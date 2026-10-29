use crate::edge::{Edge, EdgeKind, ResolutionOrigin, ResolutionStatus};
use crate::graph::filter::NodeFilter;
use crate::graph::Graph;
use crate::location::{Position, SourceRange};
use crate::node::{Node, NodeKind};

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
    use crate::error::CoreError;
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
