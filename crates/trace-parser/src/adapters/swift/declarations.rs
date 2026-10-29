use trace_core::{NodeKind, Visibility};
use tree_sitter::Node as TsNode;

use crate::syntax::HeritageRelation;
use crate::util::{child_field, find_descendant, first_named, named_children, SyntaxBuilder};

use super::types::{collect_type_refs, last_type_name};
use super::visibility::{swift_annotations, swift_visibility};

pub(crate) fn parse_type_declaration(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
) {
    let name = child_field(node, "name")
        .map(|name| builder.text(name).to_string())
        .or_else(|| {
            named_children(node)
                .into_iter()
                .find(|child| matches!(child.kind(), "type_identifier" | "simple_identifier"))
                .map(|name| builder.text(name).to_string())
        });
    let Some(name) = name else {
        return;
    };
    let is_extension = node.kind() == "extension_declaration";
    let kind = match node.kind() {
        "struct_declaration" => NodeKind::Struct,
        "enum_declaration" => NodeKind::Enum,
        "protocol_declaration" => NodeKind::Interface,
        _ => NodeKind::Class,
    };
    let modifiers = super::helpers::find_modifiers(builder, node);
    let visibility = swift_visibility(&modifiers);

    if is_extension {
        let previous = builder.set_qualifier(Some(name));
        for child in named_children(node) {
            if child.kind() == "inheritance_specifier" {
                if let Some(type_node) = child_field(child, "inherits_from") {
                    collect_type_refs(builder, type_node);
                }
            }
        }
        if let Some(body) = named_children(node).into_iter().find(|child| {
            matches!(
                child.kind(),
                "class_body" | "protocol_body" | "enum_class_body"
            )
        }) {
            for child in named_children(body) {
                super::items::parse_item(builder, child, false);
            }
        }
        builder.set_qualifier(previous);
        return;
    }

    let id = builder.add_symbol(
        kind,
        name.clone(),
        node,
        visibility,
        exported || matches!(visibility, Some(Visibility::Public)),
    );
    let decorators = swift_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    builder.push(id);
    super::declarations::parse_inheritance(builder, id, node);
    if let Some(body) = named_children(node).into_iter().find(|child| {
        matches!(
            child.kind(),
            "class_body" | "protocol_body" | "enum_class_body"
        )
    }) {
        for child in named_children(body) {
            super::items::parse_item(builder, child, false);
        }
    }
    builder.pop();
}

pub(crate) fn parse_inheritance(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    let mut index = 0usize;
    for child in named_children(node) {
        if child.kind() != "inheritance_specifier" {
            continue;
        }
        let relation = if index == 0 {
            HeritageRelation::Inherits
        } else {
            HeritageRelation::Implements
        };
        index += 1;
        if let Some(type_node) = child_field(child, "inherits_from") {
            crate::adapters::swift::types::collect_type_identifiers(
                builder,
                type_node,
                &mut |builder, name, at| builder.add_heritage(from, name, relation, at),
            );
            collect_type_refs(builder, type_node);
        }
    }
}

pub(crate) fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name").or_else(|| {
        named_children(node)
            .into_iter()
            .find(|child| child.kind() == "simple_identifier")
    }) else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let kind = if builder.in_type_scope() {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let modifiers = super::helpers::find_modifiers(builder, node);
    let visibility = swift_visibility(&modifiers);
    let id = builder.add_symbol(
        kind,
        name,
        node,
        visibility,
        exported || matches!(visibility, Some(Visibility::Public)),
    );
    if modifiers.contains("override") {
        builder.mark_override(id);
    }
    let decorators = swift_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    builder.push(id);
    super::calls::collect_parameter_types(builder, node);
    if let Some(return_type) = super::types::return_type_node(node) {
        collect_type_refs(builder, return_type);
    }
    if let Some(body) = child_field(node, "body") {
        super::items::parse_item(builder, body, false);
    }
    builder.pop();
}

pub(crate) fn parse_initializer(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let id = builder.add_symbol(NodeKind::Constructor, "init", node, None, false);
    builder.push(id);
    super::calls::collect_parameter_types(builder, node);
    if let Some(body) = child_field(node, "body") {
        super::items::parse_item(builder, body, false);
    }
    builder.pop();
}

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
    let modifiers = super::helpers::find_modifiers(builder, node);
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
        if let Some(type_name) = super::calls::infer_call_type(builder, value) {
            builder.add_local(name, type_name, node);
        }
    }
    for child in named_children(node) {
        if matches!(
            child.kind(),
            "call_expression" | "computed_property" | "closure_expression"
        ) {
            super::items::parse_item(builder, child, false);
        }
    }
}
