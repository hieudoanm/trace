use trace_core::Visibility;

pub(crate) fn go_visibility(exported: bool) -> Option<Visibility> {
    if exported {
        Some(Visibility::Public)
    } else {
        Some(Visibility::Internal)
    }
}
