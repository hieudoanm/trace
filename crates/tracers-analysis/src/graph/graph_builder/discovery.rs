//! Manifest file discovery for framework detection.

use std::path::Path;

use crate::framework::ManifestContents;

/// Known manifest basenames used for framework detection.
const MANIFEST_BASENAMES: &[&str] = &[
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
];

/// Manifest paths matched by suffix (e.g. Rails engines under `engines/*/config/`).
const MANIFEST_SUFFIXES: &[&str] = &["config/application.rb"];

/// Upper bound on manifest files read per scan; monorepos can contain
/// hundreds of `package.json`/`Cargo.toml` files.
const MAX_MANIFESTS: usize = 200;

/// Discover manifest files under `root` whose names are known framework
/// signals. Walks the repository (respecting `.gitignore`, hidden files and
/// symlinks) so monorepo sub-packages are found, not just root manifests.
/// Returns repository-relative paths to file contents.
pub fn discover_manifests(root: &Path) -> ManifestContents {
    let mut files = ManifestContents::empty();
    let walker = ignore::WalkBuilder::new(root)
        .standard_filters(true)
        .build();
    for entry in walker.flatten() {
        if files.files.len() >= MAX_MANIFESTS {
            break;
        }
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let relative_str = relative.to_string_lossy();
        let matches = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| MANIFEST_BASENAMES.contains(&name))
            || MANIFEST_SUFFIXES
                .iter()
                .any(|suffix| relative_str.ends_with(suffix));
        if !matches {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(path) {
            files.files.insert(relative_str.into_owned(), text);
        }
    }
    files
}
