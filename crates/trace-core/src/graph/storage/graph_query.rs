use std::collections::HashSet;

use crate::node::Node;

use super::graph_struct::Graph;

impl Graph {
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn edges(&self) -> &[crate::edge::Edge] {
        &self.edges
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Produce a filtered view of nodes and edges. Edges are retained only
    /// when both endpoints pass the node filter.
    pub fn filtered(
        &self,
        filter: &crate::graph::filter::NodeFilter,
    ) -> (Vec<Node>, Vec<crate::edge::Edge>) {
        let kept: HashSet<&str> = self
            .nodes
            .iter()
            .filter(|node| filter.matches(node))
            .map(|node| node.id.as_str())
            .collect();
        let nodes = self
            .nodes
            .iter()
            .filter(|node| kept.contains(node.id.as_str()))
            .cloned()
            .collect();
        let edges = self
            .edges
            .iter()
            .filter(|edge| {
                kept.contains(edge.source.as_str()) && kept.contains(edge.target.as_str())
            })
            .cloned()
            .collect();
        (nodes, edges)
    }

    pub fn node_ids(&self) -> Vec<String> {
        self.nodes.iter().map(|n| n.id.clone()).collect()
    }

    pub fn node(&self, node_id: &str) -> Option<&Node> {
        self.node_index.get(node_id).map(|&idx| &self.nodes[idx])
    }

    pub fn out_edges(&self, node_id: &str) -> Vec<&crate::edge::Edge> {
        self.out_index
            .get(node_id)
            .map(|indices| indices.iter().map(|&idx| &self.edges[idx]).collect())
            .unwrap_or_default()
    }

    pub fn in_edges(&self, node_id: &str) -> Vec<&crate::edge::Edge> {
        self.in_index
            .get(node_id)
            .map(|indices| indices.iter().map(|&idx| &self.edges[idx]).collect())
            .unwrap_or_default()
    }
}
