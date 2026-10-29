//! Framework detection: recognises application frameworks from imports,
//! decorators, base classes, calls, and package manifests.

pub mod detector;
pub mod lookup;
pub mod signatures;
pub mod types;

pub use detector::{FrameworkDetectionResult, FrameworkDetector, ManifestContents};
pub use types::{
    Framework, FrameworkConfidence, FrameworkFamily, FrameworkSignal, FrameworkSignalKind,
};
