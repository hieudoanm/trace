//! Tracers CLI entry point.

mod cli;
mod mcp;

use std::process::exit;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

use crate::cli::commands::Commands;

/// Tracers: a high-performance, multi-language code intelligence and graph
/// analysis tool.
#[derive(Parser, Debug)]
#[command(
    name = "tracers",
    version = concat!(
        env!("CARGO_PKG_VERSION"),
        " (build ",
        env!("TRACERS_BUILD_TIME"),
        ")"
    ),
    about,
    long_about = None
)]
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

fn main() {
    if let Err(error) = run() {
        // `{:#}` prints the full anyhow cause chain on one line; plain `{}`
        // would only show the outermost context and hide the real failure.
        eprintln!("error: {error:#}");
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
    let builder = fmt().with_env_filter(filter);
    if matches!(cli.command, Commands::Mcp) {
        builder.with_writer(std::io::stderr).init();
    } else {
        builder.init();
    }

    match cli.command {
        Commands::Mcp => mcp::run().map_err(Into::into),
        Commands::Scan {
            path,
            output,
            include,
            exclude,
            language,
            no_cache,
            jobs,
            config,
            write_tracers_dir,
            no_tracers_dir,
        } => crate::cli::scan::cmd_scan(
            &path,
            output.as_deref(),
            &include,
            &exclude,
            language.as_deref(),
            no_cache,
            jobs,
            config.as_deref(),
            write_tracers_dir && !no_tracers_dir,
        ),
        Commands::Graph {
            path,
            format,
            output,
            language,
            kind,
            graph,
        } => crate::cli::graph::cmd_graph(
            &path,
            format.as_deref(),
            output.as_deref(),
            language.as_deref(),
            kind.as_deref(),
            graph.as_deref(),
        ),
        Commands::Callers {
            symbol,
            path,
            max_depth,
            max_results,
            graph,
        } => crate::cli::queries::cmd_callers(
            &symbol,
            &path,
            max_depth,
            max_results,
            graph.as_deref(),
        ),
        Commands::Callees {
            symbol,
            path,
            max_depth,
            max_results,
            graph,
        } => crate::cli::queries::cmd_callees(
            &symbol,
            &path,
            max_depth,
            max_results,
            graph.as_deref(),
        ),
        Commands::Refs {
            symbol,
            path,
            max_depth,
            max_results,
            graph,
        } => {
            crate::cli::queries::cmd_refs(&symbol, &path, max_depth, max_results, graph.as_deref())
        }
        Commands::Deps {
            target,
            path,
            transitive,
            max_depth,
            max_results,
            graph,
        } => crate::cli::queries::cmd_deps(
            &target,
            &path,
            transitive,
            max_depth,
            max_results,
            graph.as_deref(),
        ),
        Commands::Impact {
            target,
            path,
            max_depth,
            max_results,
            graph,
        } => crate::cli::queries::cmd_impact(
            &target,
            &path,
            max_depth,
            max_results,
            graph.as_deref(),
        ),
        Commands::Dependents {
            target,
            path,
            max_depth,
            max_results,
            graph,
        } => crate::cli::queries::cmd_dependents(
            &target,
            &path,
            max_depth,
            max_results,
            graph.as_deref(),
        ),
        Commands::Reach {
            symbol,
            path,
            max_depth,
            max_results,
            graph,
        } => {
            crate::cli::queries::cmd_reach(&symbol, &path, max_depth, max_results, graph.as_deref())
        }
        Commands::Diagnostics {
            path,
            max_results,
            graph,
        } => crate::cli::validate::cmd_diagnostics(&path, max_results, graph.as_deref()),
        Commands::Path {
            source,
            target,
            path,
            max_depth,
            graph,
        } => crate::cli::queries::cmd_path(&source, &target, &path, max_depth, graph.as_deref()),
        Commands::Roots {
            path,
            max_results,
            graph,
        } => crate::cli::queries::cmd_roots(&path, max_results, graph.as_deref()),
        Commands::Cycles { path, graph } => {
            crate::cli::validate::cmd_cycles(&path, graph.as_deref())
        }
        Commands::Stats { path, graph } => crate::cli::validate::cmd_stats(&path, graph.as_deref()),
        Commands::Languages => crate::cli::validate::cmd_languages(),
        Commands::Frameworks {
            path,
            include,
            exclude,
            language,
        } => crate::cli::validate::cmd_frameworks(&path, &include, &exclude, language.as_deref()),
        Commands::Check {
            path,
            include,
            exclude,
            language,
        } => crate::cli::validate::cmd_check(&path, &include, &exclude, language.as_deref()),
        Commands::Validate { path, graph } => {
            crate::cli::validate::cmd_validate(&path, graph.as_deref())
        }
    }
}
