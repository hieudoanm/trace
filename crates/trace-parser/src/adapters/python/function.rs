use tree_sitter::Node as TsNode;

use crate::util::{child_field, named_children, SyntaxBuilder};

use trace_core::NodeKind;

use super::is_exported_name;
use super::python_visibility;

pub(crate) fn parse_function(
    builder: &mut SyntaxBuilder<'_>,
    node: TsNode<'_>,
    exported: bool,
    decorators: Vec<String>,
) {
    let Some(name_node) = child_field(node, "name") else {
        return;
    };
    let name = builder.text(name_node).to_string();
    let in_class = builder
        .current_symbol()
        .map(|symbol| symbol.kind == NodeKind::Class)
        .unwrap_or(false);
    let kind = if name == "__init__" {
        NodeKind::Constructor
    } else if in_class {
        NodeKind::Method
    } else {
        NodeKind::Function
    };
    let exported = exported || is_exported_name(&name);
    let id = builder.add_symbol(kind, name.clone(), node, python_visibility(&name), exported);
    builder.add_decorators(id, decorators);
    builder.push(id);
    if let Some(parameters) = child_field(node, "parameters") {
        collect_parameter_types(builder, parameters);
    }
    if let Some(return_type) = child_field(node, "return_type") {
        super::types::collect_type_refs(builder, return_type);
    }
    if let Some(body) = child_field(node, "body") {
        super::parse_item(builder, body, false, Vec::new());
    }
    builder.pop();
}

fn collect_parameter_types(builder: &mut SyntaxBuilder<'_>, parameters: TsNode<'_>) {
    for parameter in named_children(parameters) {
        match parameter.kind() {
            "typed_parameter" | "typed_default_parameter" => {
                if let Some(type_node) = child_field(parameter, "type") {
                    super::types::collect_type_refs(builder, type_node);
                }
                let name_node = child_field(parameter, "name").or_else(|| {
                    named_children(parameter)
                        .into_iter()
                        .find(|child| child.kind() == "identifier")
                });
                if let Some(name_node) = name_node {
                    if let Some(type_node) = child_field(parameter, "type") {
                        let type_name =
                            super::types::normalize_python_type(builder.text(type_node));
                        let param_name = builder.text(name_node).to_string();
                        builder.add_local(param_name, type_name, parameter);
                    }
                }
            }
            "default_parameter" => {
                if let Some(value) = child_field(parameter, "value") {
                    super::parse_item(builder, value, false, Vec::new());
                }
            }
            "list_splat_pattern" | "dictionary_splat_pattern" | "identifier" => {}
            _ => {}
        }
    }
}
