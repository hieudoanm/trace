//! Member resolution methods for `Resolver`.

use std::collections::HashSet;

use trace_core::NodeKind;

use crate::index::{FileState, SymbolRef};

use super::resolve_resolver::Resolver;
use super::resolve_types::{Resolution, Want};

impl<'a> Resolver<'a> {
    pub fn resolve_member(&self, type_symbol: &SymbolRef, name: &str, want: Want) -> Resolution {
        let mut visited = HashSet::new();
        self.resolve_member_inner(type_symbol, name, want, &mut visited)
    }

    pub(crate) fn resolve_member_inner(
        &self,
        type_symbol: &SymbolRef,
        name: &str,
        want: Want,
        visited: &mut HashSet<String>,
    ) -> Resolution {
        if !visited.insert(type_symbol.node_id.clone()) {
            return Resolution::Unresolved {
                name: name.to_string(),
            };
        }
        let key = format!("{}.{}", type_symbol.qualified, name);
        let mut candidates: Vec<SymbolRef> = self
            .index
            .by_qualified
            .get(&key)
            .cloned()
            .unwrap_or_default();
        if candidates.is_empty() {
            if let Some(state) = self.index.files.get(&type_symbol.path) {
                candidates = state
                    .symbols
                    .iter()
                    .filter(|symbol| {
                        symbol.parent == Some(type_symbol.local_id) && symbol.name == name
                    })
                    .cloned()
                    .collect();
            }
        }
        candidates.retain(|symbol| want.accepts(symbol));
        if !candidates.is_empty() {
            return choose(name, candidates);
        }
        for supertype in self.supertypes(type_symbol) {
            let resolution = self.resolve_member_inner(&supertype, name, want, visited);
            if !matches!(
                resolution,
                Resolution::Unresolved { .. } | Resolution::Ambiguous { .. }
            ) {
                return resolution;
            }
        }
        Resolution::Unresolved {
            name: name.to_string(),
        }
    }

    pub fn constructor_for(&self, type_symbol: &SymbolRef) -> Resolution {
        let Some(state) = self.index.files.get(&type_symbol.path) else {
            return Resolution::Internal(type_symbol.clone());
        };
        let candidates: Vec<SymbolRef> = state
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.parent == Some(type_symbol.local_id) && symbol.kind == NodeKind::Constructor
            })
            .cloned()
            .collect();
        if candidates.is_empty() {
            Resolution::Internal(type_symbol.clone())
        } else {
            choose(&type_symbol.name, candidates)
        }
    }

    pub fn supertypes(&self, type_symbol: &SymbolRef) -> Vec<SymbolRef> {
        let Some(state) = self.index.files.get(&type_symbol.path) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for heritage in &state.parsed.heritage {
            if heritage.from != type_symbol.local_id {
                continue;
            }
            if let Resolution::Internal(symbol) =
                self.resolve_type_name(state, Some(type_symbol.local_id), &heritage.name)
            {
                if symbol.is_type() && symbol.node_id != type_symbol.node_id {
                    out.push(symbol);
                }
            }
        }
        out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        out.dedup_by(|a, b| a.node_id == b.node_id);
        out
    }

    pub fn local_type(
        &self,
        state: &FileState,
        scope: Option<u32>,
        name: &str,
        at_byte: u32,
    ) -> Option<String> {
        let mut scopes = Vec::new();
        if let Some(scope) = scope {
            scopes.push(Some(scope));
            let mut parent = state.symbol(scope).and_then(|symbol| symbol.parent);
            while let Some(local) = parent {
                scopes.push(Some(local));
                parent = state.symbol(local).and_then(|symbol| symbol.parent);
            }
        }
        scopes.push(None);
        for scope in scopes {
            let mut best: Option<(&trace_parser::LocalBinding, u32)> = None;
            for binding in &state.parsed.locals {
                if binding.scope != scope || binding.name != name {
                    continue;
                }
                if binding.range.start.byte > at_byte {
                    continue;
                }
                let rank = binding.range.start.byte;
                if best.is_none_or(|(_, current)| rank >= current) {
                    best = Some((binding, rank));
                }
            }
            if let Some((binding, _)) = best {
                return Some(binding.type_name.clone());
            }
        }
        None
    }

    pub(crate) fn lexical_candidates(
        &self,
        state: &FileState,
        scope: Option<u32>,
        name: &str,
        want: Want,
    ) -> Vec<SymbolRef> {
        let mut candidates = Vec::new();
        if let Some(indexes) = state.top_level.get(name) {
            candidates.extend(
                indexes
                    .iter()
                    .map(|index| state.symbols[*index].clone())
                    .filter(|symbol| want.accepts(symbol)),
            );
        }
        if let Some(scope) = scope {
            let mut parent = state.symbol(scope).and_then(|symbol| symbol.parent);
            while let Some(local) = parent {
                let Some(container) = state.symbol(local) else {
                    break;
                };
                candidates.extend(
                    state
                        .symbols
                        .iter()
                        .filter(|symbol| {
                            symbol.parent == Some(local)
                                && symbol.name == name
                                && want.accepts(symbol)
                                && !matches!(
                                    container.kind,
                                    NodeKind::Function | NodeKind::Method | NodeKind::Constructor
                                )
                        })
                        .cloned(),
                );
                parent = container.parent;
            }
        }
        candidates.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        candidates.dedup_by(|a, b| a.node_id == b.node_id);
        candidates
    }
}

use super::resolve_resolution_ext::choose;
