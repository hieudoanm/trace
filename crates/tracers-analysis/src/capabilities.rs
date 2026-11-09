//! Static, honest descriptions of what each language adapter can do.

use serde::{Deserialize, Serialize};

use tracers_core::Language;
use tracers_parser::grammar_version;

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
        Language::C | Language::Cpp | Language::Dart | Language::Scala | Language::R => (
            None,
            &["Symbol extraction only; imports, calls and cross-file resolution are not implemented."],
        ),
        Language::Zig | Language::Bash | Language::Matlab | Language::GdScript => (
            None,
            &["Grammar is parsed for syntax trees only; no symbols, imports or calls are extracted."],
        ),
        Language::Markdown => (
            None,
            &["Wiki-link and markdown-link targets are recorded as imports; no symbols or calls."],
        ),
    };
    // Report what each adapter actually extracts rather than claiming
    // uniform coverage. Only the primary adapters emit imports and calls;
    // C-family and scripting adapters are symbol-only; Zig/Bash emit nothing;
    // Markdown records link targets as imports.
    let (symbol_extraction, import_resolution, call_resolution, cross_file) = match language {
        Language::Zig | Language::Bash | Language::Matlab | Language::GdScript => {
            (false, false, false, false)
        }
        Language::Markdown => (false, true, false, false),
        Language::C | Language::Cpp | Language::Dart | Language::Scala | Language::R => {
            (true, false, false, false)
        }
        _ => (true, true, true, true),
    };
    LanguageCapability {
        language,
        display_name: language.display_name().to_string(),
        files,
        grammar: grammar_version(language).to_string(),
        syntax_parsing: true,
        symbol_extraction,
        import_resolution,
        call_resolution,
        cross_file_references: cross_file,
        type_aware_resolution: false,
        semantic_integration: false,
        semantic_tool: tool.map(|t| t.to_string()),
        notes: notes.iter().map(|note| (*note).to_string()).collect(),
    }
}

pub fn all_capabilities() -> Vec<LanguageCapability> {
    // Markdown is not in Language::ALL (it is opt-in via --language to avoid
    // indexing documentation in every repository), but it is still a
    // supported language and must appear in capability reports.
    Language::ALL
        .iter()
        .copied()
        .chain(std::iter::once(Language::Markdown))
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

    #[test]
    fn stub_and_partial_adapters_report_honest_capabilities() {
        // Grammar-only adapters extract nothing.
        for language in [
            Language::Zig,
            Language::Bash,
            Language::Matlab,
            Language::GdScript,
        ] {
            let capability = capabilities(language, 0);
            assert!(!capability.symbol_extraction, "{language}");
            assert!(!capability.import_resolution, "{language}");
            assert!(!capability.call_resolution, "{language}");
            assert!(!capability.cross_file_references, "{language}");
        }
        // Symbol-only adapters extract symbols but no imports/calls.
        for language in [
            Language::C,
            Language::Cpp,
            Language::Dart,
            Language::Scala,
            Language::R,
        ] {
            let capability = capabilities(language, 0);
            assert!(capability.symbol_extraction, "{language}");
            assert!(!capability.import_resolution, "{language}");
            assert!(!capability.call_resolution, "{language}");
        }
        // Markdown records link targets as imports only.
        let markdown = capabilities(Language::Markdown, 0);
        assert!(!markdown.symbol_extraction);
        assert!(markdown.import_resolution);
        assert!(!markdown.call_resolution);
        // Primary adapters keep full claims.
        let typescript = capabilities(Language::TypeScript, 0);
        assert!(typescript.symbol_extraction);
        assert!(typescript.import_resolution);
        assert!(typescript.call_resolution);
        assert!(typescript.cross_file_references);
    }
}
