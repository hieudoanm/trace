//! Lookup functions for framework signatures.
//!
//! These functions search the [`FRAMEWORK_SIGNATURES`] database for entries
//! matching imports, decorators, base classes, calls, and manifest
//! dependencies.

use crate::framework::signatures::FRAMEWORK_SIGNATURES;
use crate::framework::types::FrameworkSignature;

/// Look up all signatures whose import prefixes match a given import path.
///
/// A prefix matches only at a package boundary: the import path must equal
/// the prefix, or be followed by `/` (a subpath import). Prefixes ending in
/// `/` (scoped package scopes like `@angular/`) also match any package within
/// that scope. This prevents `react-native` from matching the `react` prefix.
pub fn signatures_for_import(import_path: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| {
            sig.import_prefixes.iter().any(|prefix| {
                if import_path == *prefix {
                    return true;
                }
                let Some(rest) = import_path.strip_prefix(prefix) else {
                    return false;
                };
                rest.is_empty() || rest.starts_with('/') || prefix.ends_with('/')
            })
        })
        .collect()
}

/// Look up all signatures whose decorators contain a given name.
pub fn signatures_for_decorator(decorator: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| sig.decorators.iter().any(|d| d == &decorator))
        .collect()
}

/// Look up all signatures whose base classes contain a given name.
pub fn signatures_for_base_class(base: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| sig.base_classes.iter().any(|b| b == &base))
        .collect()
}

/// Look up all signatures whose calls contain a given name.
pub fn signatures_for_call(call: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| sig.calls.iter().any(|c| c == &call))
        .collect()
}

/// Look up all signatures whose manifest dependencies contain a given dep.
pub fn signatures_for_manifest_dep(dep: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| sig.manifest_deps.iter().any(|d| dep.contains(*d)))
        .collect()
}
