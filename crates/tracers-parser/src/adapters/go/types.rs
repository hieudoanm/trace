use tree_sitter::Node as TsNode;

use crate::util::{child_field, first_named, is_builtin_type, named_children, SyntaxBuilder};

pub(crate) fn collect_go_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    match node.kind() {
        "type_identifier" => {
            let name = builder.text(node).to_string();
            if !is_builtin_type(&name) {
                builder.add_type_ref(name, node);
            }
        }
        "qualified_type" => {
            let name = child_field(node, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_else(|| builder.text(node).to_string());
            if !is_builtin_type(&name) {
                builder.add_type_ref(name, node);
            }
        }
        _ => {
            for child in named_children(node) {
                collect_go_type_refs(builder, child);
            }
        }
    }
}

pub(crate) fn go_type_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    match node.kind() {
        "pointer_expression" | "parenthesized_type" | "generic_type" => first_named(node)
            .map(|child| go_type_name(builder, child))
            .unwrap_or_default(),
        "qualified_type" => child_field(node, "name")
            .map(|name| builder.text(name).to_string())
            .unwrap_or_else(|| builder.text(node).to_string()),
        "type_identifier" | "identifier" => builder.text(node).to_string(),
        _ => builder.text(node).to_string(),
    }
}
