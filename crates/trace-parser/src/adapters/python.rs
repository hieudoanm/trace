use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{CallKind, HeritageRelation, ImportData, ImportItem, ImportKind, ParsedFile};
use crate::util::{child_field, is_upper_camel, named_children, range, SyntaxBuilder};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child, false, Vec::new());
    }
    builder.finish()
}

fn parse_item(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    decorators: Vec<String>,
) {
    match node.kind() {
        "decorated_definition" => {
            let mut decorators = decorators;
            for child in named_children(node) {
                if child.kind() == "decorator" {
                    decorators.push(builder.text(child).trim_start_matches('@').to_string());
                }
            }
            if let Some(definition) = child_field(node, "definition") {
                parse_item(builder, definition, exported, decorators);
            }
        }
        "class_definition" => parse_class(builder, node, exported, decorators),
        "function_definition" => parse_function(builder, node, exported, decorators),
        "import_statement" => parse_import_statement(builder, node),
        "import_from_statement" => parse_import_from(builder, node),
        "call" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false, Vec::new());
            }
        }
        "assignment" | "annotated_assignment" => {
            parse_assignment(builder, node);
            if let Some(right) = child_field(node, "right") {
                parse_item(builder, right, false, Vec::new());
            }
        }
        "block" | "module" => {
            for child in named_children(node) {
                parse_item(builder, child, false, Vec::new());
            }
        }
        "expression_statement" => {
            for child in named_children(node) {
                parse_item(builder, child, false, Vec::new());
            }
        }
        _ => {
            for child in named_children(node) {
                parse_item(builder, child, false, Vec::new());
            }
        }
    }
}

fn parse_class(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    decorators: Vec<String>,
) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = python_visibility(&name);
    let exported = exported || !name.starts_with('_');
    let id = builder.add_symbol(NodeKind::Class, name, node, visibility, exported);
    builder.add_decorators(id, decorators);
    builder.push(id);
    if let Some(superclasses) = child_field(node, "superclasses") {
        for child in named_children(superclasses) {
            if child.kind() == "keyword_argument" {
                continue;
            }
            let heritage_name = builder.text(child).to_string();
            builder.add_heritage(id, heritage_name.clone(), HeritageRelation::Inherits, child);
            builder.add_type_ref(heritage_name, child);
        }
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false, Vec::new());
    }
    builder.pop();
}

fn parse_function(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    decorators: Vec<String>,
) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let in_class = builder
        .current_symbol()
        .map(|symbol| symbol.kind == NodeKind::Class)
        .unwrap_or(false);
    let kind = if name == "__init__" {
        NodeKind::Constructor
    } else if in_class {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let exported = exported || !name.starts_with('_');
    let id = builder.add_symbol(kind, name.clone(), node, python_visibility(&name), exported);
    builder.add_decorators(id, decorators);
    builder.push(id);
    if let Some(parameters) = child_field(node, "parameters") {
        collect_parameter_types(builder, parameters);
    }
    if let Some(return_type) = child_field(node, "return_type") {
        collect_type_refs(builder, return_type);
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false, Vec::new());
    }
    builder.pop();
}

fn collect_parameter_types(builder: &mut SyntaxBuilder<'_>, parameters: TsNode<'_>) {
    for parameter in named_children(parameters) {
        match parameter.kind() {
            "typed_parameter" | "typed_default_parameter" => {
                if let Some(type_node) = child_field(parameter, "type") {
                    collect_type_refs(builder, type_node);
                }
                let name_node = child_field(parameter, "name").or_else(|| {
                    named_children(parameter)
                        .into_iter()
                        .find(|child| child.kind() == "identifier")
                });
                if let Some(name_node) = name_node {
                    if let Some(type_node) = child_field(parameter, "type") {
                        let type_name = normalize_python_type(builder.text(type_node));
                        let param_name = builder.text(name_node).to_string();
                        builder.add_local(param_name, type_name, parameter);
                    }
                }
            }
            "default_parameter" => {
                if let Some(value) = child_field(parameter, "value") {
                    parse_item(builder, value, false, Vec::new());
                }
            }
            "list_splat_pattern" | "dictionary_splat_pattern" | "identifier" => {}
            _ => {}
        }
    }
}

