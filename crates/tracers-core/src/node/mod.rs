pub mod kind;
pub mod node_impl;
pub mod tests;
pub mod visibility;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use crate::language::Language;
use crate::location::SourceRange;

pub use self::kind::NodeKind;
pub use self::visibility::Visibility;

/// A node in the unified code graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    /// Canonical, stable within an unchanged source tree. Never an array index.
    pub id: String,
    pub kind: NodeKind,
    /// Short display name. This may be changed without affecting `id`.
    pub name: String,
    /// Language-aware qualified name when one can be derived.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualified_name: Option<String>,
    /// Repository-relative path using `/` separators.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<SourceRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
    /// Canonical ID of the lexical or structural parent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exported: Option<bool>,
    /// Small, serializable language-specific facts. Kept deliberately flat so
    /// the schema does not need to understand every language.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
