use thiserror::Error;

use trace_core::error::CoreError;

#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("file discovery failed: {0}")]
    Discovery(String),
    #[error("cannot read `{path}`: {message}")]
    Read { path: String, message: String },
    #[error("parser setup failed: {0}")]
    Parser(String),
    #[error("graph construction failed: {0}")]
    Graph(String),
    #[error("core graph error: {0}")]
    Core(#[from] CoreError),
    #[error("cache operation failed: {0}")]
    Cache(String),
    #[error("analysis did not produce a valid graph: {0}")]
    InvalidGraph(String),
}

pub type Result<T> = std::result::Result<T, AnalysisError>;
