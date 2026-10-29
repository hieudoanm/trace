//! Tests for graph construction.

use super::*;
use crate::index::RepositoryIndex;
use crate::resolve::resolve_imports;
use tempfile::tempdir;
use trace_core::{file_id, symbol_id, EdgeKind, Language, NodeKind, ResolutionStatus};
use trace_parser::{ParsedFile, SyntaxParser};

fn parse(path: &str, language: Language, source: &str) -> ParsedFile {
    let mut parser = SyntaxParser::new().unwrap();
    parser.parse(path, language, source).unwrap()
}

#[test]
fn builds_file_and_symbol_nodes() {
    let dir = tempdir().unwrap();
    let parsed = vec![parse(
        "a.ts",
        Language::TypeScript,
        "export function foo() { return 1; }\n",
    )];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    assert!(result.graph.node(&file_id("a.ts")).is_some());
    assert!(result
        .graph
        .node(&symbol_id(NodeKind::Function, "a.ts", "foo", 0))
        .is_some());
    assert_eq!(result.graph.node_count(), 4); // repository, file, module, function
}

#[test]
fn builds_call_edge_between_files() {
    let dir = tempdir().unwrap();
    let parsed = vec![
        parse(
            "math.ts",
            Language::TypeScript,
            "export function twice(n: number): number { return n * 2; }\n",
        ),
        parse(
            "main.ts",
            Language::TypeScript,
            "import { twice } from './math';\nexport function run(): number { return twice(1); }\n",
        ),
    ];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    let run_id = symbol_id(NodeKind::Function, "main.ts", "run", 0);
    let twice_id = symbol_id(NodeKind::Function, "math.ts", "twice", 0);
    let graph = &result.graph;
    let out_edges = graph.out_edges(&run_id);
    let edges: Vec<_> = out_edges
        .iter()
        .flat_map(std::iter::once)
        .filter(|e| e.kind == EdgeKind::Calls && e.target == twice_id)
        .collect();
    assert!(!edges.is_empty(), "expected a call edge run -> twice");
}

#[test]
fn records_unresolved_call() {
    let dir = tempdir().unwrap();
    let parsed = vec![parse(
        "a.ts",
        Language::TypeScript,
        "export function run() { return missing(); }\n",
    )];
    let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
    let import_pass = resolve_imports(&mut index);
    let builder = GraphBuilder::new();
    let result = builder.build_from_index(&index, &import_pass, &[]).unwrap();
    let run_id = symbol_id(NodeKind::Function, "a.ts", "run", 0);
    let graph = &result.graph;
    let out_edges = graph.out_edges(&run_id);
    let edges: Vec<_> = out_edges
        .iter()
        .flat_map(std::iter::once)
        .filter(|e| e.kind == EdgeKind::Calls)
        .collect();
    assert!(!edges.is_empty());
    assert!(
        edges
            .iter()
            .any(|e| matches!(e.status, ResolutionStatus::Unresolved)),
        "expected an unresolved call edge"
    );
    // The unresolved target must exist as a node so the graph is target-complete.
    assert!(graph.node("unresolved:missing").is_some());
}
