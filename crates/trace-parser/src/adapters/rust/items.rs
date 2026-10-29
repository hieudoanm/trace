use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::syntax::{CallKind, HeritageRelation};
use crate::util::{child_field, named_children, SyntaxBuilder};

use super::calls::{parse_call, parse_let, parse_struct_expression};
use super::function::parse_function;
use super::types::{parse_const, parse_type_item, rust_type_name, rust_visibility};
use super::use_decl::parse_use;

pub(crate) fn parse_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
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
        super::calls::collect_type_refs(builder, type_parameters);
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
                super::calls::collect_type_refs(builder, type_node);
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
        super::calls::collect_type_refs(builder, type_node);
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
            super::function::collect_function_signature(builder, item);
            if let Some(function_body) = child_field(item, "body") {
                parse_item(builder, function_body, false);
            }
            builder.pop();
        }
    }
}
