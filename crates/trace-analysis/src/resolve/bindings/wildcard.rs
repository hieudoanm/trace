use std::collections::BTreeMap;

use crate::index::{Binding, ModuleRef};

use super::super::resolve_import_pass::ImportContext;

impl<'a> ImportContext<'a> {
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
