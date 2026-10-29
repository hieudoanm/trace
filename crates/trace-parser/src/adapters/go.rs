use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{CallKind, HeritageRelation, ImportData, ImportKind, LocalBinding, ParsedFile};
use crate::util::{
    child_field, first_named, is_builtin_type, is_upper_camel, named_children, range, strip_quotes,
    SyntaxBuilder,
};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    let root = tree.root_node();
    for child in named_children(root) {
        parse_item(&mut builder, child);
    }
    builder.finish()
}

fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    match node.kind() {
        "package_clause" => {
            if let Some(name) = first_named(node) {
                builder.module_name = Some(builder.text(name).to_string());
            }
        }
        "import_declaration" => parse_imports(builder, node),
        "type_declaration" => {
            for spec in named_children(node) {
                if spec.kind() == "type_spec" {
                    parse_type_spec(builder, spec);
                }
            }
        }
        "function_declaration" => parse_function(builder, node),
        "method_declaration" => parse_method(builder, node),
        "call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
        "composite_literal" => {
            parse_composite(builder, node);
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
        "short_var_declaration" | "var_declaration" | "const_declaration" => {
            parse_declaration(builder, node);
        }
        "block" | "statement_list" | "source_file" => {
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
        "expression_statement" => {
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
        _ => {
            for child in named_children(node) {
                parse_item(builder, child);
            }
        }
    }
}

fn parse_imports(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let mut specs = Vec::new();
    collect_specs(node, &mut specs);
    for spec in specs {
        if spec.kind() != "import_spec" {
            continue;
        }
        let module = child_field(spec, "path")
            .map(|path| strip_quotes(builder.text(path)))
            .unwrap_or_default();
        if module.is_empty() {
            continue;
        }
        let alias = child_field(spec, "name").map(|name| builder.text(name).to_string());
        builder.add_import(ImportData {
            module,
            alias,
            items: Vec::new(),
            kind: ImportKind::Go,
            range: range(spec),
            is_export: false,
        });
    }
}

fn collect_specs<'a>(node: TsNode<'a>, out: &mut Vec<TsNode<'a>>) {
    if node.kind() == "import_spec" {
        out.push(node);
        return;
    }
    for child in named_children(node) {
        collect_specs(child, out);
    }
}

fn parse_type_spec(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let Some(type_node) = child_field(node, "type") else {
        return;
    };
    let kind = match type_node.kind() {
        "struct_type" => NodeKind::Struct,
        "interface_type" => NodeKind::Interface,
        _ => NodeKind::Type,
    };
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol(kind, name.clone(), node, go_visibility(exported), exported);
    builder.push(id);
    match type_node.kind() {
        "struct_type" => parse_struct(builder, id, type_node),
        "interface_type" => parse_interface_type(builder, id, type_node),
        _ => collect_go_type_refs(builder, type_node),
    }
    builder.pop();
}

fn parse_struct(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    for child in named_children(node) {
        if child.kind() != "field_declaration_list" {
            continue;
        }
        for field in named_children(child) {
            if field.kind() != "field_declaration" {
                continue;
            }
            let name = child_field(field, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_default();
            let type_node = child_field(field, "type");
            if name.is_empty() {
                // Embedded field: a struct/interface composition.
                if let Some(type_node) = type_node {
                    let embedded = go_type_name(builder, type_node);
                    if !embedded.is_empty() {
                        builder.add_heritage(from, embedded, HeritageRelation::Inherits, field);
                    }
                    collect_go_type_refs(builder, type_node);
                }
                continue;
            }
            let exported = is_upper_camel(&name);
            builder.add_symbol(
                NodeKind::Property,
                name,
                field,
                go_visibility(exported),
                exported,
            );
            if let Some(type_node) = type_node {
                collect_go_type_refs(builder, type_node);
            }
        }
    }
}

fn parse_interface_type(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    for child in named_children(node) {
        match child.kind() {
            "method_elem" => {
                let name = child_field(child, "name")
                    .map(|name| builder.text(name).to_string())
                    .unwrap_or_else(|| "anonymous".to_string());
                let exported = is_upper_camel(&name);
                builder.add_symbol(
                    NodeKind::Method,
                    name,
                    child,
                    go_visibility(exported),
                    exported,
                );
                collect_signature_types(builder, child);
            }
            "type_identifier" | "qualified_type" => {
                let name = go_type_name(builder, child);
                if !name.is_empty() {
                    builder.add_heritage(from, name, HeritageRelation::Inherits, child);
                }
                collect_go_type_refs(builder, child);
            }
            _ => {
                for nested in named_children(child) {
                    if nested.kind() == "type_identifier" || nested.kind() == "qualified_type" {
                        let name = go_type_name(builder, nested);
                        builder.add_heritage(from, name, HeritageRelation::Inherits, nested);
                    }
                }
            }
        }
    }
}

fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol(
        NodeKind::Function,
        name,
        node,
        go_visibility(exported),
        exported,
    );
    builder.push(id);
    collect_signature_types(builder, node);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body);
    }
    builder.pop();
}

