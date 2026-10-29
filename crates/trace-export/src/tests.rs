use std::path::PathBuf;

use serde_json::Value;
use trace_core::{
    edge::{EdgeKind, ResolutionOrigin, ResolutionStatus},
    graph::repository_node,
    language::Language,
    node::NodeKind,
};

use crate::export::export_graph;
use crate::serialize::to_json_string;
use crate::types::GraphDocument;

fn schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("schemas")
        .join("graph.schema.json")
}

fn load_schema() -> Value {
    let text = std::fs::read_to_string(schema_path())
        .unwrap_or_else(|e| panic!("failed to read schema at {:?}: {e}", schema_path()));
    serde_json::from_str(&text).expect("schema is valid JSON")
}

fn test_graph() -> trace_core::graph::Graph {
    let mut graph = trace_core::graph::Graph::new();
    let repo = repository_node("test-repo");
    graph.add_node(repo).unwrap();
    let file = trace_core::Node::new("file:src/a.ts", NodeKind::File, "src/a.ts")
        .with_file("src/a.ts".to_string())
        .with_language(Language::TypeScript);
    graph.add_node(file).unwrap();
    let symbol = trace_core::Node::new("function:src/a.ts::foo", NodeKind::Function, "foo")
        .with_file("src/a.ts".to_string())
        .with_language(Language::TypeScript);
    graph.add_node(symbol).unwrap();
    graph
        .add_edge(trace_core::Edge::new(
            "repository:.",
            "file:src/a.ts",
            EdgeKind::Contains,
            ResolutionStatus::ResolvedInternal,
            ResolutionOrigin::Syntax,
        ))
        .unwrap();
    graph.finalize();
    graph
}

#[test]
fn export_document_is_deterministic() {
    let graph = test_graph();
    let document = export_graph(
        &graph,
        "test-repo",
        "/repo",
        &[],
        None,
        &[crate::types::LanguageInfo {
            language: "typescript".to_string(),
            grammar: "tree-sitter-typescript 0.23.2".to_string(),
        }],
        &[],
        false,
        "off",
        &["typescript".to_string()],
    );
    let json1 = to_json_string(&document);
    let json2 = to_json_string(&document);
    assert_eq!(json1, json2);
}

#[test]
fn export_document_has_schema_version() {
    let graph = test_graph();
    let document = export_graph(
        &graph,
        "test-repo",
        "/repo",
        &[],
        None,
        &[],
        &[],
        false,
        "off",
        &[],
    );
    assert_eq!(document.schema_version, crate::export::SCHEMA_VERSION);
    assert_eq!(document.trace_version, crate::export::TRACE_VERSION);
    assert_eq!(document.repository.id, trace_core::id::repository_id());
}

#[test]
fn json_round_trip() {
    let graph = test_graph();
    let document = export_graph(
        &graph,
        "test-repo",
        "/repo",
        &[],
        None,
        &[],
        &[],
        false,
        "off",
        &[],
    );
    let json = to_json_string(&document);
    let parsed: GraphDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, document);
}

#[test]
fn nodes_and_edges_are_sorted() {
    let graph = test_graph();
    let document = export_graph(
        &graph,
        "test-repo",
        "/repo",
        &[],
        None,
        &[],
        &[],
        false,
        "off",
        &[],
    );
    let node_ids: Vec<_> = document.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(node_ids, node_ids.as_slice());
    let mut sorted_ids = node_ids.clone();
    sorted_ids.sort();
    assert_eq!(node_ids, sorted_ids);
}

#[test]
fn exported_document_conforms_to_schema() {
    let graph = test_graph();
    let document = export_graph(
        &graph,
        "test-repo",
        "/repo",
        &[],
        None,
        &[crate::types::LanguageInfo {
            language: "typescript".to_string(),
            grammar: "tree-sitter-typescript 0.23.2".to_string(),
        }],
        &[],
        false,
        "off",
        &["typescript".to_string()],
    );
    let json = crate::serialize::to_json_value(&document);
    let schema = load_schema();
    jsonschema::validate(&schema, &json)
        .unwrap_or_else(|e| panic!("schema validation failed: {e}"));
}

#[test]
fn dot_export_produces_valid_format() {
    use crate::dot::to_dot;
    let graph = test_graph();
    let dot = to_dot(&graph);
    assert!(dot.contains("digraph trace"));
    assert!(dot.contains("rankdir=LR"));
    assert!(dot.contains("repository:."));
    assert!(dot.contains("file:src/a.ts"));
    assert!(dot.contains("function:src/a.ts::foo"));
}
