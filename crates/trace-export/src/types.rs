use serde::{Deserialize, Serialize};

use trace_core::graph::Stats;
use trace_core::node::NodeKind;

/// Metadata about the analysed repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMetadata {
    pub id: String,
    pub name: String,
    pub root: String,
    pub file_count: usize,
    pub language_count: usize,
}

/// Metadata about the analysis run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisMetadata {
    pub format: String,
    pub cache_enabled: bool,
    pub semantic_mode: String,
    pub languages: Vec<String>,
}

/// Information about one supported language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageInfo {
    pub language: String,
    pub grammar: String,
}

/// Information about one detected framework.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkInfo {
    pub name: String,
    pub display_name: String,
    pub family: String,
    pub version: Option<String>,
    pub confidence: String,
}

/// A versioned graph document suitable for JSON export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphDocument {
    pub schema_version: String,
    pub trace_version: String,
    pub repository: RepositoryMetadata,
    pub analysis: AnalysisMetadata,
    pub languages: Vec<LanguageInfo>,
    pub frameworks: Vec<FrameworkInfo>,
    pub nodes: Vec<trace_core::node::Node>,
    pub edges: Vec<trace_core::edge::Edge>,
    pub diagnostics: Vec<trace_core::diagnostic::Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<Stats>,
}

impl GraphDocument {
    pub fn file_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| n.kind == NodeKind::File)
            .count()
    }

    pub fn language_count(&self) -> usize {
        self.languages.len()
    }
}
