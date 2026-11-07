#[test]
fn detects_extensions_case_insensitively() {
    assert_eq!(
        super::Language::from_extension("TS"),
        Some(super::Language::TypeScript)
    );
    assert_eq!(
        super::Language::from_extension("kt"),
        Some(super::Language::Kotlin)
    );
    assert_eq!(
        super::Language::from_extension("kts"),
        Some(super::Language::Kotlin)
    );
    assert_eq!(super::Language::from_extension("txt"), None);
}

#[test]
fn parses_language_names() {
    assert_eq!(
        super::Language::parse_name("TypeScript"),
        Some(super::Language::TypeScript)
    );
    assert_eq!(
        super::Language::parse_name("golang"),
        Some(super::Language::Go)
    );
    assert_eq!(super::Language::parse_name("cobol"), None);
}
