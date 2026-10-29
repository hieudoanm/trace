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
        "namespace_use_declaration" | "use_declaration" => parse_use(builder, node),
        "class_declaration" => parse_class(builder, node, _exported),
        "interface_declaration" => parse_interface(builder, node, _exported),
        "trait_declaration" => parse_trait(builder, node, _exported),
        "method_declaration" => parse_method(builder, node, _exported),
        "function_definition" => parse_function(builder, node, _exported),
        "property_declaration" => parse_property(builder, node, _exported),
        "function_call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "block" | "declaration_list" | "program" | "compound_statement" => {
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
    let text = builder.text(node);
    let name = text
        .trim_start_matches("use")
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
    let id = builder.add_symbol(NodeKind::Class, name, node, Some(Visibility::Public), true);
    builder.push(id);
    if let Some(base) = child_field(node, "base_clause") {
        for child in named_children(base) {
            if child.kind() == "name" || child.kind() == "qualified_name" {
                let heritage_name = builder.text(child).to_string();
                builder.add_heritage(id, heritage_name, HeritageRelation::Inherits, child);
            }
        }
    }
    if let Some(interfaces) = child_field(node, "class_interface_clause") {
        for child in named_children(interfaces) {
            if matches!(child.kind(), "name" | "qualified_name") {
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
    let id = builder.add_symbol(
        NodeKind::Interface,
        name,
        node,
        Some(Visibility::Public),
        true,
    );
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_trait(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let id = builder.add_symbol(NodeKind::Trait, name, node, Some(Visibility::Public), true);
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
    let in_class = builder
        .current_symbol()
        .map(|s| {
            matches!(
                s.kind,
                NodeKind::Class | NodeKind::Trait | NodeKind::Interface
            )
        })
        .unwrap_or(false);
    let kind = if in_class {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let visibility = method_visibility(node);
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

fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let id = builder.add_symbol(
        NodeKind::Function,
        name.clone(),
        node,
        Some(Visibility::Public),
        true,
    );
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_property(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let visibility = method_visibility(node);
    for child in named_children(node) {
        if child.kind() == "property_element" {
            if let Some(name_node) = child_field(child, "name") {
                let name = builder.text(name_node).trim_start_matches('$').to_string();
                let kind = if has_modifier(node, "const") {
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
}

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(func) = child_field(node, "function") {
        if func.kind() == "name" || func.kind() == "qualified_name" {
            let name = builder.text(func).to_string();
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
            return;
        }
        if func.kind() == "member_call_expression" {
            if let Some(name_node) = child_field(func, "name") {
                let name = builder.text(name_node).to_string();
                builder.add_call(None, name.clone(), name, CallKind::Method, node);
                return;
            }
        }
        if func.kind() == "scoped_call_expression" {
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

fn method_visibility(node: TsNode<'_>) -> Visibility {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "visibility_modifier" {
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
        }
    }
    Visibility::Public
}

fn has_modifier(node: TsNode<'_>, modifier: &str) -> bool {
    for child in node.children(&mut node.walk()) {
        if child.kind().contains(modifier) {
            return true;
        }
    }
    false
}
