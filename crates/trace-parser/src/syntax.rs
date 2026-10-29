use serde::{Deserialize, Serialize};

use trace_core::{Diagnostic, Language, NodeKind, SourceRange, Visibility};

/// A declaration extracted from a syntax tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolData {
    /// File-local index used to link parents, calls and type references.
    pub local_id: u32,
    pub kind: NodeKind,
    pub name: String,
    pub qualified_name: String,
    pub parent: Option<u32>,
    /// Assigned by the adapter for symbols sharing kind + qualified name.
    pub ordinal: u32,
    pub range: SourceRange,
    pub signature: Option<String>,
    pub visibility: Option<Visibility>,
    pub exported: bool,
    /// Decorator/attribute names attached to the declaration.
    #[serde(default)]
    pub decorators: Vec<String>,
    /// True when the declaration explicitly overrides a supertype member.
    #[serde(default)]
    pub overrides: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportKind {
    /// ES named/default/namespace import.
    Es,
    /// Python `import` or `from ... import ...`.
    Python,
    /// Go import declaration.
    Go,
    /// Rust `use` declaration.
    Rust,
    /// Kotlin import header.
    Kotlin,
    /// Swift import declaration (modules only).
    Swift,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportItem {
    /// Imported name. `*` means a wildcard, `default` an ES default import.
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    /// Module specifier exactly as written, before language-specific cleaning.
    pub module: String,
    /// Binding for a namespace/module import (`import * as ns`, `import x as y`).
    pub alias: Option<String>,
    pub items: Vec<ImportItem>,
    pub kind: ImportKind,
    pub range: SourceRange,
    /// True when the statement is a re-export (`export ... from ...`).
    #[serde(default)]
    pub is_export: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallKind {
    Function,
    Method,
    Constructor,
    Super,
    Macro,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallData {
    /// Enclosing callable symbol, or `None` for module-level expressions.
    pub caller: Option<u32>,
    /// Textual receiver for member calls (`this`, `c`, `path`, `Circle`, ...).
    pub receiver: Option<String>,
    /// Final callee name.
    pub name: String,
    /// Full textual callee expression, useful for diagnostics and fallbacks.
    pub full: String,
    pub kind: CallKind,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeritageRelation {
    Inherits,
    Implements,
    Extends,
    Mixin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeritageData {
    /// The declaring type.
    pub from: u32,
    /// Referenced superclass/interface/protocol name as written.
    pub name: String,
    pub relation: HeritageRelation,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeRefData {
    /// Enclosing symbol, or `None` at module level.
    pub from: Option<u32>,
    pub name: String,
    pub range: SourceRange,
}

/// A local variable binding whose type can be inferred cheaply from a
/// constructor or annotated declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalBinding {
    /// Enclosing callable symbol, or `None` at module level.
    pub scope: Option<u32>,
    pub name: String,
    /// Type or class name as written.
    pub type_name: String,
    pub range: SourceRange,
}

/// Syntax-level result for one source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedFile {
    pub path: String,
    pub language: Language,
    pub source_hash: String,
    /// Declared package/module name where the language provides one.
    pub module_name: Option<String>,
    pub symbols: Vec<SymbolData>,
    pub imports: Vec<ImportData>,
    pub calls: Vec<CallData>,
    pub type_refs: Vec<TypeRefData>,
    pub heritage: Vec<HeritageData>,
    pub locals: Vec<LocalBinding>,
    pub diagnostics: Vec<Diagnostic>,
    pub parse_error_count: usize,
}

impl ParsedFile {
    pub fn symbol(&self, local_id: u32) -> Option<&SymbolData> {
        self.symbols
            .iter()
            .find(|symbol| symbol.local_id == local_id)
    }
}
