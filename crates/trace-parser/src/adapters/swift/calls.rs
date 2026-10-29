use tree_sitter::Node as TsNode;

use crate::syntax::CallKind;
use crate::util::{child_field, first_named, is_upper_camel, named_children, SyntaxBuilder};

use super::types::collect_type_refs;

pub(crate) fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function").or_else(|| first_named(node)) else {
        return;
    };
    if function.kind() == "navigation_expression" {
        let receiver =
            child_field(function, "target").map(|target| builder.text(target).to_string());
        let name = child_field(function, "suffix")
            .and_then(|suffix| first_named(suffix))
            .map(|name| builder.text(name).to_string())
            .unwrap_or_else(|| builder.text(function).to_string());
        let full = builder.text(function).to_string();
        let kind = if name == "init" {
            CallKind::Constructor
        } else if receiver.as_deref() == Some("super") {
            CallKind::Super
        } else {
            CallKind::Method
        };
        builder.add_call(receiver, name, full, kind, node);
    } else if function.kind() == "simple_identifier" {
        let name = builder.text(function).to_string();
        let kind = if name == "super" {
            CallKind::Super
        } else if is_upper_camel(&name) {
            CallKind::Constructor
        } else {
            CallKind::Function
        };
        builder.add_call(None, name.clone(), name, kind, node);
    }
}

pub(crate) fn infer_call_type(builder: &SyntaxBuilder<'_>, value: TsNode<'_>) -> Option<String> {
    if value.kind() != "call_expression" {
        return None;
    }
    let function = child_field(value, "function").or_else(|| first_named(value))?;
    if function.kind() == "simple_identifier" {
        let name = builder.text(function);
        if is_upper_camel(name) {
            return Some(name.to_string());
        }
    }
    None
}

pub(crate) fn collect_parameter_types(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    for child in named_children(node) {
        if child.kind() == "parameter_clause" {
            for parameter in named_children(child) {
                collect_one_parameter(builder, parameter);
            }
        } else if child.kind() == "parameter" {
            collect_one_parameter(builder, child);
        }
    }
}

fn collect_one_parameter(builder: &mut SyntaxBuilder<'_>, parameter: TsNode<'_>) {
    let type_node = named_children(parameter)
        .into_iter()
        .find(|nested| nested.kind() == "user_type");
    if let Some(type_node) = type_node {
        collect_type_refs(builder, type_node);
    }
    let name_node = named_children(parameter)
        .into_iter()
        .rfind(|nested| nested.kind() == "simple_identifier");
    if let (Some(name), Some(type_node)) = (name_node, type_node) {
        let name = builder.text(name).to_string();
        let type_name = super::types::last_type_name(builder, type_node);
        builder.add_local(name, type_name, parameter);
    }
}
