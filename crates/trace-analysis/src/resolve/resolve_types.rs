//! Core types for import and symbol resolution.
//!
//! Defines the `Want`, `Resolution`, `ImportEdge`, and `ImportPass` types
//! used throughout the resolution pipeline.

use trace_core::{Diagnostic, ResolutionStatus, SourceRange};

use crate::index::{ModuleRef, SymbolRef};

/// What kind of symbol a caller is looking for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Want {
    Any,
    Type,
    Callable,
}

impl Want {
    pub(crate) fn accepts(self, symbol: &SymbolRef) -> bool {
        match self {
            Want::Any => true,
            Want::Type => symbol.is_type(),
            Want::Callable => symbol.is_callable(),
        }
    }
}

/// Outcome of resolving a name or import target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    Internal(SymbolRef),
    Module(ModuleRef),
    External {
        qualified: String,
        name: String,
    },
    ExternalNode {
        node_id: String,
        name: String,
    },
    Unresolved {
        name: String,
    },
    Ambiguous {
        name: String,
        candidates: Vec<String>,
    },
}

impl Resolution {
    pub fn status(&self) -> ResolutionStatus {
        match self {
            Resolution::Internal(_) => ResolutionStatus::ResolvedInternal,
            Resolution::Module(ModuleRef::External(_)) => ResolutionStatus::ResolvedExternal,
            Resolution::Module(_) => ResolutionStatus::ResolvedInternal,
            Resolution::External { .. } | Resolution::ExternalNode { .. } => {
                ResolutionStatus::ResolvedExternal
            }
            Resolution::Unresolved { .. } => ResolutionStatus::Unresolved,
            Resolution::Ambiguous { .. } => ResolutionStatus::Ambiguous,
        }
    }

    pub fn is_resolved(&self) -> bool {
        !matches!(
            self,
            Resolution::Unresolved { .. } | Resolution::Ambiguous { .. }
        )
    }
}

/// A single import edge discovered during the import pass.
#[derive(Debug, Clone)]
pub struct ImportEdge {
    pub from_path: String,
    pub module: ModuleRef,
    pub site: SourceRange,
    pub status: ResolutionStatus,
}

/// Result of resolving all imports in a repository.
#[derive(Debug, Clone, Default)]
pub struct ImportPass {
    pub edges: Vec<ImportEdge>,
    pub diagnostics: Vec<Diagnostic>,
    pub package_pairs: std::collections::BTreeSet<(String, String)>,
}
