use tree_sitter::Node as TsNode;

use crate::syntax::CallKind;
use crate::util::{child_field, first_named, is_upper_camel, SyntaxBuilder};

pub(crate) fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if node.kind() == "method_invocation" {
        if let Some(name_node) = child_field(node, "name") {
            let name = builder.text(name_node).to_string();
            builder.add_call(None, name.clone(), name, CallKind::Method, node);
            return;
        }
    }
    if let Some(func) = child_field(node, "function") {
        if func.kind() == "identifier" {
            let name = builder.text(func).to_string();
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
            return;
        }
        if func.kind() == "field_expression" {
            if let Some(field) = child_field(func, "field") {
                let name = builder.text(field).to_string();
                builder.add_call(None, name.clone(), name, CallKind::Method, node);
                return;
            }
        }
    }
    // fallback
    if let Some(first) = first_named(node) {
        let name = builder.text(first).to_string();
        if is_upper_camel(&name) {
            builder.add_call(None, name.clone(), name, CallKind::Constructor, node);
        } else if !name.is_empty() {
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
        }
    }
}
