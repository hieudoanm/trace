use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use crate::language::Language;
use crate::location::SourceRange;

/// Kinds of source-code entities represented as graph nodes.
///
/// New variants may be added without changing the graph engine; consumers
/// should treat unknown kinds as opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Repository,
    File,
    Module,
    Package,
    Namespace,
    Class,
    Struct,
    Interface,
    Trait,
    Enum,
    Function,
    Method,
    Constructor,
    Variable,
    Constant,
    Type,
    Property,
    Parameter,
    ExternalSymbol,
    Unknown,
}

impl NodeKind {
    pub const ALL: [NodeKind; 20] = [
        NodeKind::Repository,
        NodeKind::File,
        NodeKind::Module,
        NodeKind::Package,
        NodeKind::Namespace,
        NodeKind::Class,
        NodeKind::Struct,
        NodeKind::Interface,
        NodeKind::Trait,
        NodeKind::Enum,
        NodeKind::Function,
        NodeKind::Method,
        NodeKind::Constructor,
        NodeKind::Variable,
        NodeKind::Constant,
        NodeKind::Type,
        NodeKind::Property,
        NodeKind::Parameter,
        NodeKind::ExternalSymbol,
        NodeKind::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            NodeKind::Repository => "repository",
            NodeKind::File => "file",
            NodeKind::Module => "module",
            NodeKind::Package => "package",
            NodeKind::Namespace => "namespace",
            NodeKind::Class => "class",
            NodeKind::Struct => "struct",
            NodeKind::Interface => "interface",
            NodeKind::Trait => "trait",
            NodeKind::Enum => "enum",
            NodeKind::Function => "function",
            NodeKind::Method => "method",
            NodeKind::Constructor => "constructor",
            NodeKind::Variable => "variable",
            NodeKind::Constant => "constant",
            NodeKind::Type => "type",
            NodeKind::Property => "property",
            NodeKind::Parameter => "parameter",
            NodeKind::ExternalSymbol => "external_symbol",
            NodeKind::Unknown => "unknown",
        }
    }

    pub fn parse_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.as_str() == name)
    }

    /// Whether this kind can own nested declarations.
    pub fn is_container(self) -> bool {
        matches!(
            self,
            NodeKind::Repository
                | NodeKind::File
                | NodeKind::Module
                | NodeKind::Package
                | NodeKind::Namespace
                | NodeKind::Class
                | NodeKind::Struct
                | NodeKind::Interface
                | NodeKind::Trait
                | NodeKind::Enum
        )
    }

    /// Whether this kind is callable and can be the source of a call edge.
    pub fn is_callable(self) -> bool {
        matches!(
            self,
            NodeKind::Function | NodeKind::Method | NodeKind::Constructor | NodeKind::Unknown
        )
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Protected,
    Private,
    Internal,
    Package,
    Unknown,
}

impl Visibility {
    pub fn parse(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "public" | "pub" | "open" => Visibility::Public,
            "protected" => Visibility::Protected,
            "private" | "priv" => Visibility::Private,
            "internal" | "fileprivate" => Visibility::Internal,
            "package" => Visibility::Package,
            _ => Visibility::Unknown,
        }
    }
}

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

impl Node {
    pub fn new(id: impl Into<String>, kind: NodeKind, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            name: name.into(),
            qualified_name: None,
            file: None,
            range: None,
            language: None,
            parent: None,
            signature: None,
            visibility: None,
            exported: None,
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_range(mut self, range: SourceRange) -> Self {
        self.range = Some(range);
        self
    }

    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn with_qualified_name(mut self, qualified: impl Into<String>) -> Self {
        self.qualified_name = Some(qualified.into());
        self
    }

    pub fn with_exported(mut self, exported: bool) -> Self {
        self.exported = Some(exported);
        self
    }

    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }

    pub fn display_label(&self) -> String {
        match &self.qualified_name {
            Some(q) if q != &self.name => format!("{} ({})", self.name, self.kind),
            _ => format!("{} ({})", self.name, self.kind),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_kind_names_round_trip() {
        for kind in NodeKind::ALL {
            assert_eq!(NodeKind::parse_name(kind.as_str()), Some(kind));
        }
    }

    #[test]
    fn builder_sets_optional_fields() {
        let node = Node::new("function:a.ts::foo", NodeKind::Function, "foo")
            .with_file("a.ts")
            .with_language(Language::TypeScript)
            .with_qualified_name("foo");
        assert_eq!(node.file.as_deref(), Some("a.ts"));
        assert_eq!(node.language, Some(Language::TypeScript));
    }
}
