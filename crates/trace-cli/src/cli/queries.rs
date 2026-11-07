//! Query commands: callers, callees, refs, deps, impact, path, roots.

use std::path::Path;

use anyhow::{anyhow, Result};
use trace_core::query::resolve_unique;
use trace_core::{file_id, module_id, package_id, EdgeKind, Graph, Node, Query, QueryOptions};

use crate::cli::scan::build_graph_for_path;

fn build_graph(path: &Path) -> Result<Graph> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    Ok(result.graph)
}

/// Resolve a user-supplied selector to a node.
///
/// Accepts canonical node IDs (`function:src/a.ts::foo`, `file:src/a.ts`),
/// repository-relative file/module/package paths, and symbol selectors
/// (qualified name, short name, or ID tail).
fn resolve_target(graph: &Graph, selector: &str) -> Result<Node> {
    if let Some(node) = graph.node(selector) {
        return Ok(node.clone());
    }
    for candidate in [file_id(selector), module_id(selector), package_id(selector)] {
        if let Some(node) = graph.node(&candidate) {
            return Ok(node.clone());
        }
    }
    resolve_unique(graph, selector)
        .cloned()
        .map_err(anyhow::Error::msg)
}

fn node_options(max_depth: usize, max_results: usize) -> QueryOptions {
    QueryOptions::default()
        .with_depth(max_depth)
        .with_limit(max_results)
}

fn location_suffix(node: &Node) -> String {
    match (&node.file, &node.range) {
        (Some(file), Some(range)) => format!(" — {file}:{}", range.start.line),
        (Some(file), None) => format!(" — {file}"),
        _ => String::new(),
    }
}

fn print_neighbors(
    heading: &str,
    selector: &str,
    neighbors: &[trace_core::Neighbor],
    max_results: usize,
) {
    println!("{heading} of `{selector}`:");
    if neighbors.is_empty() {
        println!("  (none)");
        return;
    }
    for neighbor in neighbors {
        let node = &neighbor.node;
        let origin = match neighbor.edge.origin {
            trace_core::ResolutionOrigin::Syntax => String::new(),
            other => format!(", {}", other.as_str()),
        };
        println!(
            "  - {} [{}] via {} ({}){} depth {}{}",
            node.name,
            node.kind.as_str(),
            neighbor.edge.kind.as_str(),
            neighbor.edge.status.as_str(),
            origin,
            neighbor.depth,
            location_suffix(node),
        );
        println!("    {}", node.id);
    }
    if neighbors.len() >= max_results {
        println!("  … result limit reached; refine with --max-results or --max-depth");
    }
}

pub(crate) fn cmd_callers(
    symbol: &str,
    path: &Path,
    max_depth: usize,
    max_results: usize,
) -> Result<()> {
    let graph = build_graph(path)?;
    let node = resolve_target(&graph, symbol)?;
    let query = Query::new(&graph);
    let options = node_options(max_depth, max_results);
    let callers = query.callers(&node.id, &options);
    print_neighbors("Callers", symbol, &callers, max_results);
    Ok(())
}

pub(crate) fn cmd_callees(
    symbol: &str,
    path: &Path,
    max_depth: usize,
    max_results: usize,
) -> Result<()> {
    let graph = build_graph(path)?;
    let node = resolve_target(&graph, symbol)?;
    let query = Query::new(&graph);
    let options = node_options(max_depth, max_results);
    let callees = query.callees(&node.id, &options);
    print_neighbors("Callees", symbol, &callees, max_results);
    Ok(())
}

pub(crate) fn cmd_refs(
    symbol: &str,
    path: &Path,
    max_depth: usize,
    max_results: usize,
) -> Result<()> {
    let graph = build_graph(path)?;
    let node = resolve_target(&graph, symbol)?;
    let query = Query::new(&graph);
    // A call site is also a reference, so `refs` covers calls, references
    // and type usages; `callers` remains the calls-only view.
    let options = node_options(max_depth, max_results).with_edges(&[
        EdgeKind::Calls,
        EdgeKind::References,
        EdgeKind::UsesType,
    ]);
    let references = query.reverse_traverse(&node.id, &options, &[]);
    print_neighbors("References", symbol, &references, max_results);
    Ok(())
}

