use crate::syntax::ParsedFile;
use crate::util::{named_children, SyntaxBuilder};
use trace_core::Language;
use tree_sitter::Tree;

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        let _ = child;
    }
    builder.finish()
}
