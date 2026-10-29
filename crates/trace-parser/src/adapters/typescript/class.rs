use trace_core::NodeKind;
use tree_sitter::Node as TsNode;

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::items::parse_item;

pub(crate) fn parse_class(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    kind: NodeKind,
) {
    let name = super::helpers::name_of(builder, node).unwrap_or_else(|| "default".to_string());
    let id = builder.add_symbol(
        kind,
        name,
        node,
        super::visibility::visibility(builder, node),
        exported,
    );
    builder.push(id);
    if let Some(heritage) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "class_heritage")
    {
        super::heritage::parse_heritage(builder, id, heritage);
    }
    if let Some(body) = child_field(node, "body") {
        for child in named_children(body) {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}

pub(crate) fn parse_interface(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>, exported: bool) {
    let name = super::helpers::name_of(builder, node).unwrap_or_else(|| "default".to_string());
    let id = builder.add_symbol(NodeKind::Interface, name, node, None, exported);
    builder.push(id);
    if let Some(heritage) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "extends_type_clause" || child.kind() == "class_heritage")
    {
        for child in named_children(heritage) {
            super::types::collect_type_identifiers(builder, child, &mut |builder, name, at| {
                builder.add_heritage(id, name, crate::syntax::HeritageRelation::Inherits, at)
            });
        }
    }
    if let Some(body) = child_field(node, "body") {
        for child in named_children(body) {
            parse_item(builder, child, false);
        }
    }
    builder.pop();
}
