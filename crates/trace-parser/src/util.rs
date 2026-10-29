use std::collections::HashMap;

use tree_sitter::Node as TsNode;

use trace_core::{Language, NodeKind, Position, SourceRange, Visibility};

use crate::syntax::{
    CallData, CallKind, HeritageData, HeritageRelation, ImportData, ParsedFile, SymbolData,
    TypeRefData,
};

/// Convert a tree-sitter range to Trace's source range.
pub fn range(node: TsNode<'_>) -> SourceRange {
    let start = node.start_position();
    let end = node.end_position();
    SourceRange::new(
        Position::new(
            start.row as u32 + 1,
            start.column as u32 + 1,
            node.start_byte() as u32,
        ),
        Position::new(
            end.row as u32 + 1,
            end.column as u32 + 1,
            node.end_byte() as u32,
        ),
    )
}

/// Text for a node, tolerating malformed trees.
pub fn text<'a>(source: &'a str, node: TsNode<'_>) -> &'a str {
    source.get(node.byte_range()).unwrap_or_default().trim()
}

pub fn child_field<'a>(node: TsNode<'a>, name: &str) -> Option<TsNode<'a>> {
    node.child_by_field_name(name)
}

pub fn named_children<'a>(node: TsNode<'a>) -> Vec<TsNode<'a>> {
    let mut cursor = node.walk();
    let mut out = Vec::new();
    if cursor.goto_first_child() {
        loop {
            if cursor.node().is_named() {
                out.push(cursor.node());
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    out
}

pub fn children<'a>(node: TsNode<'a>) -> Vec<TsNode<'a>> {
    let mut cursor = node.walk();
    let mut out = Vec::new();
    if cursor.goto_first_child() {
        loop {
            out.push(cursor.node());
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    out
}

pub fn first_child_of_kind<'a>(node: TsNode<'a>, kind: &str) -> Option<TsNode<'a>> {
    named_children(node)
        .into_iter()
        .find(|child| child.kind() == kind)
}

pub fn first_named<'a>(node: TsNode<'a>) -> Option<TsNode<'a>> {
    named_children(node).into_iter().next()
}

/// Find the first descendant (including self) with a given node kind.
pub fn find_descendant<'a>(node: TsNode<'a>, kind: &str) -> Option<TsNode<'a>> {
    if node.kind() == kind {
        return Some(node);
    }
    for child in named_children(node) {
        if let Some(found) = find_descendant(child, kind) {
            return Some(found);
        }
    }
    None
}

/// Visit every node depth-first. The callback returns `false` to stop the walk.
pub fn walk<'a>(node: TsNode<'a>, visit: &mut impl FnMut(TsNode<'a>) -> bool) -> bool {
    if !visit(node) {
        return false;
    }
    for child in named_children(node) {
        if !walk(child, visit) {
            return false;
        }
    }
    true
}

/// Compact one-line signature for a declaration, capped at 300 bytes.
pub fn signature(source: &str, node: TsNode<'_>) -> Option<String> {
    let mut raw = text(source, node);
    if let Some(idx) = raw.find('\n') {
        raw = &raw[..idx];
    }
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let mut out = raw.to_string();
    if out.len() > 300 {
        let mut cut = 300;
        while !out.is_char_boundary(cut) {
            cut -= 1;
        }
        out.truncate(cut);
        out.push_str("...");
    }
    Some(out)
}

pub fn strip_quotes(value: &str) -> String {
    value
        .trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .to_string()
}

pub fn is_upper_camel(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

pub fn last_segment(value: &str) -> &str {
    value.rsplit(['.', ':', '/', '\\']).next().unwrap_or(value)
}

/// Shared accumulator used by all language adapters.
pub(crate) struct SyntaxBuilder<'a> {
    pub(crate) source: &'a str,
    pub(crate) language: Language,
    pub(crate) path: &'a str,
    pub(crate) module_name: Option<String>,
    pub(crate) symbols: Vec<SymbolData>,
    pub(crate) imports: Vec<ImportData>,
    pub(crate) calls: Vec<CallData>,
    pub(crate) type_refs: Vec<TypeRefData>,
    pub(crate) heritage: Vec<HeritageData>,
    pub(crate) locals: Vec<crate::syntax::LocalBinding>,
    stack: Vec<u32>,
    qualifier_prefix: Option<String>,
    ordinals: HashMap<(NodeKind, String), u32>,
    next_local: u32,
}

