use crate::error::{CoreError, Result};
use crate::node::Node;

use super::edge_key::EdgeKey;
use super::graph_struct::Graph;

impl Graph {
    /// Add a node. Identical re-insertions are ignored; conflicting contents
    /// for the same canonical ID are rejected.
    pub fn add_node(&mut self, node: Node) -> Result<bool> {
        if let Some(&existing) = self.node_index.get(&node.id) {
            if self.nodes[existing] == node {
                return Ok(false);
            }
            return Err(CoreError::ConflictingNode(node.id));
        }
        let idx = self.nodes.len();
        self.nodes.push(node);
        self.index_node(idx);
        self.finalized = false;
        Ok(true)
    }

    /// Add an edge, aggregating call sites when all key fields match.
    pub fn add_edge(&mut self, mut edge: crate::edge::Edge) -> Result<()> {
        edge.sites.sort();
        edge.sites.dedup();
        let key = EdgeKey::from_edge(&edge);
        if let Some(&idx) = self.edge_index.get(&key) {
            let existing = &mut self.edges[idx];
            existing.sites.extend(edge.sites);
            existing.sites.sort();
            existing.sites.dedup();
            return Ok(());
        }
        let idx = self.edges.len();
        self.edge_index.insert(key, idx);
        self.out_index
            .entry(edge.source.clone())
            .or_default()
            .push(idx);
        self.in_index
            .entry(edge.target.clone())
            .or_default()
            .push(idx);
        self.edges.push(edge);
        self.finalized = false;
        Ok(())
    }
}
