use crate::graph::Graph;
use crate::node::{Node, NodeKind};

/// Convenience: resolve a selector and report ambiguity.
pub fn resolve_unique<'a>(graph: &'a Graph, selector: &str) -> Result<&'a Node, String> {
    let mut matches = graph.resolve_symbols(selector);
    if matches.is_empty() {
        return Err(format!("no symbol matches `{selector}`"));
    }
    if matches.len() > 1 {
        let mut candidate_list: Vec<String> = matches
            .iter()
            .map(|node| format!("{} [{}]", node.id, node.kind))
            .collect();
        candidate_list.sort();
        candidate_list.truncate(10);
        return Err(format!(
            "`{selector}` is ambiguous ({} matches): {}",
            matches.len(),
            candidate_list.join(", ")
        ));
    }
    Ok(matches.remove(0))
}

/// Convenience for tests and CLI display.
pub fn callable_kinds() -> [NodeKind; 4] {
    [
        NodeKind::Function,
        NodeKind::Method,
        NodeKind::Constructor,
        NodeKind::Unknown,
    ]
}
