//! Graph construction from a resolved [`RepositoryIndex`].
//!
//! This module turns parsed, indexed, and import-resolved source files into a
//! [`trace_core::Graph`]. All graph construction is syntax- and
//! lexically-scoped; compiler-assisted semantic resolution is not yet
//! integrated, so unresolved relationships are recorded explicitly rather than
//! invented.

use std::collections::BTreeMap;
use std::path::Path;

use rayon::prelude::*;
use trace_core::{
    external_id, file_id, package_id, Diagnostic, Edge, EdgeKind, Graph, Language, Node, NodeKind,
    ResolutionOrigin, ResolutionStatus, SourceRange,
};
use trace_parser::{CallData, CallKind, HeritageRelation, ParsedFile};

use crate::cache::SyntaxCache;
use crate::error::{AnalysisError, Result};
use crate::index::{Binding, FileState, ModuleRef, RepositoryIndex, SymbolRef};
use crate::resolve::{clean_name, last_dotted_segment, Resolver, Want};

/// The result of building a graph.
#[derive(Debug, Clone)]
pub struct GraphBuildResult {
    pub graph: Graph,
    pub diagnostics: Vec<Diagnostic>,
}

/// Builds a [`Graph`] from discovered, parsed, and resolved source files.
#[derive(Debug, Clone, Default)]
pub struct GraphBuilder {
    /// Whether to include external symbol nodes for unresolved imports.
    pub include_external: bool,
    /// Optional syntax cache for incremental parsing.
    pub cache: Option<SyntaxCache>,
    /// Whether to use parallel parsing (default: true).
    pub parallel: bool,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the syntax cache for incremental parsing.
    pub fn with_cache(mut self, cache: SyntaxCache) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Enable or disable parallel parsing.
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Build a graph from a repository root using default discovery options.
    pub fn build_repository(&self, root: &Path) -> Result<GraphBuildResult> {
        let options = trace_parser::discovery::DiscoveryOptions::default();
        self.build_repository_with_options(root, &options)
    }

