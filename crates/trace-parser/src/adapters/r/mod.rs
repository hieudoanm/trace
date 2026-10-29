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
        "function_definition" | "function_definition_r" | "left_assignment" => {
            // Try to find name
            if let Some(name_node) = crate::util::child_field(node, "name") {
                let name = builder.text(name_node).to_string();
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    builder.add_symbol(trace_core::NodeKind::Function, name, node, None, false);
                    return;
                }
            }
            // For assignment like f <- function() or f = function()
            for child in named_children(node) {
                if child.kind() == "identifier" && node.kind() == "binary_operator"
                    || node.kind() == "left_assignment"
                {
                    // simple case
                }
            }
        }
        _ => {}
    }
    for child in named_children(node) {
        parse_item(builder, child);
    }
}

#[allow(dead_code)]
fn find_identifier(builder: &SyntaxBuilder<'_>, node: tree_sitter::Node<'_>) -> Option<String> {
    if node.kind() == "identifier" {
        return Some(builder.text(node).to_string());
    }
    for child in named_children(node) {
        if let Some(name) = find_identifier(builder, child) {
            return Some(name);
        }
    }
    None
}
