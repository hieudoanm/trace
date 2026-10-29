use tree_sitter::Node as TsNode;

use crate::util::{is_builtin_type, named_children, SyntaxBuilder};

pub(crate) fn collect_function_types(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    for child in named_children(node) {
        match child.kind() {
            "function_value_parameters" => {
                for parameter in named_children(child) {
                    if let Some(type_node) = named_children(parameter)
                        .into_iter()
                        .find(|nested| nested.kind() == "user_type")
                    {
                        collect_type_refs(builder, type_node);
                    }
                    if parameter.kind() == "parameter" {
                        if let (Some(name), Some(type_node)) = (
                            named_children(parameter)
                                .into_iter()
                                .find(|nested| nested.kind() == "simple_identifier"),
                            named_children(parameter)
                                .into_iter()
                                .find(|nested| nested.kind() == "user_type"),
                        ) {
                            let name = builder.text(name).to_string();
                            let type_name = crate::adapters::kotlin::helpers::last_type_name(
                                builder, type_node,
                            );
                            builder.add_local(name, type_name, parameter);
                        }
                    }
                }
            }
            "user_type" => collect_type_refs(builder, child),
            "receiver_type" => collect_type_refs(builder, child),
            _ => {}
        }
    }
}

pub(crate) fn collect_type_refs(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    collect_type_identifiers(builder, node, &mut |builder, name, at| {
        builder.add_type_ref(name, at)
    });
}

pub(crate) fn collect_type_identifiers(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    add: &mut dyn FnMut(&mut SyntaxBuilder<'_>, String, TsNode<'_>),
) {
    if node.kind() == "type_identifier" {
        let name = builder.text(node).to_string();
        if !is_builtin_type(&name) {
            add(builder, name, node);
        }
        return;
    }
    for child in named_children(node) {
        collect_type_identifiers(builder, child, add);
    }
}
