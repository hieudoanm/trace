//! Build a graph directly from already-parsed files.

use std::path::Path;

use trace_parser::ParsedFile;

use crate::framework::FrameworkDetector;
use crate::index::RepositoryIndex;

use super::build_from_index::build_from_index;
use super::discovery::discover_manifests;

use super::super::graph_types::GraphBuilder;

pub(crate) fn build_from_parsed(
    builder: &GraphBuilder,
    root: &Path,
    parsed_files: Vec<ParsedFile>,
) -> crate::error::Result<crate::graph::graph_types::GraphBuildResult> {
    let mut index = RepositoryIndex::build(root, parsed_files.clone())?;
    let import_pass = crate::resolve::resolve_imports(&mut index);

    let manifests = discover_manifests(root);
    let detector = FrameworkDetector::new();
    let detection = detector.detect(root, &parsed_files, &manifests);

    let mut result = build_from_index(builder, &index, &import_pass, &detection.frameworks)?;
    result.diagnostics.extend(import_pass.diagnostics);
    result.diagnostics.sort_by_key(|d| d.sort_key());
    result.diagnostics.dedup();
    Ok(result)
}
