use std::path::Path;

use anyhow::Result;
use trace_core::Query;

use crate::cli::scan::build_graph_for_path;

pub(crate) fn cmd_cycles(path: &Path) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let graph = &result.graph;
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
            }
        }
    }
    Ok(())
}
