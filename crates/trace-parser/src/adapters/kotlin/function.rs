use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::util::{child_field, first_named, named_children, SyntaxBuilder};

use super::helpers::{
    find_modifiers, is_upper_camel, kotlin_annotations, kotlin_visibility, last_type_name,
};
use super::items::parse_item;
use super::types::collect_function_types;

pub(crate) fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "simple_identifier")
    else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let modifiers = find_modifiers(builder, node);
    let visibility = kotlin_visibility(&modifiers);
    let in_class = builder
        .current_symbol()
        .map(|symbol| {
            matches!(
                symbol.kind,
                NodeKind::Class | NodeKind::Interface | NodeKind::Enum | NodeKind::Trait
            )
        })
        .unwrap_or(false);
    let kind = if in_class {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let receiver = child_field(node, "receiver")
        .map(|receiver| last_type_name(builder, receiver))
        .filter(|name| !name.is_empty());
    let id = if let Some(receiver) = receiver {
        let qualified = match builder.current_symbol() {
            Some(parent) => format!("{}.{}.{}", parent.qualified_name, receiver, name),
            None => format!("{receiver}.{name}"),
        };
        let parent = builder.parent_local();
        builder.add_symbol_qualified(
            kind,
            name,
            qualified,
            parent,
            node,
            visibility,
            exported || visibility == Some(Visibility::Public),
        )
    } else {
        builder.add_symbol(
            kind,
            name,
            node,
            visibility,
            exported || visibility == Some(Visibility::Public),
        )
    };
    if modifiers.contains("override") {
        builder.mark_override(id);
    }
    let decorators = kotlin_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    builder.push(id);
    collect_function_types(builder, node);
    for child in named_children(node) {
        if child.kind() == "function_body" {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

pub(crate) fn parse_property(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = named_children(node).into_iter().find_map(|child| {
        if child.kind() == "variable_declaration" {
            super::helpers::first_of_kind(child, "simple_identifier")
        } else {
            None
        }
    }) else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let modifiers = find_modifiers(builder, node);
    let visibility = kotlin_visibility(&modifiers);
    let immutable = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "binding_pattern_kind")
        .map(|kind| builder.text(kind) == "val")
        .unwrap_or(false);
    let in_class = builder
        .current_symbol()
        .map(|symbol| {
            matches!(
                symbol.kind,
                NodeKind::Class | NodeKind::Interface | NodeKind::Enum
            )
        })
        .unwrap_or(false);
    let kind = if in_class {
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
        exported || visibility == Some(Visibility::Public),
    );
    if modifiers.contains("override") {
        builder.mark_override(id);
    }
    let decorators = kotlin_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    let type_node = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "user_type");
    if let Some(type_node) = type_node {
        super::types::collect_type_refs(builder, type_node);
        builder.add_local(name, last_type_name(builder, type_node), node);
    } else if let Some(call) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "call_expression")
    {
        if let Some(function) = first_named(call) {
            let callee = builder.text(function);
            if is_upper_camel(callee.rsplit('.').next().unwrap_or(callee)) {
                builder.add_local(name, callee.to_string(), node);
            }
        }
    }
    for child in named_children(node) {
        if child.kind() == "call_expression" {
            parse_item(builder, child, false);
        }
    }
}

pub(crate) fn parse_secondary_constructor(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let id = builder.add_symbol(NodeKind::Constructor, "constructor", node, None, false);
    builder.push(id);
    collect_function_types(builder, node);
    for child in named_children(node) {
        if matches!(
            child.kind(),
            "constructor_delegation_call" | "statements" | "block"
        ) {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}
