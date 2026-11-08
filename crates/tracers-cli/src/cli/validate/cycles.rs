use std::path::Path;

use anyhow::Result;
use tracers_core::Query;

use crate::cli::scan::load_or_build_graph;

pub(crate) fn cmd_cycles(path: &Path, graph_file: Option<&Path>) -> Result<()> {
    let (graph, _diagnostics) = load_or_build_graph(path, graph_file)?;
    let graph = &graph;
    let query = Query::new(graph);
    let cycles = query.cycles(&[]);

    println!("Dependency cycles:");
    if cycles.is_empty() {
        println!("No dependency cycles detected.");
    } else {
        for (idx, component) in cycles.iter().enumerate() {
            println!("  cycle {}: {} node(s)", idx + 1, component.len());
            for node in component {
                println!("    - {} [{}]", node.name, node.kind.as_str());
                println!("      {}", node.id);
            }
        }
    }
    Ok(())
}
