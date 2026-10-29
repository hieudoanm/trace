use tree_sitter::Node as TsNode;

use super::SyntaxBuilder;

impl<'a> SyntaxBuilder<'a> {
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
            range: super::super::tree_sitter::range(node),
        });
    }
}
