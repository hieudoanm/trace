use std::collections::BTreeMap;

use crate::edge::EdgeKind;
use crate::graph::lookup::{find_by_name, find_by_qualified_name, node};
use crate::graph::storage::Graph;
use crate::node::Node;

pub fn resolve_symbols<'a>(graph: &'a Graph, selector: &str) -> Vec<&'a Node> {
    if let Some(n) = node(graph, selector) {
        return vec![n];
    }
    let mut out: Vec<&Node> = find_by_qualified_name(graph, selector);
    if out.is_empty() {
        out = find_by_name(graph, selector);
    }
    if out.is_empty() {
        let needle = selector.replace("::", ".");
        out = graph
            .nodes()
            .iter()
            .filter(|n| {
                let qualified = n.qualified_name.as_deref().unwrap_or("");
                let normalized = qualified.replace("::", ".");
                let id_tail =
                    n.id.split_once(':')
                        .map(|(_, rest)| rest.replace("::", "."))
                        .unwrap_or_default();
                normalized == needle
                    || normalized.ends_with(&format!(".{needle}"))
                    || id_tail == needle
                    || id_tail.ends_with(&format!(".{needle}"))
            })
            .collect();
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out.dedup_by(|a, b| a.id == b.id);
    out
}

pub fn edge_kind_counts(graph: &Graph) -> BTreeMap<EdgeKind, usize> {
    let mut counts = BTreeMap::new();
    for edge in graph.edges() {
        *counts.entry(edge.kind).or_insert(0) += 1;
    }
    counts
}
