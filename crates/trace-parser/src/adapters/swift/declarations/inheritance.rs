use tree_sitter::Node as TsNode;

use crate::syntax::HeritageRelation;
use crate::util::{child_field, named_children, SyntaxBuilder};

use super::super::types::{collect_type_identifiers, collect_type_refs};

pub(crate) fn parse_inheritance(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    let mut index = 0usize;
    for child in named_children(node) {
        if child.kind() != "inheritance_specifier" {
            continue;
        }
        let relation = if index == 0 {
            HeritageRelation::Inherits
        } else {
            HeritageRelation::Implements
        };
        index += 1;
        if let Some(type_node) = child_field(child, "inherits_from") {
            collect_type_identifiers(builder, type_node, &mut |builder, name, at| {
                builder.add_heritage(from, name, relation, at)
            });
            collect_type_refs(builder, type_node);
        }
    }
}
