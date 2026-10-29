use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::super::types::rust_visibility;

pub(crate) fn parse_module(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
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
            super::parse_item(builder, child, child_exported);
        }
        builder.pop();
    }
}
