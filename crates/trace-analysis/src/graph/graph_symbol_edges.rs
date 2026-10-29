//! Symbol-level edge construction (calls, type references, heritage).

use trace_core::{external_id, Edge, EdgeKind, Graph, ResolutionOrigin, ResolutionStatus};
use trace_parser::HeritageRelation;

use crate::index::{FileState, RepositoryIndex};
use crate::resolve::resolve_resolver::Resolver;

use super::graph_types::{CallTarget, GraphBuilder};

/// Add call, type-reference, and heritage edges for all symbols in a file.
pub(crate) fn add_symbol_edges(
    builder: &GraphBuilder,
    graph: &mut Graph,
    index: &RepositoryIndex,
    state: &FileState,
    path: &str,
) -> crate::error::Result<()> {
    let _ = path;
    let resolver = Resolver::new(index);

    // Call edges.
    for call in &state.parsed.calls {
        let caller_id = call
            .caller
            .and_then(|local| state.symbol(local))
            .map(|symbol| symbol.node_id.clone())
            .unwrap_or_else(|| state.module_id.clone());

        let target = builder.resolve_call_target(index, state, &resolver, call);
        match target {
            CallTarget::Internal(node_id) => {
                graph.add_edge(
                    Edge::new(
                        caller_id.clone(),
                        node_id,
                        EdgeKind::Calls,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Lexical,
                    )
                    .at(call.range),
                )?;
            }
            CallTarget::External(qualified) => {
                let node_id = external_id(state.language, &qualified);
                if builder.include_external {
                    builder.add_external_node(graph, state.language, &qualified);
                }
                graph.add_edge(
                    Edge::new(
                        caller_id.clone(),
                        node_id,
                        EdgeKind::Calls,
                        ResolutionStatus::ResolvedExternal,
                        ResolutionOrigin::Lexical,
                    )
                    .at(call.range),
                )?;
            }
            CallTarget::Unresolved => {
                builder.add_unresolved_node(graph, state.language, &call.name);
                graph.add_edge(
                    Edge::new(
                        caller_id.clone(),
                        format!("unresolved:{}", call.name),
                        EdgeKind::Calls,
                        ResolutionStatus::Unresolved,
                        ResolutionOrigin::Syntax,
                    )
                    .at(call.range),
                )?;
            }
            CallTarget::Ambiguous(candidates) => {
                for candidate in candidates {
                    graph.add_edge(
                        Edge::new(
                            caller_id.clone(),
                            candidate,
                            EdgeKind::Calls,
                            ResolutionStatus::Ambiguous,
                            ResolutionOrigin::Syntax,
                        )
                        .at(call.range),
                    )?;
                }
            }
        }
    }

    // Type reference edges.
    for type_ref in &state.parsed.type_refs {
        let from_id = type_ref
            .from
            .and_then(|local| state.symbol(local))
            .map(|symbol| symbol.node_id.clone())
            .unwrap_or_else(|| state.module_id.clone());

        let resolution = resolver.resolve_type_name(state, type_ref.from, &type_ref.name);
        match resolution {
            crate::resolve::Resolution::Internal(symbol) => {
                graph.add_edge(
                    Edge::new(
                        from_id.clone(),
                        symbol.node_id,
                        EdgeKind::UsesType,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Syntax,
                    )
                    .at(type_ref.range),
                )?;
            }
            crate::resolve::Resolution::External { qualified, .. } => {
                let node_id = external_id(state.language, &qualified);
                if builder.include_external {
                    builder.add_external_node(graph, state.language, &qualified);
                }
                graph.add_edge(
                    Edge::new(
                        from_id.clone(),
                        node_id,
                        EdgeKind::UsesType,
                        ResolutionStatus::ResolvedExternal,
                        ResolutionOrigin::Syntax,
                    )
                    .at(type_ref.range),
                )?;
            }
            crate::resolve::Resolution::Module(module) => {
                let target = builder.module_target_node(index, &module_id_str(&module));
                graph.add_edge(
                    Edge::new(
                        from_id.clone(),
                        target,
                        EdgeKind::UsesType,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Syntax,
                    )
                    .at(type_ref.range),
                )?;
            }
            _ => {
                builder.add_unresolved_node(graph, state.language, &type_ref.name);
                graph.add_edge(
                    Edge::new(
                        from_id.clone(),
                        format!("unresolved:{}", type_ref.name),
                        EdgeKind::UsesType,
                        ResolutionStatus::Unresolved,
                        ResolutionOrigin::Syntax,
                    )
                    .at(type_ref.range),
                )?;
            }
        }
    }

    // Heritage edges (inherits / implements).
    for heritage in &state.parsed.heritage {
        let from_symbol = state.symbol(heritage.from);
        let Some(from_symbol) = from_symbol else {
            continue;
        };
        let resolution = resolver.resolve_type_name(state, Some(heritage.from), &heritage.name);
        let relation = match heritage.relation {
            HeritageRelation::Implements => EdgeKind::Implements,
            _ => EdgeKind::Inherits,
        };
        match resolution {
            crate::resolve::Resolution::Internal(symbol) => {
                graph.add_edge(
                    Edge::new(
                        from_symbol.node_id.clone(),
                        symbol.node_id,
                        relation,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Syntax,
                    )
                    .at(heritage.range),
                )?;
            }
            crate::resolve::Resolution::External { qualified, .. } => {
                let node_id = external_id(state.language, &qualified);
                if builder.include_external {
                    builder.add_external_node(graph, state.language, &qualified);
                }
                graph.add_edge(
                    Edge::new(
                        from_symbol.node_id.clone(),
                        node_id,
                        relation,
                        ResolutionStatus::ResolvedExternal,
                        ResolutionOrigin::Syntax,
                    )
                    .at(heritage.range),
                )?;
            }
            _ => {
                builder.add_unresolved_node(graph, state.language, &heritage.name);
                graph.add_edge(
                    Edge::new(
                        from_symbol.node_id.clone(),
                        format!("unresolved:{}", heritage.name),
                        relation,
                        ResolutionStatus::Unresolved,
                        ResolutionOrigin::Syntax,
                    )
                    .at(heritage.range),
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn module_id_str(module: &crate::index::ModuleRef) -> String {
    match module {
        crate::index::ModuleRef::File(path) => trace_core::file_id(path),
        crate::index::ModuleRef::Package(package) => trace_core::package_id(package),
        crate::index::ModuleRef::External(name) => {
            format!("external:unknown:{name}")
        }
    }
}