fn parse_method(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let receiver_type = child_field(node, "receiver")
        .and_then(|receiver| {
            named_children(receiver)
                .into_iter()
                .find(|child| child.kind() == "parameter_declaration")
        })
        .and_then(|parameter| child_field(parameter, "type"))
        .map(|type_node| go_type_name(builder, type_node))
        .unwrap_or_else(|| "receiver".to_string());
    let qualified = format!("{receiver_type}.{name}");
    let parent = builder.find_symbol_by_qualified(&receiver_type);
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol_qualified(
        NodeKind::Method,
        name,
        qualified,
        parent,
        node,
        go_visibility(exported),
        exported,
    );
    builder.push(id);
    collect_signature_types(builder, node);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body);
    }
    builder.pop();
}

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function") else {
        return;
    };
    match function.kind() {
        "identifier" => {
            let name = builder.text(function).to_string();
            if is_builtin_type(&name) {
                return;
            }
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
        }
        "selector_expression" => {
            let full = builder.text(function).to_string();
            let receiver =
                child_field(function, "operand").map(|operand| builder.text(operand).to_string());
            let name = child_field(function, "field")
                .map(|field| builder.text(field).to_string())
                .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).to_string());
            builder.add_call(receiver, name, full, CallKind::Method, node);
        }
        _ => {
            let full = builder.text(function).to_string();
            let name = full.rsplit('.').next().unwrap_or(&full).trim().to_string();
            builder.add_call(None, name, full, CallKind::Function, node);
        }
    }
}

fn parse_composite(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(type_node) = child_field(node, "type") {
        let name = go_type_name(builder, type_node);
        if !name.is_empty() && !is_builtin_type(&name) {
            builder.add_call(None, name.clone(), name, CallKind::Constructor, node);
        }
        collect_go_type_refs(builder, type_node);
    }
}

fn parse_declaration(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let mut specs = Vec::new();
    collect_specs_named(node, &mut specs);
    for spec in specs {
        let name = child_field(spec, "name")
            .or_else(|| {
                child_field(spec, "left")
                    .and_then(|left| first_named(left))
                    .filter(|name| name.kind() == "identifier")
            })
            .map(|name| builder.text(name).to_string());
        let Some(name) = name else {
            continue;
        };
        if name.contains('.') {
            continue;
        }
        let kind = if spec.kind() == "const_spec" {
            NodeKind::Constant
        } else {
            NodeKind::Variable
        };
        let exported = is_upper_camel(&name);
        builder.add_symbol(
            kind,
            name.clone(),
            spec,
            go_visibility(exported),
            exported && builder.caller().is_none(),
        );
        let type_name = child_field(spec, "type")
            .map(|type_node| go_type_name(builder, type_node))
            .or_else(|| {
                child_field(spec, "right").and_then(|right| {
                    let value = first_named(right)?;
                    if value.kind() == "composite_literal" {
                        child_field(value, "type").map(|type_node| go_type_name(builder, type_node))
                    } else {
                        None
                    }
                })
            });
        if let Some(type_name) = type_name.filter(|name| !name.is_empty()) {
            builder.locals.push(LocalBinding {
                scope: builder.caller(),
                name,
                type_name,
                range: range(spec),
            });
        }
        if let Some(right) = child_field(spec, "right") {
            for child in named_children(right) {
                parse_item(builder, child);
            }
        }
        if let Some(value) = child_field(spec, "value") {
            parse_item(builder, value);
        }
    }
}

fn collect_specs_named<'a>(node: TsNode<'a>, out: &mut Vec<TsNode<'a>>) {
    if matches!(
        node.kind(),
        "var_spec" | "const_spec" | "short_var_declaration"
    ) {
        out.push(node);
        return;
    }
    for child in named_children(node) {
        collect_specs_named(child, out);
    }
}

fn collect_signature_types(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(parameters) = child_field(node, "parameters") {
        for parameter in named_children(parameters) {
            if let Some(type_node) = child_field(parameter, "type") {
                collect_go_type_refs(builder, type_node);
            }
        }
    }
    if let Some(result) = child_field(node, "result") {
        collect_go_type_refs(builder, result);
    }
    if let Some(type_parameters) = child_field(node, "type_parameters") {
        collect_go_type_refs(builder, type_parameters);
    }
}

fn collect_go_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    match node.kind() {
        "type_identifier" => {
            let name = builder.text(node).to_string();
            if !is_builtin_type(&name) {
                builder.add_type_ref(name, node);
            }
        }
        "qualified_type" => {
            let name = child_field(node, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_else(|| builder.text(node).to_string());
            if !is_builtin_type(&name) {
                builder.add_type_ref(name, node);
            }
        }
        _ => {
            for child in named_children(node) {
                collect_go_type_refs(builder, child);
            }
        }
    }
}

fn go_type_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    match node.kind() {
        "pointer_expression" | "parenthesized_type" | "generic_type" => first_named(node)
            .map(|child| go_type_name(builder, child))
            .unwrap_or_default(),
        "qualified_type" => child_field(node, "name")
            .map(|name| builder.text(name).to_string())
            .unwrap_or_else(|| builder.text(node).to_string()),
        "type_identifier" | "identifier" => builder.text(node).to_string(),
        _ => builder.text(node).to_string(),
    }
}

fn go_visibility(exported: bool) -> Option<Visibility> {
    if exported {
        Some(Visibility::Public)
    } else {
        Some(Visibility::Internal)
    }
}
