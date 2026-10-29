use tree_sitter::Node as TsNode;

pub(crate) fn name_of(
    builder: &crate::util::SyntaxBuilder<'_>,
    node: TsNode<'_>,
) -> Option<String> {
    crate::util::child_field(node, "name")
        .map(|name| builder.text(name).to_string())
        .filter(|name| !name.is_empty())
}

#[allow(dead_code)]
pub(crate) fn is_default_import(node: TsNode<'_>) -> bool {
    crate::util::first_child_of_kind(node, "identifier").is_some()
}