    /// Build a graph from a repository root with custom discovery options.
    pub fn build_repository_with_options(
        &self,
        root: &Path,
        options: &trace_parser::discovery::DiscoveryOptions,
    ) -> Result<GraphBuildResult> {
        use trace_parser::discovery::discover;

        let report = discover(root, options).map_err(AnalysisError::Discovery)?;

        let cache_ref = self.cache.clone();
        let files: Vec<_> = report.files.to_vec();

        let parsed_files: Vec<Result<trace_parser::ParsedFile>> = if self.parallel {
            // Parallel parsing with rayon
            files
                .par_iter()
                .map(|file| {
                    let source = std::fs::read_to_string(&file.absolute).map_err(|error| {
                        AnalysisError::Read {
                            path: file.absolute.display().to_string(),
                            message: error.to_string(),
                        }
                    })?;

                    // Calculate source hash for caching
                    let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();

                    // Try to load from cache if available
                    let parsed = if let Some(cache) = &cache_ref {
                        if let Some(cached) = cache.load(&file.path, file.language, &source_hash) {
                            cached
                        } else {
                            let mut parser = trace_parser::parser::SyntaxParser::new()
                                .map_err(|error| AnalysisError::Parser(error.to_string()))?;
                            let parsed = parser
                                .parse(&file.path, file.language, &source)
                                .map_err(|error| AnalysisError::Parser(error.to_string()))?;
                            cache.store(&parsed);
                            parsed
                        }
                    } else {
                        let mut parser = trace_parser::parser::SyntaxParser::new()
                            .map_err(|error| AnalysisError::Parser(error.to_string()))?;
                        parser
                            .parse(&file.path, file.language, &source)
                            .map_err(|error| AnalysisError::Parser(error.to_string()))?
                    };

                    Ok(parsed)
                })
                .collect()
        } else {
            // Sequential parsing
            let mut parser = trace_parser::parser::SyntaxParser::new()
                .map_err(|error| AnalysisError::Parser(error.to_string()))?;

            files
                .iter()
                .map(|file| {
                    let source = std::fs::read_to_string(&file.absolute).map_err(|error| {
                        AnalysisError::Read {
                            path: file.absolute.display().to_string(),
                            message: error.to_string(),
                        }
                    })?;

                    // Calculate source hash for caching
                    let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();

                    // Try to load from cache if available
                    let parsed = if let Some(cache) = &cache_ref {
                        if let Some(cached) = cache.load(&file.path, file.language, &source_hash) {
                            cached
                        } else {
                            let parsed = parser
                                .parse(&file.path, file.language, &source)
                                .map_err(|error| AnalysisError::Parser(error.to_string()))?;
                            cache.store(&parsed);
                            parsed
                        }
                    } else {
                        parser
                            .parse(&file.path, file.language, &source)
                            .map_err(|error| AnalysisError::Parser(error.to_string()))?
                    };

                    Ok(parsed)
                })
                .collect()
        };

        // Collect results and diagnostics
        let mut diagnostics = Vec::new();
        let mut parsed_files_ok = Vec::new();
        for result in parsed_files {
            match result {
                Ok(parsed) => {
                    diagnostics.extend(parsed.diagnostics.clone());
                    parsed_files_ok.push(parsed);
                }
                Err(e) => return Err(e),
            }
        }

        parsed_files_ok.sort_by(|a, b| a.path.cmp(&b.path));

        let mut index = RepositoryIndex::build(root, parsed_files_ok)?;
        let import_pass = crate::resolve::resolve_imports(&mut index);

        let mut result = self.build_from_index(&index, &import_pass);
        diagnostics.extend(import_pass.diagnostics);
        diagnostics.extend(report.diagnostics.clone());
        diagnostics.sort_by_key(|d| d.sort_key());
        result.diagnostics.extend(diagnostics);
        result.diagnostics.sort_by_key(|d| d.sort_key());
        result.diagnostics.dedup();
        Ok(result)
    }

    /// Build a graph directly from already-parsed files.
    pub fn build_from_parsed(
        &self,
        root: &Path,
        parsed_files: Vec<ParsedFile>,
    ) -> Result<GraphBuildResult> {
        let mut index = RepositoryIndex::build(root, parsed_files)?;
        let import_pass = crate::resolve::resolve_imports(&mut index);
        let mut result = self.build_from_index(&index, &import_pass);
        result.diagnostics.extend(import_pass.diagnostics);
        result.diagnostics.sort_by_key(|d| d.sort_key());
        result.diagnostics.dedup();
        Ok(result)
    }

    /// Build a graph from a resolved index and import pass.
    pub fn build_from_index(
        &self,
        index: &RepositoryIndex,
        import_pass: &crate::resolve::ImportPass,
    ) -> GraphBuildResult {
        let mut graph = Graph::new();
        let diagnostics = Vec::new();

        let repository = trace_core::graph::repository_node("repository");
        let repository_id = repository.id.clone();
        graph.add_node(repository).unwrap();

        // Import edges aggregated by (from file, module target) for diagnostics.
        let import_edges: BTreeMap<(String, String), Vec<SourceRange>> = import_pass
            .edges
            .iter()
            .fold(BTreeMap::new(), |mut acc, edge| {
                let target = self.module_target_node(index, &module_id_str(&edge.module));
                acc.entry((edge.from_path.clone(), target))
                    .or_default()
                    .push(edge.site);
                acc
            });

        for (path, state) in &index.files {
            self.add_file_nodes_and_edges(&mut graph, index, state, path);
        }

        // Repository contains every file and package.
        for path in index.files.keys() {
            graph
                .add_edge(Edge::new(
                    repository_id.clone(),
                    file_id(path),
                    EdgeKind::Contains,
                    ResolutionStatus::ResolvedInternal,
                    ResolutionOrigin::Syntax,
                ))
                .unwrap();
        }
        for package_id_str in index.packages.keys() {
            graph
                .add_edge(Edge::new(
                    repository_id.clone(),
                    package_id_str.clone(),
                    EdgeKind::Contains,
                    ResolutionStatus::ResolvedInternal,
                    ResolutionOrigin::Syntax,
                ))
                .unwrap();
        }

        // File-level import edges.
        for ((from_path, target), sites) in &import_edges {
            let status = if target.starts_with("external:") {
                ResolutionStatus::ResolvedExternal
            } else {
                ResolutionStatus::ResolvedInternal
            };
            let mut edge = Edge::new(
                file_id(from_path),
                target.clone(),
                EdgeKind::Imports,
                status,
                ResolutionOrigin::Import,
            );
            for site in sites {
                edge = edge.at(*site);
            }
            graph.add_edge(edge).unwrap();
        }

        // Package containment edges.
        for (package_id_str, package) in &index.packages {
            for file_path in &package.files {
                graph
                    .add_edge(Edge::new(
                        package_id_str.clone(),
                        file_id(file_path),
                        EdgeKind::Contains,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Syntax,
                    ))
                    .unwrap();
            }
        }

        // Symbol-level edges: calls, references, heritage, exports, definitions.
        for (path, state) in &index.files {
            self.add_symbol_edges(&mut graph, index, state, path);
        }

        graph.finalize();

        GraphBuildResult { graph, diagnostics }
    }

