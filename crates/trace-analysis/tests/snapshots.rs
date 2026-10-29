use std::path::PathBuf;

use trace_analysis::graph::GraphBuilder;
use trace_export::{export_graph, LanguageInfo};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn portable_root(path: &std::path::Path) -> String {
    // Make the repository root portable by stripping the CARGO_MANIFEST_DIR
    // This ensures snapshots are deterministic across different environments
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    path.strip_prefix(manifest_dir)
        .unwrap_or(path)
        .to_str()
        .unwrap()
        .to_string()
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
        &portable_root(&path),
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
        &portable_root(&path),
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
        &portable_root(&path),
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
        &portable_root(&path),
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
        &portable_root(&path),
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
        &portable_root(&path),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["swift".to_string()],
    );

    insta::assert_json_snapshot!(document);
}

#[test]
fn snapshot_java_fixture() {
    let path = fixtures_dir().join("java");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "java".to_string(),
        grammar: "tree-sitter-java 0.23.5".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "java",
        &portable_root(&path),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["java".to_string()],
    );

    insta::assert_json_snapshot!(document);
}

#[test]
fn snapshot_csharp_fixture() {
    let path = fixtures_dir().join("csharp");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "csharp".to_string(),
        grammar: "tree-sitter-c-sharp 0.23.1".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "csharp",
        &portable_root(&path),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["csharp".to_string()],
    );

    insta::assert_json_snapshot!(document);
}

#[test]
fn snapshot_php_fixture() {
    let path = fixtures_dir().join("php");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "php".to_string(),
        grammar: "tree-sitter-php 0.23.11".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "php",
        &portable_root(&path),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["php".to_string()],
    );

    insta::assert_json_snapshot!(document);
}

#[test]
fn snapshot_ruby_fixture() {
    let path = fixtures_dir().join("ruby");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();

    let languages = vec![LanguageInfo {
        language: "ruby".to_string(),
        grammar: "tree-sitter-ruby 0.23.1".to_string(),
    }];

    let document = export_graph(
        &result.graph,
        "ruby",
        &portable_root(&path),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        false,
        "off",
        &["ruby".to_string()],
    );

    insta::assert_json_snapshot!(document);
}
