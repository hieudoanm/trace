use super::*;

#[test]
fn strips_quotes() {
    assert_eq!(strip_quotes("\"./math\""), "./math");
    assert_eq!(strip_quotes("'x'"), "x");
}

#[test]
fn last_segments() {
    assert_eq!(last_segment("crate::util::helper"), "helper");
    assert_eq!(last_segment("com.example.Helper"), "Helper");
    assert_eq!(last_segment("module"), "module");
}

#[test]
fn builtin_types_are_filtered() {
    assert!(is_builtin_type("string"));
    assert!(is_builtin_type("Self"));
    assert!(!is_builtin_type("Circle"));
}
