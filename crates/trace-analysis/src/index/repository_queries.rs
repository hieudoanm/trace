use trace_core::Language;

use crate::index::repository::RepositoryIndex;
use crate::index::types::{FileState, SymbolRef};

impl RepositoryIndex {
    pub fn symbols_in_group(&self, group: &str, name: &str, type_only: bool) -> Vec<SymbolRef> {
        let mut out = Vec::new();
        for path in self.by_group.get(group).into_iter().flatten() {
            let Some(state) = self.files.get(path) else {
                continue;
            };
            if let Some(indexes) = state.top_level.get(name) {
                for index in indexes {
                    let symbol = &state.symbols[*index];
                    if !type_only || symbol.is_type() {
                        out.push(symbol.clone());
                    }
                }
            }
        }
        out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        out.dedup_by(|a, b| a.node_id == b.node_id);
        out
    }

    pub fn symbols_in_package(&self, package: &str, name: &str) -> Vec<SymbolRef> {
        let Some(package) = self.packages.get(package) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for path in &package.files {
            let Some(state) = self.files.get(path) else {
                continue;
            };
            if let Some(indexes) = state.top_level.get(name) {
                for index in indexes {
                    let symbol = &state.symbols[*index];
                    if symbol.exported || state.language != Language::Go {
                        out.push(symbol.clone());
                    }
                }
            }
        }
        out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        out.dedup_by(|a, b| a.node_id == b.node_id);
        out
    }

    pub fn parent_type_of(&self, state: &FileState, local_id: u32) -> Option<SymbolRef> {
        let mut cursor = state.symbol(local_id).and_then(|symbol| symbol.parent);
        while let Some(parent) = cursor {
            let symbol = state.symbol(parent)?;
            if symbol.is_type() {
                return Some(symbol.clone());
            }
            cursor = symbol.parent;
        }
        // Swift extensions declare members without a local parent. Derive the
        // type from the qualified prefix when a declaration exists elsewhere.
        let symbol = state.symbol(local_id)?;
        let prefix = symbol.qualified.rsplit_once('.')?.0;
        self.resolve_type_prefix(&symbol.path, prefix)
    }

    pub fn resolve_type_prefix(&self, path: &str, prefix: &str) -> Option<SymbolRef> {
        let state = self.files.get(path)?;
        if let Some(symbol) = state
            .symbols
            .iter()
            .find(|symbol| symbol.qualified == prefix && symbol.is_type())
        {
            return Some(symbol.clone());
        }
        let candidates = self.by_qualified.get(prefix)?;
        let types: Vec<SymbolRef> = candidates
            .iter()
            .filter(|symbol| symbol.is_type())
            .cloned()
            .collect();
        if types.len() == 1 {
            types.into_iter().next()
        } else {
            None
        }
    }

    /// Resolve the structural parent node used by graph containment. Extension
    /// members attach to a type declared in another file when it can be found
    /// unambiguously; otherwise they remain module-level declarations.
    pub fn structural_parent(&self, state: &FileState, symbol: &SymbolRef) -> String {
        if let Some(parent) = symbol
            .parent
            .and_then(|local| state.symbol(local))
            .map(|parent| parent.node_id.clone())
        {
            return parent;
        }
        if let Some(prefix) = symbol.qualified.rsplit_once('.').map(|(prefix, _)| prefix) {
            if let Some(parent) = self.resolve_type_prefix(&symbol.path, prefix) {
                if parent.node_id != symbol.node_id {
                    return parent.node_id;
                }
            }
        }
        state.module_id.clone()
    }
}
