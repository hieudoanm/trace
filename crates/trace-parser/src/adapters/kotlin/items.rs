use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::syntax::{ImportData, ImportItem, ImportKind};
use crate::util::{named_children, range, SyntaxBuilder};

use super::class::{parse_class, parse_object};
use super::delegation::parse_call;
use super::function::{parse_function, parse_property, parse_secondary_constructor};
use super::helpers::first_of_kind;
use super::types::collect_type_refs;

pub(crate) fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    match node.kind() {
        "package_header" => {
            if let Some(identifier) = named_children(node)
                .into_iter()
                .find(|child| child.kind() == "identifier")
            {
                builder.module_name = Some(builder.text(identifier).to_string());
            }
        }
        "import_list" => {
            for child in named_children(node) {
                if child.kind() == "import_header" {
                    parse_import(builder, child);
                }
            }
        }
        "class_declaration" => parse_class(builder, node, exported),
        "object_declaration" | "companion_object" => parse_object(builder, node, exported),
        "function_declaration" => parse_function(builder, node, exported),
        "property_declaration" => parse_property(builder, node, exported),
        "secondary_constructor" => parse_secondary_constructor(builder, node),
        "type_alias" => {
            if let Some(name_node) = first_of_kind(node, "type_identifier") {
                let name = builder.text(name_node).to_string();
                builder.add_symbol(NodeKind::Type, name, node, None, exported);
                for child in named_children(node) {
                    if child.kind() == "user_type" {
                        collect_type_refs(builder, child);
                    }
                }
            }
        }
        "call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "class_body" | "enum_class_body" | "source_file" => {
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        _ => {
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
    }
}

pub(crate) fn parse_import(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let mut module = String::new();
    let mut alias = None;
    let mut wildcard = false;
    for child in named_children(node) {
        match child.kind() {
            "identifier" => module = builder.text(child).to_string(),
            "import_alias" => {
                alias = named_children(child)
                    .first()
                    .map(|name| builder.text(*name).to_string());
            }
            "wildcard_import" => wildcard = true,
            _ => {}
        }
    }
    if module.is_empty() {
        return;
    }
    builder.add_import(ImportData {
        module,
        alias,
        items: if wildcard {
            vec![ImportItem {
                name: "*".to_string(),
                alias: None,
            }]
        } else {
            Vec::new()
        },
        kind: ImportKind::Kotlin,
        range: range(node),
        is_export: false,
    });
}
