use tree_sitter::Node as TsNode;

use trace_core::{NodeKind, Visibility};

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::super::types::rust_visibility;

pub(crate) fn parse_trait(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
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
            super::parse_item(builder, item, false);
        }
    }
    builder.pop();
}
