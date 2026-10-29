use tree_sitter::Node as TsNode;

use crate::util::{child_field, SyntaxBuilder};

pub(crate) fn parse_call(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(function) = child_field(node, "function") else {
        return;
    };
    let full = builder.text(function).to_string();
    if function.kind() == "member_expression" {
        let receiver =
            child_field(function, "object").map(|object| builder.text(object).to_string());
        let name = child_field(function, "property")
            .map(|property| builder.text(property).to_string())
            .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).to_string());
        let kind = if receiver.as_deref() == Some("super") {
            crate::syntax::CallKind::Super
        } else {
            crate::syntax::CallKind::Method
        };
        builder.add_call(receiver, name, full, kind, node);
    } else {
        let name = function
            .child_by_field_name("property")
            .map(|property| builder.text(property).to_string())
            .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).trim().to_string());
        builder.add_call(None, name, full, crate::syntax::CallKind::Function, node);
    }
}

pub(crate) fn parse_new(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(constructor) = child_field(node, "constructor") {
        let full = builder.text(constructor).to_string();
        let receiver = child_field(constructor, "object").map(|o| builder.text(o).to_string());
        let name = child_field(constructor, "property")
            .map(|p| builder.text(p).to_string())
            .unwrap_or_else(|| full.rsplit('.').next().unwrap_or(&full).trim().to_string());
        builder.add_call(
            receiver,
            name,
            full,
            crate::syntax::CallKind::Constructor,
            node,
        );
    }
}
