use std::collections::BTreeMap;

use trace_parser::ImportData;

use crate::index::{Binding, ModuleRef};

use super::super::resolve_import_pass::ImportContext;
use super::super::resolve_path_helpers::last_path_segment;
use super::super::resolve_resolution_ext::binding_for;
use super::super::resolve_types::Resolution;

impl<'a> ImportContext<'a> {
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
}
