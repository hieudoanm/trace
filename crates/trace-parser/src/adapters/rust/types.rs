use tree_sitter::Node as TsNode;

use trace_core::Visibility;

use crate::util::{child_field, first_named, named_children, SyntaxBuilder};

pub(crate) fn parse_type_item(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    builder.add_symbol(
        trace_core::NodeKind::Type,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    if let Some(type_node) = child_field(node, "type") {
        super::calls::collect_type_refs(builder, type_node);
    }
}

pub(crate) fn parse_const(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = rust_visibility(builder, node);
    builder.add_symbol(
        trace_core::NodeKind::Constant,
        name,
        node,
        visibility,
        exported || visibility == Some(Visibility::Public),
    );
    if let Some(type_node) = child_field(node, "type") {
        super::calls::collect_type_refs(builder, type_node);
    }
    if let Some(value) = child_field(node, "value") {
        super::items::parse_item(builder, value, false);
    }
}

pub(crate) fn rust_type_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
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

pub(crate) fn rust_visibility(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> Option<Visibility> {
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
