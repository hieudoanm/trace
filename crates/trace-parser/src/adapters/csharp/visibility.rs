use trace_core::Visibility;

pub(crate) fn modifier_visibility(node: tree_sitter::Node) -> Visibility {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "modifier" {
            let t = child.to_string();
            if t.contains("public") {
                return Visibility::Public;
            }
            if t.contains("private") {
                return Visibility::Private;
            }
            if t.contains("protected") {
                return Visibility::Protected;
            }
            if t.contains("internal") {
                return Visibility::Internal;
            }
        }
    }
    Visibility::Internal
}

pub(crate) fn has_modifier(node: tree_sitter::Node, modifier: &str) -> bool {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "modifier" && child.to_string().contains(modifier) {
            return true;
        }
    }
    false
}
