//! GraphBuilder construction methods.

use std::path::Path;

use rayon::prelude::*;
use trace_core::file_id;
use trace_core::{Edge, EdgeKind, Graph, Node, NodeKind, ResolutionOrigin, ResolutionStatus};
use trace_parser::{
    discovery::{self, DiscoveryOptions},
    ParsedFile,
};

use super::graph_symbol_edges::module_id_str;
use crate::cache::SyntaxCache;
use crate::framework::{Framework, FrameworkDetector, ManifestContents};
use crate::index::RepositoryIndex;
use trace_core::SourceRange;

use super::graph_file_nodes::add_file_nodes_and_edges;
use super::graph_symbol_edges::add_symbol_edges;
use super::graph_types::{GraphBuildResult, GraphBuilder};

/// Known manifest file names used for framework detection.
const MANIFEST_FILES: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "go.mod",
    "requirements.txt",
    "pyproject.toml",
    "setup.py",
    "setup.cfg",
    "Gemfile",
    "Gemfile.lock",
    "composer.json",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "settings.gradle",
    "angular.json",
    "next.config.js",
    "next.config.mjs",
    "config/application.rb",
];

/// Discover manifest files under `root` whose names are known framework
/// signals. Returns repository-relative paths to file contents.
fn discover_manifests(root: &Path) -> ManifestContents {
    let mut files = ManifestContents::empty();
    for name in MANIFEST_FILES {
        let path = root.join(name);
        if path.is_file() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                let relative = path
                    .strip_prefix(root)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| name.to_string());
                files.files.insert(relative, text);
            }
        }
    }
    files
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the syntax cache for incremental parsing.
    pub fn with_cache(mut self, cache: SyntaxCache) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Enable or disable parallel parsing.
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Build a graph from a repository root using default discovery options.
    pub fn build_repository(&self, root: &Path) -> crate::error::Result<GraphBuildResult> {
        let options = DiscoveryOptions::default();
        self.build_repository_with_options(root, &options)
    }

    /// Build a graph from a repository root with custom discovery options.
    pub fn build_repository_with_options(
        &self,
        root: &Path,
        options: &DiscoveryOptions,
    ) -> crate::error::Result<GraphBuildResult> {
        let report =
            discovery::discover(root, options).map_err(crate::error::AnalysisError::Discovery)?;

        let cache_ref = self.cache.clone();
        let files: Vec<_> = report.files.to_vec();

        let parsed_files: Vec<Result<trace_parser::ParsedFile, crate::error::AnalysisError>> =
            if self.parallel {
                files
                    .par_iter()
                    .map(|file| {
                        let source = std::fs::read_to_string(&file.absolute).map_err(|error| {
                            crate::error::AnalysisError::Read {
                                path: file.absolute.display().to_string(),
                                message: error.to_string(),
                            }
                        })?;

                        let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();

                        let parsed =
                            if let Some(cache) = &cache_ref {
                                if let Some(cached) =
                                    cache.load(&file.path, file.language, &source_hash)
                                {
                                    cached
                                } else {
                                    let mut parser = trace_parser::parser::SyntaxParser::new()
                                        .map_err(|error| {
                                            crate::error::AnalysisError::Parser(error.to_string())
                                        })?;
                                    let parsed = parser
                                        .parse(&file.path, file.language, &source)
                                        .map_err(|error| {
                                        crate::error::AnalysisError::Parser(error.to_string())
                                    })?;
                                    cache.store(&parsed);
                                    parsed
                                }
                            } else {
                                let mut parser = trace_parser::parser::SyntaxParser::new()
                                    .map_err(|error| {
                                        crate::error::AnalysisError::Parser(error.to_string())
                                    })?;
                                parser.parse(&file.path, file.language, &source).map_err(
                                    |error| crate::error::AnalysisError::Parser(error.to_string()),
                                )?
                            };

                        Ok(parsed)
                    })
                    .collect()
            } else {
                let mut parser = trace_parser::parser::SyntaxParser::new()
                    .map_err(|error| crate::error::AnalysisError::Parser(error.to_string()))?;

                files
                    .iter()
                    .map(|file| {
                        let source = std::fs::read_to_string(&file.absolute).map_err(|error| {
                            crate::error::AnalysisError::Read {
                                path: file.absolute.display().to_string(),
                                message: error.to_string(),
                            }
                        })?;

                        let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();

                        let parsed =
                            if let Some(cache) = &cache_ref {
                                if let Some(cached) =
                                    cache.load(&file.path, file.language, &source_hash)
                                {
                                    cached
                                } else {
                                    let parsed = parser
                                        .parse(&file.path, file.language, &source)
                                        .map_err(|error| {
                                        crate::error::AnalysisError::Parser(error.to_string())
                                    })?;
                                    cache.store(&parsed);
                                    parsed
                                }
                            } else {
                                parser.parse(&file.path, file.language, &source).map_err(
                                    |error| crate::error::AnalysisError::Parser(error.to_string()),
                                )?
                            };

                        Ok(parsed)
                    })
                    .collect()
            };

        let mut diagnostics = Vec::new();
        let mut parsed_files_ok = Vec::new();
        for result in parsed_files {
            let parsed = result?;
            diagnostics.extend(parsed.diagnostics.clone());
            parsed_files_ok.push(parsed);
        }

        parsed_files_ok.sort_by(|a, b| a.path.cmp(&b.path));

        // Framework detection: run before index construction takes ownership
        // of parsed_files_ok. Imports, decorators, base classes, and calls
        // are all available from the parsed files.
        let manifests = discover_manifests(root);
        let detector = FrameworkDetector::new();
        let detection = detector.detect(root, &parsed_files_ok, &manifests);

        let mut index = RepositoryIndex::build(root, parsed_files_ok)?;
        let import_pass = crate::resolve::resolve_imports(&mut index);

        let mut result = self.build_from_index(&index, &import_pass, &detection.frameworks)?;
        diagnostics.extend(import_pass.diagnostics);
        diagnostics.extend(report.diagnostics.clone());
        diagnostics.sort_by_key(|d| d.sort_key());
        result.diagnostics.extend(diagnostics);
        result.diagnostics.sort_by_key(|d| d.sort_key());
        result.diagnostics.dedup();
        Ok(result)
    }

    /// Build a graph directly from already-parsed files.
    pub fn build_from_parsed(
        &self,
        root: &Path,
        parsed_files: Vec<ParsedFile>,
    ) -> crate::error::Result<GraphBuildResult> {
        let mut index = RepositoryIndex::build(root, parsed_files.clone())?;
        let import_pass = crate::resolve::resolve_imports(&mut index);

        let manifests = discover_manifests(root);
        let detector = FrameworkDetector::new();
        let detection = detector.detect(root, &parsed_files, &manifests);

        let mut result = self.build_from_index(&index, &import_pass, &detection.frameworks)?;
        result.diagnostics.extend(import_pass.diagnostics);
        result.diagnostics.sort_by_key(|d| d.sort_key());
        result.diagnostics.dedup();
        Ok(result)
    }

    /// Build a graph from a resolved index and import pass.
    pub fn build_from_index(
        &self,
        index: &RepositoryIndex,
        import_pass: &crate::resolve::ImportPass,
        frameworks: &[Framework],
    ) -> crate::error::Result<GraphBuildResult> {
        let mut graph = Graph::new();
        let diagnostics = Vec::new();

        let repository = trace_core::graph::repository_node("repository");
        let repository_id = repository.id.clone();
        graph.add_node(repository)?;

        // Add framework nodes and edges from the repository node.
        for framework in frameworks {
            let framework_id =
                trace_core::id::framework_id(&framework.name, framework.version.as_deref());
            let mut framework_node = Node::new(
                framework_id.clone(),
                NodeKind::Framework,
                framework.display_name.clone(),
            );
            framework_node
                .metadata
                .insert("framework".to_string(), framework.name.clone());
            framework_node
                .metadata
                .insert("family".to_string(), framework.family.as_str().to_string());
            framework_node.metadata.insert(
                "confidence".to_string(),
                framework.confidence.as_str().to_string(),
            );
            graph.add_node(framework_node)?;

            graph.add_edge(Edge::new(
                repository_id.clone(),
                framework_id,
                EdgeKind::UsesFramework,
                ResolutionStatus::ResolvedInternal,
                ResolutionOrigin::Syntax,
            ))?;
        }

        // Import edges aggregated by (from file, module target) for diagnostics.
        let import_edges: std::collections::BTreeMap<(String, String), Vec<SourceRange>> =
            import_pass
                .edges
                .iter()
                .fold(std::collections::BTreeMap::new(), |mut acc, edge| {
                    let target = self.module_target_node(index, &module_id_str(&edge.module));
                    acc.entry((edge.from_path.clone(), target))
                        .or_default()
                        .push(edge.site);
                    acc
                });

        for (path, state) in &index.files {
            add_file_nodes_and_edges(self, &mut graph, index, state, path)?;
        }

        // Repository contains every file and package.
        for path in index.files.keys() {
            graph.add_edge(Edge::new(
                repository_id.clone(),
                file_id(path),
                EdgeKind::Contains,
                ResolutionStatus::ResolvedInternal,
                ResolutionOrigin::Syntax,
            ))?;
        }
        for package_id_str in index.packages.keys() {
            graph.add_edge(Edge::new(
                repository_id.clone(),
                package_id_str.clone(),
                EdgeKind::Contains,
                ResolutionStatus::ResolvedInternal,
                ResolutionOrigin::Syntax,
            ))?;
        }

        // File-level import edges.
        for ((from_path, target), sites) in &import_edges {
            let status = if target.starts_with("external:") {
                ResolutionStatus::ResolvedExternal
            } else {
                ResolutionStatus::ResolvedInternal
            };
            // Ensure external import targets have a corresponding node so the
            // edge target is never dangling.
            if target.starts_with("external:") {
                if let Some(state) = index.files.get(from_path) {
                    let qualified = target.strip_prefix("external:unknown:").unwrap_or(target);
                    let node = Node::new(
                        target.clone(),
                        NodeKind::ExternalSymbol,
                        qualified.to_string(),
                    )
                    .with_language(state.language);
                    let _ = graph.add_node(node);
                }
            }
            let mut edge = Edge::new(
                file_id(from_path),
                target.clone(),
                EdgeKind::Imports,
                status,
                ResolutionOrigin::Import,
            );
            for site in sites {
                edge = edge.at(*site);
            }
            graph.add_edge(edge)?;
        }

        // Package containment edges.
        for (package_id_str, package) in &index.packages {
            for file_path in &package.files {
                graph.add_edge(Edge::new(
                    package_id_str.clone(),
                    file_id(file_path),
                    EdgeKind::Contains,
                    ResolutionStatus::ResolvedInternal,
                    ResolutionOrigin::Syntax,
                ))?;
            }
        }

        // Symbol-level edges: calls, references, heritage, exports, definitions.
        for (path, state) in &index.files {
            add_symbol_edges(self, &mut graph, index, state, path)?;
        }

        graph.finalize();

        Ok(GraphBuildResult {
            graph,
            diagnostics,
            frameworks: frameworks.to_vec(),
        })
    }
}
