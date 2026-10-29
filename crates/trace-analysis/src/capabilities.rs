//! Static, honest descriptions of what each language adapter can do.

use serde::{Deserialize, Serialize};

use trace_core::Language;
use trace_parser::grammar_version;

/// Capability report for one language. These values describe this release's
/// implementation, not theoretical possibilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageCapability {
    pub language: Language,
    pub display_name: String,
    pub files: usize,
    pub grammar: String,
    pub syntax_parsing: bool,
    pub symbol_extraction: bool,
    pub import_resolution: bool,
    pub call_resolution: bool,
    pub cross_file_references: bool,
    pub type_aware_resolution: bool,
    pub semantic_integration: bool,
    /// Optional external tool that a future semantic integration could use.
    pub semantic_tool: Option<String>,
    pub notes: Vec<String>,
}

pub fn capabilities(language: Language, files: usize) -> LanguageCapability {
    let (tool, notes): (Option<&str>, &[&str]) = match language {
        Language::TypeScript | Language::Tsx | Language::JavaScript | Language::Jsx => (
            Some("tsc"),
            &[
                "Relative ES module imports are resolved.",
                "tsconfig path aliases are not applied in this release.",
                "Method resolution uses lexical scope and simple local type inference.",
            ],
        ),
        Language::Python => (
            Some("python3"),
            &[
                "Absolute and relative module imports are resolved against discovered files.",
                "Dynamic imports, monkey-patching and decorator effects are not modelled.",
            ],
        ),
        Language::Go => (
            Some("go"),
            &[
                "Imports resolve through go.mod module paths and package directories.",
                "Interface method sets are reported syntactically; satisfaction is not type-checked.",
            ],
        ),
        Language::Rust => (
            Some("rust-analyzer"),
            &[
                "crate/self/super use paths and simple module files are resolved.",
                "Macro expansion and trait method resolution are not performed.",
            ],
        ),
        Language::Kotlin => (
            Some("kotlinc"),
            &[
                "Imports and same-package declarations are resolved.",
                "Overload selection and extension resolution are conservative.",
            ],
        ),
        Language::Swift => (
            Some("swiftc"),
            &[
                "Files in the same Swift package/module root share a lexical scope.",
                "Protocol witness and dynamic dispatch resolution are not performed.",
            ],
        ),
        Language::Java => (
            Some("javac"),
            &[
                "Imports and same-package declarations are resolved.",
                "Overloads and type erasure are handled conservatively.",
            ],
        ),
        Language::CSharp => (
            Some("csc"),
            &[
                "Using directives and namespace scope are considered.",
                "Advanced generics and LINQ are not fully modelled.",
            ],
        ),
        Language::Php => (
            None,
            &[
                "Use statements and namespaced declarations are resolved.",
                "Dynamic includes and magic methods are not modelled.",
            ],
        ),
        Language::Ruby => (
            None,
            &[
                "require/require_relative are tracked.",
                "Dynamic method definition and metaprogramming are not modelled.",
            ],
        ),
        Language::C | Language::Cpp | Language::Dart | Language::Scala | Language::R | Language::Zig | Language::Matlab | Language::GdScript | Language::Bash | Language::Markdown => (
            None,
            &["Syntax-level parsing only; no semantic integration."],
        ),
    };
    LanguageCapability {
        language,
        display_name: language.display_name().to_string(),
        files,
        grammar: grammar_version(language).to_string(),
        syntax_parsing: true,
        symbol_extraction: true,
        import_resolution: true,
        call_resolution: true,
        cross_file_references: true,
        type_aware_resolution: false,
        semantic_integration: false,
        semantic_tool: tool.map(|t| t.to_string()),
        notes: notes.iter().map(|note| (*note).to_string()).collect(),
    }
}

pub fn all_capabilities() -> Vec<LanguageCapability> {
    Language::ALL
        .iter()
        .copied()
        .map(|language| capabilities(language, 0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_language_claims_type_aware_resolution() {
        for capability in all_capabilities() {
            assert!(capability.syntax_parsing);
            assert!(!capability.type_aware_resolution);
            assert!(!capability.semantic_integration);
            assert!(!capability.grammar.is_empty());
        }
    }
}
