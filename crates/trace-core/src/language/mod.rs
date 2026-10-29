pub mod methods;
pub mod tests;
pub mod types;

pub use crate::language::types::{Language, LanguageFamily};

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
