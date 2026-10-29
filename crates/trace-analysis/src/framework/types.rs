//! Framework detection data structures.
//!
//! Describes a detected framework, the evidence that led to the detection,
//! and the confidence level. Framework detection is deliberately conservative:
//! a framework is only reported when an import, decorator, inheritance, call,
//! or package manifest actually signals its use.

use serde::{Deserialize, Serialize};

/// A detected framework with its identifying metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Framework {
    /// Canonical framework name (e.g. "react", "django", "express").
    pub name: String,
    /// Human-readable display name (e.g. "React", "Django", "Express").
    pub display_name: String,
    /// Language family the framework belongs to.
    pub family: FrameworkFamily,
    /// Optional version hint (from manifest or import, never asserted).
    pub version: Option<String>,
    /// Confidence in the detection.
    pub confidence: FrameworkConfidence,
    /// The signals that caused this framework to be detected.
    pub signals: Vec<FrameworkSignal>,
}

/// Broad family a framework belongs to, used for grouping and coloring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkFamily {
    EcmaScript,
    Python,
    Go,
    Rust,
    Kotlin,
    Java,
    CSharp,
    Php,
    Ruby,
}

impl FrameworkFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            FrameworkFamily::EcmaScript => "ecmascript",
            FrameworkFamily::Python => "python",
            FrameworkFamily::Go => "go",
            FrameworkFamily::Rust => "rust",
            FrameworkFamily::Kotlin => "kotlin",
            FrameworkFamily::Java => "java",
            FrameworkFamily::CSharp => "csharp",
            FrameworkFamily::Php => "php",
            FrameworkFamily::Ruby => "ruby",
        }
    }
}

/// Confidence that a framework is actually used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkConfidence {
    /// Multiple independent signals (import + decorator + base class).
    High,
    /// One strong signal (import of the core package).
    Medium,
    /// One weak signal (transitive import or naming heuristic).
    Low,
}

impl FrameworkConfidence {
    pub fn as_str(self) -> &'static str {
        match self {
            FrameworkConfidence::High => "high",
            FrameworkConfidence::Medium => "medium",
            FrameworkConfidence::Low => "low",
        }
    }
}

/// One piece of evidence for a framework detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FrameworkSignal {
    /// What kind of evidence this is.
    pub kind: FrameworkSignalKind,
    /// The matched text (import path, decorator name, base class, etc.).
    pub value: String,
    /// File where the signal was observed, if any.
    pub file: Option<String>,
    /// Line number where the signal was observed, if any.
    pub line: Option<usize>,
}

/// Kind of detection signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkSignalKind {
    /// Import/require statement references a framework package.
    Import,
    /// Decorator or annotation matches a framework pattern.
    Decorator,
    /// Class extends a framework base class.
    BaseClass,
    /// Function call matches a framework API entry point.
    Call,
    /// Package manifest lists the framework as a dependency.
    Manifest,
}

impl FrameworkSignalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FrameworkSignalKind::Import => "import",
            FrameworkSignalKind::Decorator => "decorator",
            FrameworkSignalKind::BaseClass => "base_class",
            FrameworkSignalKind::Call => "call",
            FrameworkSignalKind::Manifest => "manifest",
        }
    }
}

/// One framework signature used to match source against a known framework.
#[derive(Debug, Clone)]
pub struct FrameworkSignature {
    pub name: &'static str,
    pub display_name: &'static str,
    pub family: FrameworkFamily,
    /// Import paths that indicate the framework (substring match).
    pub import_prefixes: &'static [&'static str],
    /// Decorator/annotation names that indicate the framework.
    pub decorators: &'static [&'static str],
    /// Base class names that indicate the framework.
    pub base_classes: &'static [&'static str],
    /// Function/method calls that indicate the framework.
    pub calls: &'static [&'static str],
    /// Manifest files whose contents indicate the framework.
    pub manifest_files: &'static [&'static str],
    /// Manifest dependency substrings that indicate the framework.
    pub manifest_deps: &'static [&'static str],
    /// Confidence when only an import matches (no decorator/base class/call).
    pub import_confidence: FrameworkConfidence,
    /// Confidence when a decorator or base class matches.
    pub pattern_confidence: FrameworkConfidence,
    /// Confidence when only a manifest dependency matches.
    pub manifest_confidence: FrameworkConfidence,
}
