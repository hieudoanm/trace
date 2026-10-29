pub mod go;
pub mod kotlin;
pub mod rust;
pub mod wildcard;

use std::collections::BTreeMap;

use trace_parser::ImportData;

use crate::index::{Binding, ModuleRef};

use super::resolve_import_pass::ImportContext;
use super::resolve_resolution_ext::binding_for;

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
}
