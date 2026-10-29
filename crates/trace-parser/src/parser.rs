use std::path::Path;

use thiserror::Error;
use tree_sitter::{Parser, Tree};

use trace_core::{Diagnostic, DiagnosticKind, Language, SourceRange};

use crate::adapters;
use crate::language::tree_sitter_language;
use crate::syntax::ParsedFile;
use crate::util::range;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("parser setup failed: {0}")]
    Setup(String),
    #[error("tree-sitter returned no syntax tree")]
    NoTree,
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("cannot read file: {0}")]
    Io(String),
    #[error("file exceeds the {0} byte size limit")]
    TooLarge(u64),
    #[error("file appears to be binary")]
    Binary,
    #[error("file is not valid UTF-8")]
    InvalidUtf8,
}

/// One reusable tree-sitter parser. `Parser` is not `Sync`, so analysis
/// creates one per worker thread.
pub struct SyntaxParser {
    parser: Parser,
}

impl SyntaxParser {
    pub fn new() -> Result<Self, ParseError> {
        let parser = Parser::new();
        Ok(Self { parser })
    }

    pub fn parse(
        &mut self,
        path: &str,
        language: Language,
        source: &str,
    ) -> Result<ParsedFile, ParseError> {
        let grammar = tree_sitter_language(language);
        self.parser
            .set_language(&grammar)
            .map_err(|error| ParseError::Setup(error.to_string()))?;
        let tree = self.parser.parse(source, None).ok_or(ParseError::NoTree)?;
        let mut parsed = adapters::parse(language, path, source, &tree);
        collect_parse_diagnostics(&mut parsed, &tree);
        Ok(parsed)
    }
}

fn collect_parse_diagnostics(parsed: &mut ParsedFile, tree: &Tree) {
    let mut errors: Vec<SourceRange> = Vec::new();
    let mut missing: Vec<SourceRange> = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.is_error() {
            errors.push(range(node));
        } else if node.is_missing() {
            missing.push(range(node));
        }
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                // Finished walking the whole tree.
                let count = errors.len() + missing.len();
                parsed.parse_error_count = count;
                let max_reported = 20;
                for site in errors.iter().take(max_reported) {
                    parsed.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticKind::ParseError,
                            "syntax error; file was partially analysed",
                        )
                        .with_file(parsed.path.clone())
                        .with_language(parsed.language)
                        .with_range(*site),
                    );
                }
                let remaining = max_reported.saturating_sub(errors.len());
                for site in missing.iter().take(remaining) {
                    parsed.diagnostics.push(
                        Diagnostic::warning(
                            DiagnosticKind::UnsupportedSyntax,
                            "parser expected a missing syntax element",
                        )
                        .with_file(parsed.path.clone())
                        .with_language(parsed.language)
                        .with_range(*site),
                    );
                }
                if count > max_reported {
                    parsed.diagnostics.push(
                        Diagnostic::warning(
                            DiagnosticKind::PartialAnalysis,
                            format!(
                                "{} parse problems in total; only {max_reported} locations reported",
                                count
                            ),
                        )
                        .with_file(parsed.path.clone())
                        .with_language(parsed.language),
                    );
                }
                return;
            }
        }
    }
}

/// Read a source file defensively. This never executes or interprets content.
pub fn read_source(path: &Path, max_bytes: u64) -> Result<String, ReadError> {
    let metadata = std::fs::metadata(path).map_err(|error| ReadError::Io(error.to_string()))?;
    if metadata.len() > max_bytes {
        return Err(ReadError::TooLarge(max_bytes));
    }
    let bytes = std::fs::read(path).map_err(|error| ReadError::Io(error.to_string()))?;
    if bytes.iter().take(8192).any(|byte| *byte == 0) {
        return Err(ReadError::Binary);
    }
    String::from_utf8(bytes).map_err(|_| ReadError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_typescript_file_and_reports_errors() {
        let mut parser = SyntaxParser::new().unwrap();
        let parsed = parser
            .parse(
                "a.ts",
                Language::TypeScript,
                "export function a() { return 1; }",
            )
            .unwrap();
        assert_eq!(parsed.language, Language::TypeScript);
        assert_eq!(parsed.parse_error_count, 0);
        assert_eq!(parsed.symbols.len(), 1);
        assert_eq!(parsed.symbols[0].name, "a");

        let broken = parser
            .parse("broken.ts", Language::TypeScript, "export function a( {")
            .unwrap();
        assert!(broken.parse_error_count > 0);
        assert!(!broken.diagnostics.is_empty());
    }

    #[test]
    fn rejects_binary_and_oversized_files() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("a.bin");
        std::fs::write(&bin, [0u8, 1, 2]).unwrap();
        assert!(matches!(read_source(&bin, 1024), Err(ReadError::Binary)));

        let big = dir.path().join("big.ts");
        std::fs::write(&big, "a".repeat(64)).unwrap();
        assert!(matches!(read_source(&big, 16), Err(ReadError::TooLarge(_))));
    }
}
