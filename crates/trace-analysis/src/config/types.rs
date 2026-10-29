use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "snake_case")]
pub struct TraceConfig {
    pub analysis: AnalysisConfig,
    pub cache: CacheConfig,
    pub output: OutputConfig,
    /// Optional external tool paths. They are validated but never executed by
    /// the syntax-only analysis pipeline.
    pub tools: std::collections::BTreeMap<String, String>,
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
            max_file_size: super::DEFAULT_MAX_FILE_SIZE,
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
            directory: super::DEFAULT_CACHE_DIR.to_string(),
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
