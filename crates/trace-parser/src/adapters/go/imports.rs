use tree_sitter::Node as TsNode;

use crate::util::{child_field, named_children, range, strip_quotes, SyntaxBuilder};

use crate::syntax::{ImportData, ImportKind};

pub(crate) fn parse_imports(builder: &mut SyntaxBuilder<'_>, node: TsNode<'_>) {
    let mut specs = Vec::new();
    collect_specs(node, &mut specs);
    for spec in specs {
        if spec.kind() != "import_spec" {
            continue;
        }
        let module = child_field(spec, "path")
            .map(|path| strip_quotes(builder.text(path)))
            .unwrap_or_default();
        if module.is_empty() {
            continue;
        }
        let alias = child_field(spec, "name").map(|name| builder.text(name).to_string());
        builder.add_import(ImportData {
            module,
            alias,
            items: Vec::new(),
            kind: ImportKind::Go,
            range: range(spec),
            is_export: false,
        });
    }
}

pub(crate) fn collect_specs<'a>(node: TsNode<'a>, out: &mut Vec<TsNode<'a>>) {
    if node.kind() == "import_spec" {
        out.push(node);
        return;
    }
    for child in named_children(node) {
        collect_specs(child, out);
    }
}

pub(crate) fn collect_specs_named<'a>(node: TsNode<'a>, out: &mut Vec<TsNode<'a>>) {
    if matches!(
        node.kind(),
        "var_spec" | "const_spec" | "short_var_declaration"
    ) {
        out.push(node);
        return;
    }
    for child in named_children(node) {
        collect_specs_named(child, out);
    }
}
