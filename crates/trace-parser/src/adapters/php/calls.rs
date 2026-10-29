use tree_sitter::Node as TsNode;

use crate::syntax::CallKind;
use crate::util::{child_field, is_upper_camel, SyntaxBuilder};

pub(crate) fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(func) = child_field(node, "function") {
        if func.kind() == "name" || func.kind() == "qualified_name" {
            let name = builder.text(func).to_string();
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
            return;
        }
        if func.kind() == "member_call_expression" {
            if let Some(name_node) = child_field(func, "name") {
                let name = builder.text(name_node).to_string();
                builder.add_call(None, name.clone(), name, CallKind::Method, node);
                return;
            }
        }
        if func.kind() == "scoped_call_expression" {
            if let Some(name_node) = child_field(func, "name") {
                let name = builder.text(name_node).to_string();
                builder.add_call(None, name.clone(), name, CallKind::Method, node);
                return;
            }
        }
    }
    if let Some(first) = node.named_child(0) {
        let name = builder.text(first).to_string();
        if is_upper_camel(&name) {
            builder.add_call(None, name.clone(), name, CallKind::Constructor, node);
        } else if !name.is_empty() {
            builder.add_call(None, name.clone(), name, CallKind::Function, node);
        }
    }
}
