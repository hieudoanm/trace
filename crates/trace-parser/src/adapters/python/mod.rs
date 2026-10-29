pub mod assignment;
pub mod call;
pub mod class;
pub mod function;
pub mod imports;
pub mod types;
pub mod visibility;

use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, Visibility};

use crate::syntax::ParsedFile;
use crate::util::{child_field, named_children, SyntaxBuilder};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child, false, Vec::new());
    }
    builder.finish()
}

fn parse_item(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    decorators: Vec<String>,
) {
    match node.kind() {
        "decorated_definition" => {
            let mut decorators = decorators;
            for child in named_children(node) {
                if child.kind() == "decorator" {
                    decorators.push(builder.text(child).trim_start_matches('@').to_string());
                }
            }
            if let Some(definition) = child_field(node, "definition") {
                parse_item(builder, definition, exported, decorators);
            }
        }
        "class_definition" => class::parse_class(builder, node, exported, decorators),
        "function_definition" => function::parse_function(builder, node, exported, decorators),
        "import_statement" => imports::parse_import_statement(builder, node),
        "import_from_statement" => imports::parse_import_from(builder, node),
        "call" => {
            call::parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false, Vec::new());
            }
        }
        "assignment" | "annotated_assignment" => {
            assignment::parse_assignment(builder, node);
            if let Some(right) = child_field(node, "right") {
                parse_item(builder, right, false, Vec::new());
            }
        }
        "block" | "module" => {
            for child in named_children(node) {
                parse_item(builder, child, false, Vec::new());
            }
        }
        "expression_statement" => {
            for child in named_children(node) {
                parse_item(builder, child, false, Vec::new());
            }
        }
        _ => {
            for child in named_children(node) {
                parse_item(builder, child, false, Vec::new());
            }
        }
    }
}

pub(crate) fn is_exported_name(name: &str) -> bool {
    !name.starts_with('_')
}

pub(crate) fn python_visibility(name: &str) -> Option<Visibility> {
    visibility::python_visibility(name)
}
