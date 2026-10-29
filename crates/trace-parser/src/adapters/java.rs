use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{CallKind, HeritageRelation, ImportData, ImportKind, ParsedFile};
use crate::util::{child_field, first_named, is_upper_camel, named_children, range, SyntaxBuilder};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child, false);
    }
    builder.finish()
}

fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    match node.kind() {
        "import_declaration" => parse_import(builder, node),
        "class_declaration" => parse_class(builder, node, _exported),
        "interface_declaration" => parse_interface(builder, node, _exported),
        "enum_declaration" => parse_enum(builder, node, _exported),
        "method_declaration" => parse_method(builder, node, _exported),
        "constructor_declaration" => parse_constructor(builder, node),
        "field_declaration" => parse_field(builder, node, _exported),
        "call_expression" | "method_invocation" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "block" | "class_body" | "interface_body" | "enum_body" | "program"
        | "compilation_unit" => {
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

fn parse_import(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let text = builder.text(node);
    let name = text
        .trim_start_matches("import")
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_start_matches("static")
        .trim()
        .to_string();
    builder.add_import(ImportData {
        kind: ImportKind::Rust,
        module: name.clone(),
        alias: None,
        items: vec![],
        is_export: false,
        range: range(node),
    });
}

fn parse_class(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Class,
        name,
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
    if let Some(superclass) = child_field(node, "superclass") {
        let heritage_name = builder.text(superclass).to_string();
        builder.add_heritage(
            id,
            heritage_name.clone(),
            HeritageRelation::Inherits,
            superclass,
        );
    }
    if let Some(interfaces) = child_field(node, "interfaces") {
        for child in named_children(interfaces) {
            if child.kind() == "type_list" {
                for t in named_children(child) {
                    let heritage_name = builder.text(t).to_string();
                    builder.add_heritage(id, heritage_name, HeritageRelation::Implements, t);
                }
            } else if child.kind() != "keyword" {
                let heritage_name = builder.text(child).to_string();
                builder.add_heritage(id, heritage_name, HeritageRelation::Implements, child);
            }
        }
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_interface(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Interface,
        name,
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_enum(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Enum,
        name,
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_method(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = modifier_visibility(node);
    let in_class = builder
        .current_symbol()
        .map(|s| matches!(s.kind, NodeKind::Class | NodeKind::Interface))
        .unwrap_or(false);
    let kind = if in_class {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let id = builder.add_symbol(
        kind,
        name.clone(),
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
    if let Some(params) = child_field(node, "parameters") {
        for child in named_children(params) {
            if child.kind() == "formal_parameter" {
                if let Some(t) = child_field(child, "type") {
                    builder.add_type_ref(builder.text(t).to_string(), t);
                }
            }
        }
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_constructor(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let name = builder
        .current_symbol()
        .map(|s| s.name.clone())
        .unwrap_or_default();
    let visibility = modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Constructor,
        name,
        node,
        Some(visibility),
        visibility == Visibility::Public,
    );
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_field(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let visibility = modifier_visibility(node);
    let is_const = has_modifier(node, "final") || has_modifier(node, "static");
    if let Some(declarator) = child_field(node, "declarator") {
        if let Some(name_node) = child_field(declarator, "name") {
            let name = builder.text(name_node).to_string();
            if let Some(t) = child_field(node, "type") {
                builder.add_type_ref(builder.text(t).to_string(), t);
            }
            let kind = if is_const {
                NodeKind::Constant
            } else {
                NodeKind::Property
            };
            builder.add_symbol(
                kind,
                name,
                node,
                Some(visibility),
                visibility == Visibility::Public,
            );
        }
    }
}

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if node.kind() == "method_invocation" {
        if let Some(name_node) = child_field(node, "name") {
            let name = builder.text(name_node).to_string();
            builder.add_call(None, name.clone(), name, CallKind::Method, node);
            return;
        }
    }
    if let Some(func) = child_field(node, "function") {
        if func.kind() == "identifier" {
            let name = builder.text(func).to_string();
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
            return;
        }
        if func.kind() == "field_expression" {
            if let Some(field) = child_field(func, "field") {
                let name = builder.text(field).to_string();
                builder.add_call(None, name.clone(), name, CallKind::Method, node);
                return;
            }
        }
    }
    // fallback
    if let Some(first) = first_named(node) {
        let name = builder.text(first).to_string();
        if is_upper_camel(&name) {
            builder.add_call(None, name.clone(), name, CallKind::Constructor, node);
        } else if !name.is_empty() {
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
        }
    }
}

fn modifier_visibility(node: TsNode<'_>) -> Visibility {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "modifiers" {
            let text = child.to_string();
            if text.contains("public") {
                return Visibility::Public;
            }
            if text.contains("private") {
                return Visibility::Private;
            }
            if text.contains("protected") {
                return Visibility::Protected;
            }
        }
        if child.kind() == "public" {
            return Visibility::Public;
        }
        if child.kind() == "private" {
            return Visibility::Private;
        }
        if child.kind() == "protected" {
            return Visibility::Protected;
        }
    }
    Visibility::Internal
}

fn has_modifier(node: TsNode<'_>, modifier: &str) -> bool {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "modifiers" && child.to_string().contains(modifier) {
            return true;
        }
        if child.kind() == modifier {
            return true;
        }
    }
    false
}
