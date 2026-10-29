use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::parse_item;
use super::visibility::{has_modifier, method_visibility};

pub(crate) fn parse_method(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let in_class = builder
        .current_symbol()
        .map(|s| {
            matches!(
                s.kind,
                NodeKind::Class | NodeKind::Trait | NodeKind::Interface
            )
        })
        .unwrap_or(false);
    let kind = if in_class {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let visibility = method_visibility(node);
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

pub(crate) fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let id = builder.add_symbol(
        NodeKind::Function,
        name.clone(),
        node,
        Some(Visibility::Public),
        true,
    );
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

pub(crate) fn parse_property(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let visibility = method_visibility(node);
    for child in named_children(node) {
        if child.kind() == "property_element" {
            if let Some(name_node) = child_field(child, "name") {
                let name = builder.text(name_node).trim_start_matches('$').to_string();
                let kind = if has_modifier(node, "const") {
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
}
