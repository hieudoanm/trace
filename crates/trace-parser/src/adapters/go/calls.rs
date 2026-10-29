use tree_sitter::Node as TsNode;

use crate::syntax::CallKind;
use crate::util::{child_field, is_builtin_type, named_children, SyntaxBuilder};

use super::types::collect_go_type_refs;

pub(crate) fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function") else {
        return;
    };
    match function.kind() {
        "identifier" => {
            let name = builder.text(function).to_string();
            if is_builtin_type(&name) {
                return;
            }
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
        }
        "selector_expression" => {
            let full = builder.text(function).to_string();
            let receiver =
                child_field(function, "operand").map(|operand| builder.text(operand).to_string());
            let name = child_field(function, "field")
                .map(|field| builder.text(field).to_string())
                .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).to_string());
            builder.add_call(receiver, name, full, CallKind::Method, node);
        }
        _ => {
            let full = builder.text(function).to_string();
            let name = full.rsplit('.').next().unwrap_or(&full).trim().to_string();
            builder.add_call(None, name, full, CallKind::Function, node);
        }
    }
}

pub(crate) fn parse_composite(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(type_node) = child_field(node, "type") {
        let name = super::types::go_type_name(builder, type_node);
        if !name.is_empty() && !is_builtin_type(&name) {
            builder.add_call(None, name.clone(), name, CallKind::Constructor, node);
        }
        collect_go_type_refs(builder, type_node);
    }
}

pub(crate) fn collect_signature_types(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(parameters) = child_field(node, "parameters") {
        for parameter in named_children(parameters) {
            if let Some(type_node) = child_field(parameter, "type") {
                collect_go_type_refs(builder, type_node);
            }
        }
    }
    if let Some(result) = child_field(node, "result") {
        collect_go_type_refs(builder, result);
    }
    if let Some(type_parameters) = child_field(node, "type_parameters") {
        collect_go_type_refs(builder, type_parameters);
    }
}
