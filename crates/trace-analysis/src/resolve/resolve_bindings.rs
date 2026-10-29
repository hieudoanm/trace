//! Binding methods for `ImportContext`.
//!
//! Handles language-specific import binding: ES, Python, Go, Rust, Kotlin,
//! Swift, and wildcard imports.

use std::collections::BTreeMap;

use trace_parser::ImportData;

use crate::index::{Binding, ModuleRef};

use super::resolve_import_pass::ImportContext;
use super::resolve_path_helpers::last_path_segment;
use super::resolve_resolution_ext::binding_for;
use super::resolve_types::Resolution;

impl<'a> ImportContext<'a> {
    pub(crate) fn bind_es(
        &mut self,
        import: &ImportData,
        module: &ModuleRef,
        bindings: &mut BTreeMap<String, Binding>,
        allow_private: bool,
    ) {
        let external_name = match module {
            ModuleRef::External(name) => Some(name.as_str()),
            _ => None,
        };
        if import.items.is_empty() {
            if let Some(alias) = &import.alias {
                bindings.insert(alias.clone(), Binding::Module(module.clone()));
            }
            return;
        }
        for item in &import.items {
            if item.name == "*" {
                if let Some(alias) = &item.alias {
                    bindings.insert(alias.clone(), Binding::Module(module.clone()));
                }
                continue;
            }
            let resolution =
                self.resolve_module_name(module, &item.name, allow_private, external_name);
            let local = item.alias.clone().unwrap_or_else(|| item.name.clone());
            bindings.insert(local, binding_for(resolution));
        }
        if let Some(alias) = &import.alias {
            bindings.insert(alias.clone(), Binding::Module(module.clone()));
        }
    }

    pub(crate) fn bind_python(
        &mut self,
        import: &ImportData,
        module: &ModuleRef,
        bindings: &mut BTreeMap<String, Binding>,
        _allow_private: bool,
    ) {
        if import.items.is_empty() {
            if let Some(alias) = &import.alias {
                bindings.insert(alias.clone(), Binding::Module(module.clone()));
            }
            return;
        }
        // `import pkg.mod` binds `pkg`, while `pkg.mod` remains available
        // as a namespace path for qualified receivers.
        let dotted = import.module.trim_start_matches('.').to_string();
        if dotted.is_empty() {
            return;
        }
        bindings.insert(dotted.clone(), Binding::Module(module.clone()));
        let segments: Vec<&str> = dotted.split('.').collect();
        for length in 1..segments.len() {
            let prefix = segments[..length].join(".");
            if let Some(prefix_module) = self.python_module(&prefix, &self.path) {
                bindings.insert(prefix, Binding::Module(prefix_module));
            }
        }
        for item in &import.items {
            if item.name == "*" {
                self.bind_wildcard(module, bindings, true);
                continue;
            }
            let resolution = self.resolve_module_name(module, &item.name, true, None);
            let local = item.alias.clone().unwrap_or_else(|| item.name.clone());
            bindings.insert(local, binding_for(resolution));
        }
    }

    pub(crate) fn bind_go(
        &mut self,
        import: &ImportData,
        module: &ModuleRef,
        bindings: &mut BTreeMap<String, Binding>,
    ) {
        let local = import.alias.clone().unwrap_or_else(|| match module {
            ModuleRef::Package(package) => self
                .index
                .packages
                .get(package)
                .map(|package| package.name.clone())
                .unwrap_or_else(|| {
                    import
                        .module
                        .rsplit('/')
                        .next()
                        .unwrap_or(&import.module)
                        .to_string()
                }),
            ModuleRef::External(name) => name.rsplit('/').next().unwrap_or(name).to_string(),
            ModuleRef::File(path) => stem(path).to_string(),
        });
        if !local.is_empty() && local != "_" {
            bindings.insert(local, Binding::Module(module.clone()));
        }
    }

    pub(crate) fn bind_rust(
        &mut self,
        import: &ImportData,
        module: &ModuleRef,
        bindings: &mut BTreeMap<String, Binding>,
        allow_private: bool,
    ) {
        let raw = import.module.as_str();
        if let Some(base) = raw.strip_suffix("::*").or_else(|| raw.strip_suffix("::*;")) {
            let module = self
                .resolve_module_path(base)
                .unwrap_or_else(|| module.clone());
            self.bind_wildcard(&module, bindings, true);
            return;
        }
        let last = last_path_segment(raw);
        if last.is_empty() || last == "*" {
            return;
        }
        let local = import.alias.clone().unwrap_or_else(|| last.to_string());
        if matches!(module, ModuleRef::File(path) if self.rust_module_tail(path) == last) {
            bindings.insert(local, Binding::Module(module.clone()));
            return;
        }
        let resolution = self.resolve_module_name(module, last, allow_private, None);
        match resolution {
            Resolution::Unresolved { .. } => {
                bindings.insert(local, Binding::Module(module.clone()));
            }
            other => {
                bindings.insert(local, binding_for(other));
            }
        }
    }

    pub(crate) fn bind_kotlin(
        &mut self,
        import: &ImportData,
        module: &ModuleRef,
        bindings: &mut BTreeMap<String, Binding>,
        allow_private: bool,
    ) {
        if import.items.iter().any(|item| item.name == "*") {
            self.bind_wildcard(module, bindings, allow_private);
            return;
        }
        let last = last_path_segment(&import.module);
        if last.is_empty() || last == "*" {
            return;
        }
        let local = import.alias.clone().unwrap_or_else(|| last.to_string());
        let resolution = self.resolve_module_name(module, last, allow_private, None);
        match resolution {
            Resolution::Unresolved { .. } => {
                bindings.insert(local, Binding::Module(module.clone()));
            }
            other => {
                bindings.insert(local, binding_for(other));
            }
        }
    }

    pub(crate) fn bind_wildcard(
        &mut self,
        module: &ModuleRef,
        bindings: &mut BTreeMap<String, Binding>,
        allow_private: bool,
    ) {
        match module {
            ModuleRef::File(path) => {
                if let Some(state) = self.index.files.get(path) {
                    if allow_private {
                        for (name, indexes) in &state.top_level {
                            for index in indexes {
                                bindings.insert(
                                    name.clone(),
                                    Binding::Symbol(state.symbols[*index].clone()),
                                );
                            }
                        }
                    } else {
                        for (name, ids) in &state.exports {
                            for id in ids {
                                if let Some(symbol) = self.index.symbol(id) {
                                    bindings.insert(name.clone(), Binding::Symbol(symbol.clone()));
                                }
                            }
                        }
                    }
                }
            }
            ModuleRef::Package(package) => {
                for file_state in self
                    .index
                    .packages
                    .get(package)
                    .into_iter()
                    .flat_map(|package| &package.files)
                    .filter_map(|path| self.index.files.get(path))
                {
                    for symbol in &file_state.symbols {
                        if let Some(indexes) = file_state.top_level.get(&symbol.name) {
                            if indexes.iter().any(|index| {
                                file_state.symbols[*index].node_id == symbol.node_id
                                    && (symbol.exported || allow_private)
                            }) {
                                bindings
                                    .insert(symbol.name.clone(), Binding::Symbol(symbol.clone()));
                            }
                        }
                    }
                }
            }
            ModuleRef::External(name) => {
                bindings.insert(
                    name.clone(),
                    Binding::Module(ModuleRef::External(name.clone())),
                );
            }
        }
    }
}

use crate::index::stem;
