use trace_core::Visibility;

pub(crate) fn swift_visibility(modifiers: &str) -> Option<Visibility> {
    for word in modifiers.split_whitespace() {
        match word {
            "private" | "fileprivate" => return Some(Visibility::Private),
            "public" | "open" => return Some(Visibility::Public),
            "internal" => return Some(Visibility::Internal),
            "package" => return Some(Visibility::Package),
            _ => {}
        }
    }
    Some(Visibility::Internal)
}

pub(crate) fn swift_annotations(modifiers: &str) -> Vec<String> {
    modifiers
        .split('@')
        .skip(1)
        .map(|annotation| {
            annotation
                .split(['(', ' ', '\n'])
                .next()
                .unwrap_or(annotation)
                .trim()
                .to_string()
        })
        .filter(|annotation| !annotation.is_empty())
        .collect()
}
