use tree_sitter::Node as TsNode;

use crate::util::{named_children, range, SyntaxBuilder};

pub(crate) fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    match node.kind() {
        "import_declaration" => {
            let module = named_children(node)
                .into_iter()
                .find(|child| child.kind() == "identifier")
                .map(|module| builder.text(module).to_string())
                .unwrap_or_default();
            if !module.is_empty() {
                builder.add_import(crate::syntax::ImportData {
                    module,
                    alias: None,
                    items: Vec::new(),
                    kind: crate::syntax::ImportKind::Swift,
                    range: range(node),
                    is_export: false,
                });
            }
        }
        "class_declaration"
        | "struct_declaration"
        | "enum_declaration"
        | "protocol_declaration"
        | "extension_declaration"
        | "actor_declaration" => {
            super::declarations::parse_type_declaration(builder, node, exported);
        }
        "function_declaration" | "protocol_function_declaration" => {
            super::declarations::parse_function(builder, node, exported);
        }
        "init_declaration" => {
            super::declarations::parse_initializer(builder, node);
        }
        "property_declaration" => {
            super::declarations::parse_property(builder, node, exported);
        }
        "call_expression" => {
            super::calls::parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "class_body" | "protocol_body" | "statements" | "source_file" => {
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
