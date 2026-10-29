use std::path::Path;

use trace_core::Language;

use super::types::{DiscoveredFile, DiscoveryReport, SkipReason, SkippedFile};

pub(crate) fn discover_single_file(
    root: &Path,
    options: &super::types::DiscoveryOptions,
) -> Result<super::types::DiscoveryReport, String> {
    let original = root
        .canonicalize()
        .map_err(|error| format!("cannot access `{}`: {error}", root.display()))?;
    let relative = original
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    let Some(language) = original
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(Language::from_extension)
    else {
        return Err(format!(
            "`{}` is not a supported source file",
            original.display()
        ));
    };
    if !options.languages.is_empty() && !options.languages.contains(&language) {
        return Ok(DiscoveryReport {
            root: original
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf(),
            files: Vec::new(),
            skipped: vec![SkippedFile {
                path: relative,
                reason: SkipReason::Excluded,
                detail: "language filtered out".to_string(),
            }],
            unsupported_files: 0,
            ignored_files: 0,
            diagnostics: Vec::new(),
        });
    }
    Ok(DiscoveryReport {
        root: original
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
        files: vec![DiscoveredFile {
            path: relative,
            absolute: original,
            language,
        }],
        skipped: Vec::new(),
        unsupported_files: 0,
        ignored_files: 0,
        diagnostics: Vec::new(),
    })
}
