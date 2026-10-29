use crate::edge::Edge;
use crate::graph::storage::Graph;
use crate::node::Node;

pub fn node<'a>(graph: &'a Graph, node_id: &str) -> Option<&'a Node> {
    graph.node_index.get(node_id).map(|&idx| &graph.nodes[idx])
}

pub fn require_node<'a>(
    graph: &'a Graph,
    node_id: &str,
) -> Result<&'a Node, crate::error::CoreError> {
    node(graph, node_id).ok_or_else(|| crate::error::CoreError::UnknownNode(node_id.to_string()))
}

pub fn find_by_name<'a>(graph: &'a Graph, name: &str) -> Vec<&'a Node> {
    graph
        .name_index
        .get(name)
        .map(|ids| ids.iter().filter_map(|id| node(graph, id)).collect())
        .unwrap_or_default()
}

pub fn find_by_qualified_name<'a>(graph: &'a Graph, qualified: &str) -> Vec<&'a Node> {
    graph
        .qualified_index
        .get(qualified)
        .map(|ids| ids.iter().filter_map(|id| node(graph, id)).collect())
        .unwrap_or_default()
}

pub fn find_by_kind(graph: &Graph, kind: crate::node::NodeKind) -> Vec<&Node> {
    graph
        .kind_index
        .get(&kind)
        .map(|ids| ids.iter().filter_map(|id| node(graph, id)).collect())
        .unwrap_or_default()
}

pub fn out_edges<'a>(graph: &'a Graph, node_id: &str) -> Vec<&'a Edge> {
    graph
        .out_index
        .get(node_id)
        .map(|indices| indices.iter().map(|&idx| &graph.edges[idx]).collect())
        .unwrap_or_default()
}

pub fn in_edges<'a>(graph: &'a Graph, node_id: &str) -> Vec<&'a Edge> {
    graph
        .in_index
        .get(node_id)
        .map(|indices| indices.iter().map(|&idx| &graph.edges[idx]).collect())
        .unwrap_or_default()
}
