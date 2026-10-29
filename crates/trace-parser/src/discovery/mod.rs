pub mod constants;
pub mod filters;
pub mod types;

use std::path::Path;

use ignore::WalkBuilder;

use trace_core::{Diagnostic, DiagnosticKind, Language};

pub use constants::DEFAULT_EXCLUDED_DIRS;
pub use filters::{compile_globs, looks_generated, relative_path};
pub use types::{DiscoveredFile, DiscoveryOptions, DiscoveryReport, SkipReason, SkippedFile};

pub fn discover(root: &Path, options: &DiscoveryOptions) -> Result<DiscoveryReport, String> {
    let original_root = root.to_path_buf();
    let single_file = root.is_file();
    let scan_root = if single_file {
        root.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        root.to_path_buf()
    };
    let root = scan_root
        .canonicalize()
        .map_err(|error| format!("cannot access `{}`: {error}", scan_root.display()))?;
    let mut diagnostics = Vec::new();
    let includes = compile_globs(&options.includes, "--include")?;
    let excludes = compile_globs(&options.excludes, "--exclude")?;

    let mut files = Vec::new();
    let mut skipped = Vec::new();
    let mut unsupported_files = 0usize;
    let ignored_files = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    if single_file {
        let original = original_root
            .canonicalize()
            .map_err(|error| format!("cannot access `{}`: {error}", original_root.display()))?;
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
                diagnostics,
            });
        }
        return Ok(DiscoveryReport {
            root: original
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf(),
            files: vec![DiscoveredFile {
                path: relative,
                absolute: original,
                language,
            }],
            skipped,
            unsupported_files: 0,
            ignored_files: 0,
            diagnostics,
        });
    }

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
                    pruned.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
        ignored_files: ignored_files.load(std::sync::atomic::Ordering::Relaxed),
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(root: &Path, path: &str, content: &str) {
        let full = root.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, content).unwrap();
    }

    #[test]
    fn discovers_supported_files_and_respects_excludes() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, "src/a.ts", "export function a() {}");
        write(root, "src/skip.py", "def b(): pass");
        write(root, "node_modules/dep/index.js", "module.exports = {}");
        write(root, "README.md", "# hi");
        write(root, "src/generated.generated.ts", "export const x = 1;");
        let options = DiscoveryOptions {
            excludes: vec!["src/skip.py".into()],
            ..Default::default()
        };
        let report = discover(root, &options).unwrap();
        let paths: Vec<&str> = report.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, vec!["src/a.ts"]);
        assert_eq!(report.unsupported_files, 1);
        assert!(report.skipped.iter().any(|s| s.path == "src/skip.py"));
        assert!(report
            .skipped
            .iter()
            .any(|s| s.path == "src/generated.generated.ts"));
    }

    #[test]
    fn includes_act_as_a_whitelist() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, "src/a.ts", "export function a() {}");
        write(root, "src/b.ts", "export function b() {}");
        let options = DiscoveryOptions {
            includes: vec!["src/a.ts".into()],
            ..Default::default()
        };
        let report = discover(root, &options).unwrap();
        assert_eq!(report.files.len(), 1);
        assert_eq!(report.files[0].path, "src/a.ts");
    }

    #[test]
    fn language_filter_is_applied() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, "a.ts", "export const a = 1;");
        write(root, "b.py", "b = 1");
        let options = DiscoveryOptions {
            languages: vec![Language::Python],
            ..Default::default()
        };
        let report = discover(root, &options).unwrap();
        assert_eq!(report.files.len(), 1);
        assert_eq!(report.files[0].language, Language::Python);
    }

    #[test]
    fn generated_detection_is_conservative() {
        assert!(looks_generated("src/api.pb.go"));
        assert!(looks_generated("web/app.min.js"));
        assert!(!looks_generated("src/generate.rs"));
    }
}
