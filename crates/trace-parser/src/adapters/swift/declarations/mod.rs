pub mod function;
pub mod inheritance;
pub mod initializer;
pub mod property;

pub(crate) use self::function::parse_function;
pub(crate) use self::initializer::parse_initializer;
pub(crate) use self::property::parse_property;

use trace_core::{NodeKind, Visibility};
use tree_sitter::Node as TsNode;

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::types::collect_type_refs;
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
    inheritance::parse_inheritance(builder, id, node);
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
