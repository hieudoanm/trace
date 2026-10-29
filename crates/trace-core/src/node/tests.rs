#[test]
fn node_kind_names_round_trip() {
    for kind in super::NodeKind::ALL {
        assert_eq!(super::NodeKind::parse_name(kind.as_str()), Some(kind));
    }
}

#[test]
fn builder_sets_optional_fields() {
    let node = super::Node::new("function:a.ts::foo", super::NodeKind::Function, "foo")
        .with_file("a.ts")
        .with_language(super::Language::TypeScript)
        .with_qualified_name("foo");
    assert_eq!(node.file.as_deref(), Some("a.ts"));
    assert_eq!(node.language, Some(super::Language::TypeScript));
}
