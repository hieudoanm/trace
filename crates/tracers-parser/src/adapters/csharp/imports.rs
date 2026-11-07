use tree_sitter::Node as TsNode;

use crate::syntax::{ImportData, ImportKind};
use crate::util::{range, SyntaxBuilder};

pub(crate) fn parse_using(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let text = builder.text(node);
    let name = text
        .trim_start_matches("using")
        .trim()
        .trim_end_matches(';')
        .trim()
        .to_string();
    builder.add_import(ImportData {
        kind: ImportKind::Rust,
        module: name,
        items: vec![],
        alias: None,
        is_export: false,
        range: range(node),
    });
}