fn parse_assignment(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(left) = child_field(node, "left") else {
        return;
    };
    if left.kind() != "identifier" {
        return;
    }
    let name = builder.text(left).to_string();
    let scope_kind = builder.current_symbol().map(|symbol| symbol.kind);
    let kind = match scope_kind {
        Some(NodeKind::Class) => NodeKind::Property,
        None if name.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) => NodeKind::Constant,
        _ => NodeKind::Variable,
    };
    let exported = scope_kind.is_none() && !name.starts_with('_');
    let id = builder.add_symbol(kind, name.clone(), node, python_visibility(&name), exported);
    let _ = id;
    let mut inferred = None;
    if let Some(type_node) = child_field(node, "type") {
        collect_type_refs(builder, type_node);
        inferred = Some(normalize_python_type(builder.text(type_node)));
    }
    if inferred.is_none() {
        if let Some(right) = child_field(node, "right") {
            if right.kind() == "call" {
                if let Some(function) = child_field(right, "function") {
                    let callee = builder.text(function);
                    if is_upper_camel(callee.rsplit('.').next().unwrap_or(callee)) {
                        inferred = Some(callee.rsplit('.').next().unwrap_or(callee).to_string());
                    }
                }
            }
        }
    }
    if let Some(type_name) = inferred {
        builder.add_local(name, type_name, node);
    }
}

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function") else {
        return;
    };
    let full = builder.text(function).to_string();
    if function.kind() == "attribute" {
        let receiver = child_field(function, "object").map(|o| builder.text(o).to_string());
        let name = child_field(function, "attribute")
            .map(|a| builder.text(a).to_string())
            .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).to_string());
        let kind = if name == "__init__" {
            CallKind::Constructor
        } else {
            CallKind::Method
        };
        builder.add_call(receiver, name, full, kind, node);
    } else {
        let name = full.rsplit('.').next().unwrap_or(&full).trim().to_string();
        let kind = if is_upper_camel(&name) {
            CallKind::Constructor
        } else if name == "super" {
            CallKind::Super
        } else {
            CallKind::Function
        };
        builder.add_call(None, name, full, kind, node);
    }
}

fn parse_import_statement(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    for child in named_children(node) {
        match child.kind() {
            "dotted_name" => {
                let module = builder.text(child).to_string();
                builder.add_import(ImportData {
                    module,
                    alias: None,
                    items: Vec::new(),
                    kind: ImportKind::Python,
                    range: range(child),
                    is_export: false,
                });
            }
            "aliased_import" => {
                let module = child_field(child, "name")
                    .map(|n| builder.text(n).to_string())
                    .unwrap_or_default();
                let alias = child_field(child, "alias").map(|a| builder.text(a).to_string());
                builder.add_import(ImportData {
                    module,
                    alias,
                    items: Vec::new(),
                    kind: ImportKind::Python,
                    range: range(child),
                    is_export: false,
                });
            }
            _ => {}
        }
    }
}

fn parse_import_from(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let module = child_field(node, "module_name")
        .map(|m| builder.text(m).to_string())
        .unwrap_or_default();
    let mut items = Vec::new();
    for child in named_children(node) {
        match child.kind() {
            "dotted_name" | "identifier" => {
                // The module name itself is a field, not a separate child in
                // most trees; guard anyway.
                if Some(child) == child_field(node, "module_name") {
                    continue;
                }
                items.push(ImportItem {
                    name: builder.text(child).to_string(),
                    alias: None,
                });
            }
            "aliased_import" => {
                let name = child_field(child, "name")
                    .map(|n| builder.text(n).to_string())
                    .unwrap_or_default();
                let alias = child_field(child, "alias").map(|a| builder.text(a).to_string());
                items.push(ImportItem { name, alias });
            }
            "wildcard_import" => items.push(ImportItem {
                name: "*".to_string(),
                alias: None,
            }),
            _ => {}
        }
    }
    builder.add_import(ImportData {
        module,
        alias: None,
        items,
        kind: ImportKind::Python,
        range: range(node),
        is_export: false,
    });
}

fn collect_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if node.kind() == "identifier" {
        builder.add_type_ref(builder.text(node).to_string(), node);
        return;
    }
    for child in named_children(node) {
        collect_type_refs(builder, child);
    }
}

fn normalize_python_type(raw: &str) -> String {
    raw.trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .split(['[', '|', ' '])
        .next()
        .unwrap_or(raw)
        .trim()
        .to_string()
}

fn python_visibility(name: &str) -> Option<Visibility> {
    if name.starts_with("__") && !name.ends_with("__") {
        Some(Visibility::Private)
    } else if name.starts_with('_') && !name.ends_with("__") {
        Some(Visibility::Protected)
    } else {
        Some(Visibility::Public)
    }
}
