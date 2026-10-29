//! Tree-sitter based syntax extraction.
//!
//! Parsing produces a language-neutral [`syntax::ParsedFile`]. Resolution and
//! graph construction live in `trace-analysis`; this crate only reports what
//! the syntax tree actually contains.

pub mod adapters;
pub mod discovery;
pub mod language;
pub mod parser;
pub mod syntax;
pub mod util;

pub use discovery::{DiscoveredFile, DiscoveryOptions, DiscoveryReport, SkippedFile};
pub use language::{grammar_version, tree_sitter_language};
pub use parser::{read_source, ParseError, SyntaxParser};
pub use syntax::{
    CallData, CallKind, HeritageData, HeritageRelation, ImportData, ImportItem, ImportKind,
    LocalBinding, ParsedFile, SymbolData, TypeRefData,
};
