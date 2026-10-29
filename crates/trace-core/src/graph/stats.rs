use std::collections::BTreeMap;

use crate::diagnostic::Diagnostic;
use crate::edge::ResolutionStatus;
use crate::graph::storage::Graph;
use crate::node::NodeKind;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub files: usize,
    pub packages: usize,
    pub nodes: usize,
    pub edges: usize,
    pub symbols: usize,
    pub unresolved_edges: usize,
    pub diagnostics: usize,
    #[serde(default)]
    pub by_node_kind: BTreeMap<String, usize>,
    #[serde(default)]
    pub by_edge_kind: BTreeMap<String, usize>,
    #[serde(default)]
    pub by_language: BTreeMap<String, usize>,
    #[serde(default)]
    pub by_status: BTreeMap<String, usize>,
    #[serde(default)]
    pub by_framework: BTreeMap<String, usize>,
}

impl Stats {
    pub fn compute(graph: &Graph, diagnostics: &[Diagnostic]) -> Stats {
        let mut by_node_kind = BTreeMap::new();
        let mut by_language = BTreeMap::new();
        let mut by_edge_kind = BTreeMap::new();
        let mut by_status = BTreeMap::new();
        let mut by_framework = BTreeMap::new();
        let mut files = 0usize;
        let mut packages = 0usize;
        for node in graph.nodes() {
            *by_node_kind
                .entry(node.kind.as_str().to_string())
                .or_insert(0) += 1;
            if node.kind == NodeKind::File {
                files += 1;
            }
            if node.kind == NodeKind::Package {
                packages += 1;
            }
            if node.kind == NodeKind::Framework {
                if let Some(fw) = node.metadata.get("framework") {
                    *by_framework.entry(fw.clone()).or_insert(0) += 1;
                }
            }
            if let Some(language) = node.language {
                *by_language
                    .entry(language.as_str().to_string())
                    .or_insert(0) += 1;
            }
        }
        for edge in graph.edges() {
            *by_edge_kind
                .entry(edge.kind.as_str().to_string())
                .or_insert(0) += 1;
            *by_status
                .entry(edge.status.as_str().to_string())
                .or_insert(0) += 1;
        }
        let unresolved_edges = graph
            .edges()
            .iter()
            .filter(|edge| {
                matches!(
                    edge.status,
                    ResolutionStatus::Unresolved
                        | ResolutionStatus::Ambiguous
                        | ResolutionStatus::SyntaxOnly
                )
            })
            .count();
        Stats {
            files,
            packages,
            nodes: graph.node_count(),
            edges: graph.edge_count(),
            symbols: graph
                .nodes()
                .iter()
                .filter(|n| {
                    !matches!(
                        n.kind,
                        NodeKind::Repository
                            | NodeKind::File
                            | NodeKind::Module
                            | NodeKind::Package
                            | NodeKind::ExternalSymbol
                    )
                })
                .count(),
            unresolved_edges,
            diagnostics: diagnostics.len(),
            by_node_kind,
            by_edge_kind,
            by_language,
            by_status,
            by_framework,
        }
    }
}
