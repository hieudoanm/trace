use std::collections::BTreeMap;

use trace_parser::ImportData;

use crate::index::{Binding, ModuleRef};

use super::super::resolve_import_pass::ImportContext;
use super::super::resolve_path_helpers::last_path_segment;
use super::super::resolve_resolution_ext::binding_for;
use super::super::resolve_types::Resolution;

impl<'a> ImportContext<'a> {
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
}
