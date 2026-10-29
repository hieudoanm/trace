use tree_sitter::Node as TsNode;

use crate::syntax::{ImportData, ImportItem, ImportKind};
use crate::util::{child_field, named_children, range, SyntaxBuilder};

pub(crate) fn parse_import_statement(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    for child in named_children(node) {
        match child.kind() {
            "dotted_name" => {
                let module = builder.text(child).to_string();
                builder.add_import(ImportData {
                    module,
                    alias: None,
                    items: Vec::new(),
                    kind: ImportKind::Python,
                    range: range(child),
                    is_export: false,
                });
            }
            "aliased_import" => {
                let module = child_field(child, "name")
                    .map(|n| builder.text(n).to_string())
                    .unwrap_or_default();
                let alias = child_field(child, "alias").map(|a| builder.text(a).to_string());
                builder.add_import(ImportData {
                    module,
                    alias,
                    items: Vec::new(),
                    kind: ImportKind::Python,
                    range: range(child),
                    is_export: false,
                });
            }
            _ => {}
        }
    }
}

pub(crate) fn parse_import_from(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let module = child_field(node, "module_name")
        .map(|m| builder.text(m).to_string())
        .unwrap_or_default();
    let mut items = Vec::new();
    for child in named_children(node) {
        match child.kind() {
            "dotted_name" | "identifier" => {
                if Some(child) == child_field(node, "module_name") {
                    continue;
                }
                items.push(ImportItem {
                    name: builder.text(child).to_string(),
                    alias: None,
                });
            }
            "aliased_import" => {
                let name = child_field(child, "name")
                    .map(|n| builder.text(n).to_string())
                    .unwrap_or_default();
                let alias = child_field(child, "alias").map(|a| builder.text(a).to_string());
                items.push(ImportItem { name, alias });
            }
            "wildcard_import" => items.push(ImportItem {
                name: "*".to_string(),
                alias: None,
            }),
            _ => {}
        }
    }
    builder.add_import(ImportData {
        module,
        alias: None,
        items,
        kind: ImportKind::Python,
        range: range(node),
        is_export: false,
    });
}
