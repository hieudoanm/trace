use std::path::PathBuf;

use trace_analysis::graph::GraphBuilder;
use trace_export::{export_graph, LanguageInfo};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/languages")
}

fn portable_root(path: &std::path::Path) -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    path.strip_prefix(manifest_dir)
        .unwrap_or(path)
        .to_str()
        .unwrap()
        .replace('\\', "/")
        .to_string()
}

fn export_for(
    language: &str,
    grammar: &str,
    path: &std::path::Path,
) -> trace_export::GraphDocument {
    let builder = GraphBuilder::new();
    let result = builder.build_repository(path).unwrap();

    let languages = vec![LanguageInfo {
        language: language.to_string(),
        grammar: grammar.to_string(),
    }];

    export_graph(
        &result.graph,
        language,
        &portable_root(path),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        &[],
        false,
        "off",
        &[language.to_string()],
    )
}

#[test]
fn snapshot_python_fixture() {
    let path = fixtures_dir().join("python");
    let document = export_for("python", "tree-sitter-python 0.25.0", &path);
    insta::assert_json_snapshot!(document);
}
