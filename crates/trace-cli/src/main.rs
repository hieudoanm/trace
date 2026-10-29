//! Trace CLI entry point.

mod cli;

use std::process::exit;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

use crate::cli::commands::Commands;

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
            write_trace_dir,
            no_trace_dir,
        } => crate::cli::scan::cmd_scan(
            &path,
            &format,
            output.as_deref(),
            include.as_deref(),
            exclude.as_deref(),
            language.as_deref(),
            no_cache,
            jobs,
            config.as_deref(),
            write_trace_dir && !no_trace_dir,
        ),
        Commands::Graph {
            path,
            format,
            output,
            language,
            kind,
        } => crate::cli::graph::cmd_graph(
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
        } => crate::cli::queries::cmd_callers(&symbol, &path, max_depth, max_results),
        Commands::Callees {
            symbol,
            path,
            max_depth,
            max_results,
        } => crate::cli::queries::cmd_callees(&symbol, &path, max_depth, max_results),
        Commands::Refs {
            symbol,
            path,
            max_depth,
            max_results,
        } => crate::cli::queries::cmd_refs(&symbol, &path, max_depth, max_results),
        Commands::Deps {
            target,
            path,
            transitive,
            max_depth,
        } => crate::cli::queries::cmd_deps(&target, &path, transitive, max_depth),
        Commands::Impact {
            target,
            path,
            max_depth,
            max_results,
        } => crate::cli::queries::cmd_impact(&target, &path, max_depth, max_results),
        Commands::Path {
            source,
            target,
            path,
            max_depth,
        } => crate::cli::queries::cmd_path(&source, &target, &path, max_depth),
        Commands::Cycles { path } => crate::cli::validate::cmd_cycles(&path),
        Commands::Stats { path } => crate::cli::validate::cmd_stats(&path),
        Commands::Languages => crate::cli::validate::cmd_languages(),
        Commands::Check { path } => crate::cli::validate::cmd_check(&path),
        Commands::Validate { path } => crate::cli::validate::cmd_validate(&path),
    }
}
