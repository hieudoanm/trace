use std::collections::HashMap;

use tree_sitter::Node as TsNode;

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{
    CallData, CallKind, HeritageData, HeritageRelation, ImportData, ParsedFile, SymbolData,
    TypeRefData,
};

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

    pub(crate) fn add_symbol(
        &mut self,
        kind: NodeKind,
        name: impl Into<String>,
        node: TsNode<'_>,
        visibility: Option<Visibility>,
        exported: bool,
    ) -> u32 {
        let name = name.into();
        let parent = self.stack.last().copied();
        let qualified = match self.current_symbol() {
            Some(parent_symbol) => format!("{}.{}", parent_symbol.qualified_name, name),
            None => match &self.qualifier_prefix {
                Some(prefix) => format!("{prefix}.{name}"),
                None => name.clone(),
            },
        };
        self.add_symbol_qualified(kind, name, qualified, parent, node, visibility, exported)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_symbol_qualified(
        &mut self,
        kind: NodeKind,
        name: impl Into<String>,
        qualified: impl Into<String>,
        parent: Option<u32>,
        node: TsNode<'_>,
        visibility: Option<Visibility>,
        exported: bool,
    ) -> u32 {
        let name = name.into();
        let qualified = qualified.into();
        let key = (kind, qualified.clone());
        let ordinal = *self.ordinals.entry(key).or_insert(0);
        self.ordinals.insert((kind, qualified.clone()), ordinal + 1);
        let local_id = self.next_local;
        self.next_local += 1;
        self.symbols.push(SymbolData {
            local_id,
            kind,
            name,
            qualified_name: qualified,
            parent,
            ordinal,
            range: super::tree_sitter::range(node),
            signature: super::strings::signature(self.source, node),
            visibility,
            exported,
            decorators: Vec::new(),
            overrides: false,
        });
        local_id
    }

    pub(crate) fn mark_override(&mut self, local_id: u32) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.local_id == local_id)
        {
            symbol.overrides = true;
        }
    }

    pub(crate) fn add_decorators(&mut self, local_id: u32, decorators: Vec<String>) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.local_id == local_id)
        {
            symbol.decorators = decorators;
        }
    }

    pub(crate) fn add_import(&mut self, import: ImportData) {
        self.imports.push(import);
    }

    pub(crate) fn add_call(
        &mut self,
        receiver: Option<String>,
        name: impl Into<String>,
        full: impl Into<String>,
        kind: CallKind,
        node: TsNode<'_>,
    ) {
        self.calls.push(CallData {
            caller: self.caller(),
            receiver,
            name: name.into(),
            full: full.into(),
            kind,
            range: super::tree_sitter::range(node),
        });
    }

    pub(crate) fn add_type_ref(&mut self, name: impl Into<String>, node: TsNode<'_>) {
        let name = name.into();
        if name.is_empty() || super::builtins::is_builtin_type(&name) {
            return;
        }
        self.type_refs.push(TypeRefData {
            from: self.caller(),
            name,
            range: super::tree_sitter::range(node),
        });
    }

    pub(crate) fn add_heritage(
        &mut self,
        from: u32,
        name: impl Into<String>,
        relation: HeritageRelation,
        node: TsNode<'_>,
    ) {
        self.heritage.push(HeritageData {
            from,
            name: name.into(),
            relation,
            range: super::tree_sitter::range(node),
        });
    }

    pub(crate) fn add_local(
        &mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
        node: TsNode<'_>,
    ) {
        let name = name.into();
        let type_name = type_name.into();
        if name.is_empty() || type_name.is_empty() {
            return;
        }
        self.locals.push(crate::syntax::LocalBinding {
            scope: self.caller(),
            name,
            type_name,
            range: super::tree_sitter::range(node),
        });
    }

    pub(crate) fn find_symbol_by_qualified(&self, qualified: &str) -> Option<u32> {
        self.symbols
            .iter()
            .find(|symbol| symbol.qualified_name == qualified)
            .map(|symbol| symbol.local_id)
    }

    #[allow(dead_code)]
    pub(crate) fn find_symbol_by_name(&self, name: &str) -> Option<u32> {
        self.symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .map(|symbol| symbol.local_id)
    }

    pub(crate) fn finish(self) -> ParsedFile {
        ParsedFile {
            path: self.path.to_string(),
            language: self.language,
            source_hash: blake3::hash(self.source.as_bytes()).to_hex().to_string(),
            module_name: self.module_name,
            symbols: self.symbols,
            imports: self.imports,
            calls: self.calls,
            type_refs: self.type_refs,
            heritage: self.heritage,
            locals: self.locals,
            diagnostics: Vec::new(),
            parse_error_count: 0,
        }
    }
}
