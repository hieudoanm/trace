use tree_sitter::Node as TsNode;

use crate::util::{named_children, SyntaxBuilder};

pub(crate) fn collect_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if node.kind() == "identifier" {
        builder.add_type_ref(builder.text(node).to_string(), node);
        return;
    }
    for child in named_children(node) {
        collect_type_refs(builder, child);
    }
}

pub(crate) fn normalize_python_type(raw: &str) -> String {
    raw.trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .split(['[', '|', ' '])
        .next()
        .unwrap_or(raw)
        .trim()
        .to_string()
}
