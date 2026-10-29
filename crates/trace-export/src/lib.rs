//! Deterministic JSON and DOT graph export for Trace.
//!
//! Produces a versioned graph document that serializes a [`trace_core::Graph`]
//! together with repository metadata, diagnostics, and statistics. Output is
//! sorted by canonical keys so identical inputs always produce byte-identical
//! JSON. Also supports DOT format for Graphviz visualization.

pub mod dot;
pub mod export;
pub mod serialize;
pub mod types;

#[cfg(test)]
mod tests;

pub use dot::to_dot;
pub use export::{export_graph, SCHEMA_VERSION, TRACE_VERSION};
pub use serialize::{to_json_string, to_json_value, write_json};
pub use types::{AnalysisMetadata, FrameworkInfo, GraphDocument, LanguageInfo, RepositoryMetadata};
