use std::path::Path;

use anyhow::Result;

use crate::cli::scan::load_or_build_graph;

pub(crate) fn cmd_diagnostics(
    path: &Path,
    max_results: usize,
    graph_file: Option<&Path>,
) -> Result<()> {
    let (_graph, diagnostics) = load_or_build_graph(path, graph_file)?;

    println!("Analysis diagnostics ({} total):", diagnostics.len());
    if diagnostics.is_empty() {
        println!("  (none)");
        return Ok(());
    }
    for diagnostic in diagnostics.iter().take(max_results) {
        let location = match (&diagnostic.file, &diagnostic.range) {
            (Some(file), Some(range)) => format!(" [{file}:{}]", range.start.line),
            (Some(file), None) => format!(" [{file}]"),
            _ => String::new(),
        };
        println!(
            "  {} {}: {}{}",
            diagnostic.severity.as_str(),
            diagnostic.kind.as_str(),
            diagnostic.message,
            location
        );
    }
    if diagnostics.len() > max_results {
        println!(
            "  … {} more; increase --max-results to see them",
            diagnostics.len() - max_results
        );
    }
    Ok(())
}
