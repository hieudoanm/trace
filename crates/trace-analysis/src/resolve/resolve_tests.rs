//! Tests for the resolution module.

use trace_core::{DiagnosticKind, Language};
use trace_parser::SyntaxParser;

use crate::index::{Binding, ModuleRef, RepositoryIndex};
use crate::resolve::resolve_import_pass::resolve_imports;
use crate::resolve::resolve_resolver::Resolver;
use crate::resolve::resolve_types::Resolution;

fn parse(path: &str, language: Language, source: &str) -> trace_parser::ParsedFile {
    let mut parser = SyntaxParser::new().unwrap();
    parser.parse(path, language, source).unwrap()
}

fn index(files: Vec<trace_parser::ParsedFile>) -> RepositoryIndex {
    let dir = tempfile::tempdir().unwrap();
    RepositoryIndex::build(dir.path(), files).unwrap()
}

#[test]
fn resolves_relative_typescript_imports() {
    let mut index = index(vec![
        parse(
            "src/math.ts",
            Language::TypeScript,
            "export function twice(n: number): number { return n * 2; }\nexport class Circle {}\n",
        ),
        parse(
            "src/main.ts",
            Language::TypeScript,
            "import { twice, Circle } from './math';\nexport function run(): number { const c = new Circle(); return twice(1); }\n",
        ),
    ]);
    let pass = resolve_imports(&mut index);
    assert!(pass
        .edges
        .iter()
        .any(|edge| matches!(&edge.module, ModuleRef::File(path) if path == "src/math.ts")));
    let main = index.files.get("src/main.ts").unwrap();
    assert!(
        matches!(main.bindings.get("twice"), Some(Binding::Symbol(symbol)) if symbol.name == "twice")
    );
    assert!(
        matches!(main.bindings.get("Circle"), Some(Binding::Symbol(symbol)) if symbol.name == "Circle")
    );
}

#[test]
fn does_not_resolve_same_named_functions_across_files_without_import() {
    let index = index(vec![
        parse(
            "a.ts",
            Language::TypeScript,
            "export function helper() {}\n",
        ),
        parse(
            "b.ts",
            Language::TypeScript,
            "export function run() { helper(); }\n",
        ),
    ]);
    let state = index.files.get("b.ts").unwrap();
    let resolver = Resolver::new(&index);
    let resolution = resolver.resolve_callable_name(state, Some(1), "helper");
    assert!(matches!(resolution, Resolution::Unresolved { .. }));
}

#[test]
fn resolves_python_package_imports() {
    let mut index = index(vec![
        parse("pkg/__init__.py", Language::Python, ""),
        parse("pkg/util.py", Language::Python, "def helper():\n    pass\n"),
        parse(
            "pkg/main.py",
            Language::Python,
            "from .util import helper\n\ndef run():\n    return helper()\n",
        ),
    ]);
    let pass = resolve_imports(&mut index);
    assert!(pass
        .edges
        .iter()
        .any(|edge| matches!(&edge.module, ModuleRef::File(path) if path == "pkg/util.py")));
    let main = index.files.get("pkg/main.py").unwrap();
    assert!(matches!(
        main.bindings.get("helper"),
        Some(Binding::Symbol(_))
    ));
}

#[test]
fn resolves_rust_use_paths() {
    let mut index = index(vec![
        parse(
            "src/util.rs",
            Language::Rust,
            "pub fn helper() -> i32 { 1 }\n",
        ),
        parse(
            "src/lib.rs",
            Language::Rust,
            "mod util;\nuse crate::util::helper;\npub fn run() -> i32 { helper() }\n",
        ),
    ]);
    let pass = resolve_imports(&mut index);
    assert!(pass
        .edges
        .iter()
        .any(|edge| matches!(&edge.module, ModuleRef::File(path) if path == "src/util.rs")));
    let state = index.files.get("src/lib.rs").unwrap();
    assert!(matches!(
        state.bindings.get("helper"),
        Some(Binding::Symbol(_))
    ));
}

#[test]
fn same_package_go_names_resolve_without_import() {
    let index = index(vec![
        parse(
            "pkg/a.go",
            Language::Go,
            "package pkg\n\nfunc Helper() int { return 1 }\n",
        ),
        parse(
            "pkg/b.go",
            Language::Go,
            "package pkg\n\nfunc Run() int { return Helper() }\n",
        ),
    ]);
    let state = index.files.get("pkg/b.go").unwrap();
    let resolver = Resolver::new(&index);
    let resolution = resolver.resolve_callable_name(state, Some(1), "Helper");
    assert!(matches!(resolution, Resolution::Internal(symbol) if symbol.name == "Helper"));
}

#[test]
fn reports_unresolved_relative_imports() {
    let mut index = index(vec![parse(
        "a.ts",
        Language::TypeScript,
        "import { missing } from './nope';\n",
    )]);
    let pass = resolve_imports(&mut index);
    assert!(pass
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == DiagnosticKind::UnresolvedImport));
}
