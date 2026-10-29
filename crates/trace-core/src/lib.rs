//! Core data types shared by every Trace component.
//!
//! The core crate owns the canonical graph model. It deliberately has no
//! knowledge of tree-sitter, language adapters or command-line concerns.

pub mod diagnostic;
pub mod edge;
pub mod error;
pub mod graph;
pub mod id;
pub mod language;
pub mod location;
pub mod node;
pub mod query;

pub use diagnostic::{Diagnostic, DiagnosticKind, DiagnosticSeverity};
pub use edge::{Edge, EdgeKind, ResolutionOrigin, ResolutionStatus};
pub use error::{CoreError, Result};
pub use graph::{Graph, NodeFilter, Stats};
pub use id::{external_id, file_id, module_id, package_id, repository_id, symbol_id};
pub use language::Language;
pub use location::{Position, SourceRange};
pub use node::{Node, NodeKind, Visibility};
pub use query::{ImpactNode, Neighbor, PathResult, QueryOptions};
