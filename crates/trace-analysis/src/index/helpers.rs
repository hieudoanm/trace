use trace_core::{Language, NodeKind};

pub fn is_type_kind(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Class
            | NodeKind::Struct
            | NodeKind::Interface
            | NodeKind::Trait
            | NodeKind::Enum
            | NodeKind::Type
    )
}

pub fn python_module_name(path: &str) -> String {
    let without_extension = path
        .strip_suffix(".pyi")
        .or_else(|| path.strip_suffix(".py"))
        .unwrap_or(path);
    let dotted = without_extension.replace('/', ".");
    let dotted = dotted.strip_suffix(".__init__").unwrap_or(&dotted);
    if dotted.is_empty() {
        "root".to_string()
    } else {
        dotted.to_string()
    }
}

pub fn rust_module_qualified(path: &str, crate_root: Option<&str>) -> String {
    let relative = match crate_root {
        Some(root) if !root.is_empty() => path
            .strip_prefix(root)
            .unwrap_or(path)
            .trim_start_matches('/'),
        _ => path,
    };
    let relative = relative.strip_prefix("src/").unwrap_or(relative);
    let without_extension = relative.strip_suffix(".rs").unwrap_or(relative);
    let mut parts: Vec<&str> = without_extension
        .split('/')
        .filter(|p| !p.is_empty())
        .collect();
    if matches!(parts.last(), Some(&"mod" | &"lib" | &"main")) {
        parts.pop();
    }
    if parts.is_empty() {
        "crate".to_string()
    } else {
        format!("crate::{}", parts.join("::"))
    }
}

pub fn parent_directory(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("")
}

pub fn directory_key(path: &str) -> String {
    let parent = parent_directory(path);
    if parent.is_empty() {
        ".".to_string()
    } else {
        parent.to_string()
    }
}

pub fn stem(path: &str) -> &str {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name)
}

pub fn is_python(language: Language) -> bool {
    language == Language::Python
}

pub fn is_kotlin(language: Language) -> bool {
    language == Language::Kotlin
}

pub fn is_go(language: Language) -> bool {
    language == Language::Go
}

pub fn is_rust(language: Language) -> bool {
    language == Language::Rust
}

pub fn is_swift(language: Language) -> bool {
    language == Language::Swift
}
