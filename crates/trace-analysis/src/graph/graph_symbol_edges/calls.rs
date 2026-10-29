use trace_core::{external_id, Edge, EdgeKind, ResolutionOrigin, ResolutionStatus};

use crate::index::{FileState, RepositoryIndex};
use crate::resolve::resolve_resolver::Resolver;

use super::super::graph_types::{CallTarget, GraphBuilder};

pub(crate) fn add_call_edges(
    builder: &GraphBuilder,
    graph: &mut trace_core::Graph,
    index: &RepositoryIndex,
    state: &FileState,
) -> crate::error::Result<()> {
    let resolver = Resolver::new(index);

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
    Ok(())
}
