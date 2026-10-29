pub mod function;
pub mod interface;
pub mod method;
pub mod struct_;

pub(crate) use self::function::parse_function;
pub(crate) use self::method::parse_method;

use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::syntax::LocalBinding;
use crate::util::{child_field, first_named, is_upper_camel, named_children, range, SyntaxBuilder};

use super::imports::collect_specs_named;
use super::items::parse_item;
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
        "struct_type" => struct_::parse_struct(builder, id, type_node),
        "interface_type" => interface::parse_interface_type(builder, id, type_node),
        _ => collect_go_type_refs(builder, type_node),
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
                parse_item(builder, child);
            }
        }
        if let Some(value) = child_field(spec, "value") {
            parse_item(builder, value);
        }
    }
}
