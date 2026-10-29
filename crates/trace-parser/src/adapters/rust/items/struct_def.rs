use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::super::calls::collect_type_refs;
use super::super::types::rust_visibility;

pub(crate) fn parse_struct(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
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
        collect_type_refs(builder, type_parameters);
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
                collect_type_refs(builder, type_node);
            }
        }
    }
    builder.pop();
}
