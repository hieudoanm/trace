//! Graph construction from a resolved [`RepositoryIndex`].
//!
//! This module turns parsed, indexed, and import-resolved source files into a
//! [`trace_core::Graph`]. All graph construction is syntax- and
//! lexically-scoped; compiler-assisted semantic resolution is not yet
//! integrated, so unresolved relationships are recorded explicitly rather than
//! invented.

pub mod graph_builder;
pub mod graph_file_nodes;
pub mod graph_helpers;
pub mod graph_symbol_edges;
#[cfg(test)]
pub mod graph_tests;
pub mod graph_types;

pub use graph_types::GraphBuildResult;
pub use graph_types::GraphBuilder;
