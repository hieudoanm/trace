use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::util::{child_field, SyntaxBuilder};

use super::parse_item;

pub(crate) fn parse_method(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = super::visibility::modifier_visibility(node);
    let in_type = builder
        .current_symbol()
        .map(|s| {
            matches!(
                s.kind,
                NodeKind::Class | NodeKind::Interface | NodeKind::Struct
            )
        })
        .unwrap_or(false);
    let kind = if in_type {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let id = builder.add_symbol(
        kind,
        name,
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
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
    let visibility = super::visibility::modifier_visibility(node);
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

pub(crate) fn parse_member(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let visibility = super::visibility::modifier_visibility(node);
    let is_const = super::visibility::has_modifier(node, "const")
        || super::visibility::has_modifier(node, "static");
    if let Some(decl) = child_field(node, "declaration") {
        if let Some(name_node) = child_field(decl, "name") {
            let name = builder.text(name_node).to_string();
            if let Some(t) = child_field(decl, "type") {
                builder.add_type_ref(builder.text(t).to_string(), t);
            } else if let Some(t) = child_field(node, "type") {
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
            return;
        }
    }
    // fallback
    if let Some(name_node) = child_field(node, "name") {
        let name = builder.text(name_node).to_string();
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
