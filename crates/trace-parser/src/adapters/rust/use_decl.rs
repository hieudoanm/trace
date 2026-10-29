use tree_sitter::Node as TsNode;

use crate::syntax::{ImportData, ImportKind};
use crate::util::{child_field, first_named, named_children, range, SyntaxBuilder};

pub(crate) fn parse_use(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let Some(argument) = child_field(node, "argument") else {
        return;
    };
    let mut bindings = Vec::new();
    flatten_use(builder, argument, "", &mut bindings);
    for (path, alias) in bindings {
        if path.is_empty() {
            continue;
        }
        builder.add_import(ImportData {
            module: path,
            alias,
            items: Vec::new(),
            kind: ImportKind::Rust,
            range: range(node),
            is_export: false,
        });
    }
}

pub(crate) fn flatten_use(
    builder: &SyntaxBuilder<'_>,
    node: TsNode<'_>,
    prefix: &str,
    out: &mut Vec<(String, Option<String>)>,
) {
    let join = |part: &str| {
        if prefix.is_empty() {
            part.to_string()
        } else if part.is_empty() {
            prefix.to_string()
        } else {
            format!("{prefix}::{part}")
        }
    };
    match node.kind() {
        "scoped_use_list" => {
            let base = child_field(node, "path")
                .map(|path| builder.text(path).to_string())
                .unwrap_or_default();
            let base = join(&base);
            if let Some(list) = child_field(node, "list") {
                flatten_use(builder, list, &base, out);
            }
        }
        "use_list" => {
            for child in named_children(node) {
                flatten_use(builder, child, prefix, out);
            }
        }
        "use_as_clause" => {
            let path = child_field(node, "path")
                .map(|path| join(builder.text(path)))
                .unwrap_or_default();
            let alias = child_field(node, "alias").map(|alias| builder.text(alias).to_string());
            out.push((path, alias));
        }
        "identifier" | "scoped_identifier" | "crate" | "self" | "super" => {
            out.push((join(builder.text(node)), None));
        }
        "use_wildcard" => {
            let inner = first_named(node)
                .map(|name| builder.text(name).to_string())
                .unwrap_or_default();
            if inner.ends_with('*') || inner.is_empty() {
                out.push((join(&format!("{inner}*")), None));
            } else {
                out.push((join(&format!("{inner}::*")), None));
            }
        }
        _ => {
            let text = builder.text(node);
            if !text.is_empty() {
                out.push((join(text), None));
            }
        }
    }
}
