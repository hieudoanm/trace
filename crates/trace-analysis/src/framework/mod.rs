//! Framework detection: recognises application frameworks from imports,
//! decorators, base classes, calls, and package manifests.

pub mod detector;
pub mod lookup;
pub mod manifest;
pub mod result;
pub mod scoring;
pub mod signatures;
pub mod types;

pub use detector::FrameworkDetector;
pub use manifest::ManifestContents;
pub use result::FrameworkDetectionResult;
pub use scoring::{best_confidence, extract_version, language_matches, version_from_text};
pub use types::{
    Framework, FrameworkConfidence, FrameworkFamily, FrameworkSignal, FrameworkSignalKind,
};
