use std::collections::{HashSet, VecDeque};

use crate::edge::{Edge, EdgeKind};
use crate::graph::Graph;
use crate::node::Node;

use crate::query::constants::{DEPENDENCY_EDGES, PATH_EDGES};
use crate::query::direction::Direction;
use crate::query::options::QueryOptions;
use crate::query::types::Neighbor;

/// Reusable query API over a graph. Common lookups use graph indexes.
#[derive(Debug, Clone, Copy)]
pub struct Query<'a> {
    pub(crate) graph: &'a Graph,
}

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

    pub fn reachable(&self, start: &str, options: &QueryOptions) -> Vec<Neighbor> {
        self.forward_traverse(start, options, PATH_EDGES)
    }
}
