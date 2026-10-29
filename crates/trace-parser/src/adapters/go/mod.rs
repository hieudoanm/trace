pub mod calls;
pub mod declarations;
pub mod imports;
pub mod items;
pub mod types;
pub mod visibility;

use tree_sitter::Tree;

use trace_core::Language;

use crate::syntax::ParsedFile;
use crate::util::{named_children, SyntaxBuilder};

use items::parse_item;

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    let root = tree.root_node();
    for child in named_children(root) {
        parse_item(&mut builder, child);
    }
    builder.finish()
}
