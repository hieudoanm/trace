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
use declarations::{parse_class, parse_enum, parse_interface, parse_struct};
use imports::parse_using;
use members::{parse_constructor, parse_member, parse_method};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child, false);
    }
    builder.finish()
}

fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    match node.kind() {
        "using_directive" => parse_using(builder, node),
        "class_declaration" => parse_class(builder, node, _exported),
        "interface_declaration" => parse_interface(builder, node, _exported),
        "struct_declaration" => parse_struct(builder, node, _exported),
        "enum_declaration" => parse_enum(builder, node, _exported),
        "method_declaration" => parse_method(builder, node, _exported),
        "constructor_declaration" => parse_constructor(builder, node),
        "field_declaration" | "property_declaration" => parse_member(builder, node, _exported),
        "invocation_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "block" | "class_body" | "interface_body" | "declaration_list" | "compilation_unit" => {
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
