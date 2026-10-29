use tree_sitter::{Node as TsNode, Tree};

use trace_core::{Language, NodeKind, Visibility};

use crate::syntax::{CallKind, HeritageRelation, ImportData, ImportItem, ImportKind, ParsedFile};
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
        "package_header" => {
            if let Some(identifier) = named_children(node)
                .into_iter()
                .find(|child| child.kind() == "identifier")
            {
                builder.module_name = Some(builder.text(identifier).to_string());
            }
        }
        "import_list" => {
            for child in named_children(node) {
                if child.kind() == "import_header" {
                    parse_import(builder, child);
                }
            }
        }
        "class_declaration" => parse_class(builder, node, exported),
        "object_declaration" | "companion_object" => parse_object(builder, node, exported),
        "function_declaration" => parse_function(builder, node, exported),
        "property_declaration" => parse_property(builder, node, exported),
        "secondary_constructor" => parse_secondary_constructor(builder, node),
        "type_alias" => {
            if let Some(name_node) = first_of_kind(node, "type_identifier") {
                let name = builder.text(name_node).to_string();
                builder.add_symbol(NodeKind::Type, name, node, None, exported);
                for child in named_children(node) {
                    if child.kind() == "user_type" {
                        collect_type_refs(builder, child);
                    }
                }
            }
        }
        "call_expression" => {
            parse_call(builder, node);
            for child in named_children(node) {
                parse_item(builder, child, false);
            }
        }
        "class_body" | "enum_class_body" | "source_file" => {
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
    let mut module = String::new();
    let mut alias = None;
    let mut wildcard = false;
    for child in named_children(node) {
        match child.kind() {
            "identifier" => module = builder.text(child).to_string(),
            "import_alias" => {
                alias = named_children(child)
                    .first()
                    .map(|name| builder.text(*name).to_string());
            }
            "wildcard_import" => wildcard = true,
            _ => {}
        }
    }
    if module.is_empty() {
        return;
    }
    builder.add_import(ImportData {
        module,
        alias,
        items: if wildcard {
            vec![ImportItem {
                name: "*".to_string(),
                alias: None,
            }]
        } else {
            Vec::new()
        },
        kind: ImportKind::Kotlin,
        range: range(node),
        is_export: false,
    });
}

