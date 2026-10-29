use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::syntax::HeritageRelation;
use crate::util::{child_field, named_children, SyntaxBuilder};

use super::parse_item;
use super::visibility::modifier_visibility;

pub(crate) fn parse_class(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Class,
        name,
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
    if let Some(superclass) = child_field(node, "superclass") {
        let heritage_name = builder.text(superclass).to_string();
        builder.add_heritage(
            id,
            heritage_name.clone(),
            HeritageRelation::Inherits,
            superclass,
        );
    }
    if let Some(interfaces) = child_field(node, "interfaces") {
        for child in named_children(interfaces) {
            if child.kind() == "type_list" {
                for t in named_children(child) {
                    let heritage_name = builder.text(t).to_string();
                    builder.add_heritage(id, heritage_name, HeritageRelation::Implements, t);
                }
            } else if child.kind() != "keyword" {
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
    let visibility = modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Interface,
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

pub(crate) fn parse_enum(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Enum,
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
