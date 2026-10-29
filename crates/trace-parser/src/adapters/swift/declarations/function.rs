use trace_core::{NodeKind, Visibility};
use tree_sitter::Node as TsNode;

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::super::calls::collect_parameter_types;
use super::super::items::parse_item;
use super::super::types::{collect_type_refs, return_type_node};
use super::super::visibility::{swift_annotations, swift_visibility};

pub(crate) fn parse_function(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let Some(name_node) = child_field(node, "name").or_else(|| {
        named_children(node)
            .into_iter()
            .find(|child| child.kind() == "simple_identifier")
    }) else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let kind = if builder.in_type_scope() {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let modifiers = super::super::helpers::find_modifiers(builder, node);
    let visibility = swift_visibility(&modifiers);
    let id = builder.add_symbol(
        kind,
        name,
        node,
        visibility,
        exported || matches!(visibility, Some(Visibility::Public)),
    );
    if modifiers.contains("override") {
        builder.mark_override(id);
    }
    let decorators = swift_annotations(&modifiers);
    if !decorators.is_empty() {
        builder.add_decorators(id, decorators);
    }
    builder.push(id);
    collect_parameter_types(builder, node);
    if let Some(return_type) = return_type_node(node) {
        collect_type_refs(builder, return_type);
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    }
    builder.pop();
}
