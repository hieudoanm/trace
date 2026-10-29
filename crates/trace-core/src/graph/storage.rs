use std::collections::{HashMap, HashSet};

use crate::diagnostic::Diagnostic;
use crate::edge::{Edge, EdgeKind, ResolutionStatus};
use crate::error::{CoreError, Result};
use crate::id;
use crate::node::{Node, NodeKind};

use crate::graph::filter::NodeFilter;
use crate::graph::stats::Stats;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EdgeKey {
    source: String,
    kind: EdgeKind,
    target: String,
    status: ResolutionStatus,
    origin: crate::edge::ResolutionOrigin,
}

impl EdgeKey {
    fn from_edge(edge: &Edge) -> Self {
        Self {
            source: edge.source.clone(),
            kind: edge.kind,
            target: edge.target.clone(),
            status: edge.status,
            origin: edge.origin,
        }
    }
}

/// In-memory directed attributed graph with lookup indexes.
///
/// Nodes and edges are stored in insertion order while a graph is being built.
/// [`Graph::finalize`] sorts them by canonical keys and rebuilds indexes so
/// identical inputs always produce byte-identical exports. Lookup by ID, name,
/// qualified name and kind is indexed rather than linear.
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
    finalized: bool,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

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

    fn index_node(&mut self, idx: usize) {
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

    /// Add an edge, aggregating call sites when all key fields match.
    pub fn add_edge(&mut self, mut edge: Edge) -> Result<()> {
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

    /// Sort canonical data and rebuild all indexes. Safe to call repeatedly.
    pub fn finalize(&mut self) {
        self.nodes.sort_by(|a, b| a.id.cmp(&b.id));
        self.edges.sort_by(|a, b| {
            a.source
                .cmp(&b.source)
                .then(a.kind.cmp(&b.kind))
                .then(a.target.cmp(&b.target))
                .then(a.status.cmp(&b.status))
                .then(a.origin.cmp(&b.origin))
        });
        for node in &mut self.nodes {
            if let Some(range) = &node.range {
                if !range.is_valid() {
                    node.range = None;
                }
            }
        }
        for edge in &mut self.edges {
            edge.sites.sort();
            edge.sites.dedup();
        }
        self.rebuild_indexes();
        self.finalized = true;
    }

    fn rebuild_indexes(&mut self) {
        self.node_index.clear();
        self.name_index.clear();
        self.qualified_index.clear();
        self.kind_index.clear();
        self.edge_index.clear();
        self.out_index.clear();
        self.in_index.clear();
        for idx in 0..self.nodes.len() {
            self.index_node(idx);
        }
        for (idx, edge) in self.edges.iter().enumerate() {
            self.edge_index.insert(EdgeKey::from_edge(edge), idx);
            self.out_index
                .entry(edge.source.clone())
                .or_default()
                .push(idx);
            self.in_index
                .entry(edge.target.clone())
                .or_default()
                .push(idx);
        }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn edges(&self) -> &[Edge] {
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
    pub fn filtered(&self, filter: &NodeFilter) -> (Vec<Node>, Vec<Edge>) {
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

    pub fn stats(&self, diagnostics: &[Diagnostic]) -> Stats {
        Stats::compute(self, diagnostics)
    }

    pub fn validate(&self) -> Result<()> {
        let issues = crate::graph::validation::validation_issues(self);
        if issues.is_empty() {
            Ok(())
        } else {
            Err(CoreError::InvalidGraph(issues.join("; ")))
        }
    }

    /// All node IDs, useful for deterministic iteration.
    pub fn node_ids(&self) -> Vec<String> {
        self.nodes.iter().map(|n| n.id.clone()).collect()
    }

    pub fn node(&self, node_id: &str) -> Option<&Node> {
        self.node_index.get(node_id).map(|&idx| &self.nodes[idx])
    }

    pub fn out_edges(&self, node_id: &str) -> Vec<&Edge> {
        self.out_index
            .get(node_id)
            .map(|indices| indices.iter().map(|&idx| &self.edges[idx]).collect())
            .unwrap_or_default()
    }

    pub fn in_edges(&self, node_id: &str) -> Vec<&Edge> {
        self.in_index
            .get(node_id)
            .map(|indices| indices.iter().map(|&idx| &self.edges[idx]).collect())
            .unwrap_or_default()
    }

    pub fn resolve_symbols(&self, selector: &str) -> Vec<&Node> {
        crate::graph::resolve::resolve_symbols(self, selector)
    }
}

/// Build a repository node with a fixed canonical ID.
pub fn repository_node(name: &str) -> Node {
    Node::new(id::repository_id(), NodeKind::Repository, name)
}
