use serde::{Deserialize, Serialize};

use crate::location::SourceRange;

/// Relationship kinds in the unified graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Contains,
    Imports,
    Exports,
    Calls,
    References,
    Inherits,
    Implements,
    UsesType,
    Defines,
    Overrides,
    DependsOn,
    UsesFramework,
    ImplementsPattern,
    ExtendsFramework,
    RoutesTo,
    Injects,
}

impl EdgeKind {
    pub const ALL: [EdgeKind; 16] = [
        EdgeKind::Contains,
        EdgeKind::Imports,
        EdgeKind::Exports,
        EdgeKind::Calls,
        EdgeKind::References,
        EdgeKind::Inherits,
        EdgeKind::Implements,
        EdgeKind::UsesType,
        EdgeKind::Defines,
        EdgeKind::Overrides,
        EdgeKind::DependsOn,
        EdgeKind::UsesFramework,
        EdgeKind::ImplementsPattern,
        EdgeKind::ExtendsFramework,
        EdgeKind::RoutesTo,
        EdgeKind::Injects,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            EdgeKind::Contains => "contains",
            EdgeKind::Imports => "imports",
            EdgeKind::Exports => "exports",
            EdgeKind::Calls => "calls",
            EdgeKind::References => "references",
            EdgeKind::Inherits => "inherits",
            EdgeKind::Implements => "implements",
            EdgeKind::UsesType => "uses_type",
            EdgeKind::Defines => "defines",
            EdgeKind::Overrides => "overrides",
            EdgeKind::DependsOn => "depends_on",
            EdgeKind::UsesFramework => "uses_framework",
            EdgeKind::ImplementsPattern => "implements_pattern",
            EdgeKind::ExtendsFramework => "extends_framework",
            EdgeKind::RoutesTo => "routes_to",
            EdgeKind::Injects => "injects",
        }
    }

    pub fn parse_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.as_str() == name)
    }

    /// Whether traversing this edge forwards represents a dependency.
    pub fn is_dependency(self) -> bool {
        matches!(
            self,
            EdgeKind::Imports | EdgeKind::DependsOn | EdgeKind::UsesType | EdgeKind::UsesFramework
        )
    }
}

/// How confidently the target of an edge was identified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStatus {
    /// Target is a symbol or entity analysed in this repository.
    ResolvedInternal,
    /// Target is a known symbol belonging to an external dependency.
    ResolvedExternal,
    /// A relationship was observed but its target could not be identified.
    Unresolved,
    /// More than one plausible target exists and no unambiguous choice is safe.
    Ambiguous,
    /// Relationship is reported from syntax alone and was not semantically
    /// resolved (for example, a dynamic receiver).
    SyntaxOnly,
}

impl ResolutionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ResolutionStatus::ResolvedInternal => "resolved_internal",
            ResolutionStatus::ResolvedExternal => "resolved_external",
            ResolutionStatus::Unresolved => "unresolved",
            ResolutionStatus::Ambiguous => "ambiguous",
            ResolutionStatus::SyntaxOnly => "syntax_only",
        }
    }
}

/// Where a relationship came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionOrigin {
    /// Direct syntax (lexical lookup or structural relationship).
    Syntax,
    /// Import/export metadata.
    Import,
    /// Lexical scope lookup.
    Lexical,
    /// Inferred from constructor expressions or simple assignments.
    TypeInference,
    /// Cross-file naming heuristic with conservative confidence.
    Heuristic,
    /// A language compiler or semantic index supplied the fact.
    Compiler,
}

/// An aggregated directed relationship.
///
/// Call sites between the same pair of symbols are aggregated into one edge
/// with multiple `sites`. The choice keeps the exported graph small while
/// preserving every source location; the JSON schema exposes `sites` as a
/// sorted array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub kind: EdgeKind,
    pub status: ResolutionStatus,
    pub origin: ResolutionOrigin,
    /// Sorted, de-duplicated source locations for this relationship.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sites: Vec<SourceRange>,
}

impl Edge {
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        kind: EdgeKind,
        status: ResolutionStatus,
        origin: ResolutionOrigin,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            kind,
            status,
            origin,
            sites: Vec::new(),
        }
    }

    pub fn at(mut self, site: SourceRange) -> Self {
        self.sites.push(site);
        self
    }

    pub fn aggregation_key(&self) -> (&str, EdgeKind, &str, ResolutionStatus, ResolutionOrigin) {
        (
            &self.source,
            self.kind,
            &self.target,
            self.status,
            self.origin,
        )
    }

    pub fn sort_key(&self) -> (&str, EdgeKind, &str, ResolutionStatus, ResolutionOrigin) {
        self.aggregation_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_kind_names_round_trip() {
        for kind in EdgeKind::ALL {
            assert_eq!(EdgeKind::parse_name(kind.as_str()), Some(kind));
        }
    }

    #[test]
    fn dependency_classification() {
        assert!(EdgeKind::Imports.is_dependency());
        assert!(!EdgeKind::Calls.is_dependency());
    }
}
