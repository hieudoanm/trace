use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::syntax::{CallKind, LocalBinding};
use crate::util::{
    child_field, first_named, is_builtin_type, is_upper_camel, named_children, range, SyntaxBuilder,
};

use super::items::parse_item;

pub(crate) fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function") else {
        return;
    };
    match function.kind() {
        "identifier" => {
            let name = builder.text(function).to_string();
            if !is_builtin_type(&name) {
                builder.add_call(None, name.clone(), name, CallKind::Function, node);
            }
        }
        "scoped_identifier" => {
            let full = builder.text(function).to_string();
            let path = child_field(function, "path")
                .map(|path| builder.text(path).to_string())
                .unwrap_or_default();
            let name = child_field(function, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_else(|| full.rsplit("::").next().unwrap_or(&full).to_string());
            let kind = if is_upper_camel(path.rsplit("::").next().unwrap_or(&path)) {
                CallKind::Constructor
            } else {
                CallKind::Function
            };
            builder.add_call(Some(path), name, full, kind, node);
        }
        "field_expression" => {
            let full = builder.text(function).to_string();
            let receiver =
                child_field(function, "value").map(|value| builder.text(value).to_string());
            let name = child_field(function, "field")
                .map(|field| builder.text(field).to_string())
                .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).to_string());
            builder.add_call(receiver, name, full, CallKind::Method, node);
        }
        "generic_function" => {
            if let Some(inner) = child_field(function, "function") {
                let full = builder.text(inner).to_string();
                let name = full.rsplit("::").next().unwrap_or(&full).trim().to_string();
                builder.add_call(None, name, full, CallKind::Function, node);
            }
        }
        _ => {
            let full = builder.text(function).to_string();
            let name = full.rsplit("::").next().unwrap_or(&full).trim().to_string();
            if !name.is_empty() {
                builder.add_call(None, name, full, CallKind::Function, node);
            }
        }
    }
}

pub(crate) fn parse_struct_expression(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(name_node) = child_field(node, "name") {
        let name = super::types::rust_type_name(builder, name_node);
        if !name.is_empty() {
            builder.add_call(None, name.clone(), name, CallKind::Constructor, node);
        }
        collect_type_refs(builder, name_node);
    }
}

pub(crate) fn parse_let(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(pattern) = child_field(node, "pattern") else {
        if let Some(value) = child_field(node, "value") {
            parse_item(builder, value, false);
        }
        return;
    };
    let value = child_field(node, "value");
    let type_node = child_field(node, "type");
    if pattern.kind() == "identifier" {
        let name = builder.text(pattern).to_string();
        builder.add_symbol(NodeKind::Variable, name.clone(), node, None, false);
        let mut inferred =
            type_node.map(|type_node| super::types::rust_type_name(builder, type_node));
        if inferred.as_deref().unwrap_or("").is_empty() {
            inferred = value.and_then(|value| infer_expression_type(builder, value));
        }
        if let Some(type_name) = inferred.filter(|name| !name.is_empty()) {
            builder.locals.push(LocalBinding {
                scope: builder.caller(),
                name,
                type_name,
                range: range(node),
            });
        }
    }
    if let Some(type_node) = type_node {
        collect_type_refs(builder, type_node);
    }
    if let Some(value) = value {
        parse_item(builder, value, false);
    }
}

pub(crate) fn infer_expression_type(
    builder: &SyntaxBuilder<'_>,
    node: TsNode<'_>,
) -> Option<String> {
    match node.kind() {
        "struct_expression" => {
            child_field(node, "name").map(|name| super::types::rust_type_name(builder, name))
        }
        "call_expression" => {
            let function = child_field(node, "function")?;
            if function.kind() == "scoped_identifier" {
                let path = child_field(function, "path")?;
                let candidate = super::types::rust_type_name(builder, path);
                let last = candidate.rsplit("::").next().unwrap_or(&candidate);
                if is_upper_camel(last) {
                    return Some(candidate);
                }
            }
            None
        }
        "reference_expression" | "parenthesized_expression" | "await_expression" => {
            first_named(node).and_then(|inner| infer_expression_type(builder, inner))
        }
        _ => None,
    }
}

pub(crate) fn collect_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    match node.kind() {
        "type_identifier" | "identifier" => {
            let name = builder.text(node).to_string();
            if !is_builtin_type(&name) && is_upper_camel(&name) {
                builder.add_type_ref(name, node);
            }
        }
        "scoped_type_identifier" => {
            let text = builder.text(node).to_string();
            let last = text.rsplit("::").next().unwrap_or(&text).to_string();
            if !is_builtin_type(&last) {
                builder.add_type_ref(last, node);
            }
        }
        _ => {
            for child in named_children(node) {
                collect_type_refs(builder, child);
            }
        }
    }
}
