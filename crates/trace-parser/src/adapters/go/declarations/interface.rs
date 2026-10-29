use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::syntax::HeritageRelation;
use crate::util::{child_field, is_upper_camel, named_children, SyntaxBuilder};

use super::super::calls::collect_signature_types;
use super::super::types::{collect_go_type_refs, go_type_name};
use super::super::visibility::go_visibility;

pub(crate) fn parse_interface_type(builder: &mut SyntaxBuilder<'_>, from: u32, node: TsNode<'_>) {
    for child in named_children(node) {
        match child.kind() {
            "method_elem" => {
                let name = child_field(child, "name")
                    .map(|name| builder.text(name).to_string())
                    .unwrap_or_else(|| "anonymous".to_string());
                let exported = is_upper_camel(&name);
                builder.add_symbol(
                    NodeKind::Method,
                    name,
                    child,
                    go_visibility(exported),
                    exported,
                );
                collect_signature_types(builder, child);
            }
            "type_identifier" | "qualified_type" => {
                let name = go_type_name(builder, child);
                if !name.is_empty() {
                    builder.add_heritage(from, name, HeritageRelation::Inherits, child);
                }
                collect_go_type_refs(builder, child);
            }
            _ => {
                for nested in named_children(child) {
                    if nested.kind() == "type_identifier" || nested.kind() == "qualified_type" {
                        let name = go_type_name(builder, nested);
                        builder.add_heritage(from, name, HeritageRelation::Inherits, nested);
                    }
                }
            }
        }
    }
}
