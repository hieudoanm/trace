//! Analysis orchestration: symbol indexing, import resolution, graph
//! construction, and caching.
//!
//! This crate is the bridge between [`trace-parser`] (syntax extraction) and
//! [`trace-core`] (the graph model). It builds a [`RepositoryIndex`] from
//! parsed files, resolves imports and symbols, and constructs a unified
//! [`trace_core::Graph`].

pub mod cache;
pub mod capabilities;
pub mod config;
pub mod error;
pub mod framework;
pub mod graph;
pub mod index;
pub mod resolve;

pub use capabilities::{all_capabilities, capabilities, LanguageCapability};
pub use config::{
    AnalysisConfig, CacheConfig, OutputConfig, SemanticMode, TraceConfig, CONFIG_FILE_NAME,
    DEFAULT_CACHE_DIR, DEFAULT_MAX_FILE_SIZE,
};
pub use error::{AnalysisError, Result};
pub use graph::{GraphBuildResult, GraphBuilder};
pub use index::RepositoryIndex;
