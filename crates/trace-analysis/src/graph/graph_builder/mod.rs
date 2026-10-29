//! GraphBuilder construction methods.
//!
//! Builds a [`trace_core::Graph`] from discovered, parsed, and resolved source
//! files, running framework detection, import resolution, and edge construction.

pub mod build_from_index;
pub mod build_from_parsed;
pub mod build_repository;
pub mod discovery;
pub mod parsing;

use std::path::Path;

use trace_parser::discovery::DiscoveryOptions;

use crate::cache::SyntaxCache;
use crate::index::RepositoryIndex;

use super::graph_types::{GraphBuildResult, GraphBuilder};

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cache(mut self, cache: SyntaxCache) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    pub fn build_repository(&self, root: &Path) -> crate::error::Result<GraphBuildResult> {
        let options = DiscoveryOptions::default();
        build_repository::build_repository_with_options(self, root, &options, self.cache.as_ref())
    }

    pub fn build_repository_with_options(
        &self,
        root: &Path,
        options: &DiscoveryOptions,
    ) -> crate::error::Result<GraphBuildResult> {
        build_repository::build_repository_with_options(self, root, options, self.cache.as_ref())
    }

    pub fn build_from_parsed(
        &self,
        root: &Path,
        parsed_files: Vec<trace_parser::ParsedFile>,
    ) -> crate::error::Result<GraphBuildResult> {
        build_from_parsed::build_from_parsed(self, root, parsed_files)
    }

    pub fn build_from_index(
        &self,
        index: &RepositoryIndex,
        import_pass: &crate::resolve::ImportPass,
        frameworks: &[crate::framework::Framework],
    ) -> crate::error::Result<GraphBuildResult> {
        build_from_index::build_from_index(self, index, import_pass, frameworks)
    }
}
