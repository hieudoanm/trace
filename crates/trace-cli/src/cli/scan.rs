//! `scan` command implementation.

use std::path::Path;

use anyhow::{Context, Result};
use trace_analysis::{cache::SyntaxCache, config::TraceConfig, graph::GraphBuilder};
use trace_core::Language;
use trace_export::{to_dot, to_json_string, LanguageInfo};
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
        !no_cache,
        "off",
        &config.analysis.languages,
    );

    let content = if output_format == "dot" {
        to_dot(&result.graph)
    } else {
        to_json_string(&document)
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

    if write_trace_dir {
        let trace_dir = path.join(".trace");
        std::fs::create_dir_all(&trace_dir)?;
        let ext = if output_format == "dot" {
            "dot"
        } else {
            "json"
        };
        let output_path = trace_dir.join(format!("graph.{ext}"));
        std::fs::write(&output_path, &content)
            .with_context(|| format!("failed to write output to {:?}", output_path))?;
        if output_format != "dot" {
            let html_path = trace_dir.join("graph.html");
            let html = include_str!("../graph_template.html");
            std::fs::write(&html_path, html).ok();
        }
    }

    Ok(())
}
