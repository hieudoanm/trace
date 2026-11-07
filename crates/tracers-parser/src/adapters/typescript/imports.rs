use tree_sitter::Node as TsNode;

use crate::syntax::{ImportData, ImportItem, ImportKind};
use crate::util::{child_field, named_children, range, strip_quotes, SyntaxBuilder};

use super::items::parse_item;

pub(crate) fn parse_import(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let module = child_field(node, "source")
        .map(|source| strip_quotes(builder.text(source)))
        .unwrap_or_default();
    let mut items = Vec::new();
    let mut alias = None;
    if let Some(clause) = named_children(node)
        .into_iter()
        .find(|child| child.kind() == "import_clause")
    {
        for part in named_children(clause) {
            match part.kind() {
                "identifier" => items.push(ImportItem {
                    name: "default".to_string(),
                    alias: Some(builder.text(part).to_string()),
                }),
                "namespace_import" => {
                    alias = named_children(part)
                        .first()
                        .map(|name| builder.text(*name).to_string());
                }
                "named_imports" => {
                    for specifier in named_children(part) {
                        if specifier.kind() != "import_specifier" {
                            continue;
                        }
                        let Some(name_node) = child_field(specifier, "name") else {
                            continue;
                        };
                        let name = builder.text(name_node).to_string();
                        let alias = child_field(specifier, "alias")
                            .map(|alias| builder.text(alias).to_string());
                        items.push(ImportItem { name, alias });
                    }
                }
                _ => {}
            }
        }
    }
    builder.add_import(ImportData {
        module,
        alias,
        items,
        kind: ImportKind::Es,
        range: range(node),
        is_export: false,
    });
}

pub(crate) fn parse_export(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    if let Some(declaration) = child_field(node, "declaration") {
        parse_item(builder, declaration, true);
        return;
    }
    let module = child_field(node, "source")
        .map(|source| strip_quotes(builder.text(source)))
        .unwrap_or_default();
    let mut items = Vec::new();
    for child in named_children(node) {
        if child.kind() == "export_clause" {
            for specifier in named_children(child) {
                if specifier.kind() != "export_specifier" {
                    continue;
                }
                let Some(name_node) = child_field(specifier, "name") else {
                    continue;
                };
                let name = builder.text(name_node).to_string();
                let alias =
                    child_field(specifier, "alias").map(|alias| builder.text(alias).to_string());
                items.push(ImportItem { name, alias });
            }
        } else if child.kind() == "namespace_export" {
            items.push(ImportItem {
                name: "*".to_string(),
                alias: named_children(child)
                    .first()
                    .map(|name| builder.text(*name).to_string()),
            });
        }
    }
    if !items.is_empty() {
        builder.add_import(ImportData {
            module,
            alias: None,
            items,
            kind: ImportKind::Es,
            range: range(node),
            is_export: true,
        });
    }
}
