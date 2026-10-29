//! Repository symbol index.
//!
//! The index is built once from syntax results and is immutable during
//! resolution except for per-file import bindings, which are filled in a
//! separate pass. All maps are ordered so iteration is deterministic.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use trace_core::{file_id, module_id, package_id, symbol_id, Language, NodeKind};
use trace_parser::{ParsedFile, SymbolData};

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolRef {
    pub node_id: String,
    pub path: String,
    pub local_id: u32,
    pub kind: NodeKind,
    pub name: String,
    pub qualified: String,
    pub parent: Option<u32>,
    pub exported: bool,
}

impl SymbolRef {
    pub fn is_type(&self) -> bool {
        is_type_kind(self.kind)
    }

    pub fn is_callable(&self) -> bool {
        matches!(
            self.kind,
            NodeKind::Function | NodeKind::Method | NodeKind::Constructor
        )
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleRef {
    File(String),
    Package(String),
    External(String),
}

#[derive(Debug, Clone)]
pub enum Binding {
    Symbol(SymbolRef),
    Module(ModuleRef),
    ExternalSymbol {
        node_id: String,
        name: String,
    },
    Unresolved {
        name: String,
    },
    Ambiguous {
        name: String,
        candidates: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct FileState {
    pub path: String,
    pub language: Language,
    pub parsed: ParsedFile,
    pub file_id: String,
    pub module_id: String,
    pub module_name: String,
    pub module_qualified: String,
    pub package: Option<String>,
    pub group: Option<String>,
    pub crate_root: Option<String>,
    pub swift_root: Option<String>,
    pub symbols: Vec<SymbolRef>,
    /// File-local symbol index by parser `local_id`.
    pub local_index: BTreeMap<u32, usize>,
    /// Exported name to canonical node IDs. Re-exports are added after index
    /// construction.
    pub exports: BTreeMap<String, Vec<String>>,
    /// All top-level declarations, including non-exported ones.
    pub top_level: BTreeMap<String, Vec<usize>>,
    pub bindings: BTreeMap<String, Binding>,
}

impl FileState {
    pub fn symbol(&self, local_id: u32) -> Option<&SymbolRef> {
        self.local_index
            .get(&local_id)
            .and_then(|index| self.symbols.get(*index))
    }

    pub fn is_top_level(&self, symbol: &SymbolRef) -> bool {
        symbol.parent.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct PackageState {
    pub id: String,
    pub name: String,
    pub language: Language,
    pub directory: Option<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoModule {
    pub directory: String,
    pub module: String,
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryIndex {
    pub files: BTreeMap<String, FileState>,
    pub by_id: BTreeMap<String, SymbolRef>,
    pub by_name: BTreeMap<String, Vec<SymbolRef>>,
    pub by_qualified: BTreeMap<String, Vec<SymbolRef>>,
    pub by_directory: BTreeMap<String, Vec<String>>,
    pub by_group: BTreeMap<String, Vec<String>>,
    pub by_python_module: BTreeMap<String, Vec<String>>,
    pub by_kotlin_package: BTreeMap<String, Vec<String>>,
    pub by_declared_module: BTreeMap<String, Vec<String>>,
    pub packages: BTreeMap<String, PackageState>,
    pub go_modules: Vec<GoModule>,
}

impl RepositoryIndex {
    pub fn build(root: &Path, parsed_files: Vec<ParsedFile>) -> Result<Self> {
        let paths: BTreeSet<String> = parsed_files.iter().map(|file| file.path.clone()).collect();
        let go_modules = discover_go_modules(root, &parsed_files);
        let mut index = RepositoryIndex {
            go_modules,
            ..RepositoryIndex::default()
        };

        for parsed in parsed_files {
            let state = build_file_state(root, &paths, &index.go_modules, parsed);
            index
                .by_directory
                .entry(directory_key(&state.path))
                .or_default()
                .push(state.path.clone());
            if let Some(group) = &state.group {
                index
                    .by_group
                    .entry(group.clone())
                    .or_default()
                    .push(state.path.clone());
            }
            if state.language == Language::Python {
                let dotted = python_module_name(&state.path);
                index
                    .by_python_module
                    .entry(dotted)
                    .or_default()
                    .push(state.path.clone());
            }
            if state.language == Language::Kotlin {
                if let Some(package) = state.parsed.module_name.clone() {
                    index
                        .by_kotlin_package
                        .entry(package.clone())
                        .or_default()
                        .push(state.path.clone());
                    index
                        .by_declared_module
                        .entry(package)
                        .or_default()
                        .push(state.path.clone());
                }
            }
            if state.language == Language::Go {
                if let Some(package) = state.parsed.module_name.clone() {
                    index
                        .by_declared_module
                        .entry(package)
                        .or_default()
                        .push(state.path.clone());
                }
            }
            if let Some(package_id) = &state.package {
                let entry =
                    index
                        .packages
                        .entry(package_id.clone())
                        .or_insert_with(|| PackageState {
                            id: package_id.clone(),
                            name: package_display_name(&state),
                            language: state.language,
                            directory: state.package.as_ref().map(|_| directory_key(&state.path)),
                            files: Vec::new(),
                        });
                entry.files.push(state.path.clone());
            }
            index.files.insert(state.path.clone(), state);
        }

        for paths in index.by_directory.values_mut() {
            paths.sort();
        }
        for paths in index.by_group.values_mut() {
            paths.sort();
        }
        for paths in index.by_python_module.values_mut() {
            paths.sort();
        }
        for paths in index.by_kotlin_package.values_mut() {
            paths.sort();
        }
        for paths in index.by_declared_module.values_mut() {
            paths.sort();
        }
        for package in index.packages.values_mut() {
            package.files.sort();
            package.files.dedup();
        }

        let mut by_id = BTreeMap::new();
        let mut by_name: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
        let mut by_qualified: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
        for state in index.files.values() {
            for symbol in &state.symbols {
                by_id.insert(symbol.node_id.clone(), symbol.clone());
                by_name
                    .entry(symbol.name.clone())
                    .or_default()
                    .push(symbol.clone());
                by_qualified
                    .entry(symbol.qualified.clone())
                    .or_default()
                    .push(symbol.clone());
            }
        }
        for symbols in by_name.values_mut() {
            symbols.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        }
        for symbols in by_qualified.values_mut() {
            symbols.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        }
        index.by_id = by_id;
        index.by_name = by_name;
        index.by_qualified = by_qualified;
        Ok(index)
    }

    pub fn file(&self, path: &str) -> Option<&FileState> {
        self.files.get(path)
    }

    pub fn symbol(&self, node_id: &str) -> Option<&SymbolRef> {
        self.by_id.get(node_id)
    }

    pub fn symbols_in_group(&self, group: &str, name: &str, type_only: bool) -> Vec<SymbolRef> {
        let mut out = Vec::new();
        for path in self.by_group.get(group).into_iter().flatten() {
            let Some(state) = self.files.get(path) else {
                continue;
            };
            if let Some(indexes) = state.top_level.get(name) {
                for index in indexes {
                    let symbol = &state.symbols[*index];
                    if !type_only || symbol.is_type() {
                        out.push(symbol.clone());
                    }
                }
            }
        }
        out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        out.dedup_by(|a, b| a.node_id == b.node_id);
        out
    }

    pub fn symbols_in_package(&self, package: &str, name: &str) -> Vec<SymbolRef> {
        let Some(package) = self.packages.get(package) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for path in &package.files {
            let Some(state) = self.files.get(path) else {
                continue;
            };
            if let Some(indexes) = state.top_level.get(name) {
                for index in indexes {
                    let symbol = &state.symbols[*index];
                    if symbol.exported || state.language != Language::Go {
                        out.push(symbol.clone());
                    }
                }
            }
        }
        out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        out.dedup_by(|a, b| a.node_id == b.node_id);
        out
    }

    pub fn parent_type_of(&self, state: &FileState, local_id: u32) -> Option<SymbolRef> {
        let mut cursor = state.symbol(local_id).and_then(|symbol| symbol.parent);
        while let Some(parent) = cursor {
            let symbol = state.symbol(parent)?;
            if symbol.is_type() {
                return Some(symbol.clone());
            }
            cursor = symbol.parent;
        }
        // Swift extensions declare members without a local parent. Derive the
        // type from the qualified prefix when a declaration exists elsewhere.
        let symbol = state.symbol(local_id)?;
        let prefix = symbol.qualified.rsplit_once('.')?.0;
        self.resolve_type_prefix(&symbol.path, prefix)
    }

    pub fn resolve_type_prefix(&self, path: &str, prefix: &str) -> Option<SymbolRef> {
        let state = self.files.get(path)?;
        if let Some(symbol) = state
            .symbols
            .iter()
            .find(|symbol| symbol.qualified == prefix && symbol.is_type())
        {
            return Some(symbol.clone());
        }
        let candidates = self.by_qualified.get(prefix)?;
        let types: Vec<SymbolRef> = candidates
            .iter()
            .filter(|symbol| symbol.is_type())
            .cloned()
            .collect();
        if types.len() == 1 {
            types.into_iter().next()
        } else {
            None
        }
    }

    /// Resolve the structural parent node used by graph containment. Extension
    /// members attach to a type declared in another file when it can be found
    /// unambiguously; otherwise they remain module-level declarations.
    pub fn structural_parent(&self, state: &FileState, symbol: &SymbolRef) -> String {
        if let Some(parent) = symbol
            .parent
            .and_then(|local| state.symbol(local))
            .map(|parent| parent.node_id.clone())
        {
            return parent;
        }
        if let Some(prefix) = symbol.qualified.rsplit_once('.').map(|(prefix, _)| prefix) {
            if let Some(parent) = self.resolve_type_prefix(&symbol.path, prefix) {
                if parent.node_id != symbol.node_id {
                    return parent.node_id;
                }
            }
        }
        state.module_id.clone()
    }

    pub fn files_in_package(&self, package: &str) -> Vec<String> {
        self.packages
            .get(package)
            .map(|package| package.files.clone())
            .unwrap_or_default()
    }
}

fn build_file_state(
    root: &Path,
    paths: &BTreeSet<String>,
    go_modules: &[GoModule],
    parsed: ParsedFile,
) -> FileState {
    let path = parsed.path.clone();
    let language = parsed.language;
    let directory = directory_key(&path);
    let package = package_for(root, paths, go_modules, &parsed);
    let crate_root = if language == Language::Rust {
        nearest_manifest(root, &path, "Cargo.toml")
    } else {
        None
    };
    let swift_root = if language == Language::Swift {
        nearest_manifest(root, &path, "Package.swift")
    } else {
        None
    };
    let group = match language {
        Language::Go => Some(format!("go:{}", directory)),
        Language::Python => Some(match &package {
            Some(id) => format!("python-package:{}", id.trim_start_matches("package:")),
            None => format!("python-directory:{directory}"),
        }),
        Language::Kotlin => parsed
            .module_name
            .as_ref()
            .map(|package| format!("kotlin-package:{package}"))
            .or_else(|| Some(format!("kotlin-directory:{directory}"))),
        Language::Rust => Some(format!(
            "rust-crate:{}",
            crate_root.as_deref().unwrap_or("")
        )),
        Language::Swift => Some(format!(
            "swift-module:{}",
            swift_root.as_deref().unwrap_or("")
        )),
        Language::Java | Language::Php | Language::Ruby | Language::CSharp => {
            Some(format!("{}:{}", language.as_str(), directory))
        }
        _ => None,
    };
    let module_name = module_name_for(&parsed, &path, crate_root.as_deref());
    let module_qualified = module_qualified_for(&parsed, &path, crate_root.as_deref());
    let mut symbols = Vec::with_capacity(parsed.symbols.len());
    for symbol in &parsed.symbols {
        symbols.push(symbol_ref(&path, symbol));
    }
    let mut local_index = BTreeMap::new();
    let mut exports: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut top_level: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, symbol) in symbols.iter().enumerate() {
        local_index.insert(symbol.local_id, index);
        if symbol.parent.is_none() {
            top_level
                .entry(symbol.name.clone())
                .or_default()
                .push(index);
            if symbol.exported {
                exports
                    .entry(symbol.name.clone())
                    .or_default()
                    .push(symbol.node_id.clone());
            }
        }
    }
    for ids in exports.values_mut() {
        ids.sort();
        ids.dedup();
    }
    FileState {
        path,
        language,
        file_id: file_id(&parsed.path),
        module_id: module_id(&parsed.path),
        module_name,
        module_qualified,
        package,
        group,
        crate_root,
        swift_root,
        symbols,
        local_index,
        exports,
        top_level,
        bindings: BTreeMap::new(),
        parsed,
    }
}

fn symbol_ref(path: &str, symbol: &SymbolData) -> SymbolRef {
    SymbolRef {
        node_id: symbol_id(symbol.kind, path, &symbol.qualified_name, symbol.ordinal),
        path: path.to_string(),
        local_id: symbol.local_id,
        kind: symbol.kind,
        name: symbol.name.clone(),
        qualified: symbol.qualified_name.clone(),
        parent: symbol.parent,
        exported: symbol.exported,
    }
}

fn package_for(
    _root: &Path,
    paths: &BTreeSet<String>,
    _go_modules: &[GoModule],
    parsed: &ParsedFile,
) -> Option<String> {
    match parsed.language {
        Language::Go => {
            let directory = directory_key(&parsed.path);
            Some(package_id(if directory == "." { "" } else { &directory }))
        }
        Language::Python => {
            let mut directory = directory_key(&parsed.path);
            loop {
                let init_py = if directory == "." {
                    "__init__.py".to_string()
                } else {
                    format!("{directory}/__init__.py")
                };
                let init_pyi = init_py.trim_end_matches(".py").to_string() + ".pyi";
                if paths.contains(&init_py) || paths.contains(&init_pyi) {
                    return Some(package_id(if directory == "." { "" } else { &directory }));
                }
                if directory == "." {
                    return None;
                }
                directory = parent_directory(&directory).to_string();
                if directory.is_empty() {
                    directory = ".".to_string();
                }
            }
        }
        Language::Kotlin => parsed
            .module_name
            .as_ref()
            .map(|package| package_id(&format!("kotlin/{}", package.replace('.', "/")))),
        _ => None,
    }
}

fn package_display_name(state: &FileState) -> String {
    match state.language {
        Language::Go => state
            .parsed
            .module_name
            .clone()
            .unwrap_or_else(|| directory_key(&state.path)),
        Language::Python => {
            let directory = directory_key(&state.path);
            if directory == "." {
                "root".to_string()
            } else {
                directory.trim_start_matches("package:").replace('/', ".")
            }
        }
        Language::Kotlin => state
            .parsed
            .module_name
            .clone()
            .unwrap_or_else(|| directory_key(&state.path)),
        _ => directory_key(&state.path),
    }
}

fn module_name_for(parsed: &ParsedFile, path: &str, crate_root: Option<&str>) -> String {
    match parsed.language {
        Language::Go | Language::Kotlin => parsed
            .module_name
            .clone()
            .unwrap_or_else(|| stem(path).to_string()),
        Language::Python => python_module_name(path),
        Language::Rust => rust_module_qualified(path, crate_root),
        _ => stem(path).to_string(),
    }
}

fn module_qualified_for(parsed: &ParsedFile, path: &str, crate_root: Option<&str>) -> String {
    match parsed.language {
        Language::Go | Language::Kotlin => parsed
            .module_name
            .clone()
            .unwrap_or_else(|| stem(path).to_string()),
        Language::Python => python_module_name(path),
        Language::Rust => rust_module_qualified(path, crate_root),
        _ => stem(path).to_string(),
    }
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

fn nearest_manifest(root: &Path, path: &str, manifest: &str) -> Option<String> {
    let mut directory = directory_key(path);
    loop {
        let relative = if directory == "." {
            manifest.to_string()
        } else {
            format!("{directory}/{manifest}")
        };
        if root.join(&relative).is_file() {
            return Some(if directory == "." {
                String::new()
            } else {
                directory
            });
        }
        if directory == "." {
            return None;
        }
        directory = parent_directory(&directory).to_string();
        if directory.is_empty() {
            directory = ".".to_string();
        }
    }
}

fn discover_go_modules(root: &Path, parsed_files: &[ParsedFile]) -> Vec<GoModule> {
    let mut manifests = BTreeSet::new();
    for parsed in parsed_files {
        if parsed.language != Language::Go {
            continue;
        }
        let mut directory = directory_key(&parsed.path);
        loop {
            let relative = if directory == "." {
                "go.mod".to_string()
            } else {
                format!("{directory}/go.mod")
            };
            if root.join(&relative).is_file() {
                manifests.insert(relative);
                break;
            }
            if directory == "." {
                break;
            }
            directory = parent_directory(&directory).to_string();
            if directory.is_empty() {
                directory = ".".to_string();
            }
        }
    }
    let mut modules = Vec::new();
    for manifest in manifests {
        let Ok(text) = std::fs::read_to_string(root.join(&manifest)) else {
            continue;
        };
        let module = text.lines().find_map(|line| {
            line.trim()
                .strip_prefix("module ")
                .map(|value| value.trim().trim_matches('"').to_string())
        });
        let Some(module) = module else {
            continue;
        };
        let directory = parent_directory(&manifest).to_string();
        modules.push(GoModule {
            directory: if directory.is_empty() {
                ".".to_string()
            } else {
                directory
            },
            module,
        });
    }
    modules.sort_by(|a, b| a.directory.cmp(&b.directory));
    modules
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

#[cfg(test)]
mod tests {
    use super::*;
    use trace_parser::SyntaxParser;

    fn parse(path: &str, language: Language, source: &str) -> ParsedFile {
        let mut parser = SyntaxParser::new().unwrap();
        parser.parse(path, language, source).unwrap()
    }

    #[test]
    fn indexes_files_and_top_level_symbols() {
        let dir = tempfile::tempdir().unwrap();
        let parsed = vec![
            parse(
                "a.py",
                Language::Python,
                "import b\nclass A:\n    def m(self):\n        pass\n",
            ),
            parse("b.py", Language::Python, "def helper():\n    pass\n"),
        ];
        let index = RepositoryIndex::build(dir.path(), parsed).unwrap();
        assert_eq!(index.files.len(), 2);
        let a = index.files.get("a.py").unwrap();
        assert_eq!(a.module_name, "a");
        assert!(a.exports.contains_key("A"));
        assert!(index.by_name.contains_key("m"));
        assert!(index.by_python_module.contains_key("b"));
    }

    #[test]
    fn python_packages_are_detected_from_init_files() {
        let dir = tempfile::tempdir().unwrap();
        let parsed = vec![
            parse("pkg/__init__.py", Language::Python, ""),
            parse("pkg/mod.py", Language::Python, "def f():\n    pass\n"),
        ];
        let index = RepositoryIndex::build(dir.path(), parsed).unwrap();
        let module = index.files.get("pkg/mod.py").unwrap();
        assert_eq!(module.module_name, "pkg.mod");
        let package = index.packages.values().next().unwrap();
        assert_eq!(package.files.len(), 2);
    }

    #[test]
    fn rust_module_paths_use_crate_roots() {
        assert_eq!(rust_module_qualified("src/lib.rs", Some("")), "crate");
        assert_eq!(
            rust_module_qualified("src/foo/mod.rs", Some("")),
            "crate::foo"
        );
        assert_eq!(
            rust_module_qualified("crate/src/foo/bar.rs", Some("crate")),
            "crate::foo::bar"
        );
    }

    #[test]
    fn directory_helpers_are_stable() {
        assert_eq!(directory_key("src/a.ts"), "src");
        assert_eq!(directory_key("a.ts"), ".");
        assert_eq!(parent_directory("src/deep/a.ts"), "src/deep");
        assert_eq!(stem("src/a.test.ts"), "a.test");
    }
}
