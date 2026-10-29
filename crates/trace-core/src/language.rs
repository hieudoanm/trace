use std::fmt;

use serde::{Deserialize, Serialize};

/// Languages recognised by Trace.
///
/// JavaScript and JSX are companions of TypeScript rather than additional
/// primary languages, but they are represented as distinct languages so that
/// parser selection and reporting stay unambiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[serde(rename = "typescript")]
    TypeScript,
    #[serde(rename = "tsx")]
    Tsx,
    #[serde(rename = "javascript")]
    JavaScript,
    #[serde(rename = "jsx")]
    Jsx,
    #[serde(rename = "python")]
    Python,
    #[serde(rename = "go")]
    Go,
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "kotlin")]
    Kotlin,
    #[serde(rename = "swift")]
    Swift,
    #[serde(rename = "java")]
    Java,
    #[serde(rename = "csharp")]
    CSharp,
    #[serde(rename = "php")]
    Php,
    #[serde(rename = "ruby")]
    Ruby,
}

impl Language {
    pub const ALL: [Language; 13] = [
        Language::TypeScript,
        Language::Tsx,
        Language::JavaScript,
        Language::Jsx,
        Language::Python,
        Language::Go,
        Language::Rust,
        Language::Kotlin,
        Language::Swift,
        Language::Java,
        Language::CSharp,
        Language::Php,
        Language::Ruby,
    ];

    /// Primary languages listed in the project specification.
    pub const PRIMARY: [Language; 10] = [
        Language::TypeScript,
        Language::Python,
        Language::Go,
        Language::Rust,
        Language::Kotlin,
        Language::Swift,
        Language::Java,
        Language::CSharp,
        Language::Php,
        Language::Ruby,
    ];

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "ts" | "mts" | "cts" => Some(Language::TypeScript),
            "tsx" => Some(Language::Tsx),
            "js" | "mjs" | "cjs" => Some(Language::JavaScript),
            "jsx" => Some(Language::Jsx),
            "py" | "pyi" => Some(Language::Python),
            "go" => Some(Language::Go),
            "rs" => Some(Language::Rust),
            "kt" | "kts" => Some(Language::Kotlin),
            "swift" => Some(Language::Swift),
            "java" => Some(Language::Java),
            "cs" => Some(Language::CSharp),
            "php" => Some(Language::Php),
            "rb" => Some(Language::Ruby),
            _ => None,
        }
    }

    /// Parse a user-provided language name.
    pub fn parse_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "typescript" | "ts" => Some(Language::TypeScript),
            "tsx" => Some(Language::Tsx),
            "javascript" | "js" => Some(Language::JavaScript),
            "jsx" => Some(Language::Jsx),
            "python" | "py" => Some(Language::Python),
            "go" | "golang" => Some(Language::Go),
            "rust" | "rs" => Some(Language::Rust),
            "kotlin" | "kt" => Some(Language::Kotlin),
            "swift" => Some(Language::Swift),
            "java" => Some(Language::Java),
            "csharp" | "c#" | "cs" => Some(Language::CSharp),
            "php" => Some(Language::Php),
            "ruby" | "rb" => Some(Language::Ruby),
            _ => None,
        }
    }

    /// Canonical lowercase name used in JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Language::TypeScript => "typescript",
            Language::Tsx => "tsx",
            Language::JavaScript => "javascript",
            Language::Jsx => "jsx",
            Language::Python => "python",
            Language::Go => "go",
            Language::Rust => "rust",
            Language::Kotlin => "kotlin",
            Language::Swift => "swift",
            Language::Java => "java",
            Language::CSharp => "csharp",
            Language::Php => "php",
            Language::Ruby => "ruby",
        }
    }

    /// Human-readable display name.
    pub fn display_name(self) -> &'static str {
        match self {
            Language::TypeScript => "TypeScript",
            Language::Tsx => "TSX",
            Language::JavaScript => "JavaScript",
            Language::Jsx => "JSX",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Rust => "Rust",
            Language::Kotlin => "Kotlin",
            Language::Swift => "Swift",
            Language::Java => "Java",
            Language::CSharp => "C#",
            Language::Php => "PHP",
            Language::Ruby => "Ruby",
        }
    }

    /// Whether this is one of the six primary target languages. TSX/JS/JSX
    /// are companions covered by the TypeScript and JavaScript adapters.
    pub fn is_primary(self) -> bool {
        Self::PRIMARY.contains(&self)
    }

    pub fn family(self) -> LanguageFamily {
        match self {
            Language::TypeScript | Language::Tsx | Language::JavaScript | Language::Jsx => {
                LanguageFamily::EcmaScript
            }
            Language::Python => LanguageFamily::Python,
            Language::Go => LanguageFamily::Go,
            Language::Rust => LanguageFamily::Rust,
            Language::Kotlin => LanguageFamily::Kotlin,
            Language::Swift => LanguageFamily::Swift,
            Language::Java => LanguageFamily::Java,
            Language::CSharp => LanguageFamily::CSharp,
            Language::Php => LanguageFamily::Php,
            Language::Ruby => LanguageFamily::Ruby,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageFamily {
    EcmaScript,
    Python,
    Go,
    Rust,
    Kotlin,
    Swift,
    Java,
    CSharp,
    Php,
    Ruby,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_extensions_case_insensitively() {
        assert_eq!(Language::from_extension("TS"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("kt"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("kts"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("txt"), None);
    }

    #[test]
    fn parses_language_names() {
        assert_eq!(
            Language::parse_name("TypeScript"),
            Some(Language::TypeScript)
        );
        assert_eq!(Language::parse_name("golang"), Some(Language::Go));
        assert_eq!(Language::parse_name("cobol"), None);
    }
}
