use std::path::{Path, PathBuf};
use std::process::exit;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};

use trace_analysis::{
    cache::SyntaxCache,
    capabilities::all_capabilities,
    config::{TraceConfig, DEFAULT_CACHE_DIR},
    graph::GraphBuilder,
};
use trace_export::{to_dot, to_json_string, LanguageInfo};
use trace_parser::discovery::DiscoveryOptions;

/// Trace: a high-performance, multi-language code intelligence and graph
/// analysis tool.
#[derive(Parser, Debug)]
#[command(name = "trace", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging.
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Silence all non-error output.
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyse a repository and generate a graph.
    Scan {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output format: json or dot.
        #[arg(short = 'f', long, default_value = "json")]
        format: String,

        /// Write output to PATH instead of stdout.
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Include only files matching this glob pattern.
        #[arg(long)]
        include: Option<String>,

        /// Exclude files matching this glob pattern.
        #[arg(long)]
        exclude: Option<String>,

        /// Only analyse the given language(s).
        #[arg(long)]
        language: Option<String>,

        /// Disable the syntax cache.
        #[arg(long)]
        no_cache: bool,

        /// Number of parallel jobs.
        #[arg(long)]
        jobs: Option<usize>,

        /// Path to a trace.toml configuration file.
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Load or construct a graph and provide filtered graph output.
    Graph {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output format: json or dot.
        #[arg(short = 'f', long, default_value = "json")]
        format: String,

        /// Write output to PATH instead of stdout.
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Filter by language.
        #[arg(long)]
        language: Option<String>,

        /// Filter by node kind.
        #[arg(long)]
        kind: Option<String>,
    },
    /// Display callers of a symbol.
    Callers {
        /// Symbol selector (ID, qualified name, or short name).
        symbol: String,

        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Maximum traversal depth.
        #[arg(long, default_value = "32")]
        max_depth: usize,

        /// Maximum number of results.
        #[arg(long, default_value = "1000")]
        max_results: usize,
    },
    /// Display callees of a symbol.
    Callees {
        /// Symbol selector (ID, qualified name, or short name).
        symbol: String,

        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Maximum traversal depth.
        #[arg(long, default_value = "32")]
        max_depth: usize,

        /// Maximum number of results.
        #[arg(long, default_value = "1000")]
        max_results: usize,
    },
    /// List references to a symbol.
    Refs {
        /// Symbol selector (ID, qualified name, or short name).
        symbol: String,

        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Maximum traversal depth.
        #[arg(long, default_value = "32")]
        max_depth: usize,

        /// Maximum number of results.
        #[arg(long, default_value = "1000")]
        max_results: usize,
    },
    /// Display direct or transitive dependencies of a target.
    Deps {
        /// Target symbol or file selector.
        target: String,

        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Follow transitive dependencies.
        #[arg(long)]
        transitive: bool,

        /// Maximum traversal depth.
        #[arg(long, default_value = "32")]
        max_depth: usize,
    },
    /// Display static impact analysis for a target.
    Impact {
        /// Target symbol or file selector.
        target: String,

        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Maximum traversal depth.
        #[arg(long, default_value = "32")]
        max_depth: usize,

        /// Maximum number of results.
        #[arg(long, default_value = "1000")]
        max_results: usize,
    },
    /// Find a path between two symbols or nodes.
    Path {
        /// Source symbol or node selector.
        source: String,

        /// Target symbol or node selector.
        target: String,

        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Maximum traversal depth.
        #[arg(long, default_value = "32")]
        max_depth: usize,
    },
    /// Detect dependency cycles.
    Cycles {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Report graph statistics.
    Stats {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Report supported languages and analysis capabilities.
    Languages,
    /// Validate configuration, discovery, and parser availability.
    Check {
        /// Repository path to check (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else if cli.quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::new("warn")
    };
    fmt().with_env_filter(filter).init();

    match cli.command {
        Commands::Scan {
            path,
            format,
            output,
            include,
            exclude,
            language,
            no_cache,
            jobs,
            config,
        } => cmd_scan(
            &path,
            &format,
            output.as_deref(),
            include.as_deref(),
            exclude.as_deref(),
            language.as_deref(),
            no_cache,
            jobs,
            config.as_deref(),
        ),
        Commands::Graph {
            path,
            format,
            output,
            language,
            kind,
        } => cmd_graph(
            &path,
            &format,
            output.as_deref(),
            language.as_deref(),
            kind.as_deref(),
        ),
        Commands::Callers {
            symbol,
            path,
            max_depth,
            max_results,
        } => cmd_callers(&symbol, &path, max_depth, max_results),
        Commands::Callees {
            symbol,
            path,
            max_depth,
            max_results,
        } => cmd_callees(&symbol, &path, max_depth, max_results),
        Commands::Refs {
            symbol,
            path,
            max_depth,
            max_results,
        } => cmd_refs(&symbol, &path, max_depth, max_results),
        Commands::Deps {
            target,
            path,
            transitive,
            max_depth,
        } => cmd_deps(&target, &path, transitive, max_depth),
        Commands::Impact {
            target,
            path,
            max_depth,
            max_results,
        } => cmd_impact(&target, &path, max_depth, max_results),
        Commands::Path {
            source,
            target,
            path,
            max_depth,
        } => cmd_path(&source, &target, &path, max_depth),
        Commands::Cycles { path } => cmd_cycles(&path),
        Commands::Stats { path } => cmd_stats(&path),
        Commands::Languages => cmd_languages(),
        Commands::Check { path } => cmd_check(&path),
    }
}

fn build_graph_for_path(
    path: &Path,
    include: Option<&str>,
    exclude: Option<&str>,
    language: Option<&str>,
    no_cache: bool,
    jobs: Option<usize>,
    _config: Option<&std::path::Path>,
) -> Result<(trace_analysis::graph::GraphBuildResult, TraceConfig)> {
    let (config, _config_source) =
        TraceConfig::load(path, _config).context("failed to load configuration")?;

    let mut options = DiscoveryOptions::default();
    if let Some(include) = include {
        options.includes = vec![include.to_string()];
    }
    if let Some(exclude) = exclude {
        options.excludes = vec![exclude.to_string()];
    }
    if let Some(language) = language {
        let lang = trace_core::Language::parse_name(language)
            .with_context(|| format!("unsupported language: {language}"))?;
        options.languages = vec![lang];
    }
    options.max_file_size = config.analysis.max_file_size;
    options.follow_symlinks = config.analysis.follow_symlinks;
    options.include_generated = config.analysis.include_generated;

    let cache = SyntaxCache::from_config(path, &config, no_cache);
    let parallel = jobs.unwrap_or(0) != 1; // Disable parallel if jobs is explicitly 1

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

#[allow(clippy::too_many_arguments)]
fn cmd_scan(
    path: &Path,
    output_format: &str,
    output: Option<&std::path::Path>,
    include: Option<&str>,
    exclude: Option<&str>,
    language: Option<&str>,
    no_cache: bool,
    jobs: Option<usize>,
    config: Option<&std::path::Path>,
) -> Result<()> {
    let (result, config) =
        build_graph_for_path(path, include, exclude, language, no_cache, jobs, config)?;

    let languages = all_capabilities()
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
            let trace_dir = path.join(".trace");
            std::fs::create_dir_all(&trace_dir)?;
            let ext = if output_format == "dot" { "dot" } else { "json" };
            let output_path = trace_dir.join(format!("graph.{ext}"));
            std::fs::write(&output_path, &content)
                .with_context(|| format!("failed to write output to {:?}", output_path))?;
            println!("Graph written to {}", output_path.display());
            if output_format != "dot" {
                let html_path = trace_dir.join("graph.html");
                let html = include_str!("graph_template.html");
                std::fs::write(&html_path, html).ok();
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_graph(
    path: &Path,
    format: &str,
    output: Option<&std::path::Path>,
    language: Option<&str>,
    kind: Option<&str>,
) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, language, false, None, None)?;

    let graph = &result.graph;
    let filter = if let Some(kind_str) = kind {
        let kind = trace_core::NodeKind::parse_name(kind_str)
            .with_context(|| format!("unsupported node kind: {kind_str}"))?;
        let lang = language
            .map(|l| {
                trace_core::Language::parse_name(l)
                    .with_context(|| format!("unsupported language: {l}"))
            })
            .transpose()?;
        trace_core::graph::NodeFilter {
            languages: lang.into_iter().collect(),
            kinds: vec![kind],
            ..Default::default()
        }
    } else if let Some(lang_str) = language {
        let lang = trace_core::Language::parse_name(lang_str)
            .with_context(|| format!("unsupported language: {lang_str}"))?;
        trace_core::graph::NodeFilter {
            languages: vec![lang],
            ..Default::default()
        }
    } else {
        trace_core::graph::NodeFilter::default()
    };

    let (nodes, edges) = graph.filtered(&filter);

    let languages = all_capabilities()
        .iter()
        .map(|cap| LanguageInfo {
            language: cap.language.as_str().to_string(),
            grammar: cap.grammar.clone(),
        })
        .collect::<Vec<_>>();

    let diagnostics = result.diagnostics.clone();
    let stats = Some(graph.stats(&diagnostics));

    let content = if format == "dot" {
        to_dot(graph)
    } else {
        let document = trace_export::GraphDocument {
            schema_version: trace_export::SCHEMA_VERSION.to_string(),
            trace_version: trace_export::TRACE_VERSION.to_string(),
            repository: trace_export::RepositoryMetadata {
                id: trace_core::id::repository_id(),
                name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("repository")
                    .to_string(),
                root: path.to_str().unwrap_or(".").to_string(),
                file_count: nodes
                    .iter()
                    .filter(|n| n.kind == trace_core::NodeKind::File)
                    .count(),
                language_count: languages.len(),
            },
            analysis: trace_export::AnalysisMetadata {
                format: format.to_string(),
                cache_enabled: true,
                semantic_mode: "off".to_string(),
                languages: vec![],
            },
            languages,
            nodes,
            edges,
            diagnostics,
            statistics: stats,
        };
        to_json_string(&document)
    };

    match output {
        Some(output_path) => {
            std::fs::write(output_path, &content)
                .with_context(|| format!("failed to write output to {:?}", output_path))?;
        }
        None => {
            println!("{content}");
        }
    }

    Ok(())
}

fn cmd_callers(symbol: &str, path: &Path, max_depth: usize, max_results: usize) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let query = trace_core::query::Query::new(&result.graph);
    let node = trace_core::query::resolve_unique(&result.graph, symbol)
        .map_err(|e| anyhow::anyhow!("cannot resolve symbol: {symbol}: {e}"))?;
    let options = trace_core::QueryOptions::default()
        .with_depth(max_depth)
        .with_limit(max_results);
    let callers = query.callers(node.id.as_str(), &options);
    print_neighbors("callers", symbol, &callers);
    Ok(())
}

fn cmd_callees(symbol: &str, path: &Path, max_depth: usize, max_results: usize) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let query = trace_core::query::Query::new(&result.graph);
    let node = trace_core::query::resolve_unique(&result.graph, symbol)
        .map_err(|e| anyhow::anyhow!("cannot resolve symbol: {symbol}: {e}"))?;
    let options = trace_core::QueryOptions::default()
        .with_depth(max_depth)
        .with_limit(max_results);
    let callees = query.callees(node.id.as_str(), &options);
    print_neighbors("callees", symbol, &callees);
    Ok(())
}

fn cmd_refs(symbol: &str, path: &Path, max_depth: usize, max_results: usize) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let query = trace_core::query::Query::new(&result.graph);
    let node = trace_core::query::resolve_unique(&result.graph, symbol)
        .map_err(|e| anyhow::anyhow!("cannot resolve symbol: {symbol}: {e}"))?;
    let options = trace_core::QueryOptions::default()
        .with_depth(max_depth)
        .with_limit(max_results);
    let refs = query.references(node.id.as_str(), &options);
    print_neighbors("references", symbol, &refs);
    Ok(())
}

