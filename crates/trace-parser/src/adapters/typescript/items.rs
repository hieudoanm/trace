use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::calls::{parse_call, parse_new};
use super::class::parse_class;
use super::function::{parse_function, parse_function_named};
use super::imports::{parse_export, parse_import};
use super::types::collect_type_refs;
use super::variables::parse_variable;
use super::visibility::visibility;

pub(crate) fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    match node.kind() {
        "export_statement" => parse_export(builder, node),
        "import_statement" => parse_import(builder, node),
        "class_declaration" | "abstract_class_declaration" => {
            parse_class(builder, node, exported, NodeKind::Class);
        }
        "interface_declaration" => {
            super::class::parse_interface(builder, node, exported);
        }
        "function_declaration" | "generator_function_declaration" => {
            parse_function(builder, node, exported, NodeKind::Function);
        }
        "method_definition" | "abstract_method_signature" => {
            let name =
                super::helpers::name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            let kind = if name == "constructor" {
                NodeKind::Constructor
            } else {
                NodeKind::Method
            };
            parse_function_named(builder, node, exported, kind, name);
        }
        "method_signature" => {
            let name =
                super::helpers::name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            let id = builder.add_symbol(
                NodeKind::Method,
                name,
                node,
                visibility(builder, node),
                exported,
            );
            super::types::collect_signature_types(builder, node);
            let _ = id;
        }
        "property_signature" | "public_field_definition" | "field_definition" => {
            let name =
                super::helpers::name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            builder.add_symbol(
                NodeKind::Property,
                name,
                node,
                visibility(builder, node),
                false,
            );
            super::types::collect_signature_types(builder, node);
            if let Some(value) = child_field(node, "value") {
                parse_item(builder, value, false);
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            for child in named_children(node) {
                if child.kind() == "variable_declarator" {
                    parse_variable(builder, child, exported);
                }
            }
        }
        "enum_declaration" => {
            let name =
                super::helpers::name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            let id = builder.add_symbol(
                NodeKind::Enum,
                name,
                node,
                visibility(builder, node),
                exported,
            );
            let _ = id;
        }
        "type_alias_declaration" => {
            let name =
                super::helpers::name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            builder.add_symbol(
                NodeKind::Type,
                name,
                node,
                visibility(builder, node),
                exported,
            );
            if let Some(value) = child_field(node, "value") {
                collect_type_refs(builder, value);
            }
        }
        "internal_module" | "module" => {
            let name =
                super::helpers::name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            let id = builder.add_symbol(
                NodeKind::Namespace,
                name,
                node,
                visibility(builder, node),
                exported,
            );
            builder.push(id);
            if let Some(body) = child_field(node, "body") {
                for child in named_children(body) {
                    parse_item(builder, child, false);
                }
            }
            builder.pop();
        }
        "call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "new_expression" => {
            parse_new(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "class_body" | "interface_body" | "statement_block" | "program" => {
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
