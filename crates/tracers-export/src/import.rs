//! Import a previously exported graph document back into a [`Graph`].
//!
//! This lets query commands run against `.tracers/graph.json` (or any
//! exported document) without rescanning the repository. Only schema
//! version `1.0` documents are accepted; anything else is rejected rather
//! than silently misread.

use std::path::{Path, PathBuf};

use thiserror::Error;
use tracers_core::error::Result as CoreResult;
use tracers_core::Graph;

use crate::export::SCHEMA_VERSION;
use crate::types::GraphDocument;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("cannot read graph file `{path}`: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid graph JSON in `{path}`: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unsupported graph schema version `{found}` (expected `{SCHEMA_VERSION}`)")]
    SchemaVersion { found: String },
    #[error(transparent)]
    Graph(#[from] tracers_core::CoreError),
}

/// Rebuild a [`Graph`] from an exported document's nodes and edges.
pub fn graph_from_document(document: &GraphDocument) -> CoreResult<Graph> {
    let mut graph = Graph::new();
    for node in &document.nodes {
        graph.add_node(node.clone())?;
    }
    for edge in &document.edges {
        graph.add_edge(edge.clone())?;
    }
    graph.finalize();
    Ok(graph)
}

/// Load and schema-check an exported graph JSON document.
pub fn load_document(path: &Path) -> Result<GraphDocument, ImportError> {
    let text = std::fs::read_to_string(path).map_err(|source| ImportError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let document: GraphDocument =
        serde_json::from_str(&text).map_err(|source| ImportError::Json {
            path: path.to_path_buf(),
            source,
        })?;
    if document.schema_version != SCHEMA_VERSION {
        return Err(ImportError::SchemaVersion {
            found: document.schema_version,
        });
    }
    Ok(document)
}

/// Load a graph JSON file produced by `tracers scan`/`tracers graph`.
pub fn load_graph(path: &Path) -> Result<Graph, ImportError> {
    let document = load_document(path)?;
    Ok(graph_from_document(&document)?)
}
