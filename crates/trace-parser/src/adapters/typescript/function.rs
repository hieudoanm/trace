use trace_core::NodeKind;
use tree_sitter::Node as TsNode;

use crate::util::{child_field, named_children, SyntaxBuilder};

use super::items::parse_item;
use super::types::collect_type_refs;
use super::visibility::visibility;

pub(crate) fn parse_function(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    kind: NodeKind,
) {
    let name = super::helpers::name_of(builder, node).unwrap_or_else(|| "default".to_string());
    parse_function_named(builder, node, exported, kind, name);
}

pub(crate) fn parse_function_named(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    kind: NodeKind,
    name: String,
) {
    let id = builder.add_symbol(kind, name, node, visibility(builder, node), exported);
    builder.push(id);
    if let Some(parameters) = child_field(node, "parameters") {
        super::types::collect_signature_types(builder, parameters);
        for parameter in named_children(parameters) {
            parse_parameter_defaults(builder, parameter);
        }
    }
    if let Some(return_type) = child_field(node, "return_type") {
        collect_type_refs(builder, return_type);
    }
    if let Some(body) = child_field(node, "body") {
        parse_item(builder, body, false);
    } else if let Some(body) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "statement_block")
    {
        parse_item(builder, body, false);
    }
    builder.pop();
}

fn parse_parameter_defaults(builder: &mut SyntaxBuilder<'_>, parameter: TsNode<'_>) {
    if let Some(value) = child_field(parameter, "value") {
        parse_item(builder, value, false);
    }
    if let Some(pattern) = child_field(parameter, "pattern") {
        parse_parameter_defaults(builder, pattern);
    }
}
