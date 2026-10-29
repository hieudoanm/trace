use tree_sitter::Tree;

use trace_core::Language;

use crate::syntax::ParsedFile;
use crate::util::{named_children, SyntaxBuilder};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child);
    }
    builder.finish()
}

fn parse_item(builder: &mut SyntaxBuilder<'_>, node: tree_sitter::Node<'_>) {
    match node.kind() {
        "function_definition" | "declaration" => {
            if let Some(declarator) = crate::util::child_field(node, "declarator") {
                let name = extract_declarator_name(builder, declarator);
                if !name.is_empty() {
                    builder.add_symbol(trace_core::NodeKind::Function, name, node, None, false);
                }
            }
        }
        "struct_specifier" | "union_specifier" | "enum_specifier" => {
            if let Some(name_node) = crate::util::child_field(node, "name") {
                let name = builder.text(name_node).to_string();
                if !name.is_empty() {
                    builder.add_symbol(trace_core::NodeKind::Struct, name, node, None, false);
                }
            }
        }
        _ => {}
    }
    for child in named_children(node) {
        parse_item(builder, child);
    }
}

fn extract_declarator_name(builder: &SyntaxBuilder<'_>, node: tree_sitter::Node<'_>) -> String {
    match node.kind() {
        "function_declarator" => {
            if let Some(declarator) = crate::util::child_field(node, "declarator") {
                return extract_declarator_name(builder, declarator);
            }
            String::new()
        }
        "identifier" => builder.text(node).to_string(),
        "field_identifier" => builder.text(node).to_string(),
        "parenthesized_declarator" => {
            for child in named_children(node) {
                if let Some(name) = crate::util::child_field(child, "declarator") {
                    let result = extract_declarator_name(builder, name);
                    if !result.is_empty() {
                        return result;
                    }
                }
            }
            for child in named_children(node) {
                let result = extract_declarator_name(builder, child);
                if !result.is_empty() {
                    return result;
                }
            }
            String::new()
        }
        _ => {
            for child in named_children(node) {
                let result = extract_declarator_name(builder, child);
                if !result.is_empty() {
                    return result;
                }
            }
            String::new()
        }
    }
}
