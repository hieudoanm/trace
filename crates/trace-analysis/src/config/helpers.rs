use std::path::{Path, PathBuf};

use trace_core::Language;

use super::types::TraceConfig;

pub(crate) fn languages(config: &TraceConfig) -> Vec<Language> {
    config
        .analysis
        .languages
        .iter()
        .filter_map(|name| Language::parse_name(name))
        .collect()
}

pub(crate) fn cache_directory(config: &TraceConfig, root: &Path) -> PathBuf {
    let configured = Path::new(&config.cache.directory);
    if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        root.join(configured)
    }
}
