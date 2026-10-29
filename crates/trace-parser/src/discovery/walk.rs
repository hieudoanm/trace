use std::path::Path;
use std::sync::atomic::Ordering;

use ignore::WalkBuilder;

use trace_core::{Diagnostic, DiagnosticKind, Language};

use super::constants::DEFAULT_EXCLUDED_DIRS;
use super::filters::{compile_globs, looks_generated, relative_path};
use super::types::{DiscoveredFile, DiscoveryOptions, DiscoveryReport, SkipReason, SkippedFile};

pub(crate) fn discover_directory(
    root: &Path,
    options: &DiscoveryOptions,
) -> Result<DiscoveryReport, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("cannot access `{}`: {error}", root.display()))?;
    let mut diagnostics = Vec::new();
    let includes = compile_globs(&options.includes, "--include")?;
    let excludes = compile_globs(&options.excludes, "--exclude")?;

    let mut files = Vec::new();
    let mut skipped = Vec::new();
    let mut unsupported_files = 0usize;
    let ignored_files = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let include_patterns = options.includes.clone();
    let pruned = ignored_files.clone();
    let mut builder = WalkBuilder::new(&root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .require_git(false)
        .follow_links(options.follow_symlinks)
        .sort_by_file_path(|a, b| a.cmp(b))
        .filter_entry(move |entry| {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                return true;
            }
            let name = entry.file_name().to_string_lossy();
            if DEFAULT_EXCLUDED_DIRS.contains(&name.as_ref()) {
                let explicitly_included = include_patterns
                    .iter()
                    .any(|pattern| pattern.contains(name.as_ref()));
                if !explicitly_included {
                    pruned.fetch_add(1, Ordering::Relaxed);
                    return false;
                }
            }
            true
        });
    let walker = builder.build();

    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticKind::UnreadableFile,
                    format!("file discovery error: {error}"),
                ));
                continue;
            }
        };
        let path = entry.path();
        let relative = relative_path(&root, path);
        if relative.is_empty() {
            continue;
        }
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            continue;
        }
        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            unsupported_files += 1;
            continue;
        };
        let Some(language) = Language::from_extension(extension) else {
            unsupported_files += 1;
            continue;
        };
        if !options.languages.is_empty() && !options.languages.contains(&language) {
            skipped.push(SkippedFile {
                path: relative.clone(),
                reason: SkipReason::Excluded,
                detail: "language filtered out".to_string(),
            });
            continue;
        }
        if !excludes.is_empty() && excludes.is_match(&relative) {
            skipped.push(SkippedFile {
                path: relative.clone(),
                reason: SkipReason::Excluded,
                detail: "matched exclude pattern".to_string(),
            });
            continue;
        }
        if !includes.is_empty() && !includes.is_match(&relative) {
            skipped.push(SkippedFile {
                path: relative.clone(),
                reason: SkipReason::Excluded,
                detail: "did not match include pattern".to_string(),
            });
            continue;
        }
        if !options.include_generated && looks_generated(&relative) {
            skipped.push(SkippedFile {
                path: relative.clone(),
                reason: SkipReason::Generated,
                detail: "generated-file naming convention".to_string(),
            });
            continue;
        }
        match std::fs::metadata(path) {
            Ok(metadata) => {
                if metadata.len() > options.max_file_size {
                    skipped.push(SkippedFile {
                        path: relative.clone(),
                        reason: SkipReason::TooLarge,
                        detail: format!(
                            "{} bytes exceeds limit of {}",
                            metadata.len(),
                            options.max_file_size
                        ),
                    });
                    continue;
                }
            }
            Err(error) => {
                diagnostics.push(
                    Diagnostic::warning(
                        DiagnosticKind::UnreadableFile,
                        format!("cannot stat file: {error}"),
                    )
                    .with_file(relative.clone())
                    .with_language(language),
                );
                skipped.push(SkippedFile {
                    path: relative.clone(),
                    reason: SkipReason::Unreadable,
                    detail: error.to_string(),
                });
                continue;
            }
        }
        files.push(DiscoveredFile {
            path: relative,
            absolute: path.to_path_buf(),
            language,
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    skipped.sort_by(|a, b| a.path.cmp(&b.path));
    diagnostics.sort_by_key(|d| d.sort_key());

    Ok(DiscoveryReport {
        root,
        files,
        skipped,
        unsupported_files,
        ignored_files: ignored_files.load(Ordering::Relaxed),
        diagnostics,
    })
}
