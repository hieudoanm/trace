//! CLI command definitions.

use std::path::PathBuf;

use clap::Subcommand;

/// CLI subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
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

        /// Also write .trace/graph.json and .trace/graph.html for the visual explorer.
        #[arg(long)]
        write_trace_dir: bool,
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
    /// Display dependencies of a target.
    Deps {
        /// Target symbol or file.
        target: String,

        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Include transitive dependencies.
        #[arg(long)]
        transitive: bool,

        /// Maximum traversal depth.
        #[arg(long, default_value = "32")]
        max_depth: usize,
    },
    /// Display impact analysis for a target.
    Impact {
        /// Target symbol or file.
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
    /// Find a path between two symbols.
    Path {
        /// Source symbol.
        source: String,

        /// Target symbol.
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
    /// List supported languages.
    Languages,
    /// Validate configuration and parser availability.
    Check {
        /// Repository path to check (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Validate graph integrity: every edge target exists as a node.
    Validate {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}