fn parse_class(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = first_of_kind(node, "type_identifier") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let is_enum = named_children(node)
        .iter()
        .any(|child| child.kind() == "enum_class_body");
    let kind = if is_enum {
        NodeKind::Enum
    } else {
        NodeKind::Class
    };
    let modifiers = find_modifiers(builder, node);
    let visibility = kotlin_visibility(&modifiers);
    let id = builder.add_symbol(
        kind,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    let decorators = kotlin_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    builder.push(id);

    // Primary constructor parameters become properties/parameters and the
    // class gains an explicit constructor edge target.
    if let Some(primary) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "primary_constructor")
    {
        let constructor_id = builder.add_symbol(
            NodeKind::Constructor,
            "constructor",
            primary,
            kotlin_visibility(&find_modifiers(builder, primary)),
            false,
        );
        let _ = constructor_id;
        for parameter in named_children(primary) {
            if parameter.kind() != "class_parameter" {
                continue;
            }
            let declaration_kind = named_children(parameter)
                .into_iter()
                .find(|child| child.kind() == "binding_pattern_kind")
                .map(|kind| builder.text(kind).to_string())
                .unwrap_or_default();
            let name_node = named_children(parameter)
                .into_iter()
                .find(|child| child.kind() == "simple_identifier");
            let type_node = named_children(parameter)
                .into_iter()
                .find(|child| child.kind() == "user_type");
            if let Some(name_node) = name_node {
                let parameter_name = builder.text(name_node).to_string();
                let symbol_kind = if declaration_kind == "val" || declaration_kind == "var" {
                    NodeKind::Property
                } else {
                    NodeKind::Parameter
                };
                builder.add_symbol(
                    symbol_kind,
                    parameter_name,
                    parameter,
                    Some(Visibility::Public),
                    false,
                );
            }
            if let Some(type_node) = type_node {
                collect_type_refs(builder, type_node);
            }
        }
    }

    for child in named_children(node) {
        if child.kind() == "delegation_specifier" {
            parse_delegation(builder, id, child);
        }
    }

    if let Some(body) = named_children(node)
        .into_iter()
        .find(|child| matches!(child.kind(), "class_body" | "enum_class_body"))
    {
        for child in named_children(body) {
            if child.kind() == "enum_entry" {
                if let Some(entry_name) = first_named(child) {
                    builder.add_symbol(
                        NodeKind::Constant,
                        builder.text(entry_name).to_string(),
                        child,
                        None,
                        true,
                    );
                }
                continue;
            }
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

fn parse_object(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let name = first_of_kind(node, "type_identifier")
        .map(|name| builder.text(name).to_string())
        .unwrap_or_else(|| {
            if node.kind() == "companion_object" {
                "Companion".to_string()
            } else {
                "object".to_string()
            }
        });
    let modifiers = find_modifiers(builder, node);
    let visibility = kotlin_visibility(&modifiers);
    let id = builder.add_symbol(
        NodeKind::Class,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    builder.push(id);
    if let Some(body) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "class_body")
    {
        for child in named_children(body) {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "simple_identifier")
    else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let modifiers = find_modifiers(builder, node);
    let visibility = kotlin_visibility(&modifiers);
    let in_class = builder
        .current_symbol()
        .map(|symbol| {
            matches!(
                symbol.kind,
                NodeKind::Class | NodeKind::Interface | NodeKind::Enum | NodeKind::Trait
            )
        })
        .unwrap_or(false);
    let kind = if in_class {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let receiver = child_field(node, "receiver")
        .map(|receiver| last_type_name(builder, receiver))
        .filter(|name| !name.is_empty());
    let id = if let Some(receiver) = receiver {
        let qualified = match builder.current_symbol() {
            Some(parent) => format!("{}.{}.{}", parent.qualified_name, receiver, name),
            None => format!("{receiver}.{name}"),
        };
        let parent = builder.parent_local();
        builder.add_symbol_qualified(
            kind,
            name,
            qualified,
            parent,
            node,
            visibility,
            exported || visibility == Some(Visibility::Public),
        )
    } else {
        builder.add_symbol(
            kind,
            name,
            node,
            visibility,
            exported || visibility == Some(Visibility::Public),
        )
    };
    if modifiers.contains("override") {
        builder.mark_override(id);
    }
    let decorators = kotlin_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    builder.push(id);
    collect_function_types(builder, node);
    for child in named_children(node) {
        if child.kind() == "function_body" {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

fn parse_property(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = named_children(node).into_iter().find_map(|child| {
        if child.kind() == "variable_declaration" {
            first_of_kind(child, "simple_identifier")
        } else {
            None
        }
    }) else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let modifiers = find_modifiers(builder, node);
    let visibility = kotlin_visibility(&modifiers);
    let immutable = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "binding_pattern_kind")
        .map(|kind| builder.text(kind) == "val")
        .unwrap_or(false);
    let in_class = builder
        .current_symbol()
        .map(|symbol| {
            matches!(
                symbol.kind,
                NodeKind::Class | NodeKind::Interface | NodeKind::Enum
            )
        })
        .unwrap_or(false);
    let kind = if in_class {
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
        exported || visibility == Some(Visibility::Public),
    );
    if modifiers.contains("override") {
        builder.mark_override(id);
    }
    let decorators = kotlin_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    let type_node = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "user_type");
    if let Some(type_node) = type_node {
        collect_type_refs(builder, type_node);
        builder.add_local(name, last_type_name(builder, type_node), node);
    } else if let Some(call) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "call_expression")
    {
        if let Some(function) = first_named(call) {
            let callee = builder.text(function);
            if is_upper_camel(callee.rsplit('.').next().unwrap_or(callee)) {
                builder.add_local(name, callee.to_string(), node);
            }
        }
    }
    for child in named_children(node) {
        if child.kind() == "call_expression" {
            parse_item(builder, child, false);
        }
    }
}

fn parse_secondary_constructor(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let id = builder.add_symbol(NodeKind::Constructor, "constructor", node, None, false);
    builder.push(id);
    collect_function_types(builder, node);
    for child in named_children(node) {
        if matches!(
            child.kind(),
            "constructor_delegation_call" | "statements" | "block"
        ) {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

fn parse_delegation(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    let has_invocation = named_children(node)
        .iter()
        .any(|child| child.kind() == "constructor_invocation");
    let relation = if has_invocation {
        HeritageRelation::Inherits
    } else {
        HeritageRelation::Implements
    };
    for child in named_children(node) {
        if child.kind() == "user_type" {
            collect_type_identifiers(builder, child, &mut |builder, name, at| {
                builder.add_heritage(from, name, relation, at)
            });
            collect_type_refs(builder, child);
        } else if child.kind() == "constructor_invocation" {
            let type_node = first_of_kind(child, "user_type");
            if let Some(type_node) = type_node {
                collect_type_identifiers(builder, type_node, &mut |builder, name, at| {
                    builder.add_heritage(from, name, relation, at)
                });
                collect_type_refs(builder, type_node);
            }
            for argument in named_children(child) {
                if argument.kind() == "value_arguments" {
                    for value in named_children(argument) {
                        parse_item(builder, value, false);
                    }
                }
            }
        }
    }
}

fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let mut children = named_children(node).into_iter();
    let Some(function) = children.next() else {
        return;
    };
    if function.kind() == "navigation_expression" {
        let parts = named_children(function);
        let receiver = parts
            .first()
            .map(|target| builder.text(*target).to_string());
        let name = parts
            .iter()
            .rev()
            .find_map(|part| {
                if part.kind() == "navigation_suffix" {
                    named_children(*part)
                        .first()
                        .map(|name| builder.text(*name).to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| builder.text(function).to_string());
        let full = builder.text(function).to_string();
        let kind = if receiver.as_deref() == Some("super") {
            CallKind::Super
        } else {
            CallKind::Method
        };
        builder.add_call(receiver, name, full, kind, node);
    } else if function.kind() == "simple_identifier" {
        let name = builder.text(function).to_string();
        let kind = if is_upper_camel(&name) {
            CallKind::Constructor
        } else if name == "super" {
            CallKind::Super
        } else {
            CallKind::Function
        };
        builder.add_call(None, name.clone(), name, kind, node);
    }
}

fn collect_function_types(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    for child in named_children(node) {
        match child.kind() {
            "function_value_parameters" => {
                for parameter in named_children(child) {
                    if let Some(type_node) = named_children(parameter)
                        .into_iter()
                        .find(|nested| nested.kind() == "user_type")
                    {
                        collect_type_refs(builder, type_node);
                    }
                    if parameter.kind() == "parameter" {
                        if let (Some(name), Some(type_node)) = (
                            named_children(parameter)
                                .into_iter()
                                .find(|nested| nested.kind() == "simple_identifier"),
                            named_children(parameter)
                                .into_iter()
                                .find(|nested| nested.kind() == "user_type"),
                        ) {
                            let name = builder.text(name).to_string();
                            let type_name = last_type_name(builder, type_node);
                            builder.add_local(name, type_name, parameter);
                        }
                    }
                }
            }
            "user_type" => collect_type_refs(builder, child),
            "receiver_type" => collect_type_refs(builder, child),
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

fn last_type_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    let text = builder.text(node);
    text.rsplit('.')
        .next()
        .unwrap_or(text)
        .trim_end_matches('?')
        .to_string()
}

fn first_of_kind<'a>(node: TsNode<'a>, kind: &str) -> Option<TsNode<'a>> {
    named_children(node)
        .into_iter()
        .find(|child| child.kind() == kind)
}

fn find_modifiers(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    named_children(node)
        .into_iter()
        .find(|child| child.kind() == "modifiers")
        .map(|modifiers| builder.text(modifiers).to_string())
        .unwrap_or_default()
}

fn kotlin_visibility(modifiers: &str) -> Option<Visibility> {
    for word in modifiers.split_whitespace() {
        match word {
            "private" => return Some(Visibility::Private),
            "protected" => return Some(Visibility::Protected),
            "internal" => return Some(Visibility::Internal),
            "public" => return Some(Visibility::Public),
            _ => {}
        }
    }
    Some(Visibility::Public)
}

fn kotlin_annotations(modifiers: &str) -> Vec<String> {
    modifiers
        .split('@')
        .skip(1)
        .map(|annotation| {
            annotation
                .split_whitespace()
                .next()
                .unwrap_or(annotation)
                .trim()
                .to_string()
        })
        .filter(|annotation| !annotation.is_empty())
        .collect()
}
