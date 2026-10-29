use serde_json::Value;

use crate::types::GraphDocument;

/// Serialize a graph document to a pretty-printed JSON value.
pub fn to_json_value(document: &GraphDocument) -> Value {
    serde_json::to_value(document).expect("graph document is always serializable")
}

/// Serialize a graph document to a pretty-printed JSON string.
pub fn to_json_string(document: &GraphDocument) -> String {
    serde_json::to_string_pretty(document).expect("graph document is always serializable")
}

/// Write a graph document as deterministic JSON to a writer.
pub fn write_json<W: std::io::Write>(writer: W, document: &GraphDocument) -> std::io::Result<()> {
    serde_json::to_writer_pretty(writer, document)
        .map_err(std::io::Error::other)
        .map(|_| ())
}
