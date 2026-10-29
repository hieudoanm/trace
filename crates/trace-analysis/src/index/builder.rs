use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use trace_core::{file_id, module_id, package_id, Language};

use crate::index::helpers;
use crate::index::manifest::nearest_manifest;
use crate::index::repository::GoModule;
use crate::index::types::{symbol_ref, FileState};
use trace_parser::ParsedFile;

fn package_for(
    _root: &Path,
    paths: &BTreeSet<String>,
    _go_modules: &[GoModule],
    parsed: &ParsedFile,
) -> Option<String> {
    match parsed.language {
        Language::Go => {
            let directory = helpers::directory_key(&parsed.path);
            Some(package_id(if directory == "." { "" } else { &directory }))
        }
        Language::Python => {
            let mut directory = helpers::directory_key(&parsed.path);
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
                directory = helpers::parent_directory(&directory).to_string();
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

pub(crate) fn package_display_name(state: &FileState) -> String {
    match state.language {
        Language::Go => state
            .parsed
            .module_name
            .clone()
            .unwrap_or_else(|| helpers::directory_key(&state.path)),
        Language::Python => {
            let directory = helpers::directory_key(&state.path);
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
            .unwrap_or_else(|| helpers::directory_key(&state.path)),
        _ => helpers::directory_key(&state.path),
    }
}

fn module_name_for(parsed: &ParsedFile, path: &str, crate_root: Option<&str>) -> String {
    match parsed.language {
        Language::Go | Language::Kotlin => parsed
            .module_name
            .clone()
            .unwrap_or_else(|| helpers::stem(path).to_string()),
        Language::Python => helpers::python_module_name(path),
        Language::Rust => helpers::rust_module_qualified(path, crate_root),
        _ => helpers::stem(path).to_string(),
    }
}

fn module_qualified_for(parsed: &ParsedFile, path: &str, crate_root: Option<&str>) -> String {
    match parsed.language {
        Language::Go | Language::Kotlin => parsed
            .module_name
            .clone()
            .unwrap_or_else(|| helpers::stem(path).to_string()),
        Language::Python => helpers::python_module_name(path),
        Language::Rust => helpers::rust_module_qualified(path, crate_root),
        _ => helpers::stem(path).to_string(),
    }
}

pub(crate) fn build_file_state(
    root: &Path,
    paths: &BTreeSet<String>,
    go_modules: &[GoModule],
    parsed: ParsedFile,
) -> FileState {
    let path = parsed.path.clone();
    let language = parsed.language;
    let directory = helpers::directory_key(&path);
    let package = package_for(root, paths, go_modules, &parsed);
    let crate_root = if helpers::is_rust(language) {
        nearest_manifest(root, &path, "Cargo.toml")
    } else {
        None
    };
    let swift_root = if helpers::is_swift(language) {
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
