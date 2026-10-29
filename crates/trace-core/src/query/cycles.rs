use crate::edge::EdgeKind;
use crate::node::Node;

use crate::query::constants::DEPENDENCY_EDGES;
use crate::query::traversal::Query;

impl<'a> Query<'a> {
    /// Strongly connected components over dependency edges.
    ///
    /// Returns components sorted deterministically; single-node components are
    /// included only when a self-loop exists.
    pub fn cycles(&self, edge_kinds: &[EdgeKind]) -> Vec<Vec<Node>> {
        use petgraph::algo::tarjan_scc;
        use petgraph::graphmap::DiGraphMap;

        let kinds = if edge_kinds.is_empty() {
            DEPENDENCY_EDGES
        } else {
            edge_kinds
        };
        let mut graph: DiGraphMap<usize, ()> = DiGraphMap::new();
        let ids: Vec<&str> = self.graph.nodes().iter().map(|n| n.id.as_str()).collect();
        let index: std::collections::HashMap<&str, usize> =
            ids.iter().enumerate().map(|(idx, id)| (*id, idx)).collect();
        for node in self.graph.nodes() {
            graph.add_node(index[node.id.as_str()]);
        }
        for edge in self.graph.edges() {
            if !kinds.contains(&edge.kind) {
                continue;
            }
            let (Some(&from), Some(&to)) = (
                index.get(edge.source.as_str()),
                index.get(edge.target.as_str()),
            ) else {
                continue;
            };
            graph.add_edge(from, to, ());
        }
        let mut components: Vec<Vec<Node>> = Vec::new();
        for component in tarjan_scc(&graph) {
            let keep = component.len() > 1
                || component
                    .iter()
                    .any(|node| graph.contains_edge(*node, *node));
            if !keep {
                continue;
            }
            let mut nodes: Vec<Node> = component
                .iter()
                .filter_map(|idx| self.graph.nodes().get(*idx).cloned())
                .collect();
            nodes.sort_by(|a, b| a.id.cmp(&b.id));
            components.push(nodes);
        }
        components.sort_by(|a, b| {
            let a_key = a.first().map(|n| n.id.as_str()).unwrap_or("");
            let b_key = b.first().map(|n| n.id.as_str()).unwrap_or("");
            a_key.cmp(b_key).then(a.len().cmp(&b.len()))
        });
        components
    }
}
