use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use trace_core::Language;

use crate::index::helpers;
use crate::index::manifest::discover_go_modules;
use crate::index::types::{FileState, SymbolRef};
use trace_parser::ParsedFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoModule {
    pub directory: String,
    pub module: String,
}

#[derive(Debug, Clone)]
pub struct PackageState {
    pub id: String,
    pub name: String,
    pub language: Language,
    pub directory: Option<String>,
    pub files: Vec<String>,
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
    pub fn build(
        root: &Path,
        parsed_files: Vec<ParsedFile>,
    ) -> Result<Self, crate::error::AnalysisError> {
        let paths: BTreeSet<String> = parsed_files.iter().map(|file| file.path.clone()).collect();
        let go_modules = discover_go_modules(root, &parsed_files);
        let mut index = RepositoryIndex {
            go_modules,
            ..RepositoryIndex::default()
        };

        for parsed in parsed_files {
            let state =
                crate::index::builder::build_file_state(root, &paths, &index.go_modules, parsed);
            index
                .by_directory
                .entry(helpers::directory_key(&state.path))
                .or_default()
                .push(state.path.clone());
            if let Some(group) = &state.group {
                index
                    .by_group
                    .entry(group.clone())
                    .or_default()
                    .push(state.path.clone());
            }
            if helpers::is_python(state.language) {
                let dotted = helpers::python_module_name(&state.path);
                index
                    .by_python_module
                    .entry(dotted)
                    .or_default()
                    .push(state.path.clone());
            }
            if helpers::is_kotlin(state.language) {
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
            if helpers::is_go(state.language) {
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
                            name: crate::index::builder::package_display_name(&state),
                            language: state.language,
                            directory: state
                                .package
                                .as_ref()
                                .map(|_| helpers::directory_key(&state.path)),
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

    pub fn files_in_package(&self, package: &str) -> Vec<String> {
        self.packages
            .get(package)
            .map(|package| package.files.clone())
            .unwrap_or_default()
    }
}