    fn add_file_nodes_and_edges(
        &self,
        graph: &mut Graph,
        _index: &RepositoryIndex,
        state: &FileState,
        path: &str,
    ) {
        let file_node = Node::new(file_id(path), NodeKind::File, path.to_string())
            .with_file(path.to_string())
            .with_language(state.language);
        graph.add_node(file_node).unwrap();

        // Package node.
        if let Some(package) = &state.package {
            let package_node = Node::new(package.clone(), NodeKind::Package, package.clone())
                .with_file(path.to_string())
                .with_language(state.language);
            graph.add_node(package_node).unwrap();

            graph
                .add_edge(Edge::new(
                    package.clone(),
                    file_id(path),
                    EdgeKind::Contains,
                    ResolutionStatus::ResolvedInternal,
                    ResolutionOrigin::Syntax,
                ))
                .unwrap();
        }

        // Module node.
        let module_node = Node::new(
            state.module_id.clone(),
            NodeKind::Module,
            state.module_name.clone(),
        )
        .with_file(path.to_string())
        .with_language(state.language);
        graph.add_node(module_node).unwrap();

        graph
            .add_edge(Edge::new(
                file_id(path),
                state.module_id.clone(),
                EdgeKind::Contains,
                ResolutionStatus::ResolvedInternal,
                ResolutionOrigin::Syntax,
            ))
            .unwrap();

        // Symbol nodes.
        for symbol in &state.symbols {
            let parent = self.parent_node_id(state, symbol);
            let mut node = Node::new(symbol.node_id.clone(), symbol.kind, symbol.name.clone())
                .with_file(path.to_string())
                .with_language(state.language)
                .with_qualified_name(symbol.qualified.clone())
                .with_exported(symbol.exported);
            if let Some(parent_id) = parent {
                node = node.with_parent(parent_id);
            }
            if let Some(range) = symbol_range(state, symbol) {
                node = node.with_range(range);
            }
            graph.add_node(node).unwrap();

            // Module defines symbol.
            graph
                .add_edge(Edge::new(
                    state.module_id.clone(),
                    symbol.node_id.clone(),
                    EdgeKind::Defines,
                    ResolutionStatus::ResolvedInternal,
                    ResolutionOrigin::Syntax,
                ))
                .unwrap();

            // File contains symbol.
            graph
                .add_edge(Edge::new(
                    file_id(path),
                    symbol.node_id.clone(),
                    EdgeKind::Contains,
                    ResolutionStatus::ResolvedInternal,
                    ResolutionOrigin::Syntax,
                ))
                .unwrap();

            // File exports symbol.
            if symbol.exported {
                graph
                    .add_edge(Edge::new(
                        file_id(path),
                        symbol.node_id.clone(),
                        EdgeKind::Exports,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Syntax,
                    ))
                    .unwrap();
            }
        }

        // Exports from re-exports.
        for (name, ids) in &state.exports {
            let _ = name;
            for id in ids {
                graph
                    .add_edge(Edge::new(
                        file_id(path),
                        id.clone(),
                        EdgeKind::Exports,
                        ResolutionStatus::ResolvedInternal,
                        ResolutionOrigin::Import,
                    ))
                    .unwrap();
            }
        }
    }

