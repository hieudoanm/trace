//! Import pass: resolves every import, fills per-file bindings and re-export
//! maps, and returns dependency edges.

use std::collections::{BTreeMap, BTreeSet};

use trace_core::{Diagnostic, Language, ResolutionStatus};
use trace_parser::ImportData;

use crate::index::{Binding, FileState, ModuleRef, RepositoryIndex};

use super::resolve_types::{ImportEdge, ImportPass, Resolution};

/// Context for resolving imports within a single file.
pub struct ImportContext<'a> {
    pub index: &'a RepositoryIndex,
    pub state: &'a FileState,
    pub path: String,
    pub language: Language,
    pub diagnostics: Vec<Diagnostic>,
    pub edges: Vec<ImportEdge>,
    pub package_pairs: BTreeSet<(String, String)>,
}

impl<'a> ImportContext<'a> {
    pub fn process(
        &mut self,
        import: &ImportData,
        bindings: &mut BTreeMap<String, Binding>,
        re_exports: &mut BTreeMap<String, String>,
    ) {
        if import.kind == trace_parser::ImportKind::Es
            && import.is_export
            && import.module.is_empty()
        {
            // `export { localName as alias };`
            for item in &import.items {
                if item.name == "*" {
                    continue;
                }
                let resolution = self.resolve_local_export(&item.name);
                if let Resolution::Internal(symbol) = resolution {
                    re_exports.insert(
                        item.alias.clone().unwrap_or_else(|| item.name.clone()),
                        symbol.node_id,
                    );
                } else {
                    self.unresolved_export(import, &item.name, &resolution);
                }
            }
            return;
        }

        let module = match self.resolve_module(import) {
            Ok(module) => module,
            Err(outcome) => {
                self.report_import_failure(import, &outcome);
                return;
            }
        };

        let external = matches!(module, ModuleRef::External(_));
        self.edges.push(ImportEdge {
            from_path: self.path.clone(),
            module: module.clone(),
            site: import.range,
            status: if external {
                ResolutionStatus::ResolvedExternal
            } else {
                ResolutionStatus::ResolvedInternal
            },
        });
        self.record_package_dependency(&module);

        let allow_private = self.allow_private(&module);
        let module_name = match &module {
            ModuleRef::External(name) => Some(name.clone()),
            _ => None,
        };

        if import.is_export {
            for item in &import.items {
                if item.name == "*" {
                    if let ModuleRef::File(path) = &module {
                        if let Some(state) = self.index.files.get(path) {
                            for (name, ids) in &state.exports {
                                for id in ids {
                                    re_exports.insert(name.clone(), id.clone());
                                }
                            }
                        }
                    }
                    continue;
                }
                let resolution = self.resolve_module_name(
                    &module,
                    &item.name,
                    allow_private,
                    module_name.as_deref(),
                );
                if let Resolution::Internal(symbol) = resolution {
                    re_exports.insert(
                        item.alias.clone().unwrap_or_else(|| item.name.clone()),
                        symbol.node_id,
                    );
                } else {
                    self.unresolved_export(import, &item.name, &resolution);
                }
            }
            return;
        }

        match import.kind {
            trace_parser::ImportKind::Es => self.bind_es(import, &module, bindings, allow_private),
            trace_parser::ImportKind::Python => {
                self.bind_python(import, &module, bindings, allow_private)
            }
            trace_parser::ImportKind::Go => self.bind_go(import, &module, bindings),
            trace_parser::ImportKind::Rust => {
                self.bind_rust(import, &module, bindings, allow_private)
            }
            trace_parser::ImportKind::Kotlin => {
                self.bind_kotlin(import, &module, bindings, allow_private)
            }
            trace_parser::ImportKind::Swift => {
                if let ModuleRef::External(name) = &module {
                    bindings.insert(
                        import.alias.clone().unwrap_or_else(|| name.clone()),
                        Binding::Module(ModuleRef::External(name.clone())),
                    );
                }
            }
        }
    }
}

/// Resolve every import, fill per-file bindings and re-export maps, and return
/// dependency edges. Modules are indexed before this pass, so no source file
/// needs to be read again.
pub fn resolve_imports(index: &mut RepositoryIndex) -> ImportPass {
    let paths: Vec<String> = index.files.keys().cloned().collect();
    let mut pass = ImportPass::default();
    for path in paths {
        let Some(state) = index.files.get(&path) else {
            continue;
        };
        let imports = state.parsed.imports.clone();
        let mut context = ImportContext {
            index,
            state,
            path: path.clone(),
            language: state.language,
            diagnostics: Vec::new(),
            edges: Vec::new(),
            package_pairs: BTreeSet::new(),
        };
        let mut bindings = BTreeMap::new();
        let mut re_exports: BTreeMap<String, String> = BTreeMap::new();
        for import in &imports {
            context.process(import, &mut bindings, &mut re_exports);
        }
        pass.diagnostics.append(&mut context.diagnostics);
        pass.edges.append(&mut context.edges);
        pass.package_pairs.append(&mut context.package_pairs);
        if let Some(state) = index.files.get_mut(&path) {
            state.bindings = bindings;
            for (name, node_id) in re_exports {
                state.exports.entry(name).or_default().push(node_id);
            }
            for ids in state.exports.values_mut() {
                ids.sort();
                ids.dedup();
            }
        }
    }
    pass.diagnostics
        .sort_by_key(|diagnostic| diagnostic.sort_key());
    pass.diagnostics.dedup();
    pass
}
