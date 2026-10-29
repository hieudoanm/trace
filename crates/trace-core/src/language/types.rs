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
    #[serde(rename = "c")]
    C,
    #[serde(rename = "cpp")]
    Cpp,
    #[serde(rename = "dart")]
    Dart,
    #[serde(rename = "scala")]
    Scala,
    #[serde(rename = "r")]
    R,
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
    C,
    Cpp,
    Dart,
    Scala,
    R,
}
