//! `scan` command implementation.

use std::path::Path;

use anyhow::{Context, Result};
use trace_analysis::framework::Framework;
use trace_analysis::{cache::SyntaxCache, config::TraceConfig, graph::GraphBuilder};
use trace_core::{Diagnostic, Graph, Language};
use trace_export::{FrameworkInfo, GraphDocument, LanguageInfo};
use trace_parser::discovery::DiscoveryOptions;

/// Build a graph from a repository path with discovery and parsing options.
pub(crate) fn build_graph_for_path(
    path: &Path,
    include: Option<&str>,
    exclude: Option<&str>,
    language: Option<&str>,
    no_cache: bool,
    jobs: Option<usize>,
    config: Option<&std::path::Path>,
) -> Result<(trace_analysis::graph::GraphBuildResult, TraceConfig)> {
    let (config, _config_source) =
        TraceConfig::load(path, config).context("failed to load configuration")?;

    let mut options = DiscoveryOptions::default();
    if let Some(include) = include {
        options.includes = vec![include.to_string()];
    }
    if let Some(exclude) = exclude {
        options.excludes = vec![exclude.to_string()];
    }
    if let Some(language) = language {
        let mut languages = Vec::new();
        for name in language.split(',') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let lang = Language::parse_name(name)
                .with_context(|| format!("unsupported language: {name}"))?;
            if !languages.contains(&lang) {
                languages.push(lang);
            }
        }
        options.languages = languages;
    }
    options.max_file_size = config.analysis.max_file_size;
    options.follow_symlinks = config.analysis.follow_symlinks;
    options.include_generated = config.analysis.include_generated;

    let cache = SyntaxCache::from_config(path, &config, no_cache);
    let parallel = jobs.unwrap_or(0) != 1;

    let builder = GraphBuilder {
        include_external: true,
        cache: if cache.enabled() { Some(cache) } else { None },
        parallel,
    };

    let result = builder
        .build_repository_with_options(path, &options)
        .context("failed to build graph")?;

    Ok((result, config))
}

/// Whether the syntax cache will actually be used for this run.
pub(crate) fn cache_enabled(config: &TraceConfig, no_cache: bool) -> bool {
    config.cache.enabled && !no_cache
}

fn framework_info(framework: &Framework) -> FrameworkInfo {
    FrameworkInfo {
        name: framework.name.clone(),
        display_name: framework.display_name.clone(),
        family: framework.family.as_str().to_string(),
        version: framework.version.clone(),
        confidence: framework.confidence.as_str().to_string(),
    }
}

fn supported_languages() -> Vec<LanguageInfo> {
    trace_analysis::capabilities::all_capabilities()
        .iter()
        .map(|cap| LanguageInfo {
            language: cap.language.as_str().to_string(),
            grammar: cap.grammar.clone(),
        })
        .collect()
}

/// Repository root as stored in exports. Absolute paths under the current
/// working directory are relativized so exports stay portable; anything else
/// is kept verbatim.
fn repository_root(path: &Path) -> String {
    if path.is_absolute() {
        if let (Ok(cwd), Ok(absolute)) = (std::env::current_dir(), path.canonicalize()) {
            let cwd = cwd.canonicalize().unwrap_or(cwd);
            if let Ok(relative) = absolute.strip_prefix(&cwd) {
                let relative = relative.to_string_lossy().replace('\\', "/");
                return if relative.is_empty() {
                    ".".to_string()
                } else {
                    relative
                };
            }
        }
    }
    path.to_string_lossy().replace('\\', "/")
}

/// Build the versioned export document shared by `scan` and `graph`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_document(
    graph: &Graph,
    diagnostics: &[Diagnostic],
    frameworks: &[Framework],
    config: &TraceConfig,
    path: &Path,
    cache_enabled: bool,
) -> GraphDocument {
    let languages = supported_languages();
    let framework_infos: Vec<FrameworkInfo> = frameworks.iter().map(framework_info).collect();
    trace_export::export_graph(
        graph,
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository"),
        &repository_root(path),
        diagnostics,
        Some(graph.stats(diagnostics)),
        &languages,
        &framework_infos,
        cache_enabled,
        config.analysis.semantic.as_str(),
        &config.analysis.languages,
    )
}

