use std::collections::{HashMap, HashSet, VecDeque};

use crate::edge::{Edge, EdgeKind, ResolutionStatus};
use crate::graph::Graph;
use crate::node::{Node, NodeKind};

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

    fn allows(&self, edge: &Edge, defaults: &[EdgeKind]) -> bool {
        let kinds = if self.edge_kinds.is_empty() {
            defaults
        } else {
            &self.edge_kinds
        };
        kinds.contains(&edge.kind)
            && (self.statuses.is_empty() || self.statuses.contains(&edge.status))
    }
}

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

/// Reusable query API over a graph. Common lookups use graph indexes.
#[derive(Debug, Clone, Copy)]
pub struct Query<'a> {
    graph: &'a Graph,
}

pub const DEPENDENCY_EDGES: &[EdgeKind] = &[
    EdgeKind::Imports,
    EdgeKind::DependsOn,
    EdgeKind::UsesType,
    EdgeKind::Inherits,
    EdgeKind::Implements,
];

pub const IMPACT_EDGES: &[EdgeKind] = &[
    EdgeKind::Calls,
    EdgeKind::References,
    EdgeKind::UsesType,
    EdgeKind::Imports,
    EdgeKind::DependsOn,
    EdgeKind::Inherits,
    EdgeKind::Implements,
    EdgeKind::Overrides,
];

pub const PATH_EDGES: &[EdgeKind] = &[
    EdgeKind::Calls,
    EdgeKind::References,
    EdgeKind::UsesType,
    EdgeKind::Imports,
    EdgeKind::DependsOn,
    EdgeKind::Inherits,
    EdgeKind::Implements,
];

