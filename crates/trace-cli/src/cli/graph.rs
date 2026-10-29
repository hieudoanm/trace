//! `graph` command implementation.

use std::path::Path;

use anyhow::{Context, Result};

use super::scan::build_graph_for_path;

pub(crate) fn cmd_graph(
    path: &Path,
    output_format: &str,
    output: Option<&std::path::Path>,
    language: Option<&str>,
    _kind: Option<&str>,
) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, language, false, None, None)?;

    let content = if output_format == "dot" {
        trace_export::to_dot(&result.graph)
    } else {
        trace_export::to_json_string(&trace_export::export_graph(
            &result.graph,
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repository"),
            path.to_str().unwrap_or("."),
            &result.diagnostics,
            Some(result.graph.stats(&result.diagnostics)),
            &[],
            false,
            "off",
            &[],
        ))
    };

    match output {
        Some(output_path) => {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(output_path, &content)
                .with_context(|| format!("failed to write output to {:?}", output_path))?;
        }
        None => {
            println!("{content}");
        }
    }

    Ok(())
}
