use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{CallKind, ImportData, ImportKind, ParsedFile};
use crate::util::{child_field, is_upper_camel, named_children, range, SyntaxBuilder};

pub fn parse(language: Language, path: &str, source: &str, tree: &Tree) -> ParsedFile {
    let mut builder = SyntaxBuilder::new(source, language, path);
    for child in named_children(tree.root_node()) {
        parse_item(&mut builder, child, false);
    }
    builder.finish()
}

fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    match node.kind() {
        "call" if is_require(node, builder) => parse_require(builder, node),
        "class" => parse_class(builder, node, exported),
        "module" => parse_module(builder, node, exported),
        "method" => parse_method(builder, node, exported),
        "singleton_method" => parse_method(builder, node, exported),
        "call" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "method_call" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "body_statement" | "program" | "do_block" | "block" => {
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

fn is_require(node: TsNode<'_>, builder: &SyntaxBuilder<'_>) -> bool {
    if let Some(method) = child_field(node, "method") {
        return builder.text(method) == "require" || builder.text(method) == "require_relative";
    }
    false
}

fn parse_require(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let text = builder.text(node);
    if let Some(args) = child_field(node, "arguments") {
        if let Some(first) = named_children(args).first().copied() {
            let module = builder
                .text(first)
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            builder.add_import(ImportData {
                kind: ImportKind::Rust,
                module,
                items: vec![],
                alias: None,
                is_export: false,
                range: range(node),
            });
            return;
        }
    }
    builder.add_import(ImportData {
        kind: ImportKind::Rust,
        module: text.to_string(),
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
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_module(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, _exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let id = builder.add_symbol(NodeKind::Module, name, node, Some(Visibility::Public), true);
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
        .map(|s| matches!(s.kind, NodeKind::Class | NodeKind::Module))
        .unwrap_or(false);
    let kind = if in_class {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let id = builder.add_symbol(kind, name, node, Some(Visibility::Public), true);
    builder.push(id);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(method) = child_field(node, "method") {
        let name = builder.text(method).to_string();
        if name == "require" || name == "require_relative" {
            return;
        }
        builder.add_call(None, name.clone(), name, CallKind::Method, node);
        return;
    }
    if let Some(func) = child_field(node, "function") {
        if func.kind() == "identifier" {
            let name = builder.text(func).to_string();
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
            return;
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
