//! Module resolution methods for `ImportContext`.
//!
//! Resolves import specifiers to `ModuleRef` targets for each supported
//! language, plus Rust-specific module path helpers.

pub mod es;
pub mod go;
pub mod kotlin;
pub mod python;
pub mod rust;

use trace_core::Language;
use trace_parser::ImportData;

use crate::index::ModuleRef;

use super::resolve_import_pass::ImportContext;
use super::resolve_types::Resolution;

impl<'a> ImportContext<'a> {
    pub(crate) fn resolve_module(
        &self,
        import: &ImportData,
    ) -> std::result::Result<ModuleRef, Resolution> {
        match import.kind {
            trace_parser::ImportKind::Es => es::resolve_es(self, import),
            trace_parser::ImportKind::Python => python::resolve_python(self, import),
            trace_parser::ImportKind::Go => go::resolve_go(self, import),
            trace_parser::ImportKind::Rust => rust::resolve_rust(self, import),
            trace_parser::ImportKind::Kotlin => kotlin::resolve_kotlin(self, import),
            trace_parser::ImportKind::Swift => {
                if import.module.is_empty() {
                    Err(Resolution::Unresolved {
                        name: import.module.clone(),
                    })
                } else {
                    Ok(ModuleRef::External(import.module.clone()))
                }
            }
        }
    }

    pub(crate) fn python_module(&self, dotted: &str, from_path: &str) -> Option<ModuleRef> {
        python::python_module(self, dotted, from_path)
    }

    pub(crate) fn resolve_module_path(&self, raw: &str) -> Option<ModuleRef> {
        rust::resolve_module_path(self, raw)
    }

    pub(crate) fn rust_module_tail(&self, path: &str) -> String {
        rust::rust_module_tail(self, path)
    }

    pub(crate) fn allow_private(&self, module: &ModuleRef) -> bool {
        match module {
            ModuleRef::File(path) => {
                let Some(target) = self.index.files.get(path) else {
                    return false;
                };
                match self.language {
                    Language::Python | Language::Kotlin => true,
                    Language::Rust => target.crate_root == self.state.crate_root,
                    Language::Swift => target.swift_root == self.state.swift_root,
                    _ => false,
                }
            }
            ModuleRef::Package(_) => self.language != Language::Go,
            ModuleRef::External(_) => false,
        }
    }
}
