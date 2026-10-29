use crate::syntax::{ImportData, ImportItem, ImportKind, ParsedFile};
use crate::util::SyntaxBuilder;
use trace_core::Language;
use trace_core::{Position, SourceRange};
use tree_sitter::Tree;

pub fn parse(language: Language, path: &str, source: &str, _tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    let mut i = 0;
    while i < source.len() {
        if source[i..].starts_with("[[") {
            if let Some(end) = source[i..].find("]]") {
                let target = &source[i + 2..i + end];
                let range = SourceRange::new(
                    Position {
                        line: 1,
                        column: 1,
                        byte: i as u32,
                    },
                    Position {
                        line: 1,
                        column: 3,
                        byte: (i + 2) as u32,
                    },
                );
                builder.imports.push(ImportData {
                    module: target.to_string(),
                    alias: None,
                    items: vec![ImportItem {
                        name: target.to_string(),
                        alias: None,
                    }],
                    kind: ImportKind::Es,
                    range,
                    is_export: false,
                });
                i = i + end + 2;
                continue;
            }
        }
        if source[i..].starts_with('[') {
            if let Some(close) = source[i..].find("](") {
                let j = i + close + 2;
                if let Some(end) = source[j..].find(')') {
                    let target = &source[j..j + end];
                    let range = SourceRange::new(
                        Position {
                            line: 1,
                            column: 1,
                            byte: i as u32,
                        },
                        Position {
                            line: 1,
                            column: 2,
                            byte: (i + 1) as u32,
                        },
                    );
                    builder.imports.push(ImportData {
                        module: target.to_string(),
                        alias: None,
                        items: vec![ImportItem {
                            name: target.to_string(),
                            alias: None,
                        }],
                        kind: ImportKind::Es,
                        range,
                        is_export: false,
                    });
                    i = j + end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    builder.finish()
}
