use tree_sitter::Node as TsNode;

use crate::util::named_children;

pub(crate) fn collect_signature_types(
    builder: &mut crate::util::SyntaxBuilder<'_>,
    node: TsNode<'_>,
) {
    if let Some(type_node) = crate::util::child_field(node, "type") {
        collect_type_refs(builder, type_node);
    }
    for child in named_children(node) {
        match child.kind() {
            "type_annotation" | "type_identifier" | "generic_type" => {
                collect_type_refs(builder, child);
            }
            "required_parameter" | "optional_parameter" | "formal_parameters" => {
                collect_signature_types(builder, child);
            }
            _ => {}
        }
    }
}

pub(crate) fn collect_type_refs(builder: &mut crate::util::SyntaxBuilder<'_>, node: TsNode<'_>) {
    collect_type_identifiers(builder, node, &mut |builder, name, at| {
        builder.add_type_ref(name, at)
    });
}

pub(crate) fn collect_type_identifiers(
    builder: &mut crate::util::SyntaxBuilder<'_>,
    node: TsNode<'_>,
    add: &mut dyn FnMut(&mut crate::util::SyntaxBuilder<'_>, String, TsNode<'_>),
) {
    if node.kind() == "type_identifier" {
        add(builder, builder.text(node).to_string(), node);
        return;
    }
    for child in named_children(node) {
        collect_type_identifiers(builder, child, add);
    }
}
