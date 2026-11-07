use tree_sitter::Node as TsNode;

/// Compact one-line signature for a declaration, capped at 300 bytes.
pub fn signature(source: &str, node: TsNode<'_>) -> Option<String> {
    let mut raw = super::tree_sitter::text(source, node);
    if let Some(idx) = raw.find('\n') {
        raw = &raw[..idx];
    }
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let mut out = raw.to_string();
    if out.len() > 300 {
        let mut cut = 300;
        while !out.is_char_boundary(cut) {
            cut -= 1;
        }
        out.truncate(cut);
        out.push_str("...");
    }
    Some(out)
}

pub fn strip_quotes(value: &str) -> String {
    value
        .trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .to_string()
}

pub fn is_upper_camel(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

pub fn last_segment(value: &str) -> &str {
    value.rsplit(['.', ':', '/', '\\']).next().unwrap_or(value)
}
