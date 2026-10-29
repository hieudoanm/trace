//! Validation and info commands.

use std::path::Path;

use anyhow::{Context, Result};
use trace_core::Query;
use trace_parser::discovery::{discover, DiscoveryOptions};

use super::scan::build_graph_for_path;

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

pub(crate) fn cmd_languages() -> Result<()> {
    let capabilities = trace_analysis::capabilities::all_capabilities();
    let mut languages: Vec<&str> = capabilities
        .iter()
        .map(|cap| cap.language.display_name())
        .collect();
    languages.sort();
    languages.dedup();

    println!("Supported languages:");
    for name in &languages {
        println!("  {name}");
    }
    println!("\nCapability matrix:");
    for cap in &capabilities {
        println!(
            "  {} ({}): syntax={}, symbols={}, imports={}, calls={}, cross-file={}",
            cap.language.display_name(),
            cap.grammar,
            cap.syntax_parsing,
            cap.symbol_extraction,
            cap.import_resolution,
            cap.call_resolution,
            cap.cross_file_references,
        );
    }
    Ok(())
}

pub(crate) fn cmd_check(path: &Path) -> Result<()> {
    let (config, config_source) = trace_analysis::config::TraceConfig::load(path, None)
        .context("failed to load configuration")?;

    println!("Configuration: valid");
    if let Some(source) = &config_source {
        println!("  source: {source}");
    }

    println!("\nDiscovery:");
    let options = DiscoveryOptions {
        max_file_size: config.analysis.max_file_size,
        follow_symlinks: config.analysis.follow_symlinks,
        include_generated: config.analysis.include_generated,
        languages: config.languages(),
        ..DiscoveryOptions::default()
    };
    let report = discover(path, &options).map_err(anyhow::Error::msg)?;
    println!("  root:        {}", report.root.display());
    println!("  files:       {}", report.files.len());
    println!("  skipped:     {}", report.skipped.len());
    println!("  unsupported: {}", report.unsupported_files);

    let mut languages: Vec<trace_core::Language> =
        report.files.iter().map(|file| file.language).collect();
    languages.sort();
    languages.dedup();

    println!("\nLanguages:");
    if languages.is_empty() {
        println!("  (none detected)");
    } else {
        for language in &languages {
            let count = report
                .files
                .iter()
                .filter(|f| f.language == *language)
                .count();
            println!("  {}: {} file(s)", language.display_name(), count);
        }
    }

    let capabilities = trace_analysis::capabilities::all_capabilities();
    println!("\nParser availability:");
    for cap in &capabilities {
        let status = if cap.syntax_parsing { "ok" } else { "missing" };
        println!(
            "  {} [{}]: {}",
            cap.language.display_name(),
            cap.grammar,
            status
        );
    }

    println!("\nCheck passed.");
    Ok(())
}

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
