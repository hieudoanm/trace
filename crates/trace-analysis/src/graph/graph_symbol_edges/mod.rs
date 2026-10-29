pub mod calls;
pub mod heritage;
pub mod type_refs;

use crate::index::ModuleRef;
use trace_core::{file_id, package_id};

pub(crate) fn add_symbol_edges(
    builder: &crate::graph::graph_types::GraphBuilder,
    graph: &mut trace_core::Graph,
    index: &crate::index::RepositoryIndex,
    state: &crate::index::FileState,
    path: &str,
) -> crate::error::Result<()> {
    let _ = path;
    calls::add_call_edges(builder, graph, index, state)?;
    type_refs::add_type_ref_edges(builder, graph, index, state)?;
    heritage::add_heritage_edges(builder, graph, index, state)?;
    Ok(())
}

pub(crate) fn module_id_str(module: &ModuleRef) -> String {
    match module {
        ModuleRef::File(path) => file_id(path),
        ModuleRef::Package(package) => package_id(package),
        ModuleRef::External(name) => {
            format!("external:unknown:{name}")
        }
    }
}
