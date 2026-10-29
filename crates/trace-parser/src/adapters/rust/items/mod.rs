pub mod enum_def;
pub mod impl_def;
pub mod module;
pub mod struct_def;
pub mod trait_def;

use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::syntax::CallKind;
use crate::util::{child_field, named_children, SyntaxBuilder};

use super::calls::{parse_call, parse_let, parse_struct_expression};
use super::function::parse_function;
use super::types::{parse_const, parse_type_item};
use super::use_decl::parse_use;

pub(crate) fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    match node.kind() {
        "use_declaration" => parse_use(builder, node),
        "mod_item" => module::parse_module(builder, node, exported),
        "struct_item" | "union_item" => struct_def::parse_struct(builder, node, exported),
        "enum_item" => enum_def::parse_enum(builder, node, exported),
        "trait_item" => trait_def::parse_trait(builder, node, exported),
        "impl_item" => impl_def::parse_impl(builder, node),
        "function_item" => parse_function(builder, node, exported, NodeKind::Function),
        "function_signature_item" => parse_function(builder, node, exported, NodeKind::Method),
        "type_item" => parse_type_item(builder, node, exported),
        "const_item" | "static_item" => parse_const(builder, node, exported),
        "call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "macro_invocation" => {
            if let Some(macro_node) = child_field(node, "macro") {
                let name = builder.text(macro_node).to_string();
                builder.add_call(None, name.clone(), name, CallKind::Macro, node);
            }
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "struct_expression" => {
            parse_struct_expression(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "let_declaration" => parse_let(builder, node),
        "block" | "declaration_list" | "source_file" => {
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "expression_statement" => {
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
