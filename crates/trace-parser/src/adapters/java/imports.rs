use tree_sitter::Node as TsNode;

use crate::syntax::{ImportData, ImportKind};
use crate::util::{range, SyntaxBuilder};

pub(crate) fn parse_import(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let text = builder.text(node);
    let name = text
        .trim_start_matches("import")
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_start_matches("static")
        .trim()
        .to_string();
    builder.add_import(ImportData {
        kind: ImportKind::Rust,
        module: name.clone(),
        alias: None,
        items: vec![],
        is_export: false,
        range: range(node),
    });
}