    fn add_symbol_edges(
        &self,
        graph: &mut Graph,
        index: &RepositoryIndex,
        state: &FileState,
        path: &str,
    ) {
        let _ = path;
        let resolver = Resolver::new(index);

        // Call edges.
        for call in &state.parsed.calls {
            let caller_id = call
                .caller
                .and_then(|local| state.symbol(local))
                .map(|symbol| symbol.node_id.clone())
                .unwrap_or_else(|| state.module_id.clone());

            let target = self.resolve_call_target(index, state, &resolver, call);
            match target {
                CallTarget::Internal(node_id) => {
                    graph
                        .add_edge(
                            Edge::new(
                                caller_id.clone(),
                                node_id,
                                EdgeKind::Calls,
                                ResolutionStatus::ResolvedInternal,
                                ResolutionOrigin::Lexical,
                            )
                            .at(call.range),
                        )
                        .unwrap();
                }
                CallTarget::External(qualified) => {
                    let node_id = external_id(state.language, &qualified);
                    if self.include_external {
                        self.add_external_node(graph, state.language, &qualified);
                    }
                    graph
                        .add_edge(
                            Edge::new(
                                caller_id.clone(),
                                node_id,
                                EdgeKind::Calls,
                                ResolutionStatus::ResolvedExternal,
                                ResolutionOrigin::Lexical,
                            )
                            .at(call.range),
                        )
                        .unwrap();
                }
                CallTarget::Unresolved => {
                    graph
                        .add_edge(
                            Edge::new(
                                caller_id.clone(),
                                format!("unresolved:{}", call.name),
                                EdgeKind::Calls,
                                ResolutionStatus::Unresolved,
                                ResolutionOrigin::Syntax,
                            )
                            .at(call.range),
                        )
                        .unwrap();
                }
                CallTarget::Ambiguous(candidates) => {
                    for candidate in candidates {
                        graph
                            .add_edge(
                                Edge::new(
                                    caller_id.clone(),
                                    candidate,
                                    EdgeKind::Calls,
                                    ResolutionStatus::Ambiguous,
                                    ResolutionOrigin::Syntax,
                                )
                                .at(call.range),
                            )
                            .unwrap();
                    }
                }
            }
        }

        // Type reference edges.
        for type_ref in &state.parsed.type_refs {
            let from_id = type_ref
                .from
                .and_then(|local| state.symbol(local))
                .map(|symbol| symbol.node_id.clone())
                .unwrap_or_else(|| state.module_id.clone());

            let resolution = resolver.resolve_type_name(state, type_ref.from, &type_ref.name);
            match resolution {
                crate::resolve::Resolution::Internal(symbol) => {
                    graph
                        .add_edge(
                            Edge::new(
                                from_id.clone(),
                                symbol.node_id,
                                EdgeKind::UsesType,
                                ResolutionStatus::ResolvedInternal,
                                ResolutionOrigin::Syntax,
                            )
                            .at(type_ref.range),
                        )
                        .unwrap();
                }
                crate::resolve::Resolution::External { qualified, .. } => {
                    let node_id = external_id(state.language, &qualified);
                    if self.include_external {
                        self.add_external_node(graph, state.language, &qualified);
                    }
                    graph
                        .add_edge(
                            Edge::new(
                                from_id.clone(),
                                node_id,
                                EdgeKind::UsesType,
                                ResolutionStatus::ResolvedExternal,
                                ResolutionOrigin::Syntax,
                            )
                            .at(type_ref.range),
                        )
                        .unwrap();
                }
                crate::resolve::Resolution::Module(module) => {
                    let target = self.module_target_node(index, &module_id_str(&module));
                    graph
                        .add_edge(
                            Edge::new(
                                from_id.clone(),
                                target,
                                EdgeKind::UsesType,
                                ResolutionStatus::ResolvedInternal,
                                ResolutionOrigin::Syntax,
                            )
                            .at(type_ref.range),
                        )
                        .unwrap();
                }
                _ => {
                    graph
                        .add_edge(
                            Edge::new(
                                from_id.clone(),
                                format!("unresolved:{}", type_ref.name),
                                EdgeKind::UsesType,
                                ResolutionStatus::Unresolved,
                                ResolutionOrigin::Syntax,
                            )
                            .at(type_ref.range),
                        )
                        .unwrap();
                }
            }
        }

        // Heritage edges (inherits / implements).
        for heritage in &state.parsed.heritage {
            let from_symbol = state.symbol(heritage.from);
            let Some(from_symbol) = from_symbol else {
                continue;
            };
            let resolution = resolver.resolve_type_name(state, Some(heritage.from), &heritage.name);
            let relation = match heritage.relation {
                HeritageRelation::Implements => EdgeKind::Implements,
                _ => EdgeKind::Inherits,
            };
            match resolution {
                crate::resolve::Resolution::Internal(symbol) => {
                    graph
                        .add_edge(
                            Edge::new(
                                from_symbol.node_id.clone(),
                                symbol.node_id,
                                relation,
                                ResolutionStatus::ResolvedInternal,
                                ResolutionOrigin::Syntax,
                            )
                            .at(heritage.range),
                        )
                        .unwrap();
                }
                crate::resolve::Resolution::External { qualified, .. } => {
                    let node_id = external_id(state.language, &qualified);
                    if self.include_external {
                        self.add_external_node(graph, state.language, &qualified);
                    }
                    graph
                        .add_edge(
                            Edge::new(
                                from_symbol.node_id.clone(),
                                node_id,
                                relation,
                                ResolutionStatus::ResolvedExternal,
                                ResolutionOrigin::Syntax,
                            )
                            .at(heritage.range),
                        )
                        .unwrap();
                }
                _ => {
                    graph
                        .add_edge(
                            Edge::new(
                                from_symbol.node_id.clone(),
                                format!("unresolved:{}", heritage.name),
                                relation,
                                ResolutionStatus::Unresolved,
                                ResolutionOrigin::Syntax,
                            )
                            .at(heritage.range),
                        )
                        .unwrap();
                }
            }
        }
    }

