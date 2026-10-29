use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::syntax::{HeritageRelation, LocalBinding};
use crate::util::{child_field, first_named, is_upper_camel, named_children, range, SyntaxBuilder};

use super::calls::collect_signature_types;
use super::imports::collect_specs_named;
use super::types::{collect_go_type_refs, go_type_name};
use super::visibility::go_visibility;

pub(crate) fn parse_type_spec(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let Some(type_node) = child_field(node, "type") else {
        return;
    };
    let kind = match type_node.kind() {
        "struct_type" => NodeKind::Struct,
        "interface_type" => NodeKind::Interface,
        _ => NodeKind::Type,
    };
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol(kind, name.clone(), node, go_visibility(exported), exported);
    builder.push(id);
    match type_node.kind() {
        "struct_type" => super::declarations::parse_struct(builder, id, type_node),
        "interface_type" => super::declarations::parse_interface_type(builder, id, type_node),
        _ => collect_go_type_refs(builder, type_node),
    }
    builder.pop();
}

pub(crate) fn parse_struct(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    for child in named_children(node) {
        if child.kind() != "field_declaration_list" {
            continue;
        }
        for field in named_children(child) {
            if field.kind() != "field_declaration" {
                continue;
            }
            let name = child_field(field, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_default();
            let type_node = child_field(field, "type");
            if name.is_empty() {
                // Embedded field: a struct/interface composition.
                if let Some(type_node) = type_node {
                    let embedded = go_type_name(builder, type_node);
                    if !embedded.is_empty() {
                        builder.add_heritage(from, embedded, HeritageRelation::Inherits, field);
                    }
                    collect_go_type_refs(builder, type_node);
                }
                continue;
            }
            let exported = is_upper_camel(&name);
            builder.add_symbol(
                NodeKind::Property,
                name,
                field,
                go_visibility(exported),
                exported,
            );
            if let Some(type_node) = type_node {
                collect_go_type_refs(builder, type_node);
            }
        }
    }
}

pub(crate) fn parse_interface_type(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    for child in named_children(node) {
        match child.kind() {
            "method_elem" => {
                let name = child_field(child, "name")
                    .map(|name| builder.text(name).to_string())
                    .unwrap_or_else(|| "anonymous".to_string());
                let exported = is_upper_camel(&name);
                builder.add_symbol(
                    NodeKind::Method,
                    name,
                    child,
                    go_visibility(exported),
                    exported,
                );
                collect_signature_types(builder, child);
            }
            "type_identifier" | "qualified_type" => {
                let name = go_type_name(builder, child);
                if !name.is_empty() {
                    builder.add_heritage(from, name, HeritageRelation::Inherits, child);
                }
                collect_go_type_refs(builder, child);
            }
            _ => {
                for nested in named_children(child) {
                    if nested.kind() == "type_identifier" || nested.kind() == "qualified_type" {
                        let name = go_type_name(builder, nested);
                        builder.add_heritage(from, name, HeritageRelation::Inherits, nested);
                    }
                }
            }
        }
    }
}

pub(crate) fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol(
        NodeKind::Function,
        name,
        node,
        go_visibility(exported),
        exported,
    );
    builder.push(id);
    collect_signature_types(builder, node);
    if let Some(body) = child_field(node, "body") {
        super::items::parse_item(builder, body);
    }
    builder.pop();
}

pub(crate) fn parse_method(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let receiver_type = child_field(node, "receiver")
        .and_then(|receiver| {
            named_children(receiver)
                .into_iter()
                .find(|child| child.kind() == "parameter_declaration")
        })
        .and_then(|parameter| child_field(parameter, "type"))
        .map(|type_node| go_type_name(builder, type_node))
        .unwrap_or_else(|| "receiver".to_string());
    let qualified = format!("{receiver_type}.{name}");
    let parent = builder.find_symbol_by_qualified(&receiver_type);
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol_qualified(
        NodeKind::Method,
        name,
        qualified,
        parent,
        node,
        go_visibility(exported),
        exported,
    );
    builder.push(id);
    collect_signature_types(builder, node);
    if let Some(body) = child_field(node, "body") {
        super::items::parse_item(builder, body);
    }
    builder.pop();
}

pub(crate) fn parse_declaration(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let mut specs = Vec::new();
    collect_specs_named(node, &mut specs);
    for spec in specs {
        let name = child_field(spec, "name")
            .or_else(|| {
                child_field(spec, "left")
                    .and_then(|left| first_named(left))
                    .filter(|name| name.kind() == "identifier")
            })
            .map(|name| builder.text(name).to_string());
        let Some(name) = name else {
            continue;
        };
        if name.contains('.') {
            continue;
        }
        let kind = if spec.kind() == "const_spec" {
            NodeKind::Constant
        } else {
            NodeKind::Variable
        };
        let exported = is_upper_camel(&name);
        builder.add_symbol(
            kind,
            name.clone(),
            spec,
            go_visibility(exported),
            exported && builder.caller().is_none(),
        );
        let type_name = child_field(spec, "type")
            .map(|type_node| go_type_name(builder, type_node))
            .or_else(|| {
                child_field(spec, "right").and_then(|right| {
                    let value = first_named(right)?;
                    if value.kind() == "composite_literal" {
                        child_field(value, "type").map(|type_node| go_type_name(builder, type_node))
                    } else {
                        None
                    }
                })
            });
        if let Some(type_name) = type_name.filter(|name| !name.is_empty()) {
            builder.locals.push(LocalBinding {
                scope: builder.caller(),
                name,
                type_name,
                range: range(spec),
            });
        }
        if let Some(right) = child_field(spec, "right") {
            for child in named_children(right) {
                super::items::parse_item(builder, child);
            }
        }
        if let Some(value) = child_field(spec, "value") {
            super::items::parse_item(builder, value);
        }
    }
}
