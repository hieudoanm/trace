//! Content-addressed syntax cache.
//!
//! Only syntax extraction is cached. Resolution and graph construction always
//! run against the current index, so a stale cache entry can never change
//! resolution semantics as long as its source hash and parser version match.
//! Entries are written atomically through a temporary file in the cache
//! directory.

use std::path::{Path, PathBuf};

use trace_core::Language;
use trace_parser::{grammar_version, ParsedFile};

use crate::config::{CacheConfig, TraceConfig};

const CACHE_FORMAT_VERSION: &str = "1";
const TRACE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CacheCounts {
    pub hits: usize,
    pub misses: usize,
    pub writes: usize,
}

#[derive(Debug, Clone)]
pub struct SyntaxCache {
    directory: PathBuf,
    enabled: bool,
}

impl SyntaxCache {
    pub fn from_config(root: &Path, config: &TraceConfig, no_cache: bool) -> Self {
        Self {
            directory: config.cache_directory(root),
            enabled: config.cache.enabled && !no_cache,
        }
    }

    pub fn new(directory: PathBuf, enabled: bool) -> Self {
        Self { directory, enabled }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    fn key(&self, path: &str, language: Language, source_hash: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(CACHE_FORMAT_VERSION.as_bytes());
        hasher.update(TRACE_VERSION.as_bytes());
        hasher.update(grammar_version(language).as_bytes());
        hasher.update(language.as_str().as_bytes());
        hasher.update(path.as_bytes());
        hasher.update(source_hash.as_bytes());
        hasher.finalize().to_hex().to_string()
    }

    fn entry_path(&self, key: &str) -> PathBuf {
        self.directory
            .join("syntax")
            .join(&key[..2])
            .join(format!("{key}.json"))
    }

    /// Load a cached syntax result, validating every identity field.
    pub fn load(&self, path: &str, language: Language, source_hash: &str) -> Option<ParsedFile> {
        if !self.enabled {
            return None;
        }
        let key = self.key(path, language, source_hash);
        let entry = self.entry_path(&key);
        let bytes = std::fs::read(&entry).ok()?;
        let parsed: ParsedFile = serde_json::from_slice(&bytes).ok()?;
        if parsed.path != path || parsed.language != language || parsed.source_hash != source_hash {
            return None;
        }
        Some(parsed)
    }

    /// Store a syntax result. Cache failures are best-effort: the caller keeps
    /// the freshly parsed value and the scan remains correct.
    pub fn store(&self, parsed: &ParsedFile) -> bool {
        if !self.enabled {
            return false;
        }
        let key = self.key(&parsed.path, parsed.language, &parsed.source_hash);
        let entry = self.entry_path(&key);
        let Some(parent) = entry.parent() else {
            return false;
        };
        if std::fs::create_dir_all(parent).is_err() {
            return false;
        }
        let Ok(encoded) = serde_json::to_vec(parsed) else {
            return false;
        };
        let temporary = entry.with_extension(format!("tmp-{}", std::process::id()));
        if std::fs::write(&temporary, encoded).is_err() {
            return false;
        }
        if std::fs::rename(&temporary, &entry).is_err() {
            let _ = std::fs::remove_file(&temporary);
            return false;
        }
        true
    }
}

/// Cached vs fresh parsing is tested through this helper in integration tests.
pub fn cache_key_for(
    path: &str,
    language: Language,
    source_hash: &str,
    config: &CacheConfig,
) -> Option<String> {
    if !config.enabled {
        return None;
    }
    let cache = SyntaxCache::new(PathBuf::from(&config.directory), true);
    Some(cache.key(path, language, source_hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use trace_core::{NodeKind, SourceRange};

    fn parsed() -> ParsedFile {
        ParsedFile {
            path: "src/a.ts".to_string(),
            language: Language::TypeScript,
            source_hash: "abc".to_string(),
            module_name: None,
            symbols: Vec::new(),
            imports: Vec::new(),
            calls: Vec::new(),
            type_refs: Vec::new(),
            heritage: Vec::new(),
            locals: Vec::new(),
            diagnostics: Vec::new(),
            parse_error_count: 0,
        }
    }

    #[test]
    fn round_trips_and_rejects_changed_identity() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SyntaxCache::new(dir.path().to_path_buf(), true);
        let parsed = parsed();
        assert!(cache.store(&parsed));
        let loaded = cache.load("src/a.ts", Language::TypeScript, "abc").unwrap();
        assert_eq!(loaded, parsed);
        assert!(cache
            .load("src/b.ts", Language::TypeScript, "abc")
            .is_none());
        assert!(cache
            .load("src/a.ts", Language::TypeScript, "def")
            .is_none());
    }

    #[test]
    fn disabled_cache_never_reads_or_writes() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SyntaxCache::new(dir.path().to_path_buf(), false);
        assert!(!cache.store(&parsed()));
        assert!(cache
            .load("src/a.ts", Language::TypeScript, "abc")
            .is_none());
    }

    #[test]
    fn key_changes_with_content_and_language() {
        let config = CacheConfig::default();
        let a = cache_key_for("a.ts", Language::TypeScript, "x", &config).unwrap();
        let b = cache_key_for("a.ts", Language::TypeScript, "y", &config).unwrap();
        let c = cache_key_for("a.ts", Language::Tsx, "x", &config).unwrap();
        assert_ne!(a, b);
        assert_ne!(a, c);
        let _ = NodeKind::Function;
        let _ = SourceRange::new(
            trace_core::Position::new(1, 1, 0),
            trace_core::Position::new(1, 1, 0),
        );
    }
}