    #[allow(clippy::collapsible_match)]
    fn resolve_call_target(
        &self,
        index: &RepositoryIndex,
        state: &FileState,
        resolver: &Resolver,
        call: &CallData,
    ) -> CallTarget {
        // If there is a receiver, resolve the receiver type and look up the
        // member method on it.
        if let Some(receiver) = &call.receiver {
            let cleaned = clean_name(receiver);
            if let Some(scope) = call.caller {
                if let Some(parent) = index.parent_type_of(state, scope) {
                    let member = resolver.resolve_member(&parent, &cleaned, Want::Callable);
                    if let crate::resolve::Resolution::Internal(symbol) = member {
                        return CallTarget::Internal(symbol.node_id);
                    }
                }
            }
            // Receiver resolved as a type in bindings.
            if let Some(binding) = state.bindings.get(&cleaned) {
                match binding {
                    Binding::Symbol(symbol) if symbol.is_type() => {
                        if let Some(member) =
                            self.find_method_on_type(index, state, symbol, &call.name)
                        {
                            return CallTarget::Internal(member.node_id);
                        }
                    }
                    Binding::Module(module) => {
                        if let ModuleRef::File(path) = module {
                            if let Some(target) = index.files.get(path) {
                                if let Some(symbol) =
                                    target.top_level.get(&call.name).and_then(|indexes| {
                                        indexes.first().and_then(|i| target.symbols.get(*i))
                                    })
                                {
                                    return CallTarget::Internal(symbol.node_id.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            return match call.kind {
                CallKind::Constructor => CallTarget::External(format!("{receiver}.{}", call.name)),
                _ => CallTarget::Unresolved,
            };
        }

        // Plain function call: resolve the name through bindings and scope.
        let scope = call.caller;
        let resolution = resolver.resolve_callable_name(state, scope, &call.name);
        match resolution {
            crate::resolve::Resolution::Internal(symbol) => {
                return CallTarget::Internal(symbol.node_id);
            }
            crate::resolve::Resolution::ExternalNode { node_id, .. } => {
                return CallTarget::Internal(node_id);
            }
            crate::resolve::Resolution::Module(module) => {
                return match module {
                    ModuleRef::File(path) => {
                        if let Some(target) = index.files.get(path.as_str()) {
                            if let Some(symbol) =
                                target.top_level.get(&call.name).and_then(|indexes| {
                                    indexes.first().and_then(|i| target.symbols.get(*i))
                                })
                            {
                                CallTarget::Internal(symbol.node_id.clone())
                            } else {
                                CallTarget::Unresolved
                            }
                        } else {
                            CallTarget::Unresolved
                        }
                    }
                    ModuleRef::Package(_) => CallTarget::Unresolved,
                    ModuleRef::External(name) => {
                        CallTarget::External(format!("{name}.{}", call.name))
                    }
                };
            }
            crate::resolve::Resolution::Ambiguous { candidates, .. } => {
                return CallTarget::Ambiguous(candidates);
            }
            _ => {}
        }

        // Same-package/group fallback for callables.
        if let Some(group) = &state.group {
            let candidates = index.symbols_in_group(group, &call.name, false);
            let callables: Vec<_> = candidates.into_iter().filter(|s| s.is_callable()).collect();
            match callables.len() {
                1 => return CallTarget::Internal(callables[0].node_id.clone()),
                _ if !callables.is_empty() => {
                    return CallTarget::Ambiguous(
                        callables.into_iter().map(|s| s.node_id).collect(),
                    );
                }
                _ => {}
            }
        }

        match call.kind {
            CallKind::Constructor => CallTarget::External(call.name.clone()),
            _ => CallTarget::Unresolved,
        }
    }

    fn find_method_on_type(
        &self,
        index: &RepositoryIndex,
        _state: &FileState,
        type_symbol: &SymbolRef,
        method_name: &str,
    ) -> Option<SymbolRef> {
        let key = format!("{}.{}", type_symbol.qualified, method_name);
        if let Some(candidates) = index.by_qualified.get(&key) {
            let callables: Vec<&SymbolRef> =
                candidates.iter().filter(|s| s.is_callable()).collect();
            if callables.len() == 1 {
                return Some(callables[0].clone());
            }
        }
        None
    }

    fn parent_node_id(&self, state: &FileState, symbol: &SymbolRef) -> Option<String> {
        let Some(parent_local) = symbol.parent else {
            return Some(state.module_id.clone());
        };
        let parent_symbol = state.symbol(parent_local)?;
        Some(parent_symbol.node_id.clone())
    }

    /// Resolve a module reference to the canonical node id used as an edge
    /// target.
    fn module_target_node(&self, index: &RepositoryIndex, module_id_str: &str) -> String {
        if module_id_str.starts_with("external:") {
            return module_id_str.to_string();
        }
        // Resolve a module id back to a file node when possible.
        for (path, state) in &index.files {
            if state.module_id == *module_id_str {
                return file_id(path);
            }
        }
        // Check package id.
        if index.packages.contains_key(module_id_str) {
            return module_id_str.to_string();
        }
        module_id_str.to_string()
    }

    fn add_external_node(&self, graph: &mut Graph, language: Language, qualified: &str) {
        let node_id = external_id(language, qualified);
        let name = last_dotted_segment(qualified);
        let node =
            Node::new(node_id, NodeKind::ExternalSymbol, name.to_string()).with_language(language);
        let _ = graph.add_node(node);
    }
}

#[derive(Debug, Clone)]
enum CallTarget {
    Internal(String),
    External(String),
    Unresolved,
    Ambiguous(Vec<String>),
}

fn symbol_range(state: &FileState, symbol: &SymbolRef) -> Option<SourceRange> {
    state
        .parsed
        .symbols
        .iter()
        .find(|s| s.local_id == symbol.local_id)
        .map(|s| s.range)
}

fn module_id_str(module: &ModuleRef) -> String {
    match module {
        ModuleRef::File(path) => file_id(path),
        ModuleRef::Package(package) => package_id(package),
        ModuleRef::External(name) => format!("external:unknown:{name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::resolve_imports;
    use trace_core::symbol_id;
    use trace_parser::SyntaxParser;

    fn parse(path: &str, language: Language, source: &str) -> ParsedFile {
        let mut parser = SyntaxParser::new().unwrap();
        parser.parse(path, language, source).unwrap()
    }

    #[test]
    fn builds_file_and_symbol_nodes() {
        let dir = tempfile::tempdir().unwrap();
        let parsed = vec![parse(
            "a.ts",
            Language::TypeScript,
            "export function foo() { return 1; }\n",
        )];
        let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
        let import_pass = resolve_imports(&mut index);
        let builder = GraphBuilder::new();
        let result = builder.build_from_index(&index, &import_pass);
        assert!(result.graph.node(&file_id("a.ts")).is_some());
        assert!(result
            .graph
            .node(&symbol_id(NodeKind::Function, "a.ts", "foo", 0))
            .is_some());
        assert_eq!(result.graph.node_count(), 4); // repository, file, module, function
    }

    #[test]
    fn builds_call_edge_between_files() {
        let dir = tempfile::tempdir().unwrap();
        let parsed = vec![
            parse(
                "math.ts",
                Language::TypeScript,
                "export function twice(n: number): number { return n * 2; }\n",
            ),
            parse(
                "main.ts",
                Language::TypeScript,
                "import { twice } from './math';\nexport function run(): number { return twice(1); }\n",
            ),
        ];
        let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
        let import_pass = resolve_imports(&mut index);
        let builder = GraphBuilder::new();
        let result = builder.build_from_index(&index, &import_pass);
        let run_id = symbol_id(NodeKind::Function, "main.ts", "run", 0);
        let twice_id = symbol_id(NodeKind::Function, "math.ts", "twice", 0);
        let graph = &result.graph;
        let out_edges = graph.out_edges(&run_id);
        let edges: Vec<_> = out_edges
            .iter()
            .flat_map(std::iter::once)
            .filter(|e| e.kind == EdgeKind::Calls && e.target == twice_id)
            .collect();
        assert!(!edges.is_empty(), "expected a call edge run -> twice");
    }

    #[test]
    fn records_unresolved_call() {
        let dir = tempfile::tempdir().unwrap();
        let parsed = vec![parse(
            "a.ts",
            Language::TypeScript,
            "export function run() { return missing(); }\n",
        )];
        let mut index = RepositoryIndex::build(dir.path(), parsed).unwrap();
        let import_pass = resolve_imports(&mut index);
        let builder = GraphBuilder::new();
        let result = builder.build_from_index(&index, &import_pass);
        let run_id = symbol_id(NodeKind::Function, "a.ts", "run", 0);
        let graph = &result.graph;
        let out_edges = graph.out_edges(&run_id);
        let edges: Vec<_> = out_edges
            .iter()
            .flat_map(std::iter::once)
            .filter(|e| e.kind == EdgeKind::Calls)
            .collect();
        assert!(!edges.is_empty());
        assert!(
            edges
                .iter()
                .any(|e| matches!(e.status, ResolutionStatus::Unresolved)),
            "expected an unresolved call edge"
        );
    }
}
