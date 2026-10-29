use trace_core::Visibility;

pub(crate) fn modifier_visibility(node: tree_sitter::Node) -> Visibility {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "modifiers" {
            let text = child.to_string();
            if text.contains("public") {
                return Visibility::Public;
            }
            if text.contains("private") {
                return Visibility::Private;
            }
            if text.contains("protected") {
                return Visibility::Protected;
            }
        }
        if child.kind() == "public" {
            return Visibility::Public;
        }
        if child.kind() == "private" {
            return Visibility::Private;
        }
        if child.kind() == "protected" {
            return Visibility::Protected;
        }
    }
    Visibility::Internal
}

pub(crate) fn has_modifier(node: tree_sitter::Node, modifier: &str) -> bool {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "modifiers" && child.to_string().contains(modifier) {
            return true;
        }
        if child.kind() == modifier {
            return true;
        }
    }
    false
}
