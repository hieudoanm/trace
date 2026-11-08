//! `scan` command implementation.

use std::path::Path;

use anyhow::{Context, Result};
use tracers_analysis::framework::Framework;
use tracers_analysis::{cache::SyntaxCache, config::TracersConfig, graph::GraphBuilder};
use tracers_core::{Diagnostic, Graph, Language};
use tracers_export::{FrameworkInfo, GraphDocument, LanguageInfo};
use tracers_parser::discovery::DiscoveryOptions;

/// Build a graph from a repository path with discovery and parsing options.
/// Merge `tracers.toml` discovery defaults with CLI filters.
///
/// Config `include`/`exclude` patterns are defaults that CLI flags extend.
/// For languages the CLI flag wins outright so `--language` always narrows
/// the scan.
pub(crate) fn discovery_options(
    config: &TracersConfig,
    include: &[String],
    exclude: &[String],
    language: Option<&str>,
) -> Result<DiscoveryOptions> {
    let mut includes = config.analysis.include.clone();
    includes.extend(include.iter().cloned());
    let mut excludes = config.analysis.exclude.clone();
    excludes.extend(exclude.iter().cloned());
    let languages = if let Some(language) = language {
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
        languages
    } else {
        config.languages()
    };
    Ok(DiscoveryOptions {
        includes,
        excludes,
        languages,
        max_file_size: config.analysis.max_file_size,
        follow_symlinks: config.analysis.follow_symlinks,
        include_generated: config.analysis.include_generated,
    })
}

/// Build a graph from a repository path with discovery and parsing options.
pub(crate) fn build_graph_for_path(
    path: &Path,
    include: &[String],
    exclude: &[String],
    language: Option<&str>,
    no_cache: bool,
    jobs: Option<usize>,
    config: Option<&std::path::Path>,
) -> Result<(tracers_analysis::graph::GraphBuildResult, TracersConfig)> {
    let (config, _config_source) =
        TracersConfig::load(path, config).context("failed to load configuration")?;

    let options = discovery_options(&config, include, exclude, language)?;

    let cache = SyntaxCache::from_config(path, &config, no_cache);
    // CLI --jobs wins over analysis.jobs; absent means the default pool.
    let jobs = jobs.or(config.analysis.jobs);
    if jobs == Some(0) {
        anyhow::bail!("--jobs must be at least 1");
    }

    let builder = GraphBuilder {
        include_external: true,
        cache: if cache.enabled() { Some(cache) } else { None },
        jobs,
    };

    let result = builder
        .build_repository_with_options(path, &options)
        .context("failed to build graph")?;

    Ok((result, config))
}

/// Load a graph from an exported JSON file, or build it from source.
/// Returns the graph and its diagnostics (empty frameworks are not
/// reconstructible from the document, so callers must not rely on them).
pub(crate) fn load_or_build_graph(
    path: &Path,
    graph_file: Option<&Path>,
) -> Result<(tracers_core::Graph, Vec<tracers_core::Diagnostic>)> {
    if let Some(file) = graph_file {
        let document = tracers_export::load_document(file)?;
        let graph = tracers_export::graph_from_document(&document)?;
        return Ok((graph, document.diagnostics));
    }
    let (result, _config) = build_graph_for_path(path, &[], &[], None, false, None, None)?;
    Ok((result.graph, result.diagnostics))
}

/// Embed the graph document into the HTML explorer so `graph.html` works
/// standalone (e.g. opened directly from disk). `<` is escaped as `<`
/// inside the JSON string, which is valid JSON and cannot terminate the
/// surrounding `<script>` element.
fn graph_html(template: &str, json: &str) -> String {
    let data = json.replace('<', "\\u003c");
    let injection = format!("<script>window.__TRACERS_DATA__ = {data};</script>\n");
    match template.find("</head>") {
        Some(index) => {
            let (head, rest) = template.split_at(index);
            format!("{head}{injection}{rest}")
        }
        None => format!("{injection}{template}"),
    }
}

