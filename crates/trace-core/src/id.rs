//! Canonical identifier construction.
//!
//! IDs are deterministic functions of repository-relative paths, entity kinds,
//! qualified names and a deterministic disambiguating ordinal. They never use
//! absolute paths, timestamps, memory addresses or transient indices.
//!
//! Format (documented in `docs/architecture.md`):
//!
//! * repository: `repository:.`
//! * file:       `file:<path>`
//! * package:    `package:<dir>`
//! * module:     `module:<path>`
//! * symbol:     `<kind>:<path>::<qualified>[~<ordinal>]`
//! * external:   `external:<language>:<qualified>`
//!
//! The ordinal is assigned in source order among symbols that share a kind,
//! path and qualified name (for example overloads or same-named local
//! functions). Inserting a new earlier overload can therefore renumber the
//! later ones; this limitation is intentional and documented.

use crate::language::Language;
use crate::node::NodeKind;

pub fn repository_id() -> String {
    "repository:.".to_string()
}

pub fn file_id(path: &str) -> String {
    format!("file:{}", normalize(path))
}

pub fn package_id(dir: &str) -> String {
    format!("package:{}", normalize(dir))
}

pub fn module_id(path: &str) -> String {
    format!("module:{}", normalize(path))
}

pub fn symbol_id(kind: NodeKind, path: &str, qualified: &str, ordinal: u32) -> String {
    let base = format!("{}:{}::{}", kind.as_str(), normalize(path), qualified);
    if ordinal == 0 {
        base
    } else {
        format!("{base}~{ordinal}")
    }
}

pub fn external_id(language: Language, qualified: &str) -> String {
    format!("external:{}:{}", language.as_str(), qualified)
}

/// Normalise repository-relative paths for canonical output.
pub fn normalize(path: &str) -> String {
    let path = path.replace('\\', "/");
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_paths() {
        assert_eq!(normalize("./src/../src/a.ts"), "src/a.ts");
        assert_eq!(normalize("src\\a.ts"), "src/a.ts");
    }

    #[test]
    fn ids_are_deterministic() {
        assert_eq!(
            symbol_id(NodeKind::Function, "src/a.ts", "foo", 0),
            "function:src/a.ts::foo"
        );
        assert_eq!(
            symbol_id(NodeKind::Function, "src/a.ts", "foo", 1),
            "function:src/a.ts::foo~1"
        );
        assert_eq!(
            symbol_id(NodeKind::Function, "src/a.ts", "foo", 0),
            symbol_id(NodeKind::Function, "src/a.ts", "foo", 0)
        );
    }

    #[test]
    fn different_kinds_do_not_collide() {
        assert_ne!(
            symbol_id(NodeKind::Function, "a.ts", "X", 0),
            symbol_id(NodeKind::Method, "a.ts", "X", 0)
        );
    }
}
