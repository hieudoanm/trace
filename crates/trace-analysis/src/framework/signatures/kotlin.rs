//! Framework signatures for Kotlin family (Ktor).

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const KOTLIN_SIGNATURES: &[FrameworkSignature] = &[FrameworkSignature {
    name: "ktor",
    display_name: "Ktor",
    family: FrameworkFamily::Kotlin,
    import_prefixes: &["io.ktor"],
    decorators: &[],
    base_classes: &[],
    calls: &["embeddedServer"],
    manifest_files: &[
        "build.gradle",
        "build.gradle.kts",
        "pom.xml",
        "settings.gradle.kts",
    ],
    manifest_deps: &["ktor-server-core", "ktor-server-netty"],
    import_confidence: FrameworkConfidence::Medium,
    pattern_confidence: FrameworkConfidence::High,
    manifest_confidence: FrameworkConfidence::Medium,
}];
