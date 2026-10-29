//! Resolution extension trait and helper functions.

use trace_core::Language;

use crate::index::{Binding, SymbolRef};

use super::resolve_types::Resolution;

/// Extension trait on `Resolution` to add external module context.
pub(crate) trait ResolutionExt {
    fn or_external(
        self,
        name: &str,
        external_module: Option<&str>,
        language: Language,
    ) -> Resolution;
}

impl ResolutionExt for Resolution {
    fn or_external(
        self,
        name: &str,
        external_module: Option<&str>,
        language: Language,
    ) -> Resolution {
        match self {
            Resolution::Unresolved { .. } => match external_module {
                Some(module) => Resolution::External {
                    qualified: join_external(language, module, name),
                    name: name.to_string(),
                },
                None => Resolution::Unresolved {
                    name: name.to_string(),
                },
            },
            other => other,
        }
    }
}

/// Convert a `Resolution` into a `Binding`.
pub(crate) fn binding_for(resolution: Resolution) -> Binding {
    match resolution {
        Resolution::Internal(symbol) => Binding::Symbol(symbol),
        Resolution::Module(module) => Binding::Module(module),
        Resolution::ExternalNode { node_id, name } => Binding::ExternalSymbol { node_id, name },
        Resolution::External { qualified, name } => Binding::ExternalSymbol {
            node_id: external_id_for(&qualified),
            name,
        },
        Resolution::Unresolved { name } => Binding::Unresolved { name },
        Resolution::Ambiguous { name, candidates } => Binding::Ambiguous { name, candidates },
    }
}

/// Build an external node ID from a qualified name.
pub(crate) fn external_id_for(qualified: &str) -> String {
    // The language segment is not known here; the node ID is only used as a
    // stable lookup key and is rewritten by the graph builder when needed.
    format!("external:unresolved:{qualified}")
}

/// Choose between unresolved, internal, or ambiguous based on candidates.
pub(crate) fn choose(name: &str, mut candidates: Vec<SymbolRef>) -> Resolution {
    candidates.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    candidates.dedup_by(|a, b| a.node_id == b.node_id);
    match candidates.len() {
        0 => Resolution::Unresolved {
            name: name.to_string(),
        },
        1 => Resolution::Internal(candidates.remove(0)),
        _ => Resolution::Ambiguous {
            name: name.to_string(),
            candidates: candidates
                .into_iter()
                .map(|symbol| symbol.node_id)
                .collect(),
        },
    }
}

/// Join an external module and name with a language-specific separator.
pub(crate) fn join_external(language: Language, module: &str, name: &str) -> String {
    let separator = match language {
        trace_core::Language::Rust => "::",
        _ => ".",
    };
    format!("{module}{separator}{name}")
}
