//! Helper methods for GraphBuilder.

use trace_core::{Graph, Language, Node, NodeKind};
use trace_parser::{CallData, CallKind};

use crate::index::{FileState, ModuleRef, RepositoryIndex, SymbolRef};
use crate::resolve::resolve_path_helpers::last_path_segment;
use crate::resolve::resolve_resolver::{clean_name, Resolver};

use super::graph_types::{CallTarget, GraphBuilder};

/// Resolve the target of a call expression.
impl GraphBuilder {
    #[allow(clippy::collapsible_match)]
    pub(crate) fn resolve_call_target(
        &self,
        index: &RepositoryIndex,
        state: &FileState,
        resolver: &Resolver,
        call: &CallData,
    ) -> CallTarget {
        // If there is a receiver, resolve the receiver type and look up the
        // member method on it.
        if let Some(receiver) = &call.receiver {
            let cleaned = clean_name(receiver);
            if let Some(scope) = call.caller {
                if let Some(parent) = index.parent_type_of(state, scope) {
                    let member = resolver.resolve_member(
                        &parent,
                        &cleaned,
                        crate::resolve::resolve_types::Want::Callable,
                    );
                    if let crate::resolve::Resolution::Internal(symbol) = member {
                        return CallTarget::Internal(symbol.node_id);
                    }
                }
            }
            // Receiver resolved as a type in bindings.
            if let Some(binding) = state.bindings.get(&cleaned) {
                match binding {
                    crate::index::Binding::Symbol(symbol) if symbol.is_type() => {
                        if let Some(member) =
                            self.find_method_on_type(index, state, symbol, &call.name)
                        {
                            return CallTarget::Internal(member.node_id);
                        }
                    }
                    crate::index::Binding::Module(module) => {
                        if let ModuleRef::File(path) = module {
                            if let Some(target) = index.files.get(path) {
                                if let Some(symbol) =
                                    target.top_level.get(&call.name).and_then(|indexes| {
                                        indexes.first().and_then(|i| target.symbols.get(*i))
                                    })
                                {
                                    return CallTarget::Internal(symbol.node_id.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            return match call.kind {
                CallKind::Constructor => CallTarget::External(format!("{receiver}.{}", call.name)),
                _ => CallTarget::Unresolved,
            };
        }

        // Plain function call: resolve the name through bindings and scope.
        let scope = call.caller;
        let resolution = resolver.resolve_callable_name(state, scope, &call.name);
        match resolution {
            crate::resolve::Resolution::Internal(symbol) => {
                return CallTarget::Internal(symbol.node_id);
            }
            crate::resolve::Resolution::ExternalNode { node_id, .. } => {
                return CallTarget::Internal(node_id);
            }
            crate::resolve::Resolution::Module(module) => {
                return match module {
                    ModuleRef::File(path) => {
                        if let Some(target) = index.files.get(path.as_str()) {
                            if let Some(symbol) =
                                target.top_level.get(&call.name).and_then(|indexes| {
                                    indexes.first().and_then(|i| target.symbols.get(*i))
                                })
                            {
                                CallTarget::Internal(symbol.node_id.clone())
                            } else {
                                CallTarget::Unresolved
                            }
                        } else {
                            CallTarget::Unresolved
                        }
                    }
                    ModuleRef::Package(_) => CallTarget::Unresolved,
                    ModuleRef::External(name) => {
                        CallTarget::External(format!("{name}.{}", call.name))
                    }
                };
            }
            crate::resolve::Resolution::Ambiguous { candidates, .. } => {
                return CallTarget::Ambiguous(candidates);
            }
            _ => {}
        }

        // Same-package/group fallback for callables.
        if let Some(group) = &state.group {
            let candidates = index.symbols_in_group(group, &call.name, false);
            let callables: Vec<_> = candidates.into_iter().filter(|s| s.is_callable()).collect();
            match callables.len() {
                1 => return CallTarget::Internal(callables[0].node_id.clone()),
                _ if !callables.is_empty() => {
                    return CallTarget::Ambiguous(
                        callables.into_iter().map(|s| s.node_id).collect(),
                    );
                }
                _ => {}
            }
        }

        match call.kind {
            CallKind::Constructor => CallTarget::External(call.name.clone()),
            _ => CallTarget::Unresolved,
        }
    }

    fn find_method_on_type(
        &self,
        index: &RepositoryIndex,
        _state: &FileState,
        type_symbol: &SymbolRef,
        method_name: &str,
    ) -> Option<SymbolRef> {
        let key = format!("{}.{}", type_symbol.qualified, method_name);
        if let Some(candidates) = index.by_qualified.get(&key) {
            let callables: Vec<&SymbolRef> =
                candidates.iter().filter(|s| s.is_callable()).collect();
            if callables.len() == 1 {
                return Some(callables[0].clone());
            }
        }
        None
    }

    pub(crate) fn parent_node_id(&self, state: &FileState, symbol: &SymbolRef) -> Option<String> {
        let Some(parent_local) = symbol.parent else {
            return Some(state.module_id.clone());
        };
        let parent_symbol = state.symbol(parent_local)?;
        Some(parent_symbol.node_id.clone())
    }

    /// Resolve a module reference to the canonical node id used as an edge
    /// target.
    pub(crate) fn module_target_node(
        &self,
        index: &RepositoryIndex,
        module_id_str: &str,
    ) -> String {
        if module_id_str.starts_with("external:") {
            return module_id_str.to_string();
        }
        // Resolve a module id back to a file node when possible.
        for (path, state) in &index.files {
            if state.module_id == *module_id_str {
                return trace_core::file_id(path);
            }
        }
        // Check package id.
        if index.packages.contains_key(module_id_str) {
            return module_id_str.to_string();
        }
        module_id_str.to_string()
    }

    pub(crate) fn add_external_node(&self, graph: &mut Graph, language: Language, qualified: &str) {
        let node_id = trace_core::external_id(language, qualified);
        let name = last_path_segment(qualified);
        let node =
            Node::new(node_id, NodeKind::ExternalSymbol, name.to_string()).with_language(language);
        let _ = graph.add_node(node);
    }

    /// Ensure a placeholder `unresolved:<name>` node exists so unresolved edges
    /// always have a valid target.
    pub(crate) fn add_unresolved_node(&self, graph: &mut Graph, language: Language, name: &str) {
        let node_id = format!("unresolved:{name}");
        let node = Node::new(node_id, NodeKind::Unknown, name.to_string()).with_language(language);
        let _ = graph.add_node(node);
    }
}

/// Return the source range for a symbol.
pub(crate) fn symbol_range(
    state: &FileState,
    symbol: &SymbolRef,
) -> Option<trace_core::SourceRange> {
    state
        .parsed
        .symbols
        .iter()
        .find(|s| s.local_id == symbol.local_id)
        .map(|s| s.range)
}
