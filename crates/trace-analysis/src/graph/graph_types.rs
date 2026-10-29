//! Graph types and builder configuration.

use trace_core::Graph;

use crate::cache::SyntaxCache;
use crate::framework::Framework;

/// The result of building a graph.
#[derive(Debug, Clone)]
pub struct GraphBuildResult {
    pub graph: Graph,
    pub diagnostics: Vec<trace_core::Diagnostic>,
    /// Frameworks detected during the build.
    pub frameworks: Vec<Framework>,
}

/// Builds a [`Graph`] from discovered, parsed, and resolved source files.
#[derive(Debug, Clone, Default)]
pub struct GraphBuilder {
    /// Whether to include external symbol nodes for unresolved imports.
    pub include_external: bool,
    /// Optional syntax cache for incremental parsing.
    pub cache: Option<SyntaxCache>,
    /// Whether to use parallel parsing (default: true).
    pub parallel: bool,
}

/// Target of a call resolution.
#[derive(Debug, Clone)]
pub(crate) enum CallTarget {
    Internal(String),
    External(String),
    Unresolved,
    Ambiguous(Vec<String>),
}
