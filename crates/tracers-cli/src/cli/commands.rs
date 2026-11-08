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

        /// Path to a tracers.toml configuration file.
        #[arg(long)]
        config: Option<PathBuf>,

        /// Write .tracers/ directory with graph.json, graph.dot, and graph.html (enabled by default).
        #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = true, overrides_with = "no_tracers_dir")]
        write_tracers_dir: bool,

        /// Disable writing the .tracers directory.
        #[arg(long, overrides_with = "write_tracers_dir")]
        no_tracers_dir: bool,
    },
    /// Load or construct a graph and provide filtered graph output.
    Graph {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output format: json or dot (defaults to tracers.toml output.format, then json).
        #[arg(short = 'f', long)]
        format: Option<String>,

        /// Write output to PATH instead of stdout.
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Filter by language.
        #[arg(long)]
        language: Option<String>,

        /// Filter by node kind.
        #[arg(long)]
        kind: Option<String>,

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
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

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
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

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
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

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
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

        /// Maximum number of results.
        #[arg(long, default_value = "1000")]
        max_results: usize,

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
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

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
    },
    /// Display symbols and files that depend on a target (reverse dependencies).
    Dependents {
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

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
    },
    /// List nodes reachable from a symbol through calls, references and dependencies.
    Reach {
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

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
    },
    /// List analysis diagnostics (parse errors, unresolved imports, skipped files).
    Diagnostics {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Maximum number of results.
        #[arg(long, default_value = "1000")]
        max_results: usize,

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
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

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
    },
    /// List callable symbols with no incoming call edges.
    Roots {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Maximum number of results.
        #[arg(long, default_value = "1000")]
        max_results: usize,

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
    },
    /// Detect dependency cycles.
    Cycles {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
    },
    /// Report graph statistics.
    Stats {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
    },
    /// List supported languages.
    Languages,
    /// Report detected frameworks in the repository.
    Frameworks {
        /// Repository path to scan (defaults to current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
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

        /// Load the graph from an exported JSON file instead of rescanning.
        #[arg(long)]
        graph: Option<PathBuf>,
    },
}
