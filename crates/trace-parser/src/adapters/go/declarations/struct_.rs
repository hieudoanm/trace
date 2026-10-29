use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use super::super::types::{collect_go_type_refs, go_type_name};
use super::super::visibility::go_visibility;
use crate::syntax::HeritageRelation;
use crate::util::{child_field, is_upper_camel, named_children, SyntaxBuilder};

pub(crate) fn parse_struct(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    for child in named_children(node) {
        if child.kind() != "field_declaration_list" {
            continue;
        }
        for field in named_children(child) {
            if field.kind() != "field_declaration" {
                continue;
            }
            let name = child_field(field, "name")
                .map(|name| builder.text(name).to_string())
                .unwrap_or_default();
            let type_node = child_field(field, "type");
            if name.is_empty() {
                if let Some(type_node) = type_node {
                    let embedded = go_type_name(builder, type_node);
                    if !embedded.is_empty() {
                        builder.add_heritage(from, embedded, HeritageRelation::Inherits, field);
                    }
                    collect_go_type_refs(builder, type_node);
                }
                continue;
            }
            let exported = is_upper_camel(&name);
            builder.add_symbol(
                NodeKind::Property,
                name,
                field,
                go_visibility(exported),
                exported,
            );
            if let Some(type_node) = type_node {
                collect_go_type_refs(builder, type_node);
            }
        }
    }
}
