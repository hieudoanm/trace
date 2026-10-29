//! Build a graph from a resolved index and import pass.

use trace_core::{Edge, EdgeKind, Graph, Node, NodeKind, ResolutionOrigin, ResolutionStatus};

use crate::index::RepositoryIndex;
use crate::resolve::ImportPass;

use super::super::graph_file_nodes::add_file_nodes_and_edges;
use super::super::graph_symbol_edges::{add_symbol_edges, module_id_str};
use super::super::graph_types::{GraphBuildResult, GraphBuilder};
use crate::framework::Framework;

pub(crate) fn build_from_index(
    builder: &GraphBuilder,
    index: &RepositoryIndex,
    import_pass: &ImportPass,
    frameworks: &[Framework],
) -> crate::error::Result<GraphBuildResult> {
    let mut graph = Graph::new();

    let repository = trace_core::graph::repository_node("repository");
    let repository_id = repository.id.clone();
    graph.add_node(repository)?;

    for framework in frameworks {
        let framework_id =
            trace_core::id::framework_id(&framework.name, framework.version.as_deref());
        let mut framework_node = Node::new(
            framework_id.clone(),
            NodeKind::Framework,
            framework.display_name.clone(),
        );
        framework_node
            .metadata
            .insert("framework".to_string(), framework.name.clone());
        framework_node
            .metadata
            .insert("family".to_string(), framework.family.as_str().to_string());
        framework_node.metadata.insert(
            "confidence".to_string(),
            framework.confidence.as_str().to_string(),
        );
        graph.add_node(framework_node)?;

        graph.add_edge(Edge::new(
            repository_id.clone(),
            framework_id,
            EdgeKind::UsesFramework,
            ResolutionStatus::ResolvedInternal,
            ResolutionOrigin::Syntax,
        ))?;
    }

    let import_edges: std::collections::BTreeMap<(String, String), Vec<trace_core::SourceRange>> =
        import_pass
            .edges
            .iter()
            .fold(std::collections::BTreeMap::new(), |mut acc, edge| {
                let target = builder.module_target_node(index, &module_id_str(&edge.module));
                acc.entry((edge.from_path.clone(), target))
                    .or_default()
                    .push(edge.site);
                acc
            });

    for (path, state) in &index.files {
        add_file_nodes_and_edges(builder, &mut graph, index, state, path)?;
    }

    for path in index.files.keys() {
        graph.add_edge(Edge::new(
            repository_id.clone(),
            trace_core::file_id(path),
            EdgeKind::Contains,
            ResolutionStatus::ResolvedInternal,
            ResolutionOrigin::Syntax,
        ))?;
    }
    for package_id_str in index.packages.keys() {
        graph.add_edge(Edge::new(
            repository_id.clone(),
            package_id_str.clone(),
            EdgeKind::Contains,
            ResolutionStatus::ResolvedInternal,
            ResolutionOrigin::Syntax,
        ))?;
    }

    for ((from_path, target), sites) in &import_edges {
        let status = if target.starts_with("external:") {
            ResolutionStatus::ResolvedExternal
        } else {
            ResolutionStatus::ResolvedInternal
        };
        if target.starts_with("external:") {
            if let Some(state) = index.files.get(from_path) {
                let qualified = target.strip_prefix("external:unknown:").unwrap_or(target);
                let node = Node::new(
                    target.clone(),
                    NodeKind::ExternalSymbol,
                    qualified.to_string(),
                )
                .with_language(state.language);
                let _ = graph.add_node(node);
            }
        }
        let mut edge = Edge::new(
            trace_core::file_id(from_path),
            target.clone(),
            EdgeKind::Imports,
            status,
            ResolutionOrigin::Import,
        );
        for site in sites {
            edge = edge.at(*site);
        }
        graph.add_edge(edge)?;
    }

    for (package_id_str, package) in &index.packages {
        for file_path in &package.files {
            graph.add_edge(Edge::new(
                package_id_str.clone(),
                trace_core::file_id(file_path),
                EdgeKind::Contains,
                ResolutionStatus::ResolvedInternal,
                ResolutionOrigin::Syntax,
            ))?;
        }
    }

    for (path, state) in &index.files {
        add_symbol_edges(builder, &mut graph, index, state, path)?;
    }

    graph.finalize();

    Ok(GraphBuildResult {
        graph,
        diagnostics: Vec::new(),
        frameworks: frameworks.to_vec(),
    })
}
