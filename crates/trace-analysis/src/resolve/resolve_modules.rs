//! Module resolution methods for `ImportContext`.
//!
//! Resolves import specifiers to `ModuleRef` targets for each supported
//! language, plus Rust-specific module path helpers.

use trace_core::Language;
use trace_parser::ImportData;

use crate::index::{FileState, ModuleRef};

use super::resolve_import_pass::ImportContext;
use super::resolve_path_helpers::{
    es_candidates, join_dir, join_relative, kotlin_package_id, module_id_to_package,
};
use super::resolve_types::Resolution;
use crate::index::{directory_key, parent_directory};

impl<'a> ImportContext<'a> {
    pub(crate) fn resolve_module(
        &self,
        import: &ImportData,
    ) -> std::result::Result<ModuleRef, Resolution> {
        match import.kind {
            trace_parser::ImportKind::Es => self.resolve_es(import),
            trace_parser::ImportKind::Python => self.resolve_python(import),
            trace_parser::ImportKind::Go => self.resolve_go(import),
            trace_parser::ImportKind::Rust => self.resolve_rust(import),
            trace_parser::ImportKind::Kotlin => self.resolve_kotlin(import),
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

    pub(crate) fn resolve_es(
        &self,
        import: &ImportData,
    ) -> std::result::Result<ModuleRef, Resolution> {
        let specifier = import.module.trim();
        if specifier.starts_with('.') {
            let base = join_relative(&self.path, specifier);
            for candidate in es_candidates(&base) {
                if self.index.files.contains_key(&candidate) {
                    return Ok(ModuleRef::File(candidate));
                }
            }
            return Err(Resolution::Unresolved {
                name: specifier.to_string(),
            });
        }
        if specifier.starts_with('/') || specifier.is_empty() {
            return Err(Resolution::Unresolved {
                name: specifier.to_string(),
            });
        }
        Ok(ModuleRef::External(specifier.to_string()))
    }

    pub(crate) fn resolve_python(
        &self,
        import: &ImportData,
    ) -> std::result::Result<ModuleRef, Resolution> {
        let specifier = import.module.trim();
        if specifier.starts_with('.') {
            let dots = specifier.chars().take_while(|c| *c == '.').count();
            let rest = specifier[dots..].trim_matches('.');
            let mut directory = directory_key(&self.path);
            for _ in 1..dots {
                directory = parent_directory(&directory).to_string();
                if directory.is_empty() {
                    directory = ".".to_string();
                }
            }
            if rest.is_empty() {
                let init = join_dir(&directory, "__init__.py");
                if self.index.files.contains_key(&init) {
                    return Ok(ModuleRef::File(init));
                }
                return Err(Resolution::Unresolved {
                    name: specifier.to_string(),
                });
            }
            let relative = rest.replace('.', "/");
            let direct = join_dir(&directory, &format!("{relative}.py"));
            if self.index.files.contains_key(&direct) {
                return Ok(ModuleRef::File(direct));
            }
            let package_init = join_dir(&directory, &format!("{relative}/__init__.py"));
            if self.index.files.contains_key(&package_init) {
                return Ok(ModuleRef::File(package_init));
            }
            return Err(Resolution::Unresolved {
                name: specifier.to_string(),
            });
        }
        self.python_module(specifier, &self.path)
            .ok_or_else(|| Resolution::Unresolved {
                name: specifier.to_string(),
            })
    }

    pub(crate) fn python_module(&self, dotted: &str, from_path: &str) -> Option<ModuleRef> {
        let dotted = dotted.trim_matches('.');
        if dotted.is_empty() {
            return None;
        }
        if let Some(paths) = self.index.by_python_module.get(dotted) {
            if paths.len() == 1 {
                return Some(ModuleRef::File(paths[0].clone()));
            }
            if paths.len() > 1 {
                return None;
            }
        }
        let suffix = format!(".{dotted}");
        let matches: Vec<String> = self
            .index
            .by_python_module
            .iter()
            .filter(|(module, _)| module.ends_with(&suffix))
            .flat_map(|(_, paths)| paths.clone())
            .collect();
        if matches.len() == 1 {
            return Some(ModuleRef::File(matches[0].clone()));
        }
        let directory = parent_directory(from_path);
        let direct = if directory.is_empty() {
            format!("{}.py", dotted.replace('.', "/"))
        } else {
            format!("{directory}/{}.py", dotted.replace('.', "/"))
        };
        self.index
            .files
            .contains_key(&direct)
            .then_some(ModuleRef::File(direct))
    }

    pub(crate) fn resolve_go(
        &self,
        import: &ImportData,
    ) -> std::result::Result<ModuleRef, Resolution> {
        let path = import.module.trim();
        let mut directories = Vec::new();
        for module in &self.index.go_modules {
            if path == module.module {
                directories.push(if module.directory == "." {
                    ".".to_string()
                } else {
                    module.directory.clone()
                });
            } else if let Some(rest) = path.strip_prefix(&format!("{}/", module.module)) {
                let directory = if module.directory == "." {
                    rest.to_string()
                } else {
                    format!("{}/{}", module.directory, rest)
                };
                directories.push(directory);
            }
        }
        directories.sort();
        directories.dedup();
        directories.retain(|directory| self.index.by_directory.contains_key(directory));
        if directories.len() == 1 {
            let directory = directories.remove(0);
            return Ok(ModuleRef::Package(module_id_to_package(&directory)));
        }
        let suffix = format!("/{path}");
        for directory in self.index.by_directory.keys() {
            if *directory == path || directory.ends_with(&suffix) {
                directories.push(directory.clone());
            }
        }
        directories.sort();
        directories.dedup();
        directories.retain(|directory| {
            self.index.by_directory.get(directory).is_some_and(|files| {
                files.iter().any(|path| {
                    self.index
                        .files
                        .get(path)
                        .is_some_and(|state| state.language == Language::Go)
                })
            })
        });
        match directories.len() {
            1 => Ok(ModuleRef::Package(module_id_to_package(&directories[0]))),
            0 => Ok(ModuleRef::External(path.to_string())),
            _ => Err(Resolution::Ambiguous {
                name: path.to_string(),
                candidates: directories,
            }),
        }
    }

    pub(crate) fn resolve_rust(
        &self,
        import: &ImportData,
    ) -> std::result::Result<ModuleRef, Resolution> {
        let raw = import
            .module
            .as_str()
            .trim_end_matches('*')
            .trim_end_matches("::");
        if let Some(module) = self.resolve_module_path(raw) {
            return Ok(module);
        }
        let first = raw.split("::").next().unwrap_or(raw);
        if matches!(first, "crate" | "self" | "super") {
            return Err(Resolution::Unresolved {
                name: raw.to_string(),
            });
        }
        if raw.is_empty() {
            return Err(Resolution::Unresolved {
                name: raw.to_string(),
            });
        }
        Ok(ModuleRef::External(
            raw.split("::").next().unwrap_or(raw).to_string(),
        ))
    }

    pub(crate) fn resolve_kotlin(
        &self,
        import: &ImportData,
    ) -> std::result::Result<ModuleRef, Resolution> {
        let raw = import.module.trim();
        if let Some(package) = raw.strip_suffix(".*") {
            let id = kotlin_package_id(package);
            if self.index.packages.contains_key(&id) {
                return Ok(ModuleRef::Package(id));
            }
            return Err(Resolution::Unresolved {
                name: package.to_string(),
            });
        }
        let (package, name) = match raw.rsplit_once('.') {
            Some((package, name)) => (package, name),
            None => ("", raw),
        };
        let candidates: Vec<String> = self
            .index
            .by_kotlin_package
            .get(package)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|path| {
                self.index.files.get(path).is_some_and(|state| {
                    state
                        .top_level
                        .get(name)
                        .is_some_and(|indexes| !indexes.is_empty())
                })
            })
            .collect();
        match candidates.len() {
            1 => Ok(ModuleRef::File(candidates[0].clone())),
            0 => {
                if self.index.by_kotlin_package.contains_key(package) {
                    Ok(ModuleRef::Package(kotlin_package_id(package)))
                } else {
                    Ok(ModuleRef::External(raw.to_string()))
                }
            }
            _ => Err(Resolution::Ambiguous {
                name: raw.to_string(),
                candidates,
            }),
        }
    }

    pub(crate) fn resolve_module_path(&self, raw: &str) -> Option<ModuleRef> {
        let state = self.state;
        let mut segments: Vec<String> = raw
            .split("::")
            .filter(|segment| !segment.is_empty() && *segment != "*")
            .map(|segment| segment.to_string())
            .collect();
        if segments.is_empty() {
            return None;
        }
        let mut module_segments: Vec<String> = Vec::new();
        let first = segments.first()?.as_str();
        match first {
            "crate" => {
                segments.remove(0);
            }
            "self" => {
                segments.remove(0);
                module_segments = self.rust_current_module(state);
            }
            "super" => {
                let mut hops = 0;
                while segments.first().is_some_and(|segment| segment == "super") {
                    segments.remove(0);
                    hops += 1;
                }
                module_segments = self.rust_current_module(state);
                for _ in 0..hops {
                    module_segments.pop();
                }
            }
            _ => {}
        }
        let crate_root = state.crate_root.clone().unwrap_or_default();

        let mut full = module_segments.clone();
        full.extend(segments.iter().cloned());
        if let Some(path) = self.rust_file_for(&crate_root, &full) {
            return Some(ModuleRef::File(path));
        }
        if segments.len() > 1 {
            let mut module = module_segments.clone();
            module.extend(segments[..segments.len() - 1].iter().cloned());
            if let Some(path) = self.rust_file_for(&crate_root, &module) {
                return Some(ModuleRef::File(path));
            }
        }
        None
    }

    pub(crate) fn rust_current_module(&self, state: &FileState) -> Vec<String> {
        state
            .module_qualified
            .strip_prefix("crate::")
            .unwrap_or("")
            .split("::")
            .filter(|segment| !segment.is_empty())
            .map(|segment| segment.to_string())
            .collect()
    }

    pub(crate) fn rust_file_for(&self, crate_root: &str, segments: &[String]) -> Option<String> {
        let relative = segments.join("/");
        let mut candidates = Vec::new();
        let prefix = if crate_root.is_empty() || crate_root == "." {
            String::new()
        } else {
            format!("{crate_root}/")
        };
        if relative.is_empty() {
            candidates.push(format!("{prefix}src/lib.rs"));
            candidates.push(format!("{prefix}src/main.rs"));
            candidates.push(format!("{prefix}lib.rs"));
            candidates.push(format!("{prefix}main.rs"));
        } else {
            candidates.push(format!("{prefix}src/{relative}.rs"));
            candidates.push(format!("{prefix}src/{relative}/mod.rs"));
            candidates.push(format!("{prefix}{relative}.rs"));
            candidates.push(format!("{prefix}{relative}/mod.rs"));
        }
        candidates
            .into_iter()
            .find(|path| self.index.files.contains_key(path))
    }

    pub(crate) fn rust_module_tail(&self, path: &str) -> String {
        self.index
            .files
            .get(path)
            .map(|state| {
                state
                    .module_qualified
                    .rsplit("::")
                    .next()
                    .unwrap_or("")
                    .to_string()
            })
            .unwrap_or_else(|| crate::index::stem(path).to_string())
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
