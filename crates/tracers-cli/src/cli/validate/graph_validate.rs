use std::path::Path;

use anyhow::Result;

use crate::cli::scan::load_or_build_graph;

pub(crate) fn cmd_validate(path: &Path, graph_file: Option<&Path>) -> Result<()> {
    let graph = load_or_build_graph(path, graph_file)?.graph;
    let graph = &graph;
    let mut dangling = Vec::new();

    for edge in graph.edges() {
        if graph.node(&edge.source).is_none() {
            dangling.push(format!(
                "edge {}: source node '{}' does not exist",
                edge.kind.as_str(),
                edge.source
            ));
        }
        if graph.node(&edge.target).is_none() {
            dangling.push(format!(
                "edge {}: target node '{}' does not exist",
                edge.kind.as_str(),
                edge.target
            ));
        }
    }

    println!("Graph validation:");
    println!("  nodes:        {}", graph.nodes().len());
    println!("  edges:        {}", graph.edges().len());
    println!("  dangling:     {}", dangling.len());

    for issue in &dangling {
        println!("  ! {issue}");
    }

    if dangling.is_empty() {
        println!("\nValidation passed: every edge references existing nodes.");
    } else {
        println!(
            "\nValidation failed: {} dangling edge reference(s).",
            dangling.len()
        );
        anyhow::bail!("graph contains dangling edges");
    }

    Ok(())
}
