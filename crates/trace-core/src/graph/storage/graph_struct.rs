use std::collections::HashMap;

use crate::edge::Edge;
use crate::node::{Node, NodeKind};

use super::edge_key::EdgeKey;

#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub(crate) nodes: Vec<Node>,
    pub(crate) edges: Vec<Edge>,
    pub(crate) node_index: HashMap<String, usize>,
    pub(crate) name_index: HashMap<String, Vec<String>>,
    pub(crate) qualified_index: HashMap<String, Vec<String>>,
    pub(crate) kind_index: HashMap<NodeKind, Vec<String>>,
    pub(crate) edge_index: HashMap<EdgeKey, usize>,
    pub(crate) out_index: HashMap<String, Vec<usize>>,
    pub(crate) in_index: HashMap<String, Vec<usize>>,
    pub(crate) finalized: bool,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn index_node(&mut self, idx: usize) {
        let node = &self.nodes[idx];
        self.node_index.insert(node.id.clone(), idx);
        self.name_index
            .entry(node.name.clone())
            .or_default()
            .push(node.id.clone());
        if let Some(qualified) = &node.qualified_name {
            self.qualified_index
                .entry(qualified.clone())
                .or_default()
                .push(node.id.clone());
        }
        self.kind_index
            .entry(node.kind)
            .or_default()
            .push(node.id.clone());
    }
}
