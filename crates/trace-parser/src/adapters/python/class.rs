use tree_sitter::Node as TsNode;

use trace_core::NodeKind;

use crate::syntax::HeritageRelation;
use crate::util::{child_field, named_children, SyntaxBuilder};

use super::is_exported_name;
use super::python_visibility;

pub(crate) fn parse_class(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    decorators: Vec<String>,
) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let visibility = python_visibility(&name);
    let exported = exported || is_exported_name(&name);
    let id = builder.add_symbol(NodeKind::Class, name, node, visibility, exported);
    builder.add_decorators(id, decorators);
    builder.push(id);
    if let Some(superclasses) = child_field(node, "superclasses") {
        for child in named_children(superclasses) {
            if child.kind() == "keyword_argument" {
                continue;
            }
            let heritage_name = builder.text(child).to_string();
            builder.add_heritage(id, heritage_name.clone(), HeritageRelation::Inherits, child);
            builder.add_type_ref(heritage_name, child);
        }
    }
    if let Some(body) = child_field(node, "body") {
        super::parse_item(builder, body, false, Vec::new());
    }
    builder.pop();
}