pub(crate) fn cmd_deps(
    target: &str,
    path: &Path,
    transitive: bool,
    max_depth: usize,
) -> Result<()> {
    let graph = build_graph(path)?;
    let node = resolve_target(&graph, target)?;
    let query = Query::new(&graph);
    // Without --transitive only direct (depth-1) dependencies are shown.
    let options = node_options(if transitive { max_depth } else { 1 }, 1000);
    let deps = query.transitive_dependencies(&node.id, &options);
    println!(
        "Dependencies of `{target}`{}:",
        if transitive {
            " (transitive)"
        } else {
            " (direct)"
        }
    );
    if deps.is_empty() {
        println!("  (none)");
    } else {
        for neighbor in &deps {
            let node = &neighbor.node;
            println!(
                "  - {} [{}] via {} ({}) depth {}{}",
                node.name,
                node.kind.as_str(),
                neighbor.edge.kind.as_str(),
                neighbor.edge.status.as_str(),
                neighbor.depth,
                location_suffix(node),
            );
            println!("    {}", node.id);
        }
        if deps.len() >= options.max_results {
            println!("  … result limit reached; refine with --max-depth");
        }
    }
    Ok(())
}

pub(crate) fn cmd_impact(
    target: &str,
    path: &Path,
    max_depth: usize,
    max_results: usize,
) -> Result<()> {
    let graph = build_graph(path)?;
    let node = resolve_target(&graph, target)?;
    let query = Query::new(&graph);
    let options = node_options(max_depth, max_results);
    let impacts = query.impact(&node.id, &options);

    println!("Impact of `{target}` (static reachability, not runtime breakage):");
    if impacts.is_empty() {
        println!("  (none)");
    } else {
        for impact in &impacts {
            println!(
                "  - {} [{}] depth {}",
                impact.node.name,
                impact.node.kind.as_str(),
                impact.depth,
            );
            if impact.path.len() > 1 {
                let chain = impact
                    .path
                    .iter()
                    .map(|node| node.name.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                println!("    path: {chain}");
            }
            if !impact.via.is_empty() {
                let via = impact
                    .via
                    .iter()
                    .map(|kind| kind.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("    via: {via}");
            }
            println!("    {}", impact.node.id);
        }
        if impacts.len() >= max_results {
            println!("  … result limit reached; refine with --max-results or --max-depth");
        }
    }
    Ok(())
}

pub(crate) fn cmd_path(source: &str, target: &str, path: &Path, max_depth: usize) -> Result<()> {
    let graph = build_graph(path)?;
    let source_node = resolve_target(&graph, source)?;
    let target_node = resolve_target(&graph, target)?;
    let query = Query::new(&graph);
    let options = node_options(max_depth, usize::MAX);
    let result = query.path(&source_node.id, &target_node.id, &options);

    if !result.found {
        println!("No path found from `{source}` to `{target}` within depth {max_depth}.");
        return Ok(());
    }

    println!("Path from `{source}` to `{target}`:");
    for (index, node) in result.nodes.iter().enumerate() {
        if index == 0 {
            println!("  {}", node.id);
            continue;
        }
        let edge = result
            .edges
            .get(index - 1)
            .ok_or_else(|| anyhow!("path result is internally inconsistent"))?;
        println!(
            "  --{} ({})--> {}",
            edge.kind.as_str(),
            edge.status.as_str(),
            node.id
        );
    }
    Ok(())
}

pub(crate) fn cmd_roots(path: &Path, max_results: usize) -> Result<()> {
    let graph = build_graph(path)?;
    let query = Query::new(&graph);
    let roots = query.no_incoming_calls();

    println!("Callable symbols with no incoming `calls` edges (entry-point candidates):");
    if roots.is_empty() {
        println!("  (none)");
    } else {
        for node in roots.iter().take(max_results) {
            println!(
                "  - {} [{}]{}",
                node.name,
                node.kind.as_str(),
                location_suffix(node),
            );
            println!("    {}", node.id);
        }
        if roots.len() > max_results {
            println!(
                "  … {} more; increase --max-results to see them",
                roots.len() - max_results
            );
        }
    }
    println!(
        "\nNote: this is not a dead-code report. Dynamic dispatch, reflection,\n\
         exported entry points, tests and callbacks can all legitimately have\n\
         no statically visible caller."
    );
    Ok(())
}
