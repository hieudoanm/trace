use super::edge_key::EdgeKey;
use super::graph_struct::Graph;

impl Graph {
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
}
