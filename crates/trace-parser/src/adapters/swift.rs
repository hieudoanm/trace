use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{CallKind, HeritageRelation, ImportData, ImportKind, ParsedFile};
use crate::util::{
    child_field, find_descendant, first_named, is_builtin_type, is_upper_camel, named_children,
    range, SyntaxBuilder,
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
        "import_declaration" => {
            let module = named_children(node)
                .into_iter()
                .find(|child| child.kind() == "identifier")
                .map(|module| builder.text(module).to_string())
                .unwrap_or_default();
            if !module.is_empty() {
                builder.add_import(ImportData {
                    module,
                    alias: None,
                    items: Vec::new(),
                    kind: ImportKind::Swift,
                    range: range(node),
                    is_export: false,
                });
            }
        }
        "class_declaration"
        | "struct_declaration"
        | "enum_declaration"
        | "protocol_declaration"
        | "extension_declaration"
        | "actor_declaration" => parse_type_declaration(builder, node, exported),
        "function_declaration" | "protocol_function_declaration" => {
            parse_function(builder, node, exported)
        }
        "init_declaration" => parse_initializer(builder, node),
        "property_declaration" => parse_property(builder, node, exported),
        "call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "class_body" | "protocol_body" | "statements" | "source_file" => {
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

fn parse_type_declaration(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let name = child_field(node, "name")
        .map(|name| builder.text(name).to_string())
        .or_else(|| {
            named_children(node)
                .into_iter()
                .find(|child| matches!(child.kind(), "type_identifier" | "simple_identifier"))
                .map(|name| builder.text(name).to_string())
        });
    let Some(name) = name else {
        return;
    };
    let is_extension = node.kind() == "extension_declaration";
    let kind = match node.kind() {
        "struct_declaration" => NodeKind::Struct,
        "enum_declaration" => NodeKind::Enum,
        "protocol_declaration" => NodeKind::Interface,
        _ => NodeKind::Class,
    };
    let modifiers = find_modifiers(builder, node);
    let visibility = swift_visibility(&modifiers);

    if is_extension {
        // Methods declared in an extension belong to the named type. The type
        // itself may live in another file, so a qualifier is used instead of
        // synthesising a parent node that does not exist in this file.
        let previous = builder.set_qualifier(Some(name));
        for child in named_children(node) {
            if child.kind() == "inheritance_specifier" {
                if let Some(type_node) = child_field(child, "inherits_from") {
                    collect_type_refs(builder, type_node);
                }
            }
        }
        if let Some(body) = named_children(node).into_iter().find(|child| {
            matches!(
                child.kind(),
                "class_body" | "protocol_body" | "enum_class_body"
            )
        }) {
            for child in named_children(body) {
                parse_item(builder, child, false);
            }
        }
        builder.set_qualifier(previous);
        return;
    }

    let id = builder.add_symbol(
        kind,
        name.clone(),
        node,
        visibility,
        exported || matches!(visibility, Some(Visibility::Public)),
    );
    let decorators = swift_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    builder.push(id);
    parse_inheritance(builder, id, node);
    if let Some(body) = named_children(node).into_iter().find(|child| {
        matches!(
            child.kind(),
            "class_body" | "protocol_body" | "enum_class_body"
        )
    }) {
        for child in named_children(body) {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

fn parse_inheritance(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    let mut index = 0usize;
    for child in named_children(node) {
        if child.kind() != "inheritance_specifier" {
            continue;
        }
        let relation = if index == 0 {
            HeritageRelation::Inherits
        } else {
            HeritageRelation::Implements
        };
        index += 1;
        if let Some(type_node) = child_field(child, "inherits_from") {
            collect_type_identifiers(builder, type_node, &mut |builder, name, at| {
                builder.add_heritage(from, name, relation, at)
            });
            collect_type_refs(builder, type_node);
        }
    }
}

fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name").or_else(|| {
        named_children(node)
            .into_iter()
            .find(|child| child.kind() == "simple_identifier")
    }) else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let kind = if builder.in_type_scope() {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let modifiers = find_modifiers(builder, node);
    let visibility = swift_visibility(&modifiers);
    let id = builder.add_symbol(
        kind,
        name,
        node,
        visibility,
        exported || matches!(visibility, Some(Visibility::Public)),
    );
    if modifiers.contains("override") {
        builder.mark_override(id);
    }
    let decorators = swift_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    builder.push(id);
    collect_parameter_types(builder, node);
    if let Some(return_type) = return_type_node(node) {
        collect_type_refs(builder, return_type);
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_initializer(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let id = builder.add_symbol(NodeKind::Constructor, "init", node, None, false);
    builder.push(id);
    collect_parameter_types(builder, node);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_property(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let name_node = child_field(node, "name")
        .or_else(|| find_descendant(node, "simple_identifier"))
        .or_else(|| find_descendant(node, "pattern"));
    let Some(name_node) = name_node else {
        return;
    };
    let name = if name_node.kind() == "pattern" {
        find_descendant(name_node, "simple_identifier")
            .map(|name| builder.text(name).to_string())
            .unwrap_or_else(|| builder.text(name_node).to_string())
    } else {
        builder.text(name_node).to_string()
    };
    if name.is_empty() {
        return;
    }
    let modifiers = find_modifiers(builder, node);
    let visibility = swift_visibility(&modifiers);
    let immutable = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "value_binding_pattern")
        .map(|kind| builder.text(kind) == "let")
        .unwrap_or(false);
    let kind = if builder.in_type_scope() {
        NodeKind::Property
    } else if immutable {
        NodeKind::Constant
    } else {
        NodeKind::Variable
    };
    let id = builder.add_symbol(
        kind,
        name.clone(),
        node,
        visibility,
        exported || matches!(visibility, Some(Visibility::Public)),
    );
    if modifiers.contains("override") {
        builder.mark_override(id);
    }
    if modifiers.contains("final") {
        // `final` is retained in the signature; no separate graph flag.
    }
    let type_node = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "type_annotation")
        .and_then(|annotation| first_named(annotation));
    if let Some(type_node) = type_node {
        collect_type_refs(builder, type_node);
        builder.add_local(name.clone(), last_type_name(builder, type_node), node);
    } else if let Some(value) = child_field(node, "value") {
        if let Some(type_name) = infer_call_type(builder, value) {
            builder.add_local(name, type_name, node);
        }
    }
    for child in named_children(node) {
        if matches!(
            child.kind(),
            "call_expression" | "computed_property" | "closure_expression"
        ) {
            parse_item(builder, child, false);
        }
    }
}

fn collect_parameter_types(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    for child in named_children(node) {
        if child.kind() == "parameter_clause" {
            for parameter in named_children(child) {
                collect_one_parameter(builder, parameter);
            }
        } else if child.kind() == "parameter" {
            collect_one_parameter(builder, child);
        }
    }
}

fn collect_one_parameter(builder: &mut SyntaxBuilder<'_>, parameter: TsNode<'_>) {
    let type_node = named_children(parameter)
        .into_iter()
        .find(|nested| nested.kind() == "user_type");
    if let Some(type_node) = type_node {
        collect_type_refs(builder, type_node);
    }
    let name_node = named_children(parameter)
        .into_iter()
        .rfind(|nested| nested.kind() == "simple_identifier");
    if let (Some(name), Some(type_node)) = (name_node, type_node) {
        let name = builder.text(name).to_string();
        let type_name = last_type_name(builder, type_node);
        builder.add_local(name, type_name, parameter);
    }
}

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function").or_else(|| first_named(node)) else {
        return;
    };
    if function.kind() == "navigation_expression" {
        let receiver =
            child_field(function, "target").map(|target| builder.text(target).to_string());
        let name = child_field(function, "suffix")
            .and_then(|suffix| first_named(suffix))
            .map(|name| builder.text(name).to_string())
            .unwrap_or_else(|| builder.text(function).to_string());
        let full = builder.text(function).to_string();
        let kind = if name == "init" {
            CallKind::Constructor
        } else if receiver.as_deref() == Some("super") {
            CallKind::Super
        } else {
            CallKind::Method
        };
        builder.add_call(receiver, name, full, kind, node);
    } else if function.kind() == "simple_identifier" {
        let name = builder.text(function).to_string();
        let kind = if name == "super" {
            CallKind::Super
        } else if is_upper_camel(&name) {
            CallKind::Constructor
        } else {
            CallKind::Function
        };
        builder.add_call(None, name.clone(), name, kind, node);
    }
}

fn infer_call_type(builder: &SyntaxBuilder<'_>, value: TsNode<'_>) -> Option<String> {
    if value.kind() != "call_expression" {
        return None;
    }
    let function = child_field(value, "function").or_else(|| first_named(value))?;
    if function.kind() == "simple_identifier" {
        let name = builder.text(function);
        if is_upper_camel(name) {
            return Some(name.to_string());
        }
    }
    None
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
        let name = builder.text(node).to_string();
        if !is_builtin_type(&name) {
            add(builder, name, node);
        }
        return;
    }
    for child in named_children(node) {
        collect_type_identifiers(builder, child, add);
    }
}

