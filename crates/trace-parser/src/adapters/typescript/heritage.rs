use tree_sitter::Node as TsNode;

use crate::syntax::HeritageRelation;
use crate::util::{named_children, SyntaxBuilder};

use super::types::collect_type_identifiers;

pub(crate) fn parse_heritage(builder: &mut SyntaxBuilder<'_>, from: u32, heritage: TsNode<'_>) {
    for clause in named_children(heritage) {
        let relation = if clause.kind().contains("implements") {
            HeritageRelation::Implements
        } else {
            HeritageRelation::Inherits
        };
        if clause.kind() == "type_identifier" {
            builder.add_heritage(from, builder.text(clause).to_string(), relation, clause);
        } else {
            collect_type_identifiers(builder, clause, &mut |builder, name, at| {
                builder.add_heritage(from, name, relation, at)
            });
        }
    }
}