fn cmd_deps(target: &str, path: &Path, transitive: bool, max_depth: usize) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let query = trace_core::query::Query::new(&result.graph);
    let node = trace_core::query::resolve_unique(&result.graph, target)
        .map_err(|e| anyhow::anyhow!("cannot resolve target: {target}: {e}"))?;
    let options = trace_core::QueryOptions::default().with_depth(max_depth);
    let deps = if transitive {
        query.transitive_dependencies(node.id.as_str(), &options)
    } else {
        query.direct_dependencies(node.id.as_str(), &options)
    };
    print_neighbors("dependencies", target, &deps);
    Ok(())
}

fn cmd_impact(target: &str, path: &Path, max_depth: usize, max_results: usize) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let query = trace_core::query::Query::new(&result.graph);
    let node = trace_core::query::resolve_unique(&result.graph, target)
        .map_err(|e| anyhow::anyhow!("cannot resolve target: {target}: {e}"))?;
    let options = trace_core::QueryOptions::default()
        .with_depth(max_depth)
        .with_limit(max_results);
    let impact = query.impact(node.id.as_str(), &options);
    println!("Impact analysis for: {target}");
    println!("Static reachability only — not a claim about runtime breakage.\n");
    for impacted in &impact {
        println!("  {} (depth {})", impacted.node.name, impacted.depth);
        for (i, n) in impacted.path.iter().enumerate() {
            if i > 0 {
                print!(" -> ");
            }
            print!("{}", n.name);
        }
        println!();
    }
    Ok(())
}

