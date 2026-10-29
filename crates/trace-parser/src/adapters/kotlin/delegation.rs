use tree_sitter::Node as TsNode;

use crate::syntax::{CallKind, HeritageRelation};
use crate::util::{named_children, SyntaxBuilder};

use super::helpers::is_upper_camel;
use super::items::parse_item;
use super::types::{collect_type_identifiers, collect_type_refs};

pub(crate) fn parse_delegation(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
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
            let type_node = super::helpers::first_of_kind(child, "user_type");
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

pub(crate) fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
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
