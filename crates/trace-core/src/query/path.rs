use std::collections::{HashMap, HashSet, VecDeque};

use crate::edge::Edge;

use crate::query::constants::PATH_EDGES;
use crate::query::options::QueryOptions;
use crate::query::traversal::Query;
use crate::query::types::{PathEdge, PathResult};

impl<'a> Query<'a> {
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
}
