//! Graph types and builder configuration.

use tracers_core::Graph;

use crate::cache::SyntaxCache;
use crate::framework::Framework;

/// The result of building a graph.
#[derive(Debug, Clone)]
pub struct GraphBuildResult {
    pub graph: Graph,
    pub diagnostics: Vec<tracers_core::Diagnostic>,
    /// Frameworks detected during the build.
    pub frameworks: Vec<Framework>,
    /// Languages the scan was configured to analyse. Empty when the build
    /// path did not perform discovery (e.g. `build_from_parsed`).
    pub languages: Vec<tracers_core::Language>,
    /// Syntax-cache activity for this build, when a cache was used.
    pub cache_counts: Option<crate::cache::CacheCounts>,
}

/// Builds a [`Graph`] from discovered, parsed, and resolved source files.
#[derive(Debug, Clone, Default)]
pub struct GraphBuilder {
    /// Whether to include external symbol nodes for unresolved imports.
    pub include_external: bool,
    /// Optional syntax cache for incremental parsing.
    pub cache: Option<SyntaxCache>,
    /// Parallel parsing worker count. `None` uses the rayon global pool,
    /// `Some(1)` forces sequential parsing, `Some(n)` uses an n-thread pool.
    pub jobs: Option<usize>,
}

/// Target of a call resolution.
#[derive(Debug, Clone)]
pub(crate) enum CallTarget {
    Internal(String),
    External(String),
    Unresolved,
    Ambiguous(Vec<String>),
}
