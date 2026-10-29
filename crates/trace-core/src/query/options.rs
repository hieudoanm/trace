use crate::edge::{EdgeKind, ResolutionStatus};

/// Options for bounded graph traversals.
#[derive(Debug, Clone)]
pub struct QueryOptions {
    /// Maximum number of edges followed from the start node.
    pub max_depth: usize,
    /// Maximum number of result nodes returned.
    pub max_results: usize,
    /// Edge kinds to follow. Empty means the operation's default set.
    pub edge_kinds: Vec<EdgeKind>,
    /// Resolution statuses to follow. Empty means all statuses.
    pub statuses: Vec<ResolutionStatus>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            max_depth: 32,
            max_results: 1000,
            edge_kinds: Vec::new(),
            statuses: Vec::new(),
        }
    }
}

impl QueryOptions {
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.max_results = limit;
        self
    }

    pub fn with_edges(mut self, kinds: &[EdgeKind]) -> Self {
        self.edge_kinds = kinds.to_vec();
        self
    }

    pub(crate) fn allows(&self, edge: &crate::edge::Edge, defaults: &[EdgeKind]) -> bool {
        let kinds = if self.edge_kinds.is_empty() {
            defaults
        } else {
            &self.edge_kinds
        };
        kinds.contains(&edge.kind)
            && (self.statuses.is_empty() || self.statuses.contains(&edge.status))
    }
}
