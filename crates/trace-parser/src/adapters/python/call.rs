use tree_sitter::Node as TsNode;

use crate::syntax::CallKind;
use crate::util::{child_field, is_upper_camel, SyntaxBuilder};

pub(crate) fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function") else {
        return;
    };
    let full = builder.text(function).to_string();
    if function.kind() == "attribute" {
        let receiver = child_field(function, "object").map(|o| builder.text(o).to_string());
        let name = child_field(function, "attribute")
            .map(|a| builder.text(a).to_string())
            .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).to_string());
        let kind = if name == "__init__" {
            CallKind::Constructor
        } else {
            CallKind::Method
        };
        builder.add_call(receiver, name, full, kind, node);
    } else {
        let name = full.rsplit('.').next().unwrap_or(&full).trim().to_string();
        let kind = if is_upper_camel(&name) {
            CallKind::Constructor
        } else if name == "super" {
            CallKind::Super
        } else {
            CallKind::Function
        };
        builder.add_call(None, name, full, kind, node);
    }
}
