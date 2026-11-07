use serde::{Deserialize, Serialize};

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
    Framework,
    Unknown,
}

impl NodeKind {
    pub const ALL: [NodeKind; 21] = [
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
        NodeKind::Framework,
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
            NodeKind::Framework => "framework",
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
                | NodeKind::Framework
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
