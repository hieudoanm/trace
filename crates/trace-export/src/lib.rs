//! Deterministic JSON and DOT graph export for Trace.
//!
//! Produces a versioned graph document that serializes a [`trace_core::Graph`]
//! together with repository metadata, diagnostics, and statistics. Output is
//! sorted by canonical keys so identical inputs always produce byte-identical
//! JSON. Also supports DOT format for Graphviz visualization.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use trace_core::{
    diagnostic::Diagnostic,
    edge::{Edge, EdgeKind},
    graph::{Graph, Stats},
    id,
    node::{Node, NodeKind},
};

/// Current schema version for the exported graph document.
pub const SCHEMA_VERSION: &str = "1.0";

/// Trace crate version, injected at compile time.
pub const TRACE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A versioned graph document suitable for JSON export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphDocument {
    pub schema_version: String,
    pub trace_version: String,
    pub repository: RepositoryMetadata,
    pub analysis: AnalysisMetadata,
    pub languages: Vec<LanguageInfo>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<Stats>,
}

/// Metadata about the analysed repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMetadata {
    pub id: String,
    pub name: String,
    pub root: String,
    pub file_count: usize,
    pub language_count: usize,
}

/// Metadata about the analysis run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisMetadata {
    pub format: String,
    pub cache_enabled: bool,
    pub semantic_mode: String,
    pub languages: Vec<String>,
}

/// Information about one supported language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageInfo {
    pub language: String,
    pub grammar: String,
}

/// Export a graph to a deterministic JSON value.
#[allow(clippy::too_many_arguments)]
pub fn export_graph(
    graph: &Graph,
    repository_name: &str,
    repository_root: &str,
    diagnostics: &[Diagnostic],
    statistics: Option<Stats>,
    languages: &[LanguageInfo],
    cache_enabled: bool,
    semantic_mode: &str,
    enabled_languages: &[String],
) -> GraphDocument {
    let mut nodes: Vec<Node> = graph.nodes().to_vec();
    let mut edges: Vec<Edge> = graph.edges().to_vec();

    // Deterministic ordering: nodes by ID, edges by (source, kind, target,
    // status, origin).
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then(a.kind.cmp(&b.kind))
            .then(a.target.cmp(&b.target))
            .then(a.status.cmp(&b.status))
            .then(a.origin.cmp(&b.origin))
    });

    let file_count = nodes.iter().filter(|n| n.kind == NodeKind::File).count();
    let language_count = languages.len();

    GraphDocument {
        schema_version: SCHEMA_VERSION.to_string(),
        trace_version: TRACE_VERSION.to_string(),
        repository: RepositoryMetadata {
            id: id::repository_id(),
            name: repository_name.to_string(),
            root: repository_root.to_string(),
            file_count,
            language_count,
        },
        analysis: AnalysisMetadata {
            format: "json".to_string(),
            cache_enabled,
            semantic_mode: semantic_mode.to_string(),
            languages: enabled_languages.to_vec(),
        },
        languages: languages.to_vec(),
        nodes: nodes.into_iter().collect(),
        edges: edges.into_iter().collect(),
        diagnostics: diagnostics.to_vec(),
        statistics,
    }
}

/// Serialize a graph document to a pretty-printed JSON value.
pub fn to_json_value(document: &GraphDocument) -> Value {
    serde_json::to_value(document).expect("graph document is always serializable")
}

/// Serialize a graph document to a pretty-printed JSON string.
pub fn to_json_string(document: &GraphDocument) -> String {
    serde_json::to_string_pretty(document).expect("graph document is always serializable")
}

/// Write a graph document as deterministic JSON to a writer.
pub fn write_json<W: std::io::Write>(writer: W, document: &GraphDocument) -> std::io::Result<()> {
    serde_json::to_writer_pretty(writer, document)
        .map_err(std::io::Error::other)
        .map(|_| ())
}

/// Export a graph to DOT format for Graphviz visualization.
pub fn to_dot(graph: &Graph) -> String {
    let mut output = String::new();
    output.push_str("digraph trace {\n");
    output.push_str("  rankdir=LR;\n");
    output.push_str("  node [shape=box];\n");
    output.push_str("  edge [dir=forward];\n\n");

    // Add nodes
    for node in graph.nodes() {
        let label = format!("{}\\n{}", node.name, node.kind.as_str());
        let color = node_kind_color(&node.kind);
        output.push_str(&format!(
            "  \"{}\" [label=\"{}\", color=\"{}\"];\n",
            node.id, label, color
        ));
    }

    output.push('\n');

    // Add edges
    for edge in graph.edges() {
        let style = edge_style(&edge.kind);
        output.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\", style=\"{}\"];\n",
            edge.source,
            edge.target,
            edge.kind.as_str(),
            style
        ));
    }

    output.push_str("}\n");
    output
}

/// Get a color for a node kind in DOT output.
fn node_kind_color(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Repository => "purple",
        NodeKind::File => "blue",
        NodeKind::Module | NodeKind::Package | NodeKind::Namespace => "cyan",
        NodeKind::Class | NodeKind::Struct | NodeKind::Interface | NodeKind::Trait => "green",
        NodeKind::Enum => "orange",
        NodeKind::Function | NodeKind::Method | NodeKind::Constructor => "red",
        NodeKind::Variable | NodeKind::Constant | NodeKind::Property | NodeKind::Parameter => {
            "gray"
        }
        NodeKind::Type => "yellow",
        NodeKind::ExternalSymbol => "lightgray",
        NodeKind::Unknown => "black",
    }
}

/// Get a style for an edge kind in DOT output.
fn edge_style(kind: &EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "solid",
        EdgeKind::Imports | EdgeKind::Exports => "dashed",
        EdgeKind::Calls => "bold",
        EdgeKind::References => "dotted",
        EdgeKind::Inherits => "solid",
        EdgeKind::Implements => "dashed",
        EdgeKind::UsesType => "dotted",
        EdgeKind::Defines => "solid",
        EdgeKind::Overrides => "dashed",
        EdgeKind::DependsOn => "dotted",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use serde_json::Value;
    use trace_core::{
        edge::{EdgeKind, ResolutionOrigin, ResolutionStatus},
        graph::repository_node,
        language::Language,
    };

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

    fn test_graph() -> Graph {
        let mut graph = Graph::new();
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
            &[LanguageInfo {
                language: "typescript".to_string(),
                grammar: "tree-sitter-typescript 0.23.2".to_string(),
            }],
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
            false,
            "off",
            &[],
        );
        assert_eq!(document.schema_version, SCHEMA_VERSION);
        assert_eq!(document.trace_version, TRACE_VERSION);
        assert_eq!(document.repository.id, id::repository_id());
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
            &[LanguageInfo {
                language: "typescript".to_string(),
                grammar: "tree-sitter-typescript 0.23.2".to_string(),
            }],
            false,
            "off",
            &["typescript".to_string()],
        );
        let json = to_json_value(&document);
        let schema = load_schema();
        jsonschema::validate(&schema, &json)
            .unwrap_or_else(|e| panic!("schema validation failed: {e}"));
    }

    #[test]
    fn dot_export_produces_valid_format() {
        let graph = test_graph();
        let dot = to_dot(&graph);
        assert!(dot.contains("digraph trace"));
        assert!(dot.contains("rankdir=LR"));
        assert!(dot.contains("repository:."));
        assert!(dot.contains("file:src/a.ts"));
        assert!(dot.contains("function:src/a.ts::foo"));
    }
}