fn cmd_path(source: &str, target: &str, path: &Path, max_depth: usize) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let query = trace_core::query::Query::new(&result.graph);
    let source_node = trace_core::query::resolve_unique(&result.graph, source)
        .map_err(|e| anyhow::anyhow!("cannot resolve source: {source}: {e}"))?;
    let target_node = trace_core::query::resolve_unique(&result.graph, target)
        .map_err(|e| anyhow::anyhow!("cannot resolve target: {target}: {e}"))?;
    let options = trace_core::QueryOptions::default().with_depth(max_depth);
    let path_result = query.path(source_node.id.as_str(), target_node.id.as_str(), &options);
    if path_result.found {
        println!("Path from {source} to {target}:");
        for (i, n) in path_result.nodes.iter().enumerate() {
            if i > 0 {
                print!(" -> ");
            }
            print!("{}", n.name);
        }
        println!();
    } else {
        println!("No path from {source} to {target}");
    }
    Ok(())
}

fn cmd_cycles(path: &Path) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let query = trace_core::query::Query::new(&result.graph);
    let cycles = query.cycles(&[]);
    if cycles.is_empty() {
        println!("No dependency cycles detected.");
    } else {
        println!("Dependency cycles:");
        for cycle in &cycles {
            for (i, node) in cycle.iter().enumerate() {
                if i > 0 {
                    print!(" -> ");
                }
                print!("{}", node.name);
            }
            println!();
        }
    }
    Ok(())
}

