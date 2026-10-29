use super::super::resolve_types::{Resolution, Want};
use crate::index::{FileState, ModuleRef, SymbolRef};
use crate::resolve::resolve_resolution_ext::{choose, external_id_for, join_external};

use super::Resolver;

impl<'a> Resolver<'a> {
    pub(crate) fn navigate(
        &self,
        state: &FileState,
        resolution: Resolution,
        segment: &str,
        want: Want,
    ) -> Option<Resolution> {
        match resolution {
            Resolution::Internal(symbol) if symbol.is_type() => {
                Some(self.resolve_member(&symbol, segment, want))
            }
            Resolution::Module(module) => Some(self.resolve_module_member(state, &module, segment)),
            Resolution::External { qualified, .. } => Some(Resolution::External {
                qualified: join_external(state.language, &qualified, segment),
                name: segment.to_string(),
            }),
            Resolution::ExternalNode { node_id, .. } => Some(Resolution::ExternalNode {
                node_id: external_id_for(&format!("{node_id}.{segment}")),
                name: segment.to_string(),
            }),
            _ => None,
        }
    }

    pub(crate) fn resolve_module_member(
        &self,
        _state: &FileState,
        module: &ModuleRef,
        name: &str,
    ) -> Resolution {
        match module {
            ModuleRef::File(path) => {
                let Some(target) = self.index.files.get(path) else {
                    return Resolution::Unresolved {
                        name: name.to_string(),
                    };
                };
                let candidates: Vec<SymbolRef> = target
                    .exports
                    .get(name)
                    .into_iter()
                    .flatten()
                    .filter_map(|id| self.index.symbol(id).cloned())
                    .collect();
                if candidates.is_empty() {
                    if let Some(indexes) = target.top_level.get(name) {
                        return choose(
                            name,
                            indexes
                                .iter()
                                .map(|index| target.symbols[*index].clone())
                                .collect(),
                        );
                    }
                }
                choose(name, candidates)
            }
            ModuleRef::Package(package) => {
                choose(name, self.index.symbols_in_package(package, name))
            }
            ModuleRef::External(module) => Resolution::External {
                qualified: module.clone(),
                name: name.to_string(),
            },
        }
    }
}
