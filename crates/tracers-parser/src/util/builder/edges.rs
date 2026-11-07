use tree_sitter::Node as TsNode;

use crate::syntax::{CallData, CallKind, HeritageData, HeritageRelation, ImportData, TypeRefData};

use super::SyntaxBuilder;

impl<'a> SyntaxBuilder<'a> {
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
            range: super::super::tree_sitter::range(node),
        });
    }

    pub(crate) fn add_type_ref(&mut self, name: impl Into<String>, node: TsNode<'_>) {
        let name = name.into();
        if name.is_empty() || super::super::builtins::is_builtin_type(&name) {
            return;
        }
        self.type_refs.push(TypeRefData {
            from: self.caller(),
            name,
            range: super::super::tree_sitter::range(node),
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
            range: super::super::tree_sitter::range(node),
        });
    }
}
