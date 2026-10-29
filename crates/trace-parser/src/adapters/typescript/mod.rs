pub mod calls;
pub mod class;
pub mod function;
pub mod helpers;
pub mod heritage;
pub mod imports;
pub mod items;
pub mod types;
pub mod variables;
pub mod visibility;

use tree_sitter::Tree;

use trace_core::Language;

use crate::syntax::ParsedFile;
use crate::util::{named_children, SyntaxBuilder};

use items::parse_item;

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child, false);
    }
    builder.finish()
}
