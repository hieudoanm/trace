use trace_core::Visibility;
use tree_sitter::Node as TsNode;

use crate::util::{children, SyntaxBuilder};

pub(crate) fn visibility(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> Option<Visibility> {
    for child in children(node) {
        if child.kind() == "accessibility_modifier" {
            return Some(Visibility::parse(builder.text(child)));
        }
    }
    None
}

pub(crate) fn normalize_type_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let trimmed = trimmed
        .split(['<', '?', '|', '&', '[', ' '])
        .next()
        .unwrap_or(trimmed);
    trimmed.trim().to_string()
}
