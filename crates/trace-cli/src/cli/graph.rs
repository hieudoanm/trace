//! `graph` command implementation.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use trace_core::{Graph, NodeFilter, NodeKind};

use super::scan::{build_document, build_graph_for_path, cache_enabled};

/// Apply the `--kind` node filter, returning a filtered graph view when set.
fn apply_kind_filter(graph: &Graph, kind: Option<&str>) -> Result<Option<Graph>> {
    let Some(kind_name) = kind else {
        return Ok(None);
    };
    let kind = NodeKind::parse_name(kind_name)
        .ok_or_else(|| anyhow!("unknown node kind `{kind_name}`"))?;
    let filter = NodeFilter {
        kinds: vec![kind],
        ..NodeFilter::default()
    };
    let (nodes, edges) = graph.filtered(&filter);
    let mut view = Graph::new();
    for node in nodes {
        view.add_node(node)?;
    }
    for edge in edges {
        view.add_edge(edge)?;
    }
    view.finalize();
    Ok(Some(view))
}

pub(crate) fn cmd_graph(
    path: &Path,
    output_format: Option<&str>,
    output: Option<&std::path::Path>,
    language: Option<&str>,
    kind: Option<&str>,
) -> Result<()> {
    let (result, config) = build_graph_for_path(path, None, None, language, false, None, None)?;

    let format = output_format
        .map(str::to_string)
        .unwrap_or_else(|| config.output.format.clone());
    if format != "json" && format != "dot" {
        anyhow::bail!("unsupported graph format `{format}` (expected `json` or `dot`)");
    }

    let filtered = apply_kind_filter(&result.graph, kind)?;
    let graph = filtered.as_ref().unwrap_or(&result.graph);

    let content = if format == "dot" {
        trace_export::to_dot(graph)
    } else {
        trace_export::to_json_string(&build_document(
            graph,
            &result.diagnostics,
            &result.frameworks,
            &config,
            path,
            cache_enabled(&config, false),
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
