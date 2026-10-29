use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::util::{child_field, is_upper_camel, SyntaxBuilder};

use super::super::calls::collect_signature_types;
use super::super::items::parse_item;
use super::super::visibility::go_visibility;

pub(crate) fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol(
        NodeKind::Function,
        name,
        node,
        go_visibility(exported),
        exported,
    );
    builder.push(id);
    collect_signature_types(builder, node);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body);
    }
    builder.pop();
}