/// Print a human-readable scan summary to stdout.
fn print_summary(
    stats: &trace_core::graph::Stats,
    languages: &[trace_export::LanguageInfo],
    frameworks: &[trace_export::FrameworkInfo],
) {
    println!("=== Trace scan summary ===");
    println!("Files scanned: {}", stats.files);
    println!("Packages: {}", stats.packages);
    println!("Nodes: {}", stats.nodes);
    println!("Edges: {}", stats.edges);
    println!("Symbols: {}", stats.symbols);
    println!("Unresolved edges: {}", stats.unresolved_edges);
    println!("Diagnostics: {}", stats.diagnostics);

    if !stats.by_language.is_empty() {
        println!();
        println!("Languages detected:");
        for (lang, count) in &stats.by_language {
            println!("  {lang}: {count} nodes");
        }
    }

    if !frameworks.is_empty() {
        println!();
        println!("Frameworks detected:");
        for fw in frameworks {
            println!(
                "  {} (family={}, confidence={}, version={:?})",
                fw.name, fw.family, fw.confidence, fw.version
            );
        }
    }

    if !stats.by_node_kind.is_empty() {
        println!();
        println!("Nodes by kind:");
        for (kind, count) in &stats.by_node_kind {
            println!("  {kind}: {count}");
        }
    }

    if !stats.by_edge_kind.is_empty() {
        println!();
        println!("Edges by kind:");
        for (kind, count) in &stats.by_edge_kind {
            println!("  {kind}: {count}");
        }
    }

    if !stats.by_status.is_empty() {
        println!();
        println!("Resolution status:");
        for (status, count) in &stats.by_status {
            println!("  {status}: {count}");
        }
    }

    if !stats.by_framework.is_empty() {
        println!();
        println!("Nodes by framework:");
        for (fw, count) in &stats.by_framework {
            println!("  {fw}: {count}");
        }
    }

    let supported: Vec<&str> = languages.iter().map(|l| l.language.as_str()).collect();
    if !supported.is_empty() {
        println!();
        println!("Supported languages: {}", supported.join(", "));
    }

    println!("=== Scan complete ===");
}

/// Execute the `scan` command.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_scan(
    path: &Path,
    output: Option<&std::path::Path>,
    include: Option<&str>,
    exclude: Option<&str>,
    language: Option<&str>,
    no_cache: bool,
    jobs: Option<usize>,
    config: Option<&std::path::Path>,
    write_trace_dir: bool,
) -> Result<()> {
    let (result, config) =
        build_graph_for_path(path, include, exclude, language, no_cache, jobs, config)?;

    let languages = supported_languages();
    let framework_infos: Vec<FrameworkInfo> =
        result.frameworks.iter().map(framework_info).collect();
    let stats = result.graph.stats(&result.diagnostics);

    let document = build_document(
        &result.graph,
        &result.diagnostics,
        &result.frameworks,
        &config,
        path,
        cache_enabled(&config, no_cache),
    );

    let json_content = trace_export::to_json_string(&document);
    let dot_content = trace_export::to_dot(&result.graph);
    let html_content = include_str!("../graph_template.html");

    // Output summary to stdout if output path is not provided
    if output.is_none() {
        print_summary(&stats, &languages, &framework_infos);
    }

    // Output all formats if an output path is provided
    if let Some(output_path) = output {
        let is_dir = output_path.is_dir()
            || output_path
                .to_str()
                .is_some_and(|s| s.ends_with('/') || s.ends_with('\\'));
        if is_dir {
            std::fs::create_dir_all(output_path)
                .with_context(|| format!("failed to create output directory {:?}", output_path))?;
            std::fs::write(output_path.join("graph.json"), &json_content)
                .with_context(|| format!("failed to write {:?}", output_path.join("graph.json")))?;
            std::fs::write(output_path.join("graph.dot"), &dot_content)
                .with_context(|| format!("failed to write {:?}", output_path.join("graph.dot")))?;
            std::fs::write(output_path.join("graph.html"), html_content)
                .with_context(|| format!("failed to write {:?}", output_path.join("graph.html")))?;
        } else {
            if let Some(parent) = output_path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let json_path = output_path.with_extension("json");
            let dot_path = output_path.with_extension("dot");
            let html_path = output_path.with_extension("html");

            std::fs::write(&json_path, &json_content)
                .with_context(|| format!("failed to write {:?}", json_path))?;
            std::fs::write(&dot_path, &dot_content)
                .with_context(|| format!("failed to write {:?}", dot_path))?;
            std::fs::write(&html_path, html_content)
                .with_context(|| format!("failed to write {:?}", html_path))?;
        }
    }

    // Write all formats (.json, .dot, .html) to .trace/ directory if write_trace_dir is enabled
    if write_trace_dir {
        let trace_dir = path.join(".trace");
        std::fs::create_dir_all(&trace_dir)
            .with_context(|| format!("failed to create trace directory {:?}", trace_dir))?;

        let json_path = trace_dir.join("graph.json");
        let dot_path = trace_dir.join("graph.dot");
        let html_path = trace_dir.join("graph.html");

        std::fs::write(&json_path, &json_content)
            .with_context(|| format!("failed to write {:?}", json_path))?;
        std::fs::write(&dot_path, &dot_content)
            .with_context(|| format!("failed to write {:?}", dot_path))?;
        std::fs::write(&html_path, html_content)
            .with_context(|| format!("failed to write {:?}", html_path))?;
    }

    Ok(())
}
