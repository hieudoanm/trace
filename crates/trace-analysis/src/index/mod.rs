//! Repository symbol index.
//!
//! The index is built once from syntax results and is immutable during
//! resolution except for per-file import bindings, which are filled in a
//! separate pass. All maps are ordered so iteration is deterministic.

pub mod builder;
pub mod helpers;
pub mod manifest;
pub mod repository;
pub mod repository_queries;
pub mod types;

#[cfg(test)]
mod tests;

pub use helpers::{
    directory_key, is_go, is_kotlin, is_python, is_rust, is_swift, parent_directory,
    python_module_name, rust_module_qualified, stem,
};
pub use repository::{GoModule, PackageState, RepositoryIndex};
pub use types::{symbol_ref, Binding, FileState, ModuleRef, SymbolRef};