fn return_type_node(node: TsNode<'_>) -> Option<TsNode<'_>> {
    let mut found = None;
    for child in named_children(node) {
        match child.kind() {
            "user_type" => {
                found = Some(child);
                break;
            }
            "parameter_clause" | "parameter" => {}
            _ => {}
        }
    }
    found
}

fn last_type_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    builder
        .text(node)
        .trim_end_matches('?')
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_string()
}

fn find_modifiers(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    named_children(node)
        .into_iter()
        .find(|child| child.kind() == "modifiers")
        .map(|modifiers| builder.text(modifiers).to_string())
        .unwrap_or_default()
}

fn swift_visibility(modifiers: &str) -> Option<Visibility> {
    for word in modifiers.split_whitespace() {
        match word {
            "private" | "fileprivate" => return Some(Visibility::Private),
            "public" | "open" => return Some(Visibility::Public),
            "internal" => return Some(Visibility::Internal),
            "package" => return Some(Visibility::Package),
            _ => {}
        }
    }
    Some(Visibility::Internal)
}

fn swift_annotations(modifiers: &str) -> Vec<String> {
    modifiers
        .split('@')
        .skip(1)
        .map(|annotation| {
            annotation
                .split(['(', ' ', '\n'])
                .next()
                .unwrap_or(annotation)
                .trim()
                .to_string()
        })
        .filter(|annotation| !annotation.is_empty())
        .collect()
}
