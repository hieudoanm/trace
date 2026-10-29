use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::util::{child_field, is_upper_camel, named_children, SyntaxBuilder};

use super::super::calls::collect_signature_types;
use super::super::items::parse_item;
use super::super::types::go_type_name;
use super::super::visibility::go_visibility;

pub(crate) fn parse_method(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let receiver_type = child_field(node, "receiver")
        .and_then(|receiver| {
            named_children(receiver)
                .into_iter()
                .find(|child| child.kind() == "parameter_declaration")
        })
        .and_then(|parameter| child_field(parameter, "type"))
        .map(|type_node| go_type_name(builder, type_node))
        .unwrap_or_else(|| "receiver".to_string());
    let qualified = format!("{receiver_type}.{name}");
    let parent = builder.find_symbol_by_qualified(&receiver_type);
    let exported = is_upper_camel(&name);
    let id = builder.add_symbol_qualified(
        NodeKind::Method,
        name,
        qualified,
        parent,
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
