use tree_sitter::Node as TsNode;

use crate::util::{first_named, named_children, SyntaxBuilder};

pub(crate) fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    match node.kind() {
        "package_clause" => {
            if let Some(name) = first_named(node) {
                builder.module_name = Some(builder.text(name).to_string());
            }
        }
        "import_declaration" => super::imports::parse_imports(builder, node),
        "type_declaration" => {
            for spec in named_children(node) {
                if spec.kind() == "type_spec" {
                    super::declarations::parse_type_spec(builder, spec);
                }
            }
        }
        "function_declaration" => super::declarations::parse_function(builder, node),
        "method_declaration" => super::declarations::parse_method(builder, node),
        "call_expression" => {
            super::calls::parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
        "composite_literal" => {
            super::calls::parse_composite(builder, node);
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
        "short_var_declaration" | "var_declaration" | "const_declaration" => {
            super::declarations::parse_declaration(builder, node);
        }
        "block" | "statement_list" | "source_file" => {
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
        "expression_statement" => {
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
        _ => {
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
    }
}
