use tree_sitter::Node as TsNode;

use trace_core::{Position, SourceRange};

/// Convert a tree-sitter range to Trace's source range.
pub fn range(node: TsNode<'_>) -> SourceRange {
    let start = node.start_position();
    let end = node.end_position();
    SourceRange::new(
        Position::new(
            start.row as u32 + 1,
            start.column as u32 + 1,
            node.start_byte() as u32,
        ),
        Position::new(
            end.row as u32 + 1,
            end.column as u32 + 1,
            node.end_byte() as u32,
        ),
    )
}

/// Text for a node, tolerating malformed trees.
pub fn text<'a>(source: &'a str, node: TsNode<'_>) -> &'a str {
    source.get(node.byte_range()).unwrap_or_default().trim()
}

pub fn child_field<'a>(node: TsNode<'a>, name: &str) -> Option<TsNode<'a>> {
    node.child_by_field_name(name)
}

pub fn named_children<'a>(node: TsNode<'a>) -> Vec<TsNode<'a>> {
    let mut cursor = node.walk();
    let mut out = Vec::new();
    if cursor.goto_first_child() {
        loop {
            if cursor.node().is_named() {
                out.push(cursor.node());
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    out
}

pub fn children<'a>(node: TsNode<'a>) -> Vec<TsNode<'a>> {
    let mut cursor = node.walk();
    let mut out = Vec::new();
    if cursor.goto_first_child() {
        loop {
            out.push(cursor.node());
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    out
}

pub fn first_child_of_kind<'a>(node: TsNode<'a>, kind: &str) -> Option<TsNode<'a>> {
    named_children(node)
        .into_iter()
        .find(|child| child.kind() == kind)
}

pub fn first_named<'a>(node: TsNode<'a>) -> Option<TsNode<'a>> {
    named_children(node).into_iter().next()
}

/// Find the first descendant (including self) with a given node kind.
pub fn find_descendant<'a>(node: TsNode<'a>, kind: &str) -> Option<TsNode<'a>> {
    if node.kind() == kind {
        return Some(node);
    }
    for child in named_children(node) {
        if let Some(found) = find_descendant(child, kind) {
            return Some(found);
        }
    }
    None
}

/// Visit every node depth-first. The callback returns `false` to stop the walk.
pub fn walk<'a>(node: TsNode<'a>, visit: &mut impl FnMut(TsNode<'a>) -> bool) -> bool {
    if !visit(node) {
        return false;
    }
    for child in named_children(node) {
        if !walk(child, visit) {
            return false;
        }
    }
    true
}
