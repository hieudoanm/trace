pub mod clean_name;
pub mod navigate_methods;
pub mod resolve_binding_path;

pub(crate) use self::clean_name::clean_name;
pub use trace_core::Language;

use crate::index::{Binding, FileState, RepositoryIndex, SymbolRef};

use super::resolve_path_helpers::last_path_segment;
use super::resolve_types::{Resolution, Want};

use crate::resolve::resolve_resolution_ext::choose;

/// Layered symbol resolver used by the graph builder.
#[derive(Debug, Clone, Copy)]
pub struct Resolver<'a> {
    pub(crate) index: &'a RepositoryIndex,
}

impl<'a> Resolver<'a> {
    pub fn new(index: &'a RepositoryIndex) -> Self {
        Self { index }
    }

    pub fn index(&self) -> &'a RepositoryIndex {
        self.index
    }

    pub fn resolve_name(
        &self,
        state: &FileState,
        scope: Option<u32>,
        raw_name: &str,
        want: Want,
    ) -> Resolution {
        let name = clean_name(raw_name);
        if name.is_empty() {
            return Resolution::Unresolved { name };
        }
        if name.contains("::") || name.contains('.') {
            if let Some(resolution) = self.resolve_binding_path(state, &name, want) {
                return resolution;
            }
            let last = last_path_segment(&name).to_string();
            if let Some(resolution) = self.resolve_binding_path(state, &last, want) {
                if matches!(resolution, Resolution::Internal(_)) {
                    return resolution;
                }
            }
        }

        // Lexical scope: declarations in the same file. Sibling methods are
        // only implicit in Kotlin and Swift; other languages need an explicit
        // receiver or import.
        let mut candidates = self.lexical_candidates(state, scope, &name, want);
        if candidates.is_empty() && matches!(state.language, Language::Kotlin | Language::Swift) {
            if let Some(scope) = scope {
                if let Some(parent) = self.index.parent_type_of(state, scope) {
                    if let Resolution::Internal(member) = self.resolve_member(&parent, &name, want)
                    {
                        candidates.push(member);
                    }
                }
            }
        }
        if !candidates.is_empty() {
            return choose(&name, candidates);
        }

        // Imported bindings.
        if let Some(binding) = state.bindings.get(&name) {
            return match binding {
                Binding::Symbol(symbol) if want.accepts(symbol) => {
                    Resolution::Internal(symbol.clone())
                }
                Binding::Symbol(_) => Resolution::Unresolved { name },
                Binding::Module(module) => Resolution::Module(module.clone()),
                Binding::ExternalSymbol { node_id, name } => Resolution::ExternalNode {
                    node_id: node_id.clone(),
                    name: name.clone(),
                },
                Binding::Unresolved { name } => Resolution::Unresolved { name: name.clone() },
                Binding::Ambiguous { name, candidates } => Resolution::Ambiguous {
                    name: name.clone(),
                    candidates: candidates.clone(),
                },
            };
        }

        // Same package/module scope. Rust is excluded because modules, not
        // crates, define unqualified name scope.
        if state.language != Language::Rust {
            if let Some(group) = &state.group {
                let candidates = self
                    .index
                    .symbols_in_group(group, &name, want == Want::Type);
                let filtered: Vec<SymbolRef> = candidates
                    .into_iter()
                    .filter(|symbol| want.accepts(symbol))
                    .collect();
                if !filtered.is_empty() {
                    return choose(&name, filtered);
                }
            }
        }

        Resolution::Unresolved { name }
    }

    pub fn resolve_type_name(
        &self,
        state: &FileState,
        scope: Option<u32>,
        name: &str,
    ) -> Resolution {
        self.resolve_name(state, scope, name, Want::Type)
    }

    pub fn resolve_callable_name(
        &self,
        state: &FileState,
        scope: Option<u32>,
        name: &str,
    ) -> Resolution {
        self.resolve_name(state, scope, name, Want::Callable)
    }
}
