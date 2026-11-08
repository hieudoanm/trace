//! Repository-level graph building: discovery, parsing, index construction,
//! framework detection, and import resolution.

use std::path::Path;

use tracers_parser::discovery::DiscoveryOptions;

use crate::cache::SyntaxCache;
use crate::error::Result;
use crate::framework::FrameworkDetector;
use crate::index::RepositoryIndex;

use super::build_from_index::build_from_index;
use super::discovery::discover_manifests;
use super::parsing::parse_files;

use super::super::graph_types::GraphBuilder;

pub(crate) fn build_repository_with_options(
    builder: &GraphBuilder,
    root: &Path,
    options: &DiscoveryOptions,
    cache: Option<&SyntaxCache>,
) -> Result<crate::graph::graph_types::GraphBuildResult> {
    let report = tracers_parser::discovery::discover(root, options)
        .map_err(crate::error::AnalysisError::Discovery)?;

    tracing::debug!(
        files = report.files.len(),
        skipped = report.skipped.len(),
        unsupported = report.unsupported_files,
        jobs = ?builder.jobs,
        "discovery complete"
    );
    for skipped in &report.skipped {
        tracing::debug!(
            path = %skipped.path,
            reason = ?skipped.reason,
            detail = %skipped.detail,
            "file skipped"
        );
    }

    let files: Vec<_> = report.files.to_vec();

    let parsed_files = parse_files(&files, builder.jobs, cache)?;

    tracing::debug!(parsed = parsed_files.len(), "parsing complete");

    let mut diagnostics = Vec::new();
    let mut parsed_files_ok = Vec::new();
    for parsed in parsed_files {
        diagnostics.extend(parsed.diagnostics.clone());
        parsed_files_ok.push(parsed);
    }

    parsed_files_ok.sort_by(|a, b| a.path.cmp(&b.path));

    let manifests = discover_manifests(root);
    let detector = FrameworkDetector::new();
    let detection = detector.detect(root, &parsed_files_ok, &manifests);

    let mut index = RepositoryIndex::build(root, parsed_files_ok)?;
    let import_pass = crate::resolve::resolve_imports(&mut index);

    let mut result = build_from_index(builder, &index, &import_pass, &detection.frameworks)?;
    // Record the effective language set: an empty filter means every
    // supported language was enabled for this scan.
    result.languages = if options.languages.is_empty() {
        tracers_core::Language::ALL.to_vec()
    } else {
        options.languages.clone()
    };
    result.cache_counts = cache.map(|cache| cache.counts());
    diagnostics.extend(import_pass.diagnostics);
    diagnostics.extend(report.diagnostics.clone());
    diagnostics.sort_by_key(|d| d.sort_key());
    result.diagnostics.extend(diagnostics);
    result.diagnostics.sort_by_key(|d| d.sort_key());
    result.diagnostics.dedup();
    Ok(result)
}
