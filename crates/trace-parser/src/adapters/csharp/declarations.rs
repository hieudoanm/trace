use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::syntax::HeritageRelation;
use crate::util::{child_field, is_upper_camel, named_children, SyntaxBuilder};

use super::parse_item;

pub(crate) fn parse_class(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = super::visibility::modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Class,
        name,
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
    if let Some(base) = child_field(node, "bases") {
        for child in named_children(base) {
            if child.kind() == "base_list" {
                continue;
            }
            let heritage_name = builder.text(child).to_string();
            if is_upper_camel(&heritage_name) {
                builder.add_heritage(id, heritage_name, HeritageRelation::Inherits, child);
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
    let visibility = super::visibility::modifier_visibility(node);
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

pub(crate) fn parse_struct(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = super::visibility::modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Struct,
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
    let visibility = super::visibility::modifier_visibility(node);
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
