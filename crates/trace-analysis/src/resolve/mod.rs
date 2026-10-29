//! Import and symbol resolution.
//!
//! Resolution is deliberately layered and conservative. Every outcome records
//! whether it is internal, external, unresolved or ambiguous; name-only matches
//! across unrelated modules are not treated as resolved.

pub mod resolve_bindings;
pub mod resolve_import_misc;
pub mod resolve_import_pass;
pub mod resolve_members;
pub mod resolve_modules;
pub mod resolve_path_helpers;
pub mod resolve_resolution_ext;
pub mod resolve_resolver;
#[cfg(test)]
pub mod resolve_tests;
pub mod resolve_types;

pub use resolve_import_pass::resolve_imports;
pub use resolve_types::{ImportEdge, ImportPass, Resolution, Want};
