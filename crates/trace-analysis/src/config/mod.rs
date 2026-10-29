pub mod helpers;
pub mod tests;
pub mod types;
pub mod validate;

use std::path::{Path, PathBuf};

use trace_core::Language;

use crate::error::AnalysisError;

pub const CONFIG_FILE_NAME: &str = "trace.toml";
pub const DEFAULT_CACHE_DIR: &str = ".trace-cache";
pub const DEFAULT_MAX_FILE_SIZE: u64 = 8 * 1024 * 1024;

pub use types::{AnalysisConfig, CacheConfig, OutputConfig, SemanticMode, TraceConfig};

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

    pub fn languages(&self) -> Vec<Language> {
        helpers::languages(self)
    }

    /// Repository-relative cache directory. Absolute external cache locations
    /// are allowed but are not included in canonical graph metadata.
    pub fn cache_directory(&self, root: &Path) -> PathBuf {
        helpers::cache_directory(self, root)
    }
}
