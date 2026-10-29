use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::diagnostic::Diagnostic;
use crate::edge::{Edge, EdgeKind, ResolutionStatus};
use crate::error::{CoreError, Result};
use crate::id;
use crate::language::Language;
use crate::node::{Node, NodeKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EdgeKey {
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
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_index: HashMap<String, usize>,
    name_index: HashMap<String, Vec<String>>,
    qualified_index: HashMap<String, Vec<String>>,
    kind_index: HashMap<NodeKind, Vec<String>>,
    edge_index: HashMap<EdgeKey, usize>,
    out_index: HashMap<String, Vec<usize>>,
    in_index: HashMap<String, Vec<usize>>,
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

    pub fn node(&self, node_id: &str) -> Option<&Node> {
        self.node_index.get(node_id).map(|&idx| &self.nodes[idx])
    }

    pub fn require_node(&self, node_id: &str) -> Result<&Node> {
        self.node(node_id)
            .ok_or_else(|| CoreError::UnknownNode(node_id.to_string()))
    }

    pub fn find_by_name(&self, name: &str) -> Vec<&Node> {
        self.name_index
            .get(name)
            .map(|ids| ids.iter().filter_map(|id| self.node(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_by_qualified_name(&self, qualified: &str) -> Vec<&Node> {
        self.qualified_index
            .get(qualified)
            .map(|ids| ids.iter().filter_map(|id| self.node(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_by_kind(&self, kind: NodeKind) -> Vec<&Node> {
        self.kind_index
            .get(&kind)
            .map(|ids| ids.iter().filter_map(|id| self.node(id)).collect())
            .unwrap_or_default()
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

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
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
        let mut by_node_kind = BTreeMap::new();
        let mut by_language = BTreeMap::new();
        let mut by_edge_kind = BTreeMap::new();
        let mut by_status = BTreeMap::new();
        let mut files = 0usize;
        let mut packages = 0usize;
        for node in &self.nodes {
            *by_node_kind
                .entry(node.kind.as_str().to_string())
                .or_insert(0) += 1;
            if node.kind == NodeKind::File {
                files += 1;
            }
            if node.kind == NodeKind::Package {
                packages += 1;
            }
            if let Some(language) = node.language {
                *by_language
                    .entry(language.as_str().to_string())
                    .or_insert(0) += 1;
            }
        }
        for edge in &self.edges {
            *by_edge_kind
                .entry(edge.kind.as_str().to_string())
                .or_insert(0) += 1;
            *by_status
                .entry(edge.status.as_str().to_string())
                .or_insert(0) += 1;
        }
        let unresolved_edges = self
            .edges
            .iter()
            .filter(|edge| {
                matches!(
                    edge.status,
                    ResolutionStatus::Unresolved
                        | ResolutionStatus::Ambiguous
                        | ResolutionStatus::SyntaxOnly
                )
            })
            .count();
        Stats {
            files,
            packages,
            nodes: self.nodes.len(),
            edges: self.edges.len(),
            symbols: self
                .nodes
                .iter()
                .filter(|n| {
                    !matches!(
                        n.kind,
                        NodeKind::Repository
                            | NodeKind::File
                            | NodeKind::Module
                            | NodeKind::Package
                            | NodeKind::ExternalSymbol
                    )
                })
                .count(),
            unresolved_edges,
            diagnostics: diagnostics.len(),
            by_node_kind,
            by_edge_kind,
            by_language,
            by_status,
        }
    }

    /// Structural validation. Returns every issue found rather than only the
    /// first so that imported graphs can be inspected usefully.
    pub fn validation_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let ids: HashSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();
        if ids.len() != self.nodes.len() {
            issues.push("duplicate node ids detected".to_string());
        }
        for node in &self.nodes {
            if node.id.is_empty() {
                issues.push("node with empty id".to_string());
            }
            if node.name.is_empty() {
                issues.push(format!("node `{}` has an empty name", node.id));
            }
            if let Some(file) = &node.file {
                if file.starts_with('/') || file.contains(":\\") {
                    issues.push(format!("node `{}` has an absolute file path", node.id));
                }
            }
            if let Some(range) = &node.range {
                if !range.is_valid() {
                    issues.push(format!("node `{}` has an invalid source range", node.id));
                }
            }
            if let Some(parent) = &node.parent {
                if !ids.contains(parent.as_str()) {
                    issues.push(format!(
                        "node `{}` references missing parent `{}`",
                        node.id, parent
                    ));
                }
            }
        }
        for edge in &self.edges {
            if !ids.contains(edge.source.as_str()) {
                issues.push(format!("edge references missing source `{}`", edge.source));
            }
            if !ids.contains(edge.target.as_str()) {
                issues.push(format!("edge references missing target `{}`", edge.target));
            }
            for range in &edge.sites {
                if !range.is_valid() {
                    issues.push(format!(
                        "edge {} -{}-> {} has an invalid site range",
                        edge.source,
                        edge.kind.as_str(),
                        edge.target
                    ));
                }
            }
        }
        issues
    }

    pub fn validate(&self) -> Result<()> {
        let issues = self.validation_issues();
        if issues.is_empty() {
            Ok(())
        } else {
            Err(CoreError::InvalidGraph(issues.join("; ")))
        }
    }

    /// Resolve a user-provided symbol selector.
    ///
    /// Match order: exact canonical ID, exact qualified name, exact short name,
    /// then a dotted/path suffix match (`Class.method`). Results are sorted by
    /// ID. Multiple matches are returned so callers can report ambiguity.
    pub fn resolve_symbols(&self, selector: &str) -> Vec<&Node> {
        if let Some(node) = self.node(selector) {
            return vec![node];
        }
        let mut out: Vec<&Node> = self.find_by_qualified_name(selector);
        if out.is_empty() {
            out = self.find_by_name(selector);
        }
        if out.is_empty() {
            let needle = selector.replace("::", ".");
            out = self
                .nodes
                .iter()
                .filter(|node| {
                    let qualified = node.qualified_name.as_deref().unwrap_or("");
                    let normalized = qualified.replace("::", ".");
                    let id_tail = node
                        .id
                        .split_once(':')
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

    /// All node IDs, useful for deterministic iteration.
    pub fn node_ids(&self) -> Vec<String> {
        self.nodes.iter().map(|n| n.id.clone()).collect()
    }

    pub fn edge_kind_counts(&self) -> BTreeMap<EdgeKind, usize> {
        let mut counts = BTreeMap::new();
        for edge in &self.edges {
            *counts.entry(edge.kind).or_insert(0) += 1;
        }
        counts
    }
}

/// Filter used by `trace graph` and the query API.
#[derive(Debug, Clone, Default)]
pub struct NodeFilter {
    pub languages: Vec<Language>,
    pub kinds: Vec<NodeKind>,
    pub file_prefix: Option<String>,
    pub name: Option<String>,
    pub exported_only: bool,
    pub include_external: bool,
}

impl NodeFilter {
    pub fn matches(&self, node: &Node) -> bool {
        if !self.include_external && node.kind == NodeKind::ExternalSymbol {
            return false;
        }
        if !self.languages.is_empty() {
            match node.language {
                Some(language) if self.languages.contains(&language) => {}
                _ => return false,
            }
        }
        if !self.kinds.is_empty() && !self.kinds.contains(&node.kind) {
            return false;
        }
        if let Some(prefix) = &self.file_prefix {
            match &node.file {
                Some(file) if file.starts_with(prefix) => {}
                _ => return false,
            }
        }
        if let Some(name) = &self.name {
            let matches_name = node.name == *name
                || node.qualified_name.as_deref() == Some(name.as_str())
                || node.id == *name;
            if !matches_name {
                return false;
            }
        }
        if self.exported_only && node.exported != Some(true) {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub files: usize,
    pub packages: usize,
    pub nodes: usize,
    pub edges: usize,
    pub symbols: usize,
    pub unresolved_edges: usize,
    pub diagnostics: usize,
    #[serde(default)]
    pub by_node_kind: BTreeMap<String, usize>,
    #[serde(default)]
    pub by_edge_kind: BTreeMap<String, usize>,
    #[serde(default)]
    pub by_language: BTreeMap<String, usize>,
    #[serde(default)]
    pub by_status: BTreeMap<String, usize>,
}

/// Build a repository node with a fixed canonical ID.
pub fn repository_node(name: &str) -> Node {
    Node::new(id::repository_id(), NodeKind::Repository, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::ResolutionOrigin;
    use crate::location::{Position, SourceRange};

    fn range(byte: u32) -> SourceRange {
        SourceRange::new(
            Position::new(1, byte + 1, byte),
            Position::new(1, byte + 2, byte + 1),
        )
    }

    fn fixture_graph() -> Graph {
        let mut graph = Graph::new();
        graph
            .add_node(Node::new("function:a.ts::main", NodeKind::Function, "main"))
            .unwrap();
        graph
            .add_node(Node::new(
                "function:a.ts::helper",
                NodeKind::Function,
                "helper",
            ))
            .unwrap();
        graph
            .add_edge(
                Edge::new(
                    "function:a.ts::main",
                    "function:a.ts::helper",
                    EdgeKind::Calls,
                    ResolutionStatus::ResolvedInternal,
                    ResolutionOrigin::Lexical,
                )
                .at(range(10)),
            )
            .unwrap();
        graph.finalize();
        graph
    }

    #[test]
    fn rejects_conflicting_duplicate_nodes() {
        let mut graph = Graph::new();
        graph
            .add_node(Node::new("x", NodeKind::Function, "a"))
            .unwrap();
        let err = graph
            .add_node(Node::new("x", NodeKind::Function, "b"))
            .unwrap_err();
        assert!(matches!(err, CoreError::ConflictingNode(_)));
    }

    #[test]
    fn aggregates_edge_sites() {
        let mut graph = Graph::new();
        graph
            .add_node(Node::new("a", NodeKind::Function, "a"))
            .unwrap();
        graph
            .add_node(Node::new("b", NodeKind::Function, "b"))
            .unwrap();
        for byte in [20, 10, 20] {
            graph
                .add_edge(
                    Edge::new(
                        "a",
                        "b",
                        EdgeKind::Calls,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Lexical,
                    )
                    .at(range(byte)),
                )
                .unwrap();
        }
        graph.finalize();
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.edges()[0].sites.len(), 2);
        assert_eq!(graph.edges()[0].sites[0].start.byte, 10);
    }

    #[test]
    fn validation_reports_missing_endpoints() {
        let mut graph = Graph::new();
        graph
            .add_node(Node::new("a", NodeKind::Function, "a"))
            .unwrap();
        graph
            .add_edge(Edge::new(
                "a",
                "missing",
                EdgeKind::Calls,
                ResolutionStatus::Unresolved,
                ResolutionOrigin::Syntax,
            ))
            .unwrap();
        graph.finalize();
        assert!(graph.validate().is_err());
    }

    #[test]
    fn resolves_symbols_by_short_and_qualified_name() {
        let graph = fixture_graph();
        assert_eq!(graph.resolve_symbols("helper").len(), 1);
        assert_eq!(graph.resolve_symbols("function:a.ts::main").len(), 1);
        assert_eq!(graph.resolve_symbols("a.ts::helper").len(), 1);
        assert!(graph.resolve_symbols("nope").is_empty());
    }

    #[test]
    fn filters_nodes_and_prunes_dangling_edges() {
        let graph = fixture_graph();
        let filter = NodeFilter {
            kinds: vec![NodeKind::Function],
            ..Default::default()
        };
        let (nodes, edges) = graph.filtered(&filter);
        assert_eq!(nodes.len(), 2);
        assert_eq!(edges.len(), 1);

        let filter = NodeFilter {
            name: Some("main".into()),
            ..Default::default()
        };
        let (nodes, edges) = graph.filtered(&filter);
        assert_eq!(nodes.len(), 1);
        assert_eq!(edges.len(), 0);
    }
}
