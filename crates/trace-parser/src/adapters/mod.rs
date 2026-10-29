mod c;
mod cpp;
mod csharp;
mod dart;
mod go;
mod java;
mod kotlin;
mod php;
mod python;
mod r;
mod ruby;
mod rust;
mod scala;
mod swift;
mod typescript;

use tree_sitter::Tree;

use trace_core::Language;

use crate::syntax::ParsedFile;

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    match language {
        Language::TypeScript | Language::Tsx | Language::JavaScript | Language::Jsx => {
            typescript::parse(language, path, source, tree)
        }
        Language::Python => python::parse(language, path, source, tree),
        Language::Go => go::parse(language, path, source, tree),
        Language::Rust => rust::parse(language, path, source, tree),
        Language::Kotlin => kotlin::parse(language, path, source, tree),
        Language::Swift => swift::parse(language, path, source, tree),
        Language::Java => java::parse(language, path, source, tree),
        Language::CSharp => csharp::parse(language, path, source, tree),
        Language::Php => php::parse(language, path, source, tree),
        Language::Ruby => ruby::parse(language, path, source, tree),
        Language::C => c::parse(language, path, source, tree),
        Language::Cpp => cpp::parse(language, path, source, tree),
        Language::Dart => dart::parse(language, path, source, tree),
        Language::Scala => scala::parse(language, path, source, tree),
        Language::R => r::parse(language, path, source, tree),
    }
}
