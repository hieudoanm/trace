//! Manifest file discovery for framework detection.

use std::path::Path;

use crate::framework::ManifestContents;

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
pub(crate) fn discover_manifests(root: &Path) -> ManifestContents {
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