fn cmd_stats(path: &Path) -> Result<()> {
    let (result, _config) = build_graph_for_path(path, None, None, None, false, None, None)?;
    let stats = result.graph.stats(&result.diagnostics);
    println!("Graph statistics:");
    println!("  files:        {}", stats.files);
    println!("  packages:     {}", stats.packages);
    println!("  nodes:        {}", stats.nodes);
    println!("  edges:        {}", stats.edges);
    println!("  symbols:      {}", stats.symbols);
    println!("  unresolved:   {}", stats.unresolved_edges);
    println!("  diagnostics:  {}", stats.diagnostics);
    if !stats.by_language.is_empty() {
        println!("  by language:");
        for (lang, count) in &stats.by_language {
            println!("    {lang}: {count}");
        }
    }
    if !stats.by_node_kind.is_empty() {
        println!("  by node kind:");
        for (kind, count) in &stats.by_node_kind {
            println!("    {kind}: {count}");
        }
    }
    if !stats.by_edge_kind.is_empty() {
        println!("  by edge kind:");
        for (kind, count) in &stats.by_edge_kind {
            println!("    {kind}: {count}");
        }
    }
    Ok(())
}

fn cmd_languages() -> Result<()> {
    let capabilities = all_capabilities();
    println!("Supported languages:");
    for cap in &capabilities {
        println!(
            "  {} ({}) — syntax: {}, symbols: {}, imports: {}, calls: {}, xref: {}, semantic: {}",
            cap.display_name,
            cap.language,
            cap.syntax_parsing,
            cap.symbol_extraction,
            cap.import_resolution,
            cap.call_resolution,
            cap.cross_file_references,
            cap.semantic_integration
        );
        for note in &cap.notes {
            println!("      - {note}");
        }
    }
    Ok(())
}

fn cmd_check(path: &Path) -> Result<()> {
    let (config, _) = TraceConfig::load(path, None).context("failed to load configuration")?;
    config.validate().context("invalid configuration")?;

    let options = DiscoveryOptions::default();
    let report = trace_parser::discovery::discover(path, &options)
        .map_err(anyhow::Error::msg)
        .context("failed to discover files")?;

    println!("Configuration: valid");
    println!("  cache directory: {}", DEFAULT_CACHE_DIR);
    println!("  semantic mode: off");
    println!();
    println!("Discovery:");
    println!("  supported files: {}", report.files.len());
    println!("  skipped: {}", report.skipped.len());
    println!("  unsupported: {}", report.unsupported_files);
    println!();
    println!("Languages:");
    for cap in &all_capabilities() {
        println!("  {} — parser available", cap.display_name);
    }
    println!();
    println!("Check passed.");
    Ok(())
}

fn print_neighbors(label: &str, symbol: &str, neighbors: &[trace_core::query::Neighbor]) {
    println!("{label} of {symbol}:");
    if neighbors.is_empty() {
        println!("  (none)");
        return;
    }
    for neighbor in neighbors {
        let edge = &neighbor.edge;
        println!(
            "  {} [{}] via {:?} at depth {}",
            neighbor.node.name, neighbor.node.id, edge.kind, neighbor.depth
        );
    }
}
