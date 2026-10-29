//! Project configuration (`trace.toml`).
//!
//! Configuration is optional. Every field has a documented default and command
//! line flags override the corresponding value. Unknown keys are rejected so a
//! typo cannot silently change analysis behaviour.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use trace_core::Language;

use crate::error::AnalysisError;

pub const CONFIG_FILE_NAME: &str = "trace.toml";
pub const DEFAULT_CACHE_DIR: &str = ".trace-cache";
pub const DEFAULT_MAX_FILE_SIZE: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "snake_case")]
pub struct TraceConfig {
    pub analysis: AnalysisConfig,
    pub cache: CacheConfig,
    pub output: OutputConfig,
    /// Optional external tool paths. They are validated but never executed by
    /// the syntax-only analysis pipeline.
    pub tools: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "snake_case")]
pub struct AnalysisConfig {
    /// Enabled language names. Empty means every supported language.
    pub languages: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub include_generated: bool,
    pub follow_symlinks: bool,
    pub max_file_size: u64,
    pub semantic: SemanticMode,
    pub jobs: Option<usize>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            languages: Vec::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            include_generated: false,
            follow_symlinks: false,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            semantic: SemanticMode::Off,
            jobs: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticMode {
    /// Syntax and lexical resolution only.
    Off,
    /// Request compiler-assisted resolution when an integration exists.
    /// This release has no compiler integrations, so `auto` falls back to
    /// syntax analysis and reports the limitation as a diagnostic.
    Auto,
}

impl SemanticMode {
    pub fn as_str(self) -> &'static str {
        match self {
            SemanticMode::Off => "off",
            SemanticMode::Auto => "auto",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "snake_case")]
pub struct CacheConfig {
    pub enabled: bool,
    /// Relative paths are resolved against the repository root.
    pub directory: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: DEFAULT_CACHE_DIR.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "snake_case")]
pub struct OutputConfig {
    /// Default output format for `scan` and `graph`: `json` or `dot`.
    pub format: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
        }
    }
}

impl TraceConfig {
    /// Resolve the configuration file location and parse it.
    ///
    /// When `explicit` is supplied the file must exist. Otherwise `trace.toml`
    /// in the repository root is used when present.
    pub fn load(
        root: &Path,
        explicit: Option<&Path>,
    ) -> Result<(Self, Option<String>), AnalysisError> {
        let path = match explicit {
            Some(path) => Some(path.to_path_buf()),
            None => {
                let candidate = root.join(CONFIG_FILE_NAME);
                candidate.is_file().then_some(candidate)
            }
        };
        let Some(path) = path else {
            return Ok((Self::default(), None));
        };
        let text = std::fs::read_to_string(&path).map_err(|error| {
            AnalysisError::Config(format!("cannot read `{}`: {error}", path.display()))
        })?;
        let config: TraceConfig = toml::from_str(&text).map_err(|error| {
            AnalysisError::Config(format!("invalid `{}`: {error}", path.display()))
        })?;
        config.validate()?;
        let source = if explicit.is_some() {
            Some(path.display().to_string())
        } else {
            Some(CONFIG_FILE_NAME.to_string())
        };
        Ok((config, source))
    }

    pub fn validate(&self) -> Result<(), AnalysisError> {
        if self.analysis.max_file_size == 0 {
            return Err(AnalysisError::Config(
                "analysis.max_file_size must be greater than zero".to_string(),
            ));
        }
        if let Some(jobs) = self.analysis.jobs {
            if jobs == 0 {
                return Err(AnalysisError::Config(
                    "analysis.jobs must be greater than zero".to_string(),
                ));
            }
        }
        for name in &self.analysis.languages {
            if Language::parse_name(name).is_none() {
                return Err(AnalysisError::Config(format!(
                    "unsupported language `{name}` in analysis.languages"
                )));
            }
        }
        if self.cache.directory.trim().is_empty() {
            return Err(AnalysisError::Config(
                "cache.directory must not be empty".to_string(),
            ));
        }
        if self.cache.directory == "." {
            return Err(AnalysisError::Config(
                "cache.directory must not be the repository root".to_string(),
            ));
        }
        match self.output.format.as_str() {
            "json" | "dot" => {}
            other => {
                return Err(AnalysisError::Config(format!(
                    "unsupported output.format `{other}` (expected `json` or `dot`)"
                )));
            }
        }
        for (name, path) in &self.tools {
            if name.trim().is_empty() || path.trim().is_empty() {
                return Err(AnalysisError::Config(
                    "tools entries must have non-empty names and paths".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn languages(&self) -> Vec<Language> {
        self.analysis
            .languages
            .iter()
            .filter_map(|name| Language::parse_name(name))
            .collect()
    }

    /// Repository-relative cache directory. Absolute external cache locations
    /// are allowed but are not included in canonical graph metadata.
    pub fn cache_directory(&self, root: &Path) -> PathBuf {
        let configured = Path::new(&self.cache.directory);
        if configured.is_absolute() {
            configured.to_path_buf()
        } else {
            root.join(configured)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_syntax_only() {
        let config = TraceConfig::default();
        assert_eq!(config.analysis.semantic, SemanticMode::Off);
        assert!(config.cache.enabled);
        assert!(config.languages().is_empty());
        config.validate().unwrap();
    }

    #[test]
    fn rejects_unknown_keys_and_bad_languages() {
        let error = toml::from_str::<TraceConfig>("[analysis]\nlangauges = []\n").unwrap_err();
        assert!(error.to_string().contains("langauges"));

        let config = TraceConfig {
            analysis: AnalysisConfig {
                languages: vec!["cobol".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn loads_optional_file() {
        let dir = tempfile::tempdir().unwrap();
        let (config, source) = TraceConfig::load(dir.path(), None).unwrap();
        assert!(source.is_none());
        assert!(config.cache.enabled);

        std::fs::write(
            dir.path().join(CONFIG_FILE_NAME),
            "[analysis]\nlanguages = [\"rust\"]\n",
        )
        .unwrap();
        let (config, source) = TraceConfig::load(dir.path(), None).unwrap();
        assert_eq!(source.as_deref(), Some(CONFIG_FILE_NAME));
        assert_eq!(config.languages(), vec![Language::Rust]);
    }
}
