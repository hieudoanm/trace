pub mod edges;
pub mod finish;
pub mod locals;
pub mod symbols;

use std::collections::HashMap;

use tree_sitter::Node as TsNode;

use trace_core::{Language, NodeKind};

use crate::syntax::{CallData, HeritageData, ImportData, SymbolData, TypeRefData};

pub struct SyntaxBuilder<'a> {
    pub(crate) source: &'a str,
    pub(crate) language: Language,
    pub(crate) path: &'a str,
    pub(crate) module_name: Option<String>,
    pub(crate) symbols: Vec<SymbolData>,
    pub(crate) imports: Vec<ImportData>,
    pub(crate) calls: Vec<CallData>,
    pub(crate) type_refs: Vec<TypeRefData>,
    pub(crate) heritage: Vec<HeritageData>,
    pub(crate) locals: Vec<crate::syntax::LocalBinding>,
    stack: Vec<u32>,
    qualifier_prefix: Option<String>,
    ordinals: HashMap<(NodeKind, String), u32>,
    next_local: u32,
}

impl<'a> SyntaxBuilder<'a> {
    pub(crate) fn new(source: &'a str, language: Language, path: &'a str) -> Self {
        Self {
            source,
            language,
            path,
            module_name: None,
            symbols: Vec::new(),
            imports: Vec::new(),
            calls: Vec::new(),
            type_refs: Vec::new(),
            heritage: Vec::new(),
            locals: Vec::new(),
            stack: Vec::new(),
            qualifier_prefix: None,
            ordinals: HashMap::new(),
            next_local: 0,
        }
    }

    pub(crate) fn text(&self, node: TsNode<'_>) -> &'a str {
        super::tree_sitter::text(self.source, node)
    }

    pub(crate) fn caller(&self) -> Option<u32> {
        self.stack.last().copied()
    }

    pub(crate) fn current_symbol(&self) -> Option<&SymbolData> {
        self.stack
            .last()
            .and_then(|id| self.symbols.iter().find(|s| s.local_id == *id))
    }

    pub(crate) fn parent_local(&self) -> Option<u32> {
        self.stack.last().copied()
    }

    pub(crate) fn in_type_scope(&self) -> bool {
        self.qualifier_prefix.is_some()
            || self.current_symbol().is_some_and(|symbol| {
                matches!(
                    symbol.kind,
                    NodeKind::Class
                        | NodeKind::Struct
                        | NodeKind::Enum
                        | NodeKind::Interface
                        | NodeKind::Trait
                )
            })
    }

    pub(crate) fn set_qualifier(&mut self, qualifier: Option<String>) -> Option<String> {
        std::mem::replace(&mut self.qualifier_prefix, qualifier)
    }

    #[allow(dead_code)]
    pub(crate) fn find_symbol_by_id(&self, local_id: u32) -> Option<&SymbolData> {
        self.symbols
            .iter()
            .find(|symbol| symbol.local_id == local_id)
    }

    pub(crate) fn push(&mut self, local_id: u32) {
        self.stack.push(local_id);
    }

    pub(crate) fn pop(&mut self) {
        self.stack.pop();
    }
}
