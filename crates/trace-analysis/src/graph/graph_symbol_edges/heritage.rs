use trace_parser::HeritageRelation;

use crate::index::{FileState, RepositoryIndex};
use crate::resolve::{resolve_resolver::Resolver, Resolution};
use trace_core::{external_id, Edge, EdgeKind, ResolutionOrigin, ResolutionStatus};

use super::super::graph_types::GraphBuilder;

pub(crate) fn add_heritage_edges(
    builder: &GraphBuilder,
    graph: &mut trace_core::Graph,
    index: &RepositoryIndex,
    state: &FileState,
) -> crate::error::Result<()> {
    let resolver = Resolver::new(index);

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
            Resolution::Internal(symbol) => {
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
            Resolution::External { qualified, .. } => {
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
