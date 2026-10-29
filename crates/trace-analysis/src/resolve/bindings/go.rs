use std::collections::BTreeMap;

use trace_parser::ImportData;

use crate::index::{stem, Binding, ModuleRef};

use super::super::resolve_import_pass::ImportContext;

impl<'a> ImportContext<'a> {
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
}
