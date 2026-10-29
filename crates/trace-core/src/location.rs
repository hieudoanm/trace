use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A position in a source file.
///
/// `line` and `column` are 1-based. Columns count UTF-8 bytes from the start
/// of the line, matching tree-sitter's byte-oriented points. `byte` is the
/// 0-based byte offset from the start of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub byte: u32,
}

impl Position {
    pub fn new(line: u32, column: u32, byte: u32) -> Self {
        Self { line, column, byte }
    }
}

/// A half-open source range: `start` is inclusive, `end` exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRange {
    pub start: Position,
    pub end: Position,
}

impl SourceRange {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, other: &SourceRange) -> bool {
        self.start.byte <= other.start.byte && other.end.byte <= self.end.byte
    }

    pub fn is_valid(&self) -> bool {
        self.start.byte <= self.end.byte && self.start.line >= 1 && self.end.line >= self.start.line
    }

    pub fn ordering_key(&self) -> (u32, u32, u32) {
        (self.start.byte, self.end.byte, self.start.line)
    }
}

impl PartialOrd for SourceRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceRange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ordering_key().cmp(&other.ordering_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_contains() {
        let inner = SourceRange::new(Position::new(2, 1, 10), Position::new(2, 5, 14));
        let outer = SourceRange::new(Position::new(1, 1, 0), Position::new(3, 1, 20));
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn ranges_sort_by_byte_offset() {
        let a = SourceRange::new(Position::new(3, 1, 30), Position::new(3, 2, 31));
        let b = SourceRange::new(Position::new(1, 1, 1), Position::new(1, 2, 2));
        assert!(b < a);
    }
}
