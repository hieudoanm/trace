use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::util::{first_named, named_children, SyntaxBuilder};

use super::delegation::parse_delegation;
use super::helpers::{find_modifiers, kotlin_visibility};
use super::items::parse_item;

pub(crate) fn parse_class(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = super::helpers::first_of_kind(node, "type_identifier") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let is_enum = named_children(node)
        .iter()
        .any(|child| child.kind() == "enum_class_body");
    let kind = if is_enum {
        NodeKind::Enum
    } else {
        NodeKind::Class
    };
    let modifiers = find_modifiers(builder, node);
    let visibility = kotlin_visibility(&modifiers);
    let id = builder.add_symbol(
        kind,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    let decorators = super::helpers::kotlin_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    builder.push(id);

    if let Some(primary) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "primary_constructor")
    {
        let constructor_id = builder.add_symbol(
            NodeKind::Constructor,
            "constructor",
            primary,
            kotlin_visibility(&find_modifiers(builder, primary)),
            false,
        );
        let _ = constructor_id;
        for parameter in named_children(primary) {
            if parameter.kind() != "class_parameter" {
                continue;
            }
            let declaration_kind = named_children(parameter)
                .into_iter()
                .find(|child| child.kind() == "binding_pattern_kind")
                .map(|kind| builder.text(kind).to_string())
                .unwrap_or_default();
            let name_node = named_children(parameter)
                .into_iter()
                .find(|child| child.kind() == "simple_identifier");
            let type_node = named_children(parameter)
                .into_iter()
                .find(|child| child.kind() == "user_type");
            if let Some(name_node) = name_node {
                let parameter_name = builder.text(name_node).to_string();
                let symbol_kind = if declaration_kind == "val" || declaration_kind == "var" {
                    NodeKind::Property
                } else {
                    NodeKind::Parameter
                };
                builder.add_symbol(
                    symbol_kind,
                    parameter_name,
                    parameter,
                    Some(Visibility::Public),
                    false,
                );
            }
            if let Some(type_node) = type_node {
                super::types::collect_type_refs(builder, type_node);
            }
        }
    }

    for child in named_children(node) {
        if child.kind() == "delegation_specifier" {
            parse_delegation(builder, id, child);
        }
    }

    if let Some(body) = named_children(node)
        .into_iter()
        .find(|child| matches!(child.kind(), "class_body" | "enum_class_body"))
    {
        for child in named_children(body) {
            if child.kind() == "enum_entry" {
                if let Some(entry_name) = first_named(child) {
                    builder.add_symbol(
                        NodeKind::Constant,
                        builder.text(entry_name).to_string(),
                        child,
                        None,
                        true,
                    );
                }
                continue;
            }
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

pub(crate) fn parse_object(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let name = super::helpers::first_of_kind(node, "type_identifier")
        .map(|name| builder.text(name).to_string())
        .unwrap_or_else(|| {
            if node.kind() == "companion_object" {
                "Companion".to_string()
            } else {
                "object".to_string()
            }
        });
    let modifiers = find_modifiers(builder, node);
    let visibility = kotlin_visibility(&modifiers);
    let id = builder.add_symbol(
        NodeKind::Class,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    builder.push(id);
    if let Some(body) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "class_body")
    {
        for child in named_children(body) {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}