impl<'a> SyntaxBuilder<'a> {
    pub(crate) fn new(source: &'a str, language: Language, path: &'a str) -> Self {
        Self {
            source,
            language,
            path,
            module_name: None,
            symbols: Vec::new(),
            imports: Vec::new(),
            calls: Vec::new(),
            type_refs: Vec::new(),
            heritage: Vec::new(),
            locals: Vec::new(),
            stack: Vec::new(),
            qualifier_prefix: None,
            ordinals: HashMap::new(),
            next_local: 0,
        }
    }

    pub(crate) fn text(&self, node: TsNode<'_>) -> &'a str {
        text(self.source, node)
    }

    pub(crate) fn caller(&self) -> Option<u32> {
        self.stack.last().copied()
    }

    pub(crate) fn current_symbol(&self) -> Option<&SymbolData> {
        self.stack
            .last()
            .and_then(|id| self.symbols.iter().find(|s| s.local_id == *id))
    }

    pub(crate) fn parent_local(&self) -> Option<u32> {
        self.stack.last().copied()
    }

    /// True when declarations are nested in a type-like scope, including
    /// Swift extensions whose target type is declared in another file.
    pub(crate) fn in_type_scope(&self) -> bool {
        self.qualifier_prefix.is_some()
            || self.current_symbol().is_some_and(|symbol| {
                matches!(
                    symbol.kind,
                    NodeKind::Class
                        | NodeKind::Struct
                        | NodeKind::Enum
                        | NodeKind::Interface
                        | NodeKind::Trait
                )
            })
    }

    pub(crate) fn set_qualifier(&mut self, qualifier: Option<String>) -> Option<String> {
        std::mem::replace(&mut self.qualifier_prefix, qualifier)
    }

    #[allow(dead_code)]
    pub(crate) fn find_symbol_by_id(&self, local_id: u32) -> Option<&SymbolData> {
        self.symbols
            .iter()
            .find(|symbol| symbol.local_id == local_id)
    }

    pub(crate) fn push(&mut self, local_id: u32) {
        self.stack.push(local_id);
    }

    pub(crate) fn pop(&mut self) {
        self.stack.pop();
    }

    /// Add a declaration under the current lexical parent.
    pub(crate) fn add_symbol(
        &mut self,
        kind: NodeKind,
        name: impl Into<String>,
        node: TsNode<'_>,
        visibility: Option<Visibility>,
        exported: bool,
    ) -> u32 {
        let name = name.into();
        let parent = self.stack.last().copied();
        let qualified = match self.current_symbol() {
            Some(parent_symbol) => format!("{}.{}", parent_symbol.qualified_name, name),
            None => match &self.qualifier_prefix {
                Some(prefix) => format!("{prefix}.{name}"),
                None => name.clone(),
            },
        };
        self.add_symbol_qualified(kind, name, qualified, parent, node, visibility, exported)
    }

    /// Add a declaration with an explicit parent and qualified name. Used for
    /// constructs such as Rust `impl` blocks and Swift extensions where the
    /// containing declaration is not the lexical parent.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_symbol_qualified(
        &mut self,
        kind: NodeKind,
        name: impl Into<String>,
        qualified: impl Into<String>,
        parent: Option<u32>,
        node: TsNode<'_>,
        visibility: Option<Visibility>,
        exported: bool,
    ) -> u32 {
        let name = name.into();
        let qualified = qualified.into();
        let key = (kind, qualified.clone());
        let ordinal = *self.ordinals.entry(key).or_insert(0);
        self.ordinals.insert((kind, qualified.clone()), ordinal + 1);
        let local_id = self.next_local;
        self.next_local += 1;
        self.symbols.push(SymbolData {
            local_id,
            kind,
            name,
            qualified_name: qualified,
            parent,
            ordinal,
            range: range(node),
            signature: signature(self.source, node),
            visibility,
            exported,
            decorators: Vec::new(),
            overrides: false,
        });
        local_id
    }

