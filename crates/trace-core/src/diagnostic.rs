use crate::language::Language;
use crate::location::SourceRange;
use serde::{Deserialize, Serialize};

use crate::edge::ResolutionStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

impl DiagnosticSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Info => "info",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    ParseError,
    UnsupportedSyntax,
    MissingTooling,
    UnresolvedImport,
    AmbiguousSymbol,
    ToolFailure,
    InvalidConfiguration,
    UnreadableFile,
    PartialAnalysis,
    Cache,
    Internal,
}

impl DiagnosticKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticKind::ParseError => "parse_error",
            DiagnosticKind::UnsupportedSyntax => "unsupported_syntax",
            DiagnosticKind::MissingTooling => "missing_tooling",
            DiagnosticKind::UnresolvedImport => "unresolved_import",
            DiagnosticKind::AmbiguousSymbol => "ambiguous_symbol",
            DiagnosticKind::ToolFailure => "tool_failure",
            DiagnosticKind::InvalidConfiguration => "invalid_configuration",
            DiagnosticKind::UnreadableFile => "unreadable_file",
            DiagnosticKind::PartialAnalysis => "partial_analysis",
            DiagnosticKind::Cache => "cache",
            DiagnosticKind::Internal => "internal",
        }
    }
}

/// Structured, machine-readable diagnostic. Diagnostics never contain source
/// text, only locations and messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub kind: DiagnosticKind,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<SourceRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
}

impl Diagnostic {
    pub fn new(
        severity: DiagnosticSeverity,
        kind: DiagnosticKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            kind,
            message: message.into(),
            file: None,
            range: None,
            language: None,
        }
    }

    pub fn error(kind: DiagnosticKind, message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Error, kind, message)
    }

    pub fn warning(kind: DiagnosticKind, message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Warning, kind, message)
    }

    pub fn info(kind: DiagnosticKind, message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Info, kind, message)
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_range(mut self, range: SourceRange) -> Self {
        self.range = Some(range);
        self
    }

    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    pub fn sort_key(&self) -> (u8, String, u32, String, &'static str) {
        let severity = match self.severity {
            DiagnosticSeverity::Error => 0,
            DiagnosticSeverity::Warning => 1,
            DiagnosticSeverity::Info => 2,
        };
        (
            severity,
            self.file.clone().unwrap_or_default(),
            self.range.map(|r| r.start.byte).unwrap_or(0),
            self.message.clone(),
            self.kind.as_str(),
        )
    }
}

/// Record a resolution status for callers that only need to distinguish
/// unresolved relationships in diagnostics.
pub fn is_unresolved(status: ResolutionStatus) -> bool {
    matches!(
        status,
        ResolutionStatus::Unresolved | ResolutionStatus::Ambiguous | ResolutionStatus::SyntaxOnly
    )
}
