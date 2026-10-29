use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::syntax::HeritageRelation;
use crate::util::{child_field, named_children, SyntaxBuilder};

use super::parse_item;

pub(crate) fn parse_class(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let id = builder.add_symbol(NodeKind::Class, name, node, Some(Visibility::Public), true);
    builder.push(id);
    if let Some(base) = child_field(node, "base_clause") {
        for child in named_children(base) {
            if child.kind() == "name" || child.kind() == "qualified_name" {
                let heritage_name = builder.text(child).to_string();
                builder.add_heritage(id, heritage_name, HeritageRelation::Inherits, child);
            }
        }
    }
    if let Some(interfaces) = child_field(node, "class_interface_clause") {
        for child in named_children(interfaces) {
            if matches!(child.kind(), "name" | "qualified_name") {
                let heritage_name = builder.text(child).to_string();
                builder.add_heritage(id, heritage_name, HeritageRelation::Implements, child);
            }
        }
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

pub(crate) fn parse_interface(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let id = builder.add_symbol(
        NodeKind::Interface,
        name,
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

pub(crate) fn parse_trait(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let id = builder.add_symbol(NodeKind::Trait, name, node, Some(Visibility::Public), true);
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}
