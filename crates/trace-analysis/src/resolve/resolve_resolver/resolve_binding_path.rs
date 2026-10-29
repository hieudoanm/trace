use super::super::resolve_types::{Resolution, Want};
use crate::index::{Binding, FileState};

use super::Resolver;

impl<'a> Resolver<'a> {
    pub fn resolve_binding_path(
        &self,
        state: &FileState,
        raw: &str,
        want: Want,
    ) -> Option<Resolution> {
        let normalized = raw.replace("::", ".");
        let segments: Vec<&str> = normalized
            .split('.')
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.len() < 2 {
            return None;
        }
        for length in (1..=segments.len()).rev() {
            let key = segments[..length].join(".");
            let Some(binding) = state.bindings.get(&key) else {
                continue;
            };
            let mut resolution = match binding {
                Binding::Symbol(symbol) => Resolution::Internal(symbol.clone()),
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
            for segment in &segments[length..] {
                resolution = self.navigate(state, resolution, segment, want)?;
            }
            return Some(resolution);
        }
        None
    }
}
