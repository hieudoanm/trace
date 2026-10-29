use trace_core::{NodeKind, Visibility};
use tree_sitter::Node as TsNode;

use crate::util::{child_field, find_descendant, first_named, named_children, SyntaxBuilder};

use super::super::calls::infer_call_type;
use super::super::items::parse_item;
use super::super::types::{collect_type_refs, last_type_name};
use super::super::visibility::swift_visibility;

pub(crate) fn parse_property(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let name_node = child_field(node, "name")
        .or_else(|| find_descendant(node, "simple_identifier"))
        .or_else(|| find_descendant(node, "pattern"));
    let Some(name_node) = name_node else {
        return;
    };
    let name = if name_node.kind() == "pattern" {
        find_descendant(name_node, "simple_identifier")
            .map(|name| builder.text(name).to_string())
            .unwrap_or_else(|| builder.text(name_node).to_string())
    } else {
        builder.text(name_node).to_string()
    };
    if name.is_empty() {
        return;
    }
    let modifiers = super::super::helpers::find_modifiers(builder, node);
    let visibility = swift_visibility(&modifiers);
    let immutable = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "value_binding_pattern")
        .map(|kind| builder.text(kind) == "let")
        .unwrap_or(false);
    let kind = if builder.in_type_scope() {
        NodeKind::Property
    } else if immutable {
        NodeKind::Constant
    } else {
        NodeKind::Variable
    };
    let id = builder.add_symbol(
        kind,
        name.clone(),
        node,
        visibility,
        exported || matches!(visibility, Some(Visibility::Public)),
    );
    if modifiers.contains("override") {
        builder.mark_override(id);
    }
    if modifiers.contains("final") {
        // `final` is retained in the signature; no separate graph flag.
    }
    let type_node = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "type_annotation")
        .and_then(|annotation| first_named(annotation));
    if let Some(type_node) = type_node {
        collect_type_refs(builder, type_node);
        builder.add_local(name.clone(), last_type_name(builder, type_node), node);
    } else if let Some(value) = child_field(node, "value") {
        if let Some(type_name) = infer_call_type(builder, value) {
            builder.add_local(name, type_name, node);
        }
    }
    for child in named_children(node) {
        if matches!(
            child.kind(),
            "call_expression" | "computed_property" | "closure_expression"
        ) {
            parse_item(builder, child, false);
        }
    }
}
