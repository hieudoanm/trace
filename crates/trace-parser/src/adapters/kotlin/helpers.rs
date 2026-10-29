use tree_sitter::Node as TsNode;

use trace_core::Visibility;

use crate::util::{named_children, SyntaxBuilder};

pub(crate) fn first_of_kind<'a>(node: TsNode<'a>, kind: &str) -> Option<TsNode<'a>> {
    named_children(node)
        .into_iter()
        .find(|child| child.kind() == kind)
}

pub(crate) fn find_modifiers(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    named_children(node)
        .into_iter()
        .find(|child| child.kind() == "modifiers")
        .map(|modifiers| builder.text(modifiers).to_string())
        .unwrap_or_default()
}

pub(crate) fn kotlin_visibility(modifiers: &str) -> Option<Visibility> {
    for word in modifiers.split_whitespace() {
        match word {
            "private" => return Some(Visibility::Private),
            "protected" => return Some(Visibility::Protected),
            "internal" => return Some(Visibility::Internal),
            "public" => return Some(Visibility::Public),
            _ => {}
        }
    }
    Some(Visibility::Public)
}

pub(crate) fn kotlin_annotations(modifiers: &str) -> Vec<String> {
    modifiers
        .split('@')
        .skip(1)
        .map(|annotation| {
            annotation
                .split_whitespace()
                .next()
                .unwrap_or(annotation)
                .trim()
                .to_string()
        })
        .filter(|annotation| !annotation.is_empty())
        .collect()
}

pub(crate) fn is_upper_camel(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

pub(crate) fn last_type_name(builder: &SyntaxBuilder<'_>, node: TsNode<'_>) -> String {
    let text = builder.text(node);
    text.rsplit('.')
        .next()
        .unwrap_or(text)
        .trim_end_matches('?')
        .to_string()
}
