use crate::language::types::{Language, LanguageFamily};

impl Language {
    pub const ALL: [Language; 18] = [
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
        Language::C,
        Language::Cpp,
        Language::Dart,
        Language::Scala,
        Language::R,
    ];

    /// Primary languages listed in the project specification.
    pub const PRIMARY: [Language; 15] = [
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
        Language::C,
        Language::Cpp,
        Language::Dart,
        Language::Scala,
        Language::R,
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
            "c" | "h" => Some(Language::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Some(Language::Cpp),
            "dart" => Some(Language::Dart),
            "scala" | "sc" => Some(Language::Scala),
            "r" | "rmd" => Some(Language::R),
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
            "c" => Some(Language::C),
            "cpp" | "c++" | "cxx" | "cc" => Some(Language::Cpp),
            "dart" => Some(Language::Dart),
            "scala" => Some(Language::Scala),
            "r" => Some(Language::R),
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
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Dart => "dart",
            Language::Scala => "scala",
            Language::R => "r",
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
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Dart => "Dart",
            Language::Scala => "Scala",
            Language::R => "R",
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
            Language::C => LanguageFamily::C,
            Language::Cpp => LanguageFamily::Cpp,
            Language::Dart => LanguageFamily::Dart,
            Language::Scala => LanguageFamily::Scala,
            Language::R => LanguageFamily::R,
        }
    }
}
