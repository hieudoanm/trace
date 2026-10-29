//! Miscellaneous methods for `ImportContext`.
//!
//! Includes module name resolution, package dependency tracking, and
//! diagnostic reporting.

use trace_core::{Diagnostic, DiagnosticKind};

use crate::index::{python_module_name, ModuleRef};

use super::resolve_import_pass::ImportContext;
use super::resolve_resolution_ext::{choose, join_external, ResolutionExt};
use super::resolve_types::{Resolution, Want};

impl<'a> ImportContext<'a> {
    pub(crate) fn resolve_local_export(&self, name: &str) -> Resolution {
        let resolver = super::resolve_resolver::Resolver::new(self.index);
        resolver.resolve_name(self.state, None, name, Want::Any)
    }

    pub(crate) fn unresolved_export(
        &mut self,
        import: &trace_parser::ImportData,
        name: &str,
        resolution: &Resolution,
    ) {
        match resolution {
            Resolution::Ambiguous { candidates, .. } => {
                self.diagnostics.push(
                    Diagnostic::warning(
                        DiagnosticKind::AmbiguousSymbol,
                        format!(
                            "re-export `{name}` is ambiguous ({} candidates)",
                            candidates.len()
                        ),
                    )
                    .with_file(self.path.clone())
                    .with_language(self.language)
                    .with_range(import.range),
                );
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::warning(
                        DiagnosticKind::UnresolvedImport,
                        format!("cannot resolve re-export `{name}`"),
                    )
                    .with_file(self.path.clone())
                    .with_language(self.language)
                    .with_range(import.range),
                );
            }
        }
    }

    pub(crate) fn resolve_module_name(
        &self,
        module: &ModuleRef,
        name: &str,
        allow_private: bool,
        external_module: Option<&str>,
    ) -> Resolution {
        match module {
            ModuleRef::File(path) => {
                let Some(state) = self.index.files.get(path) else {
                    return Resolution::Unresolved {
                        name: name.to_string(),
                    };
                };
                let mut candidates: Vec<crate::index::SymbolRef> = state
                    .exports
                    .get(name)
                    .into_iter()
                    .flatten()
                    .filter_map(|id| self.index.symbol(id).cloned())
                    .collect();
                if candidates.is_empty() && allow_private {
                    if let Some(indexes) = state.top_level.get(name) {
                        candidates
                            .extend(indexes.iter().map(|index| state.symbols[*index].clone()));
                    }
                }
                if candidates.is_empty() && state.language == trace_core::Language::Python {
                    // `from package import submodule`
                    let dotted = format!("{}.{}", python_module_name(path), name);
                    if let Some(paths) = self.index.by_python_module.get(&dotted) {
                        if paths.len() == 1 {
                            return Resolution::Module(ModuleRef::File(paths[0].clone()));
                        }
                    }
                }
                choose(name, candidates)
            }
            ModuleRef::Package(package) => {
                let candidates = self.index.symbols_in_package(package, name);
                choose(name, candidates)
            }
            ModuleRef::External(module) => Resolution::External {
                qualified: join_external(self.language, module, name),
                name: name.to_string(),
            },
        }
        .or_external(name, external_module, self.language)
    }

    pub(crate) fn record_package_dependency(&mut self, module: &ModuleRef) {
        let Some(source_package) = &self.state.package else {
            return;
        };
        let target = match module {
            ModuleRef::Package(package) => Some(package.clone()),
            ModuleRef::File(path) => self
                .index
                .files
                .get(path)
                .and_then(|state| state.package.clone()),
            ModuleRef::External(_) => None,
        };
        if let Some(target) = target {
            if &target != source_package {
                self.package_pairs.insert((source_package.clone(), target));
            }
        }
    }

    pub(crate) fn report_import_failure(
        &mut self,
        import: &trace_parser::ImportData,
        outcome: &Resolution,
    ) {
        match outcome {
            Resolution::Ambiguous { candidates, .. } => {
                self.diagnostics.push(
                    Diagnostic::warning(
                        DiagnosticKind::AmbiguousSymbol,
                        format!(
                            "import `{}` matches {} modules: {}",
                            import.module,
                            candidates.len(),
                            candidates
                                .iter()
                                .take(5)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    )
                    .with_file(self.path.clone())
                    .with_language(self.language)
                    .with_range(import.range),
                );
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::warning(
                        DiagnosticKind::UnresolvedImport,
                        format!("cannot resolve import `{}`", import.module),
                    )
                    .with_file(self.path.clone())
                    .with_language(self.language)
                    .with_range(import.range),
                );
            }
        }
    }
}
