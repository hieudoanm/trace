use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{CallKind, HeritageRelation, ImportData, ImportKind, ParsedFile};
use crate::util::{
    child_field, first_named, is_builtin_type, is_upper_camel, named_children, range, SyntaxBuilder,
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
        "use_declaration" => parse_use(builder, node),
        "mod_item" => parse_module(builder, node, exported),
        "struct_item" | "union_item" => parse_struct(builder, node, exported),
        "enum_item" => parse_enum(builder, node, exported),
        "trait_item" => parse_trait(builder, node, exported),
        "impl_item" => parse_impl(builder, node),
        "function_item" => parse_function(builder, node, exported, NodeKind::Function),
        "function_signature_item" => parse_function(builder, node, exported, NodeKind::Method),
        "type_item" => parse_type_item(builder, node, exported),
        "const_item" | "static_item" => parse_const(builder, node, exported),
        "call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "macro_invocation" => {
            if let Some(macro_node) = child_field(node, "macro") {
                let name = builder.text(macro_node).to_string();
                builder.add_call(None, name.clone(), name, CallKind::Macro, node);
            }
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "struct_expression" => {
            parse_struct_expression(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "let_declaration" => parse_let(builder, node),
        "block" | "declaration_list" | "source_file" => {
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "expression_statement" => {
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

fn parse_use(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(argument) = child_field(node, "argument") else {
        return;
    };
    let mut bindings = Vec::new();
    flatten_use(builder, argument, "", &mut bindings);
    for (path, alias) in bindings {
        if path.is_empty() {
            continue;
        }
        builder.add_import(ImportData {
            module: path,
            alias,
            items: Vec::new(),
            kind: ImportKind::Rust,
            range: range(node),
            is_export: false,
        });
    }
}

fn flatten_use(
    builder: &SyntaxBuilder<'_>,
    node: TsNode<'_>,
    prefix: &str,
    out: &mut Vec<(String, Option<String>)>,
) {
    let join = |part: &str| {
        if prefix.is_empty() {
            part.to_string()
        } else if part.is_empty() {
            prefix.to_string()
        } else {
            format!("{prefix}::{part}")
        }
    };
    match node.kind() {
        "scoped_use_list" => {
            let base = child_field(node, "path")
                .map(|path| builder.text(path).to_string())
                .unwrap_or_default();
            let base = join(&base);
            if let Some(list) = child_field(node, "list") {
                flatten_use(builder, list, &base, out);
            }
        }
        "use_list" => {
            for child in named_children(node) {
                flatten_use(builder, child, prefix, out);
            }
        }
        "use_as_clause" => {
            let path = child_field(node, "path")
                .map(|path| join(builder.text(path)))
                .unwrap_or_default();
            let alias = child_field(node, "alias").map(|alias| builder.text(alias).to_string());
            out.push((path, alias));
        }
        "identifier" | "scoped_identifier" | "crate" | "self" | "super" => {
            out.push((join(builder.text(node)), None));
        }
        "use_wildcard" => {
            let inner = first_named(node)
                .map(|name| builder.text(name).to_string())
                .unwrap_or_default();
            if inner.ends_with('*') || inner.is_empty() {
                out.push((join(&format!("{inner}*")), None));
            } else {
                out.push((join(&format!("{inner}::*")), None));
            }
        }
        _ => {
            let text = builder.text(node);
            if !text.is_empty() {
                out.push((join(text), None));
            }
        }
    }
}

fn parse_module(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    let id = builder.add_symbol(
        NodeKind::Module,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    if let Some(body) = child_field(node, "body") {
        builder.push(id);
        for child in named_children(body) {
            let child_exported = rust_visibility(builder, child) == Some(Visibility::Public);
            parse_item(builder, child, child_exported);
        }
        builder.pop();
    }
}

fn parse_struct(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    let id = builder.add_symbol(
        NodeKind::Struct,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    builder.push(id);
    if let Some(type_parameters) = child_field(node, "type_parameters") {
        collect_type_refs(builder, type_parameters);
    }
    if let Some(body) = child_field(node, "body") {
        for field in named_children(body) {
            if field.kind() != "field_declaration" {
                continue;
            }
            let field_name = child_field(field, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_else(|| "field".to_string());
            builder.add_symbol(
                NodeKind::Property,
                field_name,
                field,
                rust_visibility(builder, field),
                false,
            );
            if let Some(type_node) = child_field(field, "type") {
                collect_type_refs(builder, type_node);
            }
        }
    }
    builder.pop();
}

fn parse_enum(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    let id = builder.add_symbol(
        NodeKind::Enum,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        for variant in named_children(body) {
            if variant.kind() != "enum_variant" {
                continue;
            }
            if let Some(variant_name) = child_field(variant, "name") {
                builder.add_symbol(
                    NodeKind::Constant,
                    builder.text(variant_name).to_string(),
                    variant,
                    None,
                    true,
                );
            }
        }
    }
    builder.pop();
}

fn parse_trait(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    let id = builder.add_symbol(
        NodeKind::Trait,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        for item in named_children(body) {
            parse_item(builder, item, false);
        }
    }
    builder.pop();
}

fn parse_impl(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let type_node = child_field(node, "type");
    let type_name = type_node
        .map(|type_node| rust_type_name(builder, type_node))
        .unwrap_or_default();
    if type_name.is_empty() {
        return;
    }
    let type_local = builder.find_symbol_by_qualified(&type_name);
    if let Some(trait_node) = child_field(node, "trait") {
        let trait_name = rust_type_name(builder, trait_node);
        if let Some(type_local) = type_local {
            builder.add_heritage(
                type_local,
                trait_name,
                HeritageRelation::Implements,
                trait_node,
            );
        }
    }
    if let Some(type_node) = type_node {
        collect_type_refs(builder, type_node);
    }
    if let Some(body) = child_field(node, "body") {
        for item in named_children(body) {
            if !matches!(item.kind(), "function_item" | "function_signature_item") {
                parse_item(builder, item, false);
                continue;
            }
            let Some(name_node) = child_field(item, "name") else {
                continue;
            };
            let name = builder.text(name_node).to_string();
            let visibility = rust_visibility(builder, item);
            let qualified = format!("{type_name}.{name}");
            let id = builder.add_symbol_qualified(
                NodeKind::Method,
                name,
                qualified,
                type_local,
                item,
                visibility,
                visibility == Some(Visibility::Public),
            );
            builder.push(id);
            collect_function_signature(builder, item);
            if let Some(function_body) = child_field(item, "body") {
                parse_item(builder, function_body, false);
            }
            builder.pop();
        }
    }
}

fn parse_function(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    kind: NodeKind,
) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    let id = builder.add_symbol(
        kind,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    builder.push(id);
    collect_function_signature(builder, node);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn collect_function_signature(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(parameters) = child_field(node, "parameters") {
        for parameter in named_children(parameters) {
            if let Some(type_node) = child_field(parameter, "type") {
                collect_type_refs(builder, type_node);
            }
        }
    }
    if let Some(return_type) = child_field(node, "return_type") {
        collect_type_refs(builder, return_type);
    }
    if let Some(type_parameters) = child_field(node, "type_parameters") {
        for parameter in named_children(type_parameters) {
            if let Some(name_node) = child_field(parameter, "name") {
                let type_name = builder.text(name_node).to_string();
                builder.add_type_ref(type_name, parameter);
            }
        }
    }
}

fn parse_type_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    builder.add_symbol(
        NodeKind::Type,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    if let Some(type_node) = child_field(node, "type") {
        collect_type_refs(builder, type_node);
    }
}

fn parse_const(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    builder.add_symbol(
        NodeKind::Constant,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    if let Some(type_node) = child_field(node, "type") {
        collect_type_refs(builder, type_node);
    }
    if let Some(value) = child_field(node, "value") {
        parse_item(builder, value, false);
    }
}

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function") else {
        return;
    };
    match function.kind() {
        "identifier" => {
            let name = builder.text(function).to_string();
            if !is_builtin_type(&name) {
                builder.add_call(None, name.clone(), name, CallKind::Function, node);
            }
        }
        "scoped_identifier" => {
            let full = builder.text(function).to_string();
            let path = child_field(function, "path")
                .map(|path| builder.text(path).to_string())
                .unwrap_or_default();
            let name = child_field(function, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_else(|| full.rsplit("::").next().unwrap_or(&full).to_string());
            let kind = if is_upper_camel(path.rsplit("::").next().unwrap_or(&path)) {
                CallKind::Constructor
            } else {
                CallKind::Function
            };
            builder.add_call(Some(path), name, full, kind, node);
        }
        "field_expression" => {
            let full = builder.text(function).to_string();
            let receiver =
                child_field(function, "value").map(|value| builder.text(value).to_string());
            let name = child_field(function, "field")
                .map(|field| builder.text(field).to_string())
                .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).to_string());
            builder.add_call(receiver, name, full, CallKind::Method, node);
        }
        "generic_function" => {
            if let Some(inner) = child_field(function, "function") {
                let full = builder.text(inner).to_string();
                let name = full.rsplit("::").next().unwrap_or(&full).trim().to_string();
                builder.add_call(None, name, full, CallKind::Function, node);
            }
        }
        _ => {
            let full = builder.text(function).to_string();
            let name = full.rsplit("::").next().unwrap_or(&full).trim().to_string();
            if !name.is_empty() {
                builder.add_call(None, name, full, CallKind::Function, node);
            }
        }
    }
}

fn parse_struct_expression(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(name_node) = child_field(node, "name") {
        let name = rust_type_name(builder, name_node);
        if !name.is_empty() {
            builder.add_call(None, name.clone(), name, CallKind::Constructor, node);
        }
        collect_type_refs(builder, name_node);
    }
}

fn parse_let(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(pattern) = child_field(node, "pattern") else {
        if let Some(value) = child_field(node, "value") {
            parse_item(builder, value, false);
        }
        return;
    };
    let value = child_field(node, "value");
    let type_node = child_field(node, "type");
    if pattern.kind() == "identifier" {
        let name = builder.text(pattern).to_string();
        builder.add_symbol(NodeKind::Variable, name.clone(), node, None, false);
        let mut inferred = type_node.map(|type_node| rust_type_name(builder, type_node));
        if inferred.as_deref().unwrap_or("").is_empty() {
            inferred = value.and_then(|value| infer_expression_type(builder, value));
        }
        if let Some(type_name) = inferred.filter(|name| !name.is_empty()) {
            builder.locals.push(crate::syntax::LocalBinding {
                scope: builder.caller(),
                name,
                type_name,
                range: range(node),
            });
        }
    }
    if let Some(type_node) = type_node {
        collect_type_refs(builder, type_node);
    }
    if let Some(value) = value {
        parse_item(builder, value, false);
    }
}

fn infer_expression_type(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> Option<String> {
    match node.kind() {
        "struct_expression" => child_field(node, "name").map(|name| rust_type_name(builder, name)),
        "call_expression" => {
            let function = child_field(node, "function")?;
            if function.kind() == "scoped_identifier" {
                let path = child_field(function, "path")?;
                let candidate = rust_type_name(builder, path);
                let last = candidate.rsplit("::").next().unwrap_or(&candidate);
                if is_upper_camel(last) {
                    return Some(candidate);
                }
            }
            None
        }
        "reference_expression" | "parenthesized_expression" | "await_expression" => {
            first_named(node).and_then(|inner| infer_expression_type(builder, inner))
        }
        _ => None,
    }
}

fn collect_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    match node.kind() {
        "type_identifier" | "identifier" => {
            let name = builder.text(node).to_string();
            if !is_builtin_type(&name) && is_upper_camel(&name) {
                builder.add_type_ref(name, node);
            }
        }
        "scoped_type_identifier" => {
            let text = builder.text(node).to_string();
            let last = text.rsplit("::").next().unwrap_or(&text).to_string();
            if !is_builtin_type(&last) {
                builder.add_type_ref(last, node);
            }
        }
        _ => {
            for child in named_children(node) {
                collect_type_refs(builder, child);
            }
        }
    }
}

fn rust_type_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    match node.kind() {
        "generic_type" | "reference_type" | "pointer_type" | "parenthesized_type" => {
            child_field(node, "type")
                .map(|inner| rust_type_name(builder, inner))
                .or_else(|| first_named(node).map(|inner| rust_type_name(builder, inner)))
                .unwrap_or_else(|| builder.text(node).to_string())
        }
        "scoped_type_identifier" | "type_identifier" | "identifier" => {
            builder.text(node).to_string()
        }
        _ => builder.text(node).to_string(),
    }
}

fn rust_visibility(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> Option<Visibility> {
    for child in named_children(node) {
        if child.kind() == "visibility_modifier" {
            let text = builder.text(child);
            if text.contains("crate") || text.contains("super") {
                return Some(Visibility::Internal);
            }
            return Some(Visibility::Public);
        }
    }
    None
}
