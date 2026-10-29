use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{CallKind, HeritageRelation, ImportData, ImportItem, ImportKind, ParsedFile};
use crate::util::{
    child_field, children, first_child_of_kind, is_upper_camel, named_children, range,
    strip_quotes, SyntaxBuilder,
};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child, false);
    }
    builder.finish()
}

fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    match node.kind() {
        "export_statement" => parse_export(builder, node),
        "import_statement" => parse_import(builder, node),
        "class_declaration" | "abstract_class_declaration" => {
            parse_class(builder, node, exported, NodeKind::Class);
        }
        "interface_declaration" => parse_interface(builder, node, exported),
        "function_declaration" | "generator_function_declaration" => {
            parse_function(builder, node, exported, NodeKind::Function);
        }
        "method_definition" | "abstract_method_signature" => {
            let name = name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            let kind = if name == "constructor" {
                NodeKind::Constructor
            } else {
                NodeKind::Method
            };
            parse_function_named(builder, node, exported, kind, name);
        }
        "method_signature" => {
            let name = name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            let id = builder.add_symbol(
                NodeKind::Method,
                name,
                node,
                visibility(builder, node),
                exported,
            );
            collect_signature_types(builder, node);
            let _ = id;
        }
        "property_signature" | "public_field_definition" | "field_definition" => {
            let name = name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            builder.add_symbol(
                NodeKind::Property,
                name,
                node,
                visibility(builder, node),
                false,
            );
            collect_signature_types(builder, node);
            if let Some(value) = child_field(node, "value") {
                parse_item(builder, value, false);
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            for child in named_children(node) {
                if child.kind() == "variable_declarator" {
                    parse_variable(builder, child, exported);
                }
            }
        }
        "enum_declaration" => {
            let name = name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            let id = builder.add_symbol(
                NodeKind::Enum,
                name,
                node,
                visibility(builder, node),
                exported,
            );
            let _ = id;
        }
        "type_alias_declaration" => {
            let name = name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            builder.add_symbol(
                NodeKind::Type,
                name,
                node,
                visibility(builder, node),
                exported,
            );
            if let Some(value) = child_field(node, "value") {
                collect_type_refs(builder, value);
            }
        }
        "internal_module" | "module" => {
            let name = name_of(builder, node).unwrap_or_else(|| "anonymous".to_string());
            let id = builder.add_symbol(
                NodeKind::Namespace,
                name,
                node,
                visibility(builder, node),
                exported,
            );
            builder.push(id);
            if let Some(body) = child_field(node, "body") {
                for child in named_children(body) {
                    parse_item(builder, child, false);
                }
            }
            builder.pop();
        }
        "call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "new_expression" => {
            parse_new(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "class_body" | "interface_body" | "statement_block" | "program" => {
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        _ => {
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
    }
}

fn name_of(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> Option<String> {
    child_field(node, "name")
        .map(|name| builder.text(name).to_string())
        .filter(|name| !name.is_empty())
}

fn parse_class(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool, kind: NodeKind) {
    let name = name_of(builder, node).unwrap_or_else(|| "default".to_string());
    let id = builder.add_symbol(kind, name, node, visibility(builder, node), exported);
    builder.push(id);
    if let Some(heritage) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "class_heritage")
    {
        parse_heritage(builder, id, heritage);
    }
    if let Some(body) = child_field(node, "body") {
        for child in named_children(body) {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

fn parse_interface(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let name = name_of(builder, node).unwrap_or_else(|| "default".to_string());
    let id = builder.add_symbol(NodeKind::Interface, name, node, None, exported);
    builder.push(id);
    if let Some(heritage) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "extends_type_clause" || child.kind() == "class_heritage")
    {
        for child in named_children(heritage) {
            collect_type_identifiers(builder, child, &mut |builder, name, at| {
                builder.add_heritage(id, name, HeritageRelation::Inherits, at)
            });
        }
    }
    if let Some(body) = child_field(node, "body") {
        for child in named_children(body) {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

fn parse_function(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    kind: NodeKind,
) {
    let name = name_of(builder, node).unwrap_or_else(|| "default".to_string());
    parse_function_named(builder, node, exported, kind, name);
}

fn parse_function_named(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    kind: NodeKind,
    name: String,
) {
    let id = builder.add_symbol(kind, name, node, visibility(builder, node), exported);
    builder.push(id);
    if let Some(parameters) = child_field(node, "parameters") {
        collect_signature_types(builder, parameters);
        for parameter in named_children(parameters) {
            parse_parameter_defaults(builder, parameter);
        }
    }
    if let Some(return_type) = child_field(node, "return_type") {
        collect_type_refs(builder, return_type);
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    } else if let Some(body) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "statement_block")
    {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_parameter_defaults(builder: &mut SyntaxBuilder<'_>, parameter: TsNode<'_>) {
    if let Some(value) = child_field(parameter, "value") {
        parse_item(builder, value, false);
    }
    if let Some(pattern) = child_field(parameter, "pattern") {
        parse_parameter_defaults(builder, pattern);
    }
}

fn parse_variable(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    if name_node.kind() != "identifier" {
        // Destructuring patterns and computed names do not map cleanly to a
        // single symbol; calls in their initialisers are still analysed.
        if let Some(value) = child_field(node, "value") {
            parse_item(builder, value, false);
        }
        return;
    }
    let name = builder.text(name_node).to_string();
    let value = child_field(node, "value");
    let is_function = value
        .map(|value| matches!(value.kind(), "arrow_function" | "function_expression"))
        .unwrap_or(false);
    let is_const = node
        .parent()
        .map(|parent| builder.text(parent).trim_start().starts_with("const "))
        .unwrap_or(false);
    let kind = if is_function {
        NodeKind::Function
    } else if is_const {
        NodeKind::Constant
    } else {
        NodeKind::Variable
    };
    let id = builder.add_symbol(kind, name.clone(), node, None, exported);
    if let Some(type_node) = child_field(node, "type") {
        collect_type_refs(builder, type_node);
        let type_name = normalize_type_name(builder.text(type_node));
        builder.add_local(name.clone(), type_name, node);
    }
    if let Some(value) = value {
        if value.kind() == "new_expression" {
            if let Some(constructor) = child_field(value, "constructor") {
                let type_name = normalize_type_name(builder.text(constructor));
                builder.add_local(name.clone(), type_name, node);
            }
        } else if value.kind() == "call_expression" {
            if let Some(function) = child_field(value, "function") {
                let callee = builder.text(function);
                if is_upper_camel(callee) {
                    builder.add_local(name.clone(), callee.to_string(), node);
                }
            }
        }
        if is_function {
            builder.push(id);
            parse_item(builder, value, false);
            builder.pop();
        } else {
            parse_item(builder, value, false);
        }
    }
}

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function") else {
        return;
    };
    let full = builder.text(function).to_string();
    if function.kind() == "member_expression" {
        let receiver =
            child_field(function, "object").map(|object| builder.text(object).to_string());
        let name = child_field(function, "property")
            .map(|property| builder.text(property).to_string())
            .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).to_string());
        let kind = if receiver.as_deref() == Some("super") {
            CallKind::Super
        } else {
            CallKind::Method
        };
        builder.add_call(receiver, name, full, kind, node);
    } else {
        let name = function
            .child_by_field_name("property")
            .map(|property| builder.text(property).to_string())
            .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).trim().to_string());
        builder.add_call(None, name, full, CallKind::Function, node);
    }
}

fn parse_new(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(constructor) = child_field(node, "constructor") {
        let full = builder.text(constructor).to_string();
        let receiver = child_field(constructor, "object").map(|o| builder.text(o).to_string());
        let name = child_field(constructor, "property")
            .map(|p| builder.text(p).to_string())
            .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).trim().to_string());
        builder.add_call(receiver, name, full, CallKind::Constructor, node);
    }
}

fn parse_heritage(builder: &mut SyntaxBuilder<'_>, from: u32, heritage: TsNode<'_>) {
    for clause in named_children(heritage) {
        let relation = if clause.kind().contains("implements") {
            HeritageRelation::Implements
        } else {
            HeritageRelation::Inherits
        };
        if clause.kind() == "type_identifier" {
            builder.add_heritage(from, builder.text(clause).to_string(), relation, clause);
        } else {
            collect_type_identifiers(builder, clause, &mut |builder, name, at| {
                builder.add_heritage(from, name, relation, at)
            });
        }
    }
}

fn parse_import(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let module = child_field(node, "source")
        .map(|source| strip_quotes(builder.text(source)))
        .unwrap_or_default();
    let mut items = Vec::new();
    let mut alias = None;
    if let Some(clause) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "import_clause")
    {
        for part in named_children(clause) {
            match part.kind() {
                "identifier" => items.push(ImportItem {
                    name: "default".to_string(),
                    alias: Some(builder.text(part).to_string()),
                }),
                "namespace_import" => {
                    alias = named_children(part)
                        .first()
                        .map(|name| builder.text(*name).to_string());
                }
                "named_imports" => {
                    for specifier in named_children(part) {
                        if specifier.kind() != "import_specifier" {
                            continue;
                        }
                        let Some(name_node) = child_field(specifier, "name") else {
                            continue;
                        };
                        let name = builder.text(name_node).to_string();
                        let alias = child_field(specifier, "alias")
                            .map(|alias| builder.text(alias).to_string());
                        items.push(ImportItem { name, alias });
                    }
                }
                _ => {}
            }
        }
    }
    builder.add_import(ImportData {
        module,
        alias,
        items,
        kind: ImportKind::Es,
        range: range(node),
        is_export: false,
    });
}

