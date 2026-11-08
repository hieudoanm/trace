use std::path::Path;

use anyhow::Result;

use crate::cli::scan::load_or_build_graph;

pub(crate) fn cmd_stats(path: &Path, graph_file: Option<&Path>) -> Result<()> {
    let (graph, diagnostics) = load_or_build_graph(path, graph_file)?;
    let stats = graph.stats(&diagnostics);

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
    if !stats.by_status.is_empty() {
        println!("\nResolution status:");
        for (status, count) in &stats.by_status {
            println!("  {status}: {count}");
        }
    }
    if !stats.by_framework.is_empty() {
        println!("\nBy framework:");
        for (framework, count) in &stats.by_framework {
            println!("  {framework}: {count}");
        }
    }
    Ok(())
}
