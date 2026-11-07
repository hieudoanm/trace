use std::path::{Path, PathBuf};

use tracers_core::Language;

use super::types::TracersConfig;

pub(crate) fn languages(config: &TracersConfig) -> Vec<Language> {
    config
        .analysis
        .languages
        .iter()
        .filter_map(|name| Language::parse_name(name))
        .collect()
}

pub(crate) fn cache_directory(config: &TracersConfig, root: &Path) -> PathBuf {
    let configured = Path::new(&config.cache.directory);
    if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        root.join(configured)
    }
}
