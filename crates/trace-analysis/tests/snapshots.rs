use std::path::PathBuf;

use trace_analysis::graph::GraphBuilder;
use trace_export::{export_graph, LanguageInfo};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

#[test]
fn snapshot_typescript_fixture() {
    let path = fixtures_dir().join("typescript");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "typescript".to_string(),
        grammar: "tree-sitter-typescript 0.23.2".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "typescript",
        path.to_str().unwrap(),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["typescript".to_string()],
    );

    insta::assert_json_snapshot!(document);
}

#[test]
fn snapshot_python_fixture() {
    let path = fixtures_dir().join("python");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "python".to_string(),
        grammar: "tree-sitter-python 0.25.0".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "python",
        path.to_str().unwrap(),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["python".to_string()],
    );

    insta::assert_json_snapshot!(document);
}

#[test]
fn snapshot_go_fixture() {
    let path = fixtures_dir().join("go");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "go".to_string(),
        grammar: "tree-sitter-go 0.25.0".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "go",
        path.to_str().unwrap(),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["go".to_string()],
    );

    insta::assert_json_snapshot!(document);
}

#[test]
fn snapshot_rust_fixture() {
    let path = fixtures_dir().join("rust");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "rust".to_string(),
        grammar: "tree-sitter-rust 0.24.2".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "rust",
        path.to_str().unwrap(),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["rust".to_string()],
    );

    insta::assert_json_snapshot!(document);
}

#[test]
fn snapshot_kotlin_fixture() {
    let path = fixtures_dir().join("kotlin");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "kotlin".to_string(),
        grammar: "tree-sitter-kt 0.1.0".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "kotlin",
        path.to_str().unwrap(),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["kotlin".to_string()],
    );

    insta::assert_json_snapshot!(document);
}

#[test]
fn snapshot_swift_fixture() {
    let path = fixtures_dir().join("swift");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "swift".to_string(),
        grammar: "tree-sitter-swift 0.7.4".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "swift",
        path.to_str().unwrap(),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["swift".to_string()],
    );

    insta::assert_json_snapshot!(document);
}
