use crate::edge::{Edge, EdgeKind, ResolutionOrigin, ResolutionStatus};
use crate::location::{Position, SourceRange};
use crate::node::{Node, NodeKind};
#[allow(unused_imports)]
use crate::query::convenience::resolve_unique;
#[allow(unused_imports)]
use crate::query::{Query, QueryOptions};

#[allow(dead_code)]
fn site() -> SourceRange {
    SourceRange::new(Position::new(1, 1, 0), Position::new(1, 2, 1))
}

#[allow(dead_code)]
fn graph() -> crate::graph::Graph {
    let mut graph = crate::graph::Graph::new();
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
