use crate::edge::{Edge, EdgeKind, ResolutionStatus};
use crate::node::Node;

/// A node reached during traversal together with the edge that led to it.
#[derive(Debug, Clone)]
pub struct Neighbor {
    pub node: Node,
    /// On multi-hop traversals this is the edge from the previous hop.
    pub edge: Edge,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct PathEdge {
    pub kind: EdgeKind,
    pub status: ResolutionStatus,
}

#[derive(Debug, Clone)]
pub struct PathResult {
    pub found: bool,
    pub nodes: Vec<Node>,
    pub edges: Vec<PathEdge>,
}

#[derive(Debug, Clone)]
pub struct ImpactNode {
    pub node: Node,
    /// Number of edges between the impacted node and the target.
    pub depth: usize,
    /// Shortest explanatory path from the impacted node to the target.
    pub path: Vec<Node>,
    pub via: Vec<EdgeKind>,
}
