use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

use trace_core::{Diagnostic, DiagnosticKind, Language};

use crate::util::strip_quotes;

/// Directory names pruned unconditionally unless the user supplies an explicit
/// include pattern that names them. This list is documented in the README.
pub const DEFAULT_EXCLUDED_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".trace-cache",
    ".idea",
    ".vscode",
    ".gradle",
    ".venv",
    "venv",
    "__pycache__",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    "node_modules",
    "bower_components",
    "vendor",
    "third_party",
    "target",
    "build",
    "dist",
    "out",
    "coverage",
    ".next",
    ".nuxt",
    ".cache",
    "Pods",
    "DerivedData",
    ".build",
    ".terraform",
];

#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub languages: Vec<Language>,
    pub max_file_size: u64,
    pub follow_symlinks: bool,
    pub include_generated: bool,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            includes: Vec::new(),
            excludes: Vec::new(),
            languages: Vec::new(),
            max_file_size: 8 * 1024 * 1024,
            follow_symlinks: false,
            include_generated: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredFile {
    /// Repository-relative path with `/` separators.
    pub path: String,
    pub absolute: PathBuf,
    pub language: Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    Excluded,
    Generated,
    Unsupported,
    TooLarge,
    Binary,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedFile {
    pub path: String,
    pub reason: SkipReason,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct DiscoveryReport {
    pub root: PathBuf,
    pub files: Vec<DiscoveredFile>,
    /// Supported-language files deliberately skipped, with reasons.
    pub skipped: Vec<SkippedFile>,
    /// Count of files with unrecognised extensions (not individually listed).
    pub unsupported_files: usize,
    pub ignored_files: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl DiscoveryReport {
    /// Deterministic record of skip counts for reports.
    pub fn skip_counts(&self) -> std::collections::BTreeMap<String, usize> {
        let mut counts = std::collections::BTreeMap::new();
        for skipped in &self.skipped {
            *counts
                .entry(format!("{:?}", skipped.reason).to_lowercase())
                .or_insert(0) += 1;
        }
        counts
    }
}

/// Discover supported source files under `root`.
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

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn compile_globs(patterns: &[String], flag: &str) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let pattern = strip_quotes(pattern);
        let glob = Glob::new(&pattern)
            .map_err(|error| format!("invalid {flag} pattern `{pattern}`: {error}"))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|error| format!("cannot compile {flag} patterns: {error}"))
}

/// Conservative generated-file detection based on common naming conventions.
pub fn looks_generated(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    name.contains(".min.")
        || name.contains(".generated.")
        || name.contains(".designer.")
        || name.ends_with("_generated.rs")
        || name.ends_with("_generated.go")
        || name.ends_with(".pb.go")
        || name.ends_with(".pb.swift")
        || name.ends_with(".g.swift")
        || name == "bindings.rs"
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
