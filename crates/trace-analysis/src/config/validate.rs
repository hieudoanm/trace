use crate::error::AnalysisError;

use super::types::TraceConfig;

impl TraceConfig {
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
            if trace_core::Language::parse_name(name).is_none() {
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
}
