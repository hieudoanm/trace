use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::syntax::SymbolData;

use super::SyntaxBuilder;

impl<'a> SyntaxBuilder<'a> {
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
            range: super::super::tree_sitter::range(node),
            signature: super::super::strings::signature(self.source, node),
            visibility,
            exported,
            decorators: Vec::new(),
            overrides: false,
        });
        local_id
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
}
