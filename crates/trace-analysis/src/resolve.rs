//! Import and symbol resolution.
//!
//! Resolution is deliberately layered and conservative. Every outcome records
//! whether it is internal, external, unresolved or ambiguous; name-only matches
//! across unrelated modules are not treated as resolved.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use trace_core::{Diagnostic, DiagnosticKind, Language, NodeKind, ResolutionStatus, SourceRange};
use trace_parser::{ImportData, ImportKind, LocalBinding};

use crate::index::{
    directory_key, parent_directory, python_module_name, stem, Binding, FileState, ModuleRef,
    RepositoryIndex, SymbolRef,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Want {
    Any,
    Type,
    Callable,
}

impl Want {
    fn accepts(self, symbol: &SymbolRef) -> bool {
        match self {
            Want::Any => true,
            Want::Type => symbol.is_type(),
            Want::Callable => symbol.is_callable(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    Internal(SymbolRef),
    Module(ModuleRef),
    External {
        qualified: String,
        name: String,
    },
    ExternalNode {
        node_id: String,
        name: String,
    },
    Unresolved {
        name: String,
    },
    Ambiguous {
        name: String,
        candidates: Vec<String>,
    },
}

impl Resolution {
    pub fn status(&self) -> ResolutionStatus {
        match self {
            Resolution::Internal(_) => ResolutionStatus::ResolvedInternal,
            Resolution::Module(ModuleRef::External(_)) => ResolutionStatus::ResolvedExternal,
            Resolution::Module(_) => ResolutionStatus::ResolvedInternal,
            Resolution::External { .. } | Resolution::ExternalNode { .. } => {
                ResolutionStatus::ResolvedExternal
            }
            Resolution::Unresolved { .. } => ResolutionStatus::Unresolved,
            Resolution::Ambiguous { .. } => ResolutionStatus::Ambiguous,
        }
    }

    pub fn is_resolved(&self) -> bool {
        !matches!(
            self,
            Resolution::Unresolved { .. } | Resolution::Ambiguous { .. }
        )
    }
}

#[derive(Debug, Clone)]
pub struct ImportEdge {
    pub from_path: String,
    pub module: ModuleRef,
    pub site: SourceRange,
    pub status: ResolutionStatus,
}

#[derive(Debug, Clone, Default)]
pub struct ImportPass {
    pub edges: Vec<ImportEdge>,
    pub diagnostics: Vec<Diagnostic>,
    pub package_pairs: BTreeSet<(String, String)>,
}

struct ImportContext<'a> {
    index: &'a RepositoryIndex,
    state: &'a FileState,
    path: String,
    language: Language,
    diagnostics: Vec<Diagnostic>,
    edges: Vec<ImportEdge>,
    package_pairs: BTreeSet<(String, String)>,
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

impl<'a> ImportContext<'a> {
    fn process(
        &mut self,
        import: &ImportData,
        bindings: &mut BTreeMap<String, Binding>,
        re_exports: &mut BTreeMap<String, String>,
    ) {
        if import.kind == ImportKind::Es && import.is_export && import.module.is_empty() {
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
            ImportKind::Es => self.bind_es(import, &module, bindings, allow_private),
            ImportKind::Python => self.bind_python(import, &module, bindings, allow_private),
            ImportKind::Go => self.bind_go(import, &module, bindings),
            ImportKind::Rust => self.bind_rust(import, &module, bindings, allow_private),
            ImportKind::Kotlin => self.bind_kotlin(import, &module, bindings, allow_private),
            ImportKind::Swift => {
                if let ModuleRef::External(name) = &module {
                    bindings.insert(
                        import.alias.clone().unwrap_or_else(|| name.clone()),
                        Binding::Module(ModuleRef::External(name.clone())),
                    );
                }
            }
        }
    }

    fn bind_es(
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

    fn bind_python(
        &mut self,
        import: &ImportData,
        module: &ModuleRef,
        bindings: &mut BTreeMap<String, Binding>,
        allow_private: bool,
    ) {
        if import.items.is_empty() {
            if let Some(alias) = &import.alias {
                bindings.insert(alias.clone(), Binding::Module(module.clone()));
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
            return;
        }
        for item in &import.items {
            if item.name == "*" {
                self.bind_wildcard(module, bindings, allow_private);
                continue;
            }
            let resolution = self.resolve_module_name(module, &item.name, true, None);
            let local = item.alias.clone().unwrap_or_else(|| item.name.clone());
            bindings.insert(local, binding_for(resolution));
        }
    }

    fn bind_go(
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

    fn bind_rust(
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

    fn bind_kotlin(
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

    fn bind_wildcard(
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

    fn resolve_local_export(&self, name: &str) -> Resolution {
        let resolver = Resolver::new(self.index);
        resolver.resolve_name(self.state, None, name, Want::Any)
    }

    fn unresolved_export(&mut self, import: &ImportData, name: &str, resolution: &Resolution) {
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

    fn resolve_module(&self, import: &ImportData) -> std::result::Result<ModuleRef, Resolution> {
        match import.kind {
            ImportKind::Es => self.resolve_es(import),
            ImportKind::Python => self.resolve_python(import),
            ImportKind::Go => self.resolve_go(import),
            ImportKind::Rust => self.resolve_rust(import),
            ImportKind::Kotlin => self.resolve_kotlin(import),
            ImportKind::Swift => {
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

    fn resolve_es(&self, import: &ImportData) -> std::result::Result<ModuleRef, Resolution> {
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

    fn resolve_python(&self, import: &ImportData) -> std::result::Result<ModuleRef, Resolution> {
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

    fn python_module(&self, dotted: &str, from_path: &str) -> Option<ModuleRef> {
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
        // A relative import from the current directory is also common in
        // projects that do not use `__init__.py`.
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

    fn resolve_go(&self, import: &ImportData) -> std::result::Result<ModuleRef, Resolution> {
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

    fn resolve_rust(&self, import: &ImportData) -> std::result::Result<ModuleRef, Resolution> {
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

    fn resolve_module_path(&self, raw: &str) -> Option<ModuleRef> {
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

        // Try the full path first: it may itself name a module file.
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

    fn resolve_kotlin(&self, import: &ImportData) -> std::result::Result<ModuleRef, Resolution> {
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

    fn rust_current_module(&self, state: &FileState) -> Vec<String> {
        state
            .module_qualified
            .strip_prefix("crate::")
            .unwrap_or("")
            .split("::")
            .filter(|segment| !segment.is_empty())
            .map(|segment| segment.to_string())
            .collect()
    }

    fn rust_file_for(&self, crate_root: &str, segments: &[String]) -> Option<String> {
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

    fn rust_module_tail(&self, path: &str) -> String {
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
            .unwrap_or_else(|| stem(path).to_string())
    }

    fn allow_private(&self, module: &ModuleRef) -> bool {
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

    fn resolve_module_name(
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
                let mut candidates: Vec<SymbolRef> = state
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
                if candidates.is_empty() && state.language == Language::Python {
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

    fn record_package_dependency(&mut self, module: &ModuleRef) {
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

    fn report_import_failure(&mut self, import: &ImportData, outcome: &Resolution) {
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

trait ResolutionExt {
    fn or_external(
        self,
        name: &str,
        external_module: Option<&str>,
        language: Language,
    ) -> Resolution;
}

impl ResolutionExt for Resolution {
    fn or_external(
        self,
        name: &str,
        external_module: Option<&str>,
        language: Language,
    ) -> Resolution {
        match self {
            Resolution::Unresolved { .. } => match external_module {
                Some(module) => Resolution::External {
                    qualified: join_external(language, module, name),
                    name: name.to_string(),
                },
                None => Resolution::Unresolved {
                    name: name.to_string(),
                },
            },
            other => other,
        }
    }
}

fn binding_for(resolution: Resolution) -> Binding {
    match resolution {
        Resolution::Internal(symbol) => Binding::Symbol(symbol),
        Resolution::Module(module) => Binding::Module(module),
        Resolution::ExternalNode { node_id, name } => Binding::ExternalSymbol { node_id, name },
        Resolution::External { qualified, name } => Binding::ExternalSymbol {
            node_id: external_id_for(&qualified),
            name,
        },
        Resolution::Unresolved { name } => Binding::Unresolved { name },
        Resolution::Ambiguous { name, candidates } => Binding::Ambiguous { name, candidates },
    }
}

fn external_id_for(qualified: &str) -> String {
    // The language segment is not known here; the node ID is only used as a
    // stable lookup key and is rewritten by the graph builder when needed.
    format!("external:unresolved:{qualified}")
}

fn choose(name: &str, mut candidates: Vec<SymbolRef>) -> Resolution {
    candidates.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    candidates.dedup_by(|a, b| a.node_id == b.node_id);
    match candidates.len() {
        0 => Resolution::Unresolved {
            name: name.to_string(),
        },
        1 => Resolution::Internal(candidates.remove(0)),
        _ => Resolution::Ambiguous {
            name: name.to_string(),
            candidates: candidates
                .into_iter()
                .map(|symbol| symbol.node_id)
                .collect(),
        },
    }
}

fn join_external(language: Language, module: &str, name: &str) -> String {
    let separator = match language {
        Language::Rust => "::",
        _ => ".",
    };
    format!("{module}{separator}{name}")
}

fn join_relative(from_path: &str, specifier: &str) -> String {
    let directory = parent_directory(from_path);
    let raw = if directory.is_empty() {
        specifier.to_string()
    } else {
        format!("{directory}/{specifier}")
    };
    trace_core::id::normalize(&raw)
}

fn join_dir(directory: &str, name: &str) -> String {
    if directory == "." || directory.is_empty() {
        name.to_string()
    } else {
        format!("{directory}/{name}")
    }
}

fn es_candidates(base: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if base.ends_with(".ts")
        || base.ends_with(".tsx")
        || base.ends_with(".js")
        || base.ends_with(".jsx")
        || base.ends_with(".mjs")
        || base.ends_with(".cjs")
        || base.ends_with(".mts")
        || base.ends_with(".cts")
    {
        candidates.push(base.to_string());
        if base.ends_with(".js") || base.ends_with(".jsx") || base.ends_with(".mjs") {
            let stem = base
                .trim_end_matches(".js")
                .trim_end_matches(".jsx")
                .trim_end_matches(".mjs");
            candidates.push(format!("{stem}.ts"));
            candidates.push(format!("{stem}.tsx"));
        }
    }
    for extension in [
        ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".d.ts",
    ] {
        candidates.push(format!("{base}{extension}"));
    }
    for extension in [
        "/index.ts",
        "/index.tsx",
        "/index.js",
        "/index.jsx",
        "/index.mjs",
    ] {
        candidates.push(format!("{base}{extension}"));
    }
    candidates
}

fn module_id_to_package(directory: &str) -> String {
    trace_core::package_id(if directory == "." { "" } else { directory })
}

fn kotlin_package_id(package: &str) -> String {
    trace_core::package_id(&format!("kotlin/{}", package.replace('.', "/")))
}

fn last_path_segment(path: &str) -> &str {
    path.rsplit([':', '.'])
        .find(|segment| !segment.is_empty() && *segment != "*")
        .unwrap_or("")
}

/// Layered symbol resolver used by the graph builder.
#[derive(Debug, Clone, Copy)]
pub struct Resolver<'a> {
    index: &'a RepositoryIndex,
}

impl<'a> Resolver<'a> {
    pub fn new(index: &'a RepositoryIndex) -> Self {
        Self { index }
    }

    pub fn index(&self) -> &'a RepositoryIndex {
        self.index
    }

    pub fn resolve_name(
        &self,
        state: &FileState,
        scope: Option<u32>,
        raw_name: &str,
        want: Want,
    ) -> Resolution {
        let name = clean_name(raw_name);
        if name.is_empty() {
            return Resolution::Unresolved { name };
        }
        if name.contains("::") || name.contains('.') {
            if let Some(resolution) = self.resolve_binding_path(state, &name, want) {
                return resolution;
            }
            let last = last_dotted_segment(&name).to_string();
            if let Some(resolution) = self.resolve_binding_path(state, &last, want) {
                if matches!(resolution, Resolution::Internal(_)) {
                    return resolution;
                }
            }
        }

        // Lexical scope: declarations in the same file. Sibling methods are
        // only implicit in Kotlin and Swift; other languages need an explicit
        // receiver or import.
        let mut candidates = self.lexical_candidates(state, scope, &name, want);
        if candidates.is_empty() && matches!(state.language, Language::Kotlin | Language::Swift) {
            if let Some(scope) = scope {
                if let Some(parent) = self.index.parent_type_of(state, scope) {
                    if let Resolution::Internal(member) = self.resolve_member(&parent, &name, want)
                    {
                        candidates.push(member);
                    }
                }
            }
        }
        if !candidates.is_empty() {
            return choose(&name, candidates);
        }

        // Imported bindings.
        if let Some(binding) = state.bindings.get(&name) {
            return match binding {
                Binding::Symbol(symbol) if want.accepts(symbol) => {
                    Resolution::Internal(symbol.clone())
                }
                Binding::Symbol(_) => Resolution::Unresolved { name },
                Binding::Module(module) => Resolution::Module(module.clone()),
                Binding::ExternalSymbol { node_id, name } => Resolution::ExternalNode {
                    node_id: node_id.clone(),
                    name: name.clone(),
                },
                Binding::Unresolved { name } => Resolution::Unresolved { name: name.clone() },
                Binding::Ambiguous { name, candidates } => Resolution::Ambiguous {
                    name: name.clone(),
                    candidates: candidates.clone(),
                },
            };
        }

        // Same package/module scope. Rust is excluded because modules, not
        // crates, define unqualified name scope.
        if state.language != Language::Rust {
            if let Some(group) = &state.group {
                let candidates = self
                    .index
                    .symbols_in_group(group, &name, want == Want::Type);
                let filtered: Vec<SymbolRef> = candidates
                    .into_iter()
                    .filter(|symbol| want.accepts(symbol))
                    .collect();
                if !filtered.is_empty() {
                    return choose(&name, filtered);
                }
            }
        }

        Resolution::Unresolved { name }
    }

    pub fn resolve_type_name(
        &self,
        state: &FileState,
        scope: Option<u32>,
        name: &str,
    ) -> Resolution {
        self.resolve_name(state, scope, name, Want::Type)
    }

    pub fn resolve_callable_name(
        &self,
        state: &FileState,
        scope: Option<u32>,
        name: &str,
    ) -> Resolution {
        self.resolve_name(state, scope, name, Want::Callable)
    }

    pub fn resolve_binding_path(
        &self,
        state: &FileState,
        raw: &str,
        want: Want,
    ) -> Option<Resolution> {
        let normalized = raw.replace("::", ".");
        let segments: Vec<&str> = normalized
            .split('.')
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.len() < 2 {
            return None;
        }
        for length in (1..=segments.len()).rev() {
            let key = segments[..length].join(".");
            let Some(binding) = state.bindings.get(&key) else {
                continue;
            };
            let mut resolution = match binding {
                Binding::Symbol(symbol) => Resolution::Internal(symbol.clone()),
                Binding::Module(module) => Resolution::Module(module.clone()),
                Binding::ExternalSymbol { node_id, name } => Resolution::ExternalNode {
                    node_id: node_id.clone(),
                    name: name.clone(),
                },
                Binding::Unresolved { name } => Resolution::Unresolved { name: name.clone() },
                Binding::Ambiguous { name, candidates } => Resolution::Ambiguous {
                    name: name.clone(),
                    candidates: candidates.clone(),
                },
            };
            for segment in &segments[length..] {
                resolution = self.navigate(state, resolution, segment, want)?;
            }
            return Some(resolution);
        }
        None
    }

    fn navigate(
        &self,
        state: &FileState,
        resolution: Resolution,
        segment: &str,
        want: Want,
    ) -> Option<Resolution> {
        match resolution {
            Resolution::Internal(symbol) if symbol.is_type() => {
                Some(self.resolve_member(&symbol, segment, want))
            }
            Resolution::Module(module) => Some(self.resolve_module_member(state, &module, segment)),
            Resolution::External { qualified, .. } => Some(Resolution::External {
                qualified: join_external(state.language, &qualified, segment),
                name: segment.to_string(),
            }),
            Resolution::ExternalNode { node_id, .. } => Some(Resolution::ExternalNode {
                node_id: external_id_for(&format!("{node_id}.{segment}")),
                name: segment.to_string(),
            }),
            _ => None,
        }
    }

    fn resolve_module_member(
        &self,
        _state: &FileState,
        module: &ModuleRef,
        name: &str,
    ) -> Resolution {
        match module {
            ModuleRef::File(path) => {
                let Some(target) = self.index.files.get(path) else {
                    return Resolution::Unresolved {
                        name: name.to_string(),
                    };
                };
                let candidates: Vec<SymbolRef> = target
                    .exports
                    .get(name)
                    .into_iter()
                    .flatten()
                    .filter_map(|id| self.index.symbol(id).cloned())
                    .collect();
                if candidates.is_empty() {
                    if let Some(indexes) = target.top_level.get(name) {
                        return choose(
                            name,
                            indexes
                                .iter()
                                .map(|index| target.symbols[*index].clone())
                                .collect(),
                        );
                    }
                }
                choose(name, candidates)
            }
            ModuleRef::Package(package) => {
                choose(name, self.index.symbols_in_package(package, name))
            }
            ModuleRef::External(module) => Resolution::External {
                qualified: module.clone(),
                name: name.to_string(),
            },
        }
    }

    pub fn resolve_member(&self, type_symbol: &SymbolRef, name: &str, want: Want) -> Resolution {
        let mut visited = HashSet::new();
        self.resolve_member_inner(type_symbol, name, want, &mut visited)
    }

    fn resolve_member_inner(
        &self,
        type_symbol: &SymbolRef,
        name: &str,
        want: Want,
        visited: &mut HashSet<String>,
    ) -> Resolution {
        if !visited.insert(type_symbol.node_id.clone()) {
            return Resolution::Unresolved {
                name: name.to_string(),
            };
        }
        let key = format!("{}.{}", type_symbol.qualified, name);
        let mut candidates: Vec<SymbolRef> = self
            .index
            .by_qualified
            .get(&key)
            .cloned()
            .unwrap_or_default();
        if candidates.is_empty() {
            if let Some(state) = self.index.files.get(&type_symbol.path) {
                candidates = state
                    .symbols
                    .iter()
                    .filter(|symbol| {
                        symbol.parent == Some(type_symbol.local_id) && symbol.name == name
                    })
                    .cloned()
                    .collect();
            }
        }
        candidates.retain(|symbol| want.accepts(symbol));
        if !candidates.is_empty() {
            return choose(name, candidates);
        }
        for supertype in self.supertypes(type_symbol) {
            let resolution = self.resolve_member_inner(&supertype, name, want, visited);
            if !matches!(
                resolution,
                Resolution::Unresolved { .. } | Resolution::Ambiguous { .. }
            ) {
                return resolution;
            }
        }
        Resolution::Unresolved {
            name: name.to_string(),
        }
    }

    pub fn constructor_for(&self, type_symbol: &SymbolRef) -> Resolution {
        let Some(state) = self.index.files.get(&type_symbol.path) else {
            return Resolution::Internal(type_symbol.clone());
        };
        let candidates: Vec<SymbolRef> = state
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.parent == Some(type_symbol.local_id) && symbol.kind == NodeKind::Constructor
            })
            .cloned()
            .collect();
        if candidates.is_empty() {
            Resolution::Internal(type_symbol.clone())
        } else {
            choose(&type_symbol.name, candidates)
        }
    }

    pub fn supertypes(&self, type_symbol: &SymbolRef) -> Vec<SymbolRef> {
        let Some(state) = self.index.files.get(&type_symbol.path) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for heritage in &state.parsed.heritage {
            if heritage.from != type_symbol.local_id {
                continue;
            }
            if let Resolution::Internal(symbol) =
                self.resolve_type_name(state, Some(type_symbol.local_id), &heritage.name)
            {
                if symbol.is_type() && symbol.node_id != type_symbol.node_id {
                    out.push(symbol);
                }
            }
        }
        out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        out.dedup_by(|a, b| a.node_id == b.node_id);
        out
    }

    pub fn local_type(
        &self,
        state: &FileState,
        scope: Option<u32>,
        name: &str,
        at_byte: u32,
    ) -> Option<String> {
        let mut scopes = Vec::new();
        if let Some(scope) = scope {
            scopes.push(Some(scope));
            let mut parent = state.symbol(scope).and_then(|symbol| symbol.parent);
            while let Some(local) = parent {
                scopes.push(Some(local));
                parent = state.symbol(local).and_then(|symbol| symbol.parent);
            }
        }
        scopes.push(None);
        for scope in scopes {
            let mut best: Option<(&LocalBinding, u32)> = None;
            for binding in &state.parsed.locals {
                if binding.scope != scope || binding.name != name {
                    continue;
                }
                if binding.range.start.byte > at_byte {
                    continue;
                }
                let rank = binding.range.start.byte;
                if best.is_none_or(|(_, current)| rank >= current) {
                    best = Some((binding, rank));
                }
            }
            if let Some((binding, _)) = best {
                return Some(binding.type_name.clone());
            }
        }
        None
    }

    fn lexical_candidates(
        &self,
        state: &FileState,
        scope: Option<u32>,
        name: &str,
        want: Want,
    ) -> Vec<SymbolRef> {
        let mut candidates = Vec::new();
        if let Some(indexes) = state.top_level.get(name) {
            candidates.extend(
                indexes
                    .iter()
                    .map(|index| state.symbols[*index].clone())
                    .filter(|symbol| want.accepts(symbol)),
            );
        }
        if let Some(scope) = scope {
            let mut parent = state.symbol(scope).and_then(|symbol| symbol.parent);
            while let Some(local) = parent {
                let Some(container) = state.symbol(local) else {
                    break;
                };
                candidates.extend(
                    state
                        .symbols
                        .iter()
                        .filter(|symbol| {
                            symbol.parent == Some(local)
                                && symbol.name == name
                                && want.accepts(symbol)
                                && !matches!(
                                    container.kind,
                                    NodeKind::Function | NodeKind::Method | NodeKind::Constructor
                                )
                        })
                        .cloned(),
                );
                parent = container.parent;
            }
        }
        candidates.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        candidates.dedup_by(|a, b| a.node_id == b.node_id);
        candidates
    }
}

pub fn clean_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let trimmed = trimmed.trim_start_matches(['&', '*', '?']);
    let trimmed = trimmed
        .split(['<', '[', '(', '|', '&', ' ', '<'])
        .next()
        .unwrap_or(trimmed);
    trimmed.trim().trim_end_matches('?').to_string()
}

pub fn last_dotted_segment(name: &str) -> &str {
    name.rsplit(['.', ':'])
        .find(|segment| !segment.is_empty())
        .unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use trace_parser::SyntaxParser;

    fn parse(path: &str, language: Language, source: &str) -> trace_parser::ParsedFile {
        let mut parser = SyntaxParser::new().unwrap();
        parser.parse(path, language, source).unwrap()
    }

    fn index(files: Vec<trace_parser::ParsedFile>) -> RepositoryIndex {
        let dir = tempfile::tempdir().unwrap();
        RepositoryIndex::build(dir.path(), files).unwrap()
    }

    #[test]
    fn resolves_relative_typescript_imports() {
        let mut index = index(vec![
            parse(
                "src/math.ts",
                Language::TypeScript,
                "export function twice(n: number): number { return n * 2; }\nexport class Circle {}\n",
            ),
            parse(
                "src/main.ts",
                Language::TypeScript,
                "import { twice, Circle } from './math';\nexport function run(): number { const c = new Circle(); return twice(1); }\n",
            ),
        ]);
        let pass = resolve_imports(&mut index);
        assert!(pass
            .edges
            .iter()
            .any(|edge| matches!(&edge.module, ModuleRef::File(path) if path == "src/math.ts")));
        let main = index.files.get("src/main.ts").unwrap();
        assert!(
            matches!(main.bindings.get("twice"), Some(Binding::Symbol(symbol)) if symbol.name == "twice")
        );
        assert!(
            matches!(main.bindings.get("Circle"), Some(Binding::Symbol(symbol)) if symbol.name == "Circle")
        );
    }

    #[test]
    fn does_not_resolve_same_named_functions_across_files_without_import() {
        let index = index(vec![
            parse(
                "a.ts",
                Language::TypeScript,
                "export function helper() {}\n",
            ),
            parse(
                "b.ts",
                Language::TypeScript,
                "export function run() { helper(); }\n",
            ),
        ]);
        let state = index.files.get("b.ts").unwrap();
        let resolver = Resolver::new(&index);
        let resolution = resolver.resolve_callable_name(state, Some(1), "helper");
        assert!(matches!(resolution, Resolution::Unresolved { .. }));
    }

    #[test]
    fn resolves_python_package_imports() {
        let mut index = index(vec![
            parse("pkg/__init__.py", Language::Python, ""),
            parse("pkg/util.py", Language::Python, "def helper():\n    pass\n"),
            parse(
                "pkg/main.py",
                Language::Python,
                "from .util import helper\n\ndef run():\n    return helper()\n",
            ),
        ]);
        let pass = resolve_imports(&mut index);
        assert!(pass
            .edges
            .iter()
            .any(|edge| matches!(&edge.module, ModuleRef::File(path) if path == "pkg/util.py")));
        let main = index.files.get("pkg/main.py").unwrap();
        assert!(matches!(
            main.bindings.get("helper"),
            Some(Binding::Symbol(_))
        ));
    }

    #[test]
    fn resolves_rust_use_paths() {
        let mut index = index(vec![
            parse(
                "src/util.rs",
                Language::Rust,
                "pub fn helper() -> i32 { 1 }\n",
            ),
            parse(
                "src/lib.rs",
                Language::Rust,
                "mod util;\nuse crate::util::helper;\npub fn run() -> i32 { helper() }\n",
            ),
        ]);
        let pass = resolve_imports(&mut index);
        assert!(pass
            .edges
            .iter()
            .any(|edge| matches!(&edge.module, ModuleRef::File(path) if path == "src/util.rs")));
        let state = index.files.get("src/lib.rs").unwrap();
        assert!(matches!(
            state.bindings.get("helper"),
            Some(Binding::Symbol(_))
        ));
    }

    #[test]
    fn same_package_go_names_resolve_without_import() {
        let index = index(vec![
            parse(
                "pkg/a.go",
                Language::Go,
                "package pkg\n\nfunc Helper() int { return 1 }\n",
            ),
            parse(
                "pkg/b.go",
                Language::Go,
                "package pkg\n\nfunc Run() int { return Helper() }\n",
            ),
        ]);
        let state = index.files.get("pkg/b.go").unwrap();
        let resolver = Resolver::new(&index);
        let resolution = resolver.resolve_callable_name(state, Some(1), "Helper");
        assert!(matches!(resolution, Resolution::Internal(symbol) if symbol.name == "Helper"));
    }

    #[test]
    fn reports_unresolved_relative_imports() {
        let mut index = index(vec![parse(
            "a.ts",
            Language::TypeScript,
            "import { missing } from './nope';\n",
        )]);
        let pass = resolve_imports(&mut index);
        assert!(pass
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == DiagnosticKind::UnresolvedImport));
    }
}
