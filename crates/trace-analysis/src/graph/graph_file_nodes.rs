//! File node and edge construction for GraphBuilder.

use trace_core::{Edge, EdgeKind, Graph, Node, NodeKind, ResolutionOrigin, ResolutionStatus};

use crate::index::{FileState, RepositoryIndex};

use super::graph_helpers::symbol_range;
use super::graph_types::GraphBuilder;

/// Add file, package, module, and symbol nodes plus their containment edges.
pub(crate) fn add_file_nodes_and_edges(
    builder: &GraphBuilder,
    graph: &mut Graph,
    _index: &RepositoryIndex,
    state: &FileState,
    path: &str,
) -> crate::error::Result<()> {
    let file_node = Node::new(trace_core::file_id(path), NodeKind::File, path.to_string())
        .with_file(path.to_string())
        .with_language(state.language);
    graph.add_node(file_node)?;

    // Package node.
    if let Some(package) = &state.package {
        let package_node = Node::new(package.clone(), NodeKind::Package, package.clone())
            .with_file(path.to_string())
            .with_language(state.language);
        graph.add_node(package_node)?;

        graph.add_edge(Edge::new(
            package.clone(),
            trace_core::file_id(path),
            EdgeKind::Contains,
            ResolutionStatus::ResolvedInternal,
            ResolutionOrigin::Syntax,
        ))?;
    }

    // Module node.
    let module_node = Node::new(
        state.module_id.clone(),
        NodeKind::Module,
        state.module_name.clone(),
    )
    .with_file(path.to_string())
    .with_language(state.language);
    graph.add_node(module_node)?;

    graph.add_edge(Edge::new(
        trace_core::file_id(path),
        state.module_id.clone(),
        EdgeKind::Contains,
        ResolutionStatus::ResolvedInternal,
        ResolutionOrigin::Syntax,
    ))?;

    // Symbol nodes.
    for symbol in &state.symbols {
        let parent = builder.parent_node_id(state, symbol);
        let mut node = Node::new(symbol.node_id.clone(), symbol.kind, symbol.name.clone())
            .with_file(path.to_string())
            .with_language(state.language)
            .with_qualified_name(symbol.qualified.clone())
            .with_exported(symbol.exported);
        if let Some(parent_id) = parent {
            node = node.with_parent(parent_id);
        }
        if let Some(range) = symbol_range(state, symbol) {
            node = node.with_range(range);
        }
        graph.add_node(node)?;

        // Module defines symbol.
        graph.add_edge(Edge::new(
            state.module_id.clone(),
            symbol.node_id.clone(),
            EdgeKind::Defines,
            ResolutionStatus::ResolvedInternal,
            ResolutionOrigin::Syntax,
        ))?;

        // File contains symbol.
        graph.add_edge(Edge::new(
            trace_core::file_id(path),
            symbol.node_id.clone(),
            EdgeKind::Contains,
            ResolutionStatus::ResolvedInternal,
            ResolutionOrigin::Syntax,
        ))?;

        // File exports symbol.
        if symbol.exported {
            graph.add_edge(Edge::new(
                trace_core::file_id(path),
                symbol.node_id.clone(),
                EdgeKind::Exports,
                ResolutionStatus::ResolvedInternal,
                ResolutionOrigin::Syntax,
            ))?;
        }
    }

    // Exports from re-exports.
    for ids in state.exports.values() {
        for id in ids {
            graph.add_edge(Edge::new(
                trace_core::file_id(path),
                id.clone(),
                EdgeKind::Exports,
                ResolutionStatus::ResolvedInternal,
                ResolutionOrigin::Import,
            ))?;
        }
    }
    Ok(())
}
