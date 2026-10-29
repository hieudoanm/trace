use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{CallKind, HeritageRelation, ImportData, ImportKind, ParsedFile};
use crate::util::{child_field, is_upper_camel, named_children, range, SyntaxBuilder};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child, false);
    }
    builder.finish()
}

fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    match node.kind() {
        "using_directive" => parse_using(builder, node),
        "class_declaration" => parse_class(builder, node, _exported),
        "interface_declaration" => parse_interface(builder, node, _exported),
        "struct_declaration" => parse_struct(builder, node, _exported),
        "enum_declaration" => parse_enum(builder, node, _exported),
        "method_declaration" => parse_method(builder, node, _exported),
        "constructor_declaration" => parse_constructor(builder, node),
        "field_declaration" | "property_declaration" => parse_member(builder, node, _exported),
        "invocation_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "block" | "class_body" | "interface_body" | "declaration_list" | "compilation_unit" => {
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

fn parse_using(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let text = builder.text(node);
    let name = text
        .trim_start_matches("using")
        .trim()
        .trim_end_matches(';')
        .trim()
        .to_string();
    builder.add_import(ImportData {
        kind: ImportKind::Rust,
        module: name,
        items: vec![],
        alias: None,
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
    if let Some(base) = child_field(node, "bases") {
        for child in named_children(base) {
            if child.kind() == "base_list" {
                continue;
            }
            let heritage_name = builder.text(child).to_string();
            if is_upper_camel(&heritage_name) {
                // Heuristic: if interface naming not guaranteed; treat as implements/inherits
                builder.add_heritage(id, heritage_name, HeritageRelation::Inherits, child);
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

fn parse_struct(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = modifier_visibility(node);
    let id = builder.add_symbol(
        NodeKind::Struct,
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
    let in_type = builder
        .current_symbol()
        .map(|s| {
            matches!(
                s.kind,
                NodeKind::Class | NodeKind::Interface | NodeKind::Struct
            )
        })
        .unwrap_or(false);
    let kind = if in_type {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let id = builder.add_symbol(
        kind,
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

fn parse_member(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let visibility = modifier_visibility(node);
    let is_const = has_modifier(node, "const") || has_modifier(node, "static");
    if let Some(decl) = child_field(node, "declaration") {
        if let Some(name_node) = child_field(decl, "name") {
            let name = builder.text(name_node).to_string();
            if let Some(t) = child_field(decl, "type") {
                builder.add_type_ref(builder.text(t).to_string(), t);
            } else if let Some(t) = child_field(node, "type") {
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
            return;
        }
    }
    // fallback
    if let Some(name_node) = child_field(node, "name") {
        let name = builder.text(name_node).to_string();
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

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(func) = child_field(node, "function") {
        if func.kind() == "identifier" {
            let name = builder.text(func).to_string();
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
            return;
        }
        if func.kind() == "member_access_expression" {
            if let Some(name_node) = child_field(func, "name") {
                let name = builder.text(name_node).to_string();
                builder.add_call(None, name.clone(), name, CallKind::Method, node);
                return;
            }
        }
    }
    if let Some(first) = node.named_child(0) {
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
        if child.kind() == "modifier" {
            let t = child.to_string();
            if t.contains("public") {
                return Visibility::Public;
            }
            if t.contains("private") {
                return Visibility::Private;
            }
            if t.contains("protected") {
                return Visibility::Protected;
            }
            if t.contains("internal") {
                return Visibility::Internal;
            }
        }
    }
    Visibility::Internal
}

fn has_modifier(node: TsNode<'_>, modifier: &str) -> bool {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "modifier" && child.to_string().contains(modifier) {
            return true;
        }
    }
    false
}
