use std::path::Path;

use anyhow::Result;

use crate::cli::scan::build_graph_for_path;

pub(crate) fn cmd_stats(path: &Path) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let graph = &result.graph;
    let stats = graph.stats(&result.diagnostics);

    println!("Graph statistics:");
    println!("  files:       {}", stats.files);
    println!("  packages:    {}", stats.packages);
    println!("  nodes:       {}", stats.nodes);
    println!("  edges:       {}", stats.edges);
    println!("  symbols:     {}", stats.symbols);
    println!("  unresolved:  {}", stats.unresolved_edges);
    println!("  diagnostics: {}", stats.diagnostics);

    if !stats.by_language.is_empty() {
        println!("\nBy language:");
        for (language, count) in &stats.by_language {
            println!("  {language}: {count}");
        }
    }
    if !stats.by_node_kind.is_empty() {
        println!("\nBy node kind:");
        for (kind, count) in &stats.by_node_kind {
            println!("  {kind}: {count}");
        }
    }
    if !stats.by_edge_kind.is_empty() {
        println!("\nBy edge kind:");
        for (kind, count) in &stats.by_edge_kind {
            println!("  {kind}: {count}");
        }
    }
    Ok(())
}
