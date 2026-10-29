use std::collections::BTreeMap;

use trace_core::NodeKind;
use trace_parser::{ParsedFile, SymbolData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolRef {
    pub node_id: String,
    pub path: String,
    pub local_id: u32,
    pub kind: NodeKind,
    pub name: String,
    pub qualified: String,
    pub parent: Option<u32>,
    pub exported: bool,
}

impl SymbolRef {
    pub fn is_type(&self) -> bool {
        crate::index::helpers::is_type_kind(self.kind)
    }

    pub fn is_callable(&self) -> bool {
        matches!(
            self.kind,
            NodeKind::Function | NodeKind::Method | NodeKind::Constructor
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleRef {
    File(String),
    Package(String),
    External(String),
}

#[derive(Debug, Clone)]
pub enum Binding {
    Symbol(SymbolRef),
    Module(ModuleRef),
    ExternalSymbol {
        node_id: String,
        name: String,
    },
    Unresolved {
        name: String,
    },
    Ambiguous {
        name: String,
        candidates: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct FileState {
    pub path: String,
    pub language: trace_core::Language,
    pub parsed: ParsedFile,
    pub file_id: String,
    pub module_id: String,
    pub module_name: String,
    pub module_qualified: String,
    pub package: Option<String>,
    pub group: Option<String>,
    pub crate_root: Option<String>,
    pub swift_root: Option<String>,
    pub symbols: Vec<SymbolRef>,
    /// File-local symbol index by parser `local_id`.
    pub local_index: BTreeMap<u32, usize>,
    /// Exported name to canonical node IDs. Re-exports are added after index
    /// construction.
    pub exports: BTreeMap<String, Vec<String>>,
    /// All top-level declarations, including non-exported ones.
    pub top_level: BTreeMap<String, Vec<usize>>,
    pub bindings: BTreeMap<String, Binding>,
}

impl FileState {
    pub fn symbol(&self, local_id: u32) -> Option<&SymbolRef> {
        self.local_index
            .get(&local_id)
            .and_then(|index| self.symbols.get(*index))
    }

    pub fn is_top_level(&self, symbol: &SymbolRef) -> bool {
        symbol.parent.is_none()
    }
}

pub fn symbol_ref(path: &str, symbol: &SymbolData) -> SymbolRef {
    SymbolRef {
        node_id: trace_core::symbol_id(symbol.kind, path, &symbol.qualified_name, symbol.ordinal),
        path: path.to_string(),
        local_id: symbol.local_id,
        kind: symbol.kind,
        name: symbol.name.clone(),
        qualified: symbol.qualified_name.clone(),
        parent: symbol.parent,
        exported: symbol.exported,
    }
}