impl<'a> Query<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    pub fn graph(&self) -> &'a Graph {
        self.graph
    }

    pub fn node(&self, id: &str) -> Option<&'a Node> {
        self.graph.node(id)
    }

    pub fn find(&self, selector: &str) -> Vec<&'a Node> {
        self.graph.resolve_symbols(selector)
    }

    pub fn callers(&self, node_id: &str, options: &QueryOptions) -> Vec<Neighbor> {
        self.reverse_traverse(node_id, options, &[EdgeKind::Calls])
    }

    pub fn callees(&self, node_id: &str, options: &QueryOptions) -> Vec<Neighbor> {
        self.forward_traverse(node_id, options, &[EdgeKind::Calls])
    }

    pub fn references(&self, node_id: &str, options: &QueryOptions) -> Vec<Neighbor> {
        self.reverse_traverse(
            node_id,
            options,
            &[EdgeKind::References, EdgeKind::UsesType],
        )
    }

    pub fn direct_dependencies(&self, node_id: &str, options: &QueryOptions) -> Vec<Neighbor> {
        self.forward_traverse(node_id, options, DEPENDENCY_EDGES)
    }

    pub fn transitive_dependencies(&self, node_id: &str, options: &QueryOptions) -> Vec<Neighbor> {
        self.forward_traverse(node_id, options, DEPENDENCY_EDGES)
    }

    pub fn reverse_dependencies_refs(
        &self,
        node_id: &str,
        options: &QueryOptions,
    ) -> Vec<Neighbor> {
        self.reverse_traverse(node_id, options, DEPENDENCY_EDGES)
    }

    /// Forward BFS over selected edges, deterministic in depth then node ID.
    pub fn forward_traverse(
        &self,
        start: &str,
        options: &QueryOptions,
        default_kinds: &[EdgeKind],
    ) -> Vec<Neighbor> {
        self.traverse(start, options, default_kinds, Direction::Forward)
    }

    /// Reverse BFS over selected edges, deterministic in depth then node ID.
    pub fn reverse_traverse(
        &self,
        start: &str,
        options: &QueryOptions,
        default_kinds: &[EdgeKind],
    ) -> Vec<Neighbor> {
        self.traverse(start, options, default_kinds, Direction::Reverse)
    }

    fn traverse(
        &self,
        start: &str,
        options: &QueryOptions,
        default_kinds: &[EdgeKind],
        direction: Direction,
    ) -> Vec<Neighbor> {
        if self.graph.node(start).is_none() {
            return Vec::new();
        }
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(start.to_string());
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        queue.push_back((start.to_string(), 0));
        let mut results = Vec::new();
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= options.max_depth {
                continue;
            }
            let edges = match direction {
                Direction::Forward => self.graph.out_edges(&current),
                Direction::Reverse => self.graph.in_edges(&current),
            };
            let mut next: Vec<(String, Edge)> = edges
                .into_iter()
                .filter(|edge| options.allows(edge, default_kinds))
                .map(|edge| {
                    let other = match direction {
                        Direction::Forward => &edge.target,
                        Direction::Reverse => &edge.source,
                    };
                    (other.clone(), edge.clone())
                })
                .collect();
            next.sort_by(|a, b| {
                a.0.cmp(&b.0)
                    .then(a.1.kind.cmp(&b.1.kind))
                    .then(a.1.status.cmp(&b.1.status))
            });
            for (next_id, edge) in next {
                if !visited.insert(next_id.clone()) {
                    continue;
                }
                if let Some(node) = self.graph.node(&next_id) {
                    results.push(Neighbor {
                        node: node.clone(),
                        edge,
                        depth: depth + 1,
                    });
                    if results.len() >= options.max_results {
                        return results;
                    }
                    queue.push_back((next_id, depth + 1));
                }
            }
        }
        results
    }

    /// Find one shortest path between two nodes.
    pub fn path(&self, source: &str, target: &str, options: &QueryOptions) -> PathResult {
        if self.graph.node(source).is_none() || self.graph.node(target).is_none() {
            return PathResult {
                found: false,
                nodes: Vec::new(),
                edges: Vec::new(),
            };
        }
        let mut predecessor: HashMap<String, (String, Edge)> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(source.to_string());
        let mut queue = VecDeque::new();
        queue.push_back((source.to_string(), 0usize));
        let mut found = source == target;
        while let Some((current, depth)) = queue.pop_front() {
            if found {
                break;
            }
            if depth >= options.max_depth {
                continue;
            }
            let mut edges: Vec<&Edge> = self
                .graph
                .out_edges(&current)
                .into_iter()
                .filter(|edge| options.allows(edge, PATH_EDGES))
                .collect();
            edges.sort_by(|a, b| {
                a.target
                    .cmp(&b.target)
                    .then(a.kind.cmp(&b.kind))
                    .then(a.status.cmp(&b.status))
            });
            for edge in edges {
                if !visited.insert(edge.target.clone()) {
                    continue;
                }
                predecessor.insert(edge.target.clone(), (current.clone(), edge.clone()));
                if edge.target == target {
                    found = true;
                    break;
                }
                queue.push_back((edge.target.clone(), depth + 1));
            }
        }
        if !found {
            return PathResult {
                found: false,
                nodes: Vec::new(),
                edges: Vec::new(),
            };
        }
        let mut node_ids = vec![target.to_string()];
        let mut path = Vec::new();
        let mut cursor = target.to_string();
        while cursor != source {
            let Some((previous, edge)) = predecessor.get(&cursor) else {
                break;
            };
            path.push(PathEdge {
                kind: edge.kind,
                status: edge.status,
            });
            cursor = previous.clone();
            node_ids.push(cursor.clone());
        }
        node_ids.reverse();
        path.reverse();
        let nodes = node_ids
            .iter()
            .filter_map(|id| self.graph.node(id).cloned())
            .collect();
        PathResult {
            found: true,
            nodes,
            edges: path,
        }
    }

    pub fn reachable(&self, start: &str, options: &QueryOptions) -> Vec<Neighbor> {
        self.forward_traverse(start, options, PATH_EDGES)
    }

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
        let index: HashMap<&str, usize> =
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

    /// Callable symbols with no incoming call relationship.
    ///
    /// This is *not* a dead-code detector: dynamic dispatch, reflection,
    /// exported entry points, tests and callbacks can all legitimately have no
    /// statically visible caller.
    pub fn no_incoming_calls(&self) -> Vec<&'a Node> {
        let mut nodes: Vec<&Node> = self
            .graph
            .nodes()
            .iter()
            .filter(|node| {
                node.kind.is_callable()
                    && self
                        .graph
                        .in_edges(&node.id)
                        .iter()
                        .all(|edge| edge.kind != EdgeKind::Calls)
            })
            .collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        nodes
    }

    /// Static impact reachability. This is graph reachability, not a claim
    /// about runtime breakage.
    pub fn impact(&self, target: &str, options: &QueryOptions) -> Vec<ImpactNode> {
        if self.graph.node(target).is_none() {
            return Vec::new();
        }
        let mut results = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(target.to_string());
        queue.push_back((target.to_string(), 0usize));
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= options.max_depth || results.len() >= options.max_results {
                continue;
            }
            let mut edges: Vec<&Edge> = self
                .graph
                .in_edges(&current)
                .into_iter()
                .filter(|edge| options.allows(edge, IMPACT_EDGES))
                .collect();
            edges.sort_by(|a, b| {
                a.source
                    .cmp(&b.source)
                    .then(a.kind.cmp(&b.kind))
                    .then(a.status.cmp(&b.status))
            });
            for edge in edges {
                if !visited.insert(edge.source.clone()) {
                    continue;
                }
                let Some(node) = self.graph.node(&edge.source) else {
                    continue;
                };
                // Build the explanatory path from the dependent to the target.
                let mut path = vec![node.clone()];
                path.extend(self.path_to_target(&edge.target, target, options, depth + 1));
                results.push(ImpactNode {
                    node: node.clone(),
                    depth: depth + 1,
                    path,
                    via: vec![edge.kind],
                });
                if results.len() >= options.max_results {
                    break;
                }
                queue.push_back((edge.source.clone(), depth + 1));
            }
        }
        results.sort_by(|a, b| a.depth.cmp(&b.depth).then(a.node.id.cmp(&b.node.id)));
        results
    }

    fn path_to_target(
        &self,
        from: &str,
        target: &str,
        options: &QueryOptions,
        remaining: usize,
    ) -> Vec<Node> {
        if remaining == 0 {
            return Vec::new();
        }
        let path = self.path(
            from,
            target,
            &QueryOptions {
                max_depth: remaining,
                ..options.clone()
            },
        );
        if path.found {
            path.nodes
        } else {
            Vec::new()
        }
    }

    /// Find the shortest directed edge distance between every reachable node
    /// and the start, useful for filtered graph rendering.
    pub fn distances(&self, start: &str, options: &QueryOptions) -> HashMap<String, usize> {
        let mut distances = HashMap::new();
        distances.insert(start.to_string(), 0);
        for neighbor in self.forward_traverse(start, options, PATH_EDGES) {
            distances.entry(neighbor.node.id).or_insert(neighbor.depth);
        }
        distances
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Forward,
    Reverse,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::{EdgeKind, ResolutionOrigin};
    use crate::location::{Position, SourceRange};

    fn site() -> SourceRange {
        SourceRange::new(Position::new(1, 1, 0), Position::new(1, 2, 1))
    }

    fn graph() -> Graph {
        let mut graph = Graph::new();
        for (id, name, kind) in [
            ("function:a.ts::main", "main", NodeKind::Function),
            ("function:a.ts::helper", "helper", NodeKind::Function),
            ("function:a.ts::leaf", "leaf", NodeKind::Function),
            ("file:a.ts", "a.ts", NodeKind::File),
            ("file:b.ts", "b.ts", NodeKind::File),
        ] {
            graph.add_node(Node::new(id, kind, name)).unwrap();
        }
        for (source, target, kind) in [
            (
                "function:a.ts::main",
                "function:a.ts::helper",
                EdgeKind::Calls,
            ),
            (
                "function:a.ts::helper",
                "function:a.ts::leaf",
                EdgeKind::Calls,
            ),
            ("file:a.ts", "file:b.ts", EdgeKind::Imports),
        ] {
            graph
                .add_edge(
                    Edge::new(
                        source,
                        target,
                        kind,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Lexical,
                    )
                    .at(site()),
                )
                .unwrap();
        }
        graph.finalize();
        graph
    }

    #[test]
    fn callers_and_callees_follow_depth() {
        let graph = graph();
        let query = Query::new(&graph);
        let callees = query.callees("function:a.ts::main", &QueryOptions::default());
        assert_eq!(callees.len(), 2);
        assert_eq!(callees[0].node.name, "helper");
        assert_eq!(callees[1].depth, 2);
        let callers = query.callers("function:a.ts::leaf", &QueryOptions::default());
        assert_eq!(callers.len(), 2);
        assert_eq!(callers[0].node.name, "helper");
    }

    #[test]
    fn depth_and_result_limits_are_enforced() {
        let graph = graph();
        let query = Query::new(&graph);
        let callees = query.callees(
            "function:a.ts::main",
            &QueryOptions::default().with_depth(1),
        );
        assert_eq!(callees.len(), 1);
        let callees = query.callees(
            "function:a.ts::main",
            &QueryOptions::default().with_limit(1),
        );
        assert_eq!(callees.len(), 1);
    }

    #[test]
    fn finds_shortest_path() {
        let graph = graph();
        let query = Query::new(&graph);
        let path = query.path(
            "function:a.ts::main",
            "function:a.ts::leaf",
            &QueryOptions::default(),
        );
        assert!(path.found);
        assert_eq!(path.nodes.len(), 3);
        assert_eq!(path.edges.len(), 2);
    }

    #[test]
    fn reports_missing_path() {
        let graph = graph();
        let query = Query::new(&graph);
        let path = query.path(
            "function:a.ts::leaf",
            "function:a.ts::main",
            &QueryOptions::default(),
        );
        assert!(!path.found);
        assert!(path.nodes.is_empty());
    }

    #[test]
    fn detects_cycles() {
        let mut graph = graph();
        graph
            .add_edge(
                Edge::new(
                    "function:a.ts::leaf",
                    "function:a.ts::main",
                    EdgeKind::Calls,
                    ResolutionStatus::ResolvedInternal,
                    ResolutionOrigin::Lexical,
                )
                .at(site()),
            )
            .unwrap();
        graph.finalize();
        let query = Query::new(&graph);
        let cycles = query.cycles(&[EdgeKind::Calls]);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
        let dep_cycles = query.cycles(&[]);
        assert_eq!(dep_cycles.len(), 0);
    }

    #[test]
    fn impact_explains_reverse_paths() {
        let graph = graph();
        let query = Query::new(&graph);
        let impact = query.impact("function:a.ts::leaf", &QueryOptions::default());
        let names: Vec<&str> = impact.iter().map(|i| i.node.name.as_str()).collect();
        assert_eq!(names, vec!["helper", "main"]);
        assert_eq!(impact[0].path.len(), 2);
        assert_eq!(impact[1].path.len(), 3);
    }

    #[test]
    fn dependency_traversal_uses_import_edges() {
        let graph = graph();
        let query = Query::new(&graph);
        let deps = query.transitive_dependencies("file:a.ts", &QueryOptions::default());
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].node.name, "b.ts");
    }

    #[test]
    fn unique_resolution_reports_ambiguity() {
        let mut graph = graph();
        graph
            .add_node(Node::new(
                "function:b.ts::helper",
                NodeKind::Function,
                "helper",
            ))
            .unwrap();
        graph.finalize();
        let err = resolve_unique(&graph, "helper").unwrap_err();
        assert!(err.contains("ambiguous"), "{err}");
    }

    #[test]
    fn no_incoming_calls_ignores_non_callables() {
        let graph = graph();
        let query = Query::new(&graph);
        let roots = query.no_incoming_calls();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "main");
    }
}
