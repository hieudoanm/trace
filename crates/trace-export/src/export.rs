use trace_core::{
    diagnostic::Diagnostic,
    edge::Edge,
    graph::{Graph, Stats},
    id,
    node::{Node, NodeKind},
};

use crate::types::{
    AnalysisMetadata, FrameworkInfo, GraphDocument, LanguageInfo, RepositoryMetadata,
};

/// Current schema version for the exported graph document.
pub const SCHEMA_VERSION: &str = "1.0";

/// Trace crate version, injected at compile time.
pub const TRACE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(clippy::too_many_arguments)]
pub fn export_graph(
    graph: &Graph,
    repository_name: &str,
    repository_root: &str,
    diagnostics: &[Diagnostic],
    statistics: Option<Stats>,
    languages: &[LanguageInfo],
    frameworks: &[FrameworkInfo],
    cache_enabled: bool,
    semantic_mode: &str,
    enabled_languages: &[String],
) -> GraphDocument {
    let _stats = statistics.as_ref();
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
        frameworks: frameworks.to_vec(),
        nodes: nodes.into_iter().collect(),
        edges: edges.into_iter().collect(),
        diagnostics: diagnostics.to_vec(),
        statistics,
    }
}
