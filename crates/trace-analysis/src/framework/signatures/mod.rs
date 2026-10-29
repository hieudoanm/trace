//! Framework signature database.
//!
//! Each entry describes how to recognise one framework from imports,
//! decorators, base classes, calls, and package manifests. Matches are
//! deliberately conservative: substring prefix matching on imports and exact
//! matching on decorator/base-class/call names.

pub mod csharp;
pub mod ecmascript;
pub mod go;
pub mod java;
pub mod kotlin;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;

use std::sync::OnceLock;

use crate::framework::types::FrameworkSignature;

static ALL_SIGNATURES: OnceLock<Vec<FrameworkSignature>> = OnceLock::new();

/// All known framework signatures, ordered for deterministic iteration.
pub fn all_signatures() -> &'static [FrameworkSignature] {
    ALL_SIGNATURES.get_or_init(|| {
        let mut out = Vec::new();
        out.extend_from_slice(ecmascript::ECMASCRIPT_SIGNATURES);
        out.extend_from_slice(python::PYTHON_SIGNATURES);
        out.extend_from_slice(go::GO_SIGNATURES);
        out.extend_from_slice(rust::RUST_SIGNATURES);
        out.extend_from_slice(kotlin::KOTLIN_SIGNATURES);
        out.extend_from_slice(java::JAVA_SIGNATURES);
        out.extend_from_slice(csharp::CSHARP_SIGNATURES);
        out.extend_from_slice(php::PHP_SIGNATURES);
        out.extend_from_slice(ruby::RUBY_SIGNATURES);
        out
    })
}
