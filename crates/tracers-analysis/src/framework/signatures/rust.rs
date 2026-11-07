//! Framework signatures for Rust family (Actix Web, Axum, clap.rs).

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const RUST_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "actix-web",
        display_name: "Actix Web",
        family: FrameworkFamily::Rust,
        import_prefixes: &["actix_web"],
        decorators: &["get", "post", "put", "delete", "patch"],
        base_classes: &[],
        calls: &["HttpServer", "App"],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["actix-web"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "axum",
        display_name: "Axum",
        family: FrameworkFamily::Rust,
        import_prefixes: &["axum"],
        decorators: &[],
        base_classes: &[],
        calls: &["Router", "routing"],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["axum"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "clap.rs",
        display_name: "clap.rs",
        family: FrameworkFamily::Rust,
        import_prefixes: &["clap"],
        decorators: &[],
        base_classes: &[],
        calls: &["Command"],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["clap"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
