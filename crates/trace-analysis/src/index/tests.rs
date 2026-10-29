use super::repository::RepositoryIndex;
use trace_core::Language;
use trace_parser::{ParsedFile, SyntaxParser};

fn parse(path: &str, language: Language, source: &str) -> ParsedFile {
    let mut parser = SyntaxParser::new().unwrap();
    parser.parse(path, language, source).unwrap()
}

#[test]
fn indexes_files_and_top_level_symbols() {
    let dir = tempfile::tempdir().unwrap();
    let parsed = vec![
        parse(
            "a.py",
            Language::Python,
            "import b\nclass A:\n    def m(self):\n        pass\n",
        ),
        parse("b.py", Language::Python, "def helper():\n    pass\n"),
    ];
    let index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    assert_eq!(index.files.len(), 2);
    let a = index.files.get("a.py").unwrap();
    assert_eq!(a.module_name, "a");
    assert!(a.exports.contains_key("A"));
    assert!(index.by_name.contains_key("m"));
    assert!(index.by_python_module.contains_key("b"));
}

#[test]
fn python_packages_are_detected_from_init_files() {
    let dir = tempfile::tempdir().unwrap();
    let parsed = vec![
        parse("pkg/__init__.py", Language::Python, ""),
        parse("pkg/mod.py", Language::Python, "def f():\n    pass\n"),
    ];
    let index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let module = index.files.get("pkg/mod.py").unwrap();
    assert_eq!(module.module_name, "pkg.mod");
    let package = index.packages.values().next().unwrap();
    assert_eq!(package.files.len(), 2);
}

#[test]
fn rust_module_paths_use_crate_roots() {
    use super::helpers::rust_module_qualified;
    assert_eq!(rust_module_qualified("src/lib.rs", Some("")), "crate");
    assert_eq!(
        rust_module_qualified("src/foo/mod.rs", Some("")),
        "crate::foo"
    );
    assert_eq!(
        rust_module_qualified("crate/src/foo/bar.rs", Some("crate")),
        "crate::foo::bar"
    );
}

#[test]
fn directory_helpers_are_stable() {
    use super::helpers::{directory_key, parent_directory, stem};
    assert_eq!(directory_key("src/a.ts"), "src");
    assert_eq!(directory_key("a.ts"), ".");
    assert_eq!(parent_directory("src/deep/a.ts"), "src/deep");
    assert_eq!(stem("src/a.test.ts"), "a.test");
}
