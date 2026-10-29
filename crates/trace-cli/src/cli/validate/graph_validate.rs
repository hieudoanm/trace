use std::path::Path;

use anyhow::Result;

use crate::cli::scan::build_graph_for_path;

pub(crate) fn cmd_validate(path: &Path) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;

    let graph = &result.graph;
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
