use trace_core::Visibility;

pub(crate) fn python_visibility(name: &str) -> Option<Visibility> {
    if name.starts_with("__") && !name.ends_with("__") {
        Some(Visibility::Private)
    } else if name.starts_with('_') && !name.ends_with("__") {
        Some(Visibility::Protected)
    } else {
        Some(Visibility::Public)
    }
}
