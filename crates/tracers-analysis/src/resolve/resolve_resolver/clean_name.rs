pub(crate) fn clean_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let trimmed = trimmed.trim_start_matches(['&', '*', '?']);
    let trimmed = trimmed
        .split(['<', '[', '(', '|', '&', ' ', '<'])
        .next()
        .unwrap_or(trimmed);
    trimmed.trim().trim_end_matches('?').to_string()
}
