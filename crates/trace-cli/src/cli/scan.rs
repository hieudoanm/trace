//! `scan` command implementation.

use std::path::Path;

use anyhow::{Context, Result};
use trace_analysis::{cache::SyntaxCache, config::TraceConfig, graph::GraphBuilder};
use trace_core::Language;
use trace_export::{to_dot, to_json_string, FrameworkInfo, LanguageInfo};
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
        let lang = Language::parse_name(language)
            .with_context(|| format!("unsupported language: {language}"))?;
        options.languages = vec![lang];
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

/// Execute the `scan` command.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_scan(
    path: &Path,
    output_format: &str,
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

    let languages = trace_analysis::capabilities::all_capabilities()
        .iter()
        .map(|cap| LanguageInfo {
            language: cap.language.as_str().to_string(),
            grammar: cap.grammar.clone(),
        })
        .collect::<Vec<_>>();

    let document = trace_export::export_graph(
        &result.graph,
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository"),
        path.to_str().unwrap_or("."),
        &result.diagnostics,
        Some(result.graph.stats(&result.diagnostics)),
        &languages,
        &result
            .frameworks
            .iter()
            .map(|fw| FrameworkInfo {
                name: fw.name.clone(),
                display_name: fw.display_name.clone(),
                family: fw.family.as_str().to_string(),
                version: fw.version.clone(),
                confidence: fw.confidence.as_str().to_string(),
            })
            .collect::<Vec<_>>(),
        !no_cache,
        "off",
        &config.analysis.languages,
    );

    let json_content = to_json_string(&document);
    let dot_content = to_dot(&result.graph);
    let html_content = include_str!("../graph_template.html");

    // Output to stdout if output path is not provided
    if output.is_none() {
        let content = if output_format == "dot" {
            &dot_content
        } else {
            &json_content
        };
        println!("{content}");
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
