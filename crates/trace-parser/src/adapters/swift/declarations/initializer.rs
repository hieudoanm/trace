use trace_core::NodeKind;
use tree_sitter::Node as TsNode;

use crate::util::{child_field, SyntaxBuilder};

use super::super::calls::collect_parameter_types;
use super::super::items::parse_item;

pub(crate) fn parse_initializer(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let id = builder.add_symbol(NodeKind::Constructor, "init", node, None, false);
    builder.push(id);
    collect_parameter_types(builder, node);
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}
