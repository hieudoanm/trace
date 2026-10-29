use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::parse_item;
use super::visibility::{has_modifier, modifier_visibility};

pub(crate) fn parse_method(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = modifier_visibility(node);
    let in_class = builder
        .current_symbol()
        .map(|s| matches!(s.kind, NodeKind::Class | NodeKind::Interface))
        .unwrap_or(false);
    let kind = if in_class {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let id = builder.add_symbol(
        kind,
        name.clone(),
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
    if let Some(params) = child_field(node, "parameters") {
        for child in named_children(params) {
            if child.kind() == "formal_parameter" {
                if let Some(t) = child_field(child, "type") {
                    builder.add_type_ref(builder.text(t).to_string(), t);
                }
            }
        }
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

pub(crate) fn parse_constructor(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let name = builder
        .current_symbol()
        .map(|s| s.name.clone())
        .unwrap_or_default();
    let visibility = modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Constructor,
        name,
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        super::parse_item(builder, body, false);
    }
    builder.pop();
}

pub(crate) fn parse_field(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let visibility = modifier_visibility(node);
    let is_const = has_modifier(node, "final") || has_modifier(node, "static");
    if let Some(declarator) = child_field(node, "declarator") {
        if let Some(name_node) = child_field(declarator, "name") {
            let name = builder.text(name_node).to_string();
            if let Some(t) = child_field(node, "type") {
                builder.add_type_ref(builder.text(t).to_string(), t);
            }
            let kind = if is_const {
                NodeKind::Constant
            } else {
                NodeKind::Property
            };
            builder.add_symbol(
                kind,
                name,
                node,
                Some(visibility),
                visibility == Visibility::Public,
            );
        }
    }
}
