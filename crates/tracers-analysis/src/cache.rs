//! Content-addressed syntax cache.
//!
//! Only syntax extraction is cached. Resolution and graph construction always
//! run against the current index, so a stale cache entry can never change
//! resolution semantics as long as its source hash and parser version match.
//! Entries are written atomically through a temporary file in the cache
//! directory.

use std::path::{Path, PathBuf};

use tracers_core::Language;
use tracers_parser::{grammar_version, ParsedFile};

#[cfg(test)]
use crate::config::CacheConfig;
use crate::config::TracersConfig;

const CACHE_FORMAT_VERSION: &str = "1";
const TRACERS_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CacheCounts {
    pub hits: usize,
    pub misses: usize,
    pub writes: usize,
}

#[derive(Debug, Default)]
struct CacheStats {
    hits: std::sync::atomic::AtomicUsize,
    misses: std::sync::atomic::AtomicUsize,
    writes: std::sync::atomic::AtomicUsize,
}

#[derive(Debug, Clone)]
pub struct SyntaxCache {
    directory: PathBuf,
    enabled: bool,
    /// Shared across clones so parallel workers accumulate one count.
    stats: std::sync::Arc<CacheStats>,
}

impl SyntaxCache {
    pub fn from_config(root: &Path, config: &TracersConfig, no_cache: bool) -> Self {
        let cache = Self {
            directory: config.cache_directory(root),
            enabled: config.cache.enabled && !no_cache,
            stats: std::sync::Arc::new(CacheStats::default()),
        };
        if cache.enabled && config.cache.max_age_days > 0 {
            cache.prune(std::time::Duration::from_secs(
                config.cache.max_age_days * 24 * 60 * 60,
            ));
        }
        cache
    }

    /// Evict cache entries older than `max_age`. Best-effort: failures are
    /// ignored because stale entries are already rejected by key mismatch.
    fn prune(&self, max_age: std::time::Duration) {
        let syntax_dir = self.directory.join("syntax");
        let Ok(shards) = std::fs::read_dir(&syntax_dir) else {
            return;
        };
        let cutoff = std::time::SystemTime::now() - max_age;
        for shard in shards.flatten() {
            let Ok(entries) = std::fs::read_dir(shard.path()) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let expired = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .is_ok_and(|mtime| mtime < cutoff);
                if expired {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }

    pub fn new(directory: PathBuf, enabled: bool) -> Self {
        Self {
            directory,
            enabled,
            stats: std::sync::Arc::new(CacheStats::default()),
        }
    }

    /// Accumulated cache activity across all clones of this cache.
    pub fn counts(&self) -> CacheCounts {
        use std::sync::atomic::Ordering;
        CacheCounts {
            hits: self.stats.hits.load(Ordering::Relaxed),
            misses: self.stats.misses.load(Ordering::Relaxed),
            writes: self.stats.writes.load(Ordering::Relaxed),
        }
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
        hasher.update(TRACERS_VERSION.as_bytes());
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
        let loaded = self.lookup(path, language, source_hash);
        let counter = if loaded.is_some() {
            &self.stats.hits
        } else {
            &self.stats.misses
        };
        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        loaded
    }

    fn lookup(&self, path: &str, language: Language, source_hash: &str) -> Option<ParsedFile> {
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
        self.stats
            .writes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        true
    }
}

/// Cached vs fresh parsing is tested through this helper in integration tests.
#[cfg(test)]
fn cache_key_for(
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
    use tracers_core::{NodeKind, SourceRange};

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
    fn counts_track_hits_misses_and_writes() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SyntaxCache::new(dir.path().to_path_buf(), true);
        let parsed = parsed();

        assert!(cache
            .load("src/a.ts", Language::TypeScript, "abc")
            .is_none());
        assert!(cache.store(&parsed));
        assert!(cache
            .load("src/a.ts", Language::TypeScript, "abc")
            .is_some());

        let counts = cache.counts();
        assert_eq!(counts.hits, 1);
        assert_eq!(counts.misses, 1);
        assert_eq!(counts.writes, 1);
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
    fn prune_evicts_only_expired_entries() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SyntaxCache::new(dir.path().to_path_buf(), true);
        let shard = dir.path().join("syntax").join("ab");
        std::fs::create_dir_all(&shard).unwrap();

        let old = shard.join("old.json");
        std::fs::write(&old, b"{}").unwrap();
        let fresh = shard.join("fresh.json");
        std::fs::write(&fresh, b"{}").unwrap();

        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(60 * 60 * 24 * 60);
        std::fs::File::options()
            .write(true)
            .open(&old)
            .unwrap()
            .set_modified(past)
            .unwrap();

        cache.prune(std::time::Duration::from_secs(60 * 60 * 24 * 30));

        assert!(!old.exists());
        assert!(fresh.exists());
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
            tracers_core::Position::new(1, 1, 0),
            tracers_core::Position::new(1, 1, 0),
        );
    }
}
