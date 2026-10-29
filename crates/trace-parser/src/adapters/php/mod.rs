pub mod calls;
pub mod declarations;
pub mod imports;
pub mod members;
pub mod visibility;

use tree_sitter::{Node as TsNode, Tree};

use trace_core::Language;

use crate::syntax::ParsedFile;
use crate::util::{named_children, SyntaxBuilder};

use calls::parse_call;
use declarations::{parse_class, parse_interface, parse_trait};
use imports::parse_use;
use members::{parse_function, parse_method, parse_property};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child, false);
    }
    builder.finish()
}

fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    match node.kind() {
        "namespace_use_declaration" | "use_declaration" => parse_use(builder, node),
        "class_declaration" => parse_class(builder, node, _exported),
        "interface_declaration" => parse_interface(builder, node, _exported),
        "trait_declaration" => parse_trait(builder, node, _exported),
        "method_declaration" => parse_method(builder, node, _exported),
        "function_definition" => parse_function(builder, node, _exported),
        "property_declaration" => parse_property(builder, node, _exported),
        "function_call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "block" | "declaration_list" | "program" | "compound_statement" => {
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
