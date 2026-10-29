use trace_core::Language;

extern crate tree_sitter_c_sharp;
extern crate tree_sitter_java;
extern crate tree_sitter_php;
extern crate tree_sitter_ruby;

/// Return the tree-sitter grammar for a Trace language.
pub fn tree_sitter_language(language: Language) -> tree_sitter::Language {
    match language {
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        Language::JavaScript | Language::Jsx => tree_sitter_javascript::LANGUAGE.into(),
        Language::Python => tree_sitter_python::LANGUAGE.into(),
        Language::Go => tree_sitter_go::LANGUAGE.into(),
        Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        Language::Kotlin => tree_sitter_kt::LANGUAGE.into(),
        Language::Swift => tree_sitter_swift::LANGUAGE.into(),
        Language::Java => tree_sitter_java::LANGUAGE.into(),
        Language::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
        Language::Php => tree_sitter_php::LANGUAGE_PHP.into(),
        Language::Ruby => tree_sitter_ruby::LANGUAGE.into(),
    }
}

/// Pinned grammar crate version. Included in the syntax cache key so grammar
/// upgrades invalidate cached syntax data.
pub fn grammar_version(language: Language) -> &'static str {
    match language {
        Language::TypeScript | Language::Tsx => "tree-sitter-typescript 0.23.2",
        Language::JavaScript | Language::Jsx => "tree-sitter-javascript 0.25.0",
        Language::Python => "tree-sitter-python 0.25.0",
        Language::Go => "tree-sitter-go 0.25.0",
        Language::Rust => "tree-sitter-rust 0.24.2",
        Language::Kotlin => "tree-sitter-kt 0.1.0",
        Language::Swift => "tree-sitter-swift 0.7.4",
        Language::Java => "tree-sitter-java 0.23.5",
        Language::CSharp => "tree-sitter-c-sharp 0.23.1",
        Language::Php => "tree-sitter-php 0.23.11",
        Language::Ruby => "tree-sitter-ruby 0.23.1",
    }
}

pub fn parser_runtime_version() -> &'static str {
    "tree-sitter 0.27.1"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_builds_a_grammar() {
        for language in Language::ALL {
            let grammar = tree_sitter_language(language);
            assert!(grammar.node_kind_count() > 0, "{language}");
            assert!(!grammar_version(language).is_empty());
        }
    }
}