fn parse_export(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(declaration) = child_field(node, "declaration") {
        parse_item(builder, declaration, true);
        return;
    }
    let module = child_field(node, "source")
        .map(|source| strip_quotes(builder.text(source)))
        .unwrap_or_default();
    let mut items = Vec::new();
    for child in named_children(node) {
        if child.kind() == "export_clause" {
            for specifier in named_children(child) {
                if specifier.kind() != "export_specifier" {
                    continue;
                }
                let Some(name_node) = child_field(specifier, "name") else {
                    continue;
                };
                let name = builder.text(name_node).to_string();
                let alias =
                    child_field(specifier, "alias").map(|alias| builder.text(alias).to_string());
                items.push(ImportItem { name, alias });
            }
        } else if child.kind() == "namespace_export" {
            items.push(ImportItem {
                name: "*".to_string(),
                alias: named_children(child)
                    .first()
                    .map(|name| builder.text(*name).to_string()),
            });
        }
    }
    if !items.is_empty() {
        builder.add_import(ImportData {
            module,
            alias: None,
            items,
            kind: ImportKind::Es,
            range: range(node),
            is_export: true,
        });
    }
}

fn collect_signature_types(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(type_node) = child_field(node, "type") {
        collect_type_refs(builder, type_node);
    }
    for child in named_children(node) {
        match child.kind() {
            "type_annotation" | "type_identifier" | "generic_type" => {
                collect_type_refs(builder, child);
            }
            "required_parameter" | "optional_parameter" | "formal_parameters" => {
                collect_signature_types(builder, child);
            }
            _ => {}
        }
    }
}

fn collect_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    collect_type_identifiers(builder, node, &mut |builder, name, at| {
        builder.add_type_ref(name, at)
    });
}

fn collect_type_identifiers(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    add: &mut dyn FnMut(&mut SyntaxBuilder<'_>, String, TsNode<'_>),
) {
    if node.kind() == "type_identifier" {
        add(builder, builder.text(node).to_string(), node);
        return;
    }
    for child in named_children(node) {
        collect_type_identifiers(builder, child, add);
    }
}

fn visibility(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> Option<Visibility> {
    for child in children(node) {
        if child.kind() == "accessibility_modifier" {
            return Some(Visibility::parse(builder.text(child)));
        }
    }
    None
}

fn normalize_type_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let trimmed = trimmed
        .split(['<', '?', '|', '&', '[', ' '])
        .next()
        .unwrap_or(trimmed);
    trimmed.trim().to_string()
}

#[allow(dead_code)]
fn is_default_import(node: TsNode<'_>) -> bool {
    first_child_of_kind(node, "identifier").is_some()
}
