use crate::resolve::{resolve_resolver::Resolver, Resolution};

use crate::index::{FileState, RepositoryIndex};
use trace_core::{external_id, Edge, EdgeKind, ResolutionOrigin, ResolutionStatus};

use super::super::graph_types::GraphBuilder;

pub(crate) fn add_type_ref_edges(
    builder: &GraphBuilder,
    graph: &mut trace_core::Graph,
    index: &RepositoryIndex,
    state: &FileState,
) -> crate::error::Result<()> {
    let resolver = Resolver::new(index);

    for type_ref in &state.parsed.type_refs {
        let from_id = type_ref
            .from
            .and_then(|local| state.symbol(local))
            .map(|symbol| symbol.node_id.clone())
            .unwrap_or_else(|| state.module_id.clone());

        let resolution = resolver.resolve_type_name(state, type_ref.from, &type_ref.name);
        match resolution {
            Resolution::Internal(symbol) => {
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
            Resolution::External { qualified, .. } => {
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
            Resolution::Module(module) => {
                let target = builder.module_target_node(
                    index,
                    &crate::graph::graph_symbol_edges::module_id_str(&module),
                );
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
    Ok(())
}
