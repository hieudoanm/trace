use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::util::{child_field, is_upper_camel, SyntaxBuilder};

use super::is_exported_name;
use super::python_visibility;
use super::types::{collect_type_refs, normalize_python_type};

pub(crate) fn parse_assignment(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(left) = child_field(node, "left") else {
        return;
    };
    if left.kind() != "identifier" {
        return;
    }
    let name = builder.text(left).to_string();
    let scope_kind = builder.current_symbol().map(|symbol| symbol.kind);
    let kind = match scope_kind {
        Some(NodeKind::Class) => NodeKind::Property,
        None if name.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) => NodeKind::Constant,
        _ => NodeKind::Variable,
    };
    let exported = scope_kind.is_none() && is_exported_name(&name);
    let id = builder.add_symbol(kind, name.clone(), node, python_visibility(&name), exported);
    let _ = id;
    let mut inferred = None;
    if let Some(type_node) = child_field(node, "type") {
        collect_type_refs(builder, type_node);
        inferred = Some(normalize_python_type(builder.text(type_node)));
    }
    if inferred.is_none() {
        if let Some(right) = child_field(node, "right") {
            if right.kind() == "call" {
                if let Some(function) = child_field(right, "function") {
                    let callee = builder.text(function);
                    if is_upper_camel(callee.rsplit('.').next().unwrap_or(callee)) {
                        inferred = Some(callee.rsplit('.').next().unwrap_or(callee).to_string());
                    }
                }
            }
        }
    }
    if let Some(type_name) = inferred {
        builder.add_local(name, type_name, node);
    }
}
