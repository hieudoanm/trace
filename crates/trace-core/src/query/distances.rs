use std::collections::HashMap;

use crate::query::constants::PATH_EDGES;
use crate::query::options::QueryOptions;
use crate::query::traversal::Query;

impl<'a> Query<'a> {
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
