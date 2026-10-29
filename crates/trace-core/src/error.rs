use thiserror::Error;

pub type Result<T> = std::result::Result<T, CoreError>;

/// Errors produced by core graph operations and graph files.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("node `{0}` is not present in the graph")]
    UnknownNode(String),
    #[error("duplicate node id `{0}` with conflicting contents")]
    ConflictingNode(String),
    #[error("invalid graph: {0}")]
    InvalidGraph(String),
    #[error("graph JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("graph document uses unsupported schema version `{0}` (supported: {1})")]
    UnsupportedSchema(String, String),
    #[error("graph document is malformed: {0}")]
    MalformedGraph(String),
}
