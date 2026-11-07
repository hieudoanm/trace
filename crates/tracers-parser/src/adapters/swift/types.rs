use tree_sitter::Node as TsNode;

use crate::util::{named_children, SyntaxBuilder};

pub(crate) fn collect_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    collect_type_identifiers(builder, node, &mut |builder, name, at| {
        builder.add_type_ref(name, at)
    });
}

pub(crate) fn collect_type_identifiers(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    add: &mut dyn FnMut(&mut SyntaxBuilder<'_>, String, TsNode<'_>),
) {
    if node.kind() == "type_identifier" {
        let name = builder.text(node).to_string();
        if !crate::util::is_builtin_type(&name) {
            add(builder, name, node);
        }
        return;
    }
    for child in named_children(node) {
        collect_type_identifiers(builder, child, add);
    }
}

pub(crate) fn return_type_node(node: TsNode<'_>) -> Option<TsNode<'_>> {
    let mut found = None;
    for child in named_children(node) {
        match child.kind() {
            "user_type" => {
                found = Some(child);
                break;
            }
            "parameter_clause" | "parameter" => {}
            _ => {}
        }
    }
    found
}

pub(crate) fn last_type_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    builder
        .text(node)
        .trim_end_matches('?')
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_string()
}
