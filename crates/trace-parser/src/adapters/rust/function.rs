use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::calls::collect_type_refs;
use super::types::rust_visibility;

pub(crate) fn parse_function(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    kind: NodeKind,
) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    let id = builder.add_symbol(
        kind,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    builder.push(id);
    collect_function_signature(builder, node);
    if let Some(body) = child_field(node, "body") {
        super::items::parse_item(builder, body, false);
    }
    builder.pop();
}

pub(crate) fn collect_function_signature(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(parameters) = child_field(node, "parameters") {
        for parameter in named_children(parameters) {
            if let Some(type_node) = child_field(parameter, "type") {
                collect_type_refs(builder, type_node);
            }
        }
    }
    if let Some(return_type) = child_field(node, "return_type") {
        collect_type_refs(builder, return_type);
    }
    if let Some(type_parameters) = child_field(node, "type_parameters") {
        for parameter in named_children(type_parameters) {
            if let Some(name_node) = child_field(parameter, "name") {
                let type_name = builder.text(name_node).to_string();
                builder.add_type_ref(type_name, parameter);
            }
        }
    }
}