    pub(crate) fn mark_override(&mut self, local_id: u32) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.local_id == local_id)
        {
            symbol.overrides = true;
        }
    }

    pub(crate) fn add_decorators(&mut self, local_id: u32, decorators: Vec<String>) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.local_id == local_id)
        {
            symbol.decorators = decorators;
        }
    }

    pub(crate) fn add_import(&mut self, import: ImportData) {
        self.imports.push(import);
    }

    pub(crate) fn add_call(
        &mut self,
        receiver: Option<String>,
        name: impl Into<String>,
        full: impl Into<String>,
        kind: CallKind,
        node: TsNode<'_>,
    ) {
        self.calls.push(CallData {
            caller: self.caller(),
            receiver,
            name: name.into(),
            full: full.into(),
            kind,
            range: range(node),
        });
    }

    pub(crate) fn add_type_ref(&mut self, name: impl Into<String>, node: TsNode<'_>) {
        let name = name.into();
        if name.is_empty() || is_builtin_type(&name) {
            return;
        }
        self.type_refs.push(TypeRefData {
            from: self.caller(),
            name,
            range: range(node),
        });
    }

    pub(crate) fn add_heritage(
        &mut self,
        from: u32,
        name: impl Into<String>,
        relation: HeritageRelation,
        node: TsNode<'_>,
    ) {
        self.heritage.push(HeritageData {
            from,
            name: name.into(),
            relation,
            range: range(node),
        });
    }

    pub(crate) fn add_local(
        &mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
        node: TsNode<'_>,
    ) {
        let name = name.into();
        let type_name = type_name.into();
        if name.is_empty() || type_name.is_empty() {
            return;
        }
        self.locals.push(crate::syntax::LocalBinding {
            scope: self.caller(),
            name,
            type_name,
            range: range(node),
        });
    }

    pub(crate) fn find_symbol_by_qualified(&self, qualified: &str) -> Option<u32> {
        self.symbols
            .iter()
            .find(|symbol| symbol.qualified_name == qualified)
            .map(|symbol| symbol.local_id)
    }

    #[allow(dead_code)]
    pub(crate) fn find_symbol_by_name(&self, name: &str) -> Option<u32> {
        self.symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .map(|symbol| symbol.local_id)
    }

    pub(crate) fn finish(self) -> ParsedFile {
        ParsedFile {
            path: self.path.to_string(),
            language: self.language,
            source_hash: blake3::hash(self.source.as_bytes()).to_hex().to_string(),
            module_name: self.module_name,
            symbols: self.symbols,
            imports: self.imports,
            calls: self.calls,
            type_refs: self.type_refs,
            heritage: self.heritage,
            locals: self.locals,
            diagnostics: Vec::new(),
            parse_error_count: 0,
        }
    }
}

/// Types that should not be reported as project type dependencies.
pub(crate) fn is_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "void"
            | "any"
            | "unknown"
            | "never"
            | "number"
            | "string"
            | "boolean"
            | "bool"
            | "object"
            | "bigint"
            | "symbol"
            | "null"
            | "undefined"
            | "int"
            | "int8"
            | "int16"
            | "int32"
            | "int64"
            | "uint"
            | "uint8"
            | "uint16"
            | "uint32"
            | "uint64"
            | "float"
            | "float32"
            | "float64"
            | "complex64"
            | "complex128"
            | "byte"
            | "rune"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f16"
            | "f32"
            | "f64"
            | "str"
            | "char"
            | "Self"
            | "self"
            | "this"
            | "Unit"
            | "Double"
            | "Float"
            | "Int"
            | "Long"
            | "Short"
            | "Byte"
            | "Boolean"
            | "String"
            | "Char"
            | "Any"
            | "Nothing"
            | "Void"
            | "Never"
            | "Error"
            | "error"
            | "Type"
            | "Map"
            | "List"
            | "Array"
            | "Set"
            | "Option"
            | "Result"
            | "Vec"
            | "Box"
            | "HashMap"
            | "HashSet"
            | "Dictionary"
            | "Optional"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_quotes() {
        assert_eq!(strip_quotes("\"./math\""), "./math");
        assert_eq!(strip_quotes("'x'"), "x");
    }

    #[test]
    fn last_segments() {
        assert_eq!(last_segment("crate::util::helper"), "helper");
        assert_eq!(last_segment("com.example.Helper"), "Helper");
        assert_eq!(last_segment("module"), "module");
    }

    #[test]
    fn builtin_types_are_filtered() {
        assert!(is_builtin_type("string"));
        assert!(is_builtin_type("Self"));
        assert!(!is_builtin_type("Circle"));
    }
}