/// Whether the syntax cache will actually be used for this run.
pub(crate) fn cache_enabled(config: &TracersConfig, no_cache: bool) -> bool {
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
    tracers_analysis::capabilities::all_capabilities()
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

/// Names of the languages enabled for a build, for export metadata.
/// Falls back to the raw config list when the build did not record them.
pub(crate) fn enabled_language_names(
    result_languages: &[tracers_core::Language],
    config: &TracersConfig,
) -> Vec<String> {
    if result_languages.is_empty() {
        return config.analysis.languages.clone();
    }
    let mut names: Vec<String> = result_languages
        .iter()
        .map(|language| language.as_str().to_string())
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Build the versioned export document shared by `scan` and `graph`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_document(
    graph: &Graph,
    diagnostics: &[Diagnostic],
    frameworks: &[Framework],
    config: &TracersConfig,
    path: &Path,
    cache_enabled: bool,
    enabled_languages: &[String],
) -> GraphDocument {
    let languages = supported_languages();
    let framework_infos: Vec<FrameworkInfo> = frameworks.iter().map(framework_info).collect();
    tracers_export::export_graph(
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
        enabled_languages,
    )
}

/// Print a human-readable scan summary to stdout.
fn print_summary(
    stats: &tracers_core::graph::Stats,
    languages: &[tracers_export::LanguageInfo],
    frameworks: &[tracers_export::FrameworkInfo],
) {
    println!("=== Tracers scan summary ===");
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
    include: &[String],
    exclude: &[String],
    language: Option<&str>,
    no_cache: bool,
    jobs: Option<usize>,
    config: Option<&std::path::Path>,
    write_tracers_dir: bool,
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
        &enabled_language_names(&result.languages, &config),
    );

    let json_content = tracers_export::to_json_string(&document);
    let dot_content = tracers_export::to_dot(&result.graph);
    let html_content = graph_html(include_str!("../graph_template.html"), &json_content);

    // Output summary to stdout if output path is not provided
    if output.is_none() {
        print_summary(&stats, &languages, &framework_infos);
        if let Some(counts) = &result.cache_counts {
            println!(
                "Cache: {} hits, {} misses, {} writes",
                counts.hits, counts.misses, counts.writes
            );
        }
        if let Some(discovery) = &result.discovery {
            if discovery.skipped > 0 || discovery.unsupported_files > 0 {
                let reasons = discovery
                    .skip_reasons
                    .iter()
                    .map(|(reason, count)| format!("{reason}={count}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                println!(
                    "Discovery: {} files, {} skipped ({}), {} unsupported, {} gitignored",
                    discovery.files,
                    discovery.skipped,
                    reasons,
                    discovery.unsupported_files,
                    discovery.ignored_files,
                );
            }
        }
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
            std::fs::write(output_path.join("graph.html"), html_content.as_bytes())
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
            std::fs::write(&html_path, html_content.as_bytes())
                .with_context(|| format!("failed to write {:?}", html_path))?;
        }
    }

    // Write all formats (.json, .dot, .html) to .tracers/ directory if write_tracers_dir is enabled
    if write_tracers_dir {
        let tracers_dir = path.join(".tracers");
        std::fs::create_dir_all(&tracers_dir)
            .with_context(|| format!("failed to create tracers directory {:?}", tracers_dir))?;

        let json_path = tracers_dir.join("graph.json");
        let dot_path = tracers_dir.join("graph.dot");
        let html_path = tracers_dir.join("graph.html");

        std::fs::write(&json_path, &json_content)
            .with_context(|| format!("failed to write {:?}", json_path))?;
        std::fs::write(&dot_path, &dot_content)
            .with_context(|| format!("failed to write {:?}", dot_path))?;
        std::fs::write(&html_path, html_content.as_bytes())
            .with_context(|| format!("failed to write {:?}", html_path))?;

        println!("Wrote graph export to {}", tracers_dir.display());
    }

    Ok(())
}
