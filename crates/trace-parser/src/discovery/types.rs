use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use trace_core::{Diagnostic, Language};
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
    pub skipped: Vec<SkippedFile>,
    pub unsupported_files: usize,
    pub ignored_files: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl DiscoveryReport {
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
