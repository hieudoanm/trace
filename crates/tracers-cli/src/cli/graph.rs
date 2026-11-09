//! `graph` command implementation.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use tracers_core::{Graph, Language, NodeFilter, NodeKind};

use super::scan::{build_document, build_graph_for_path, cache_enabled};

/// Build a node filter from `--kind` and `--language` selectors.
fn node_filter(kind: Option<&str>, language: Option<&str>) -> Result<NodeFilter> {
    let mut filter = NodeFilter::default();
    if let Some(kind_name) = kind {
        for name in kind_name.split(',') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let kind =
                NodeKind::parse_name(name).ok_or_else(|| anyhow!("unknown node kind `{name}`"))?;
            if !filter.kinds.contains(&kind) {
                filter.kinds.push(kind);
            }
        }
    }
    if let Some(language) = language {
        for name in language.split(',') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let language = Language::parse_name(name)
                .ok_or_else(|| anyhow!("unsupported language: {name}"))?;
            if !filter.languages.contains(&language) {
                filter.languages.push(language);
            }
        }
    }
    Ok(filter)
}

/// Return a filtered graph view, or `None` when no filters are set.
fn apply_filter(graph: &Graph, filter: &NodeFilter) -> Result<Option<Graph>> {
    if filter.kinds.is_empty() && filter.languages.is_empty() {
        return Ok(None);
    }
    let (nodes, edges) = graph.filtered(filter);
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

fn write_or_print(output: Option<&std::path::Path>, content: &str) -> Result<()> {
    match output {
        Some(output_path) => {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(output_path, content)
                .with_context(|| format!("failed to write output to {:?}", output_path))?;
        }
        None => {
            println!("{content}");
        }
    }
    Ok(())
}

/// Emit `graph` from an already-exported document, applying node filters.
fn graph_from_file(
    file: &Path,
    format: &str,
    output: Option<&std::path::Path>,
    filter: &NodeFilter,
) -> Result<()> {
    let mut document = tracers_export::load_document(file)?;
    let graph = tracers_export::graph_from_document(&document)?;
    let view = apply_filter(&graph, filter)?;
    let graph = view.as_ref().unwrap_or(&graph);

    let content = if format == "dot" {
        tracers_export::to_dot(graph)
    } else {
        document.nodes = graph.nodes().to_vec();
        document.edges = graph.edges().to_vec();
        document.repository.file_count = document.file_count();
        document.statistics = Some(graph.stats(&document.diagnostics));
        tracers_export::to_json_string(&document)
    };
    write_or_print(output, &content)
}

pub(crate) fn cmd_graph(
    path: &Path,
    output_format: Option<&str>,
    output: Option<&std::path::Path>,
    language: Option<&str>,
    kind: Option<&str>,
    graph_file: Option<&Path>,
) -> Result<()> {
    let filter = node_filter(kind, language)?;

    // The format default lives in tracers.toml; load it cheaply even when the
    // graph itself comes from an exported file.
    let (config, _source) = tracers_analysis::config::TracersConfig::load(path, None)
        .context("failed to load configuration")?;
    let format = output_format
        .map(str::to_string)
        .unwrap_or_else(|| config.output.format.clone());
    if format != "json" && format != "dot" {
        anyhow::bail!("unsupported graph format `{format}` (expected `json` or `dot`)");
    }

    if let Some(file) = graph_file {
        return graph_from_file(file, &format, output, &filter);
    }

    let (result, config) = build_graph_for_path(path, &[], &[], language, false, None, None)?;
    let filtered = apply_filter(&result.graph, &filter)?;
    let graph = filtered.as_ref().unwrap_or(&result.graph);

    let content = if format == "dot" {
        tracers_export::to_dot(graph)
    } else {
        tracers_export::to_json_string(&build_document(
            graph,
            &result.diagnostics,
            &result.frameworks,
            &config,
            path,
            cache_enabled(&config, false),
            &super::scan::enabled_language_names(&result.languages, &config),
        ))
    };

    write_or_print(output, &content)
}
