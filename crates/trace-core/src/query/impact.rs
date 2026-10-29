use std::collections::{HashSet, VecDeque};

use crate::edge::EdgeKind;
use crate::node::Node;

use crate::query::constants::IMPACT_EDGES;
use crate::query::options::QueryOptions;
use crate::query::traversal::Query;
use crate::query::types::ImpactNode;

impl<'a> Query<'a> {
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
            let mut edges: Vec<&crate::edge::Edge> = self
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
}
