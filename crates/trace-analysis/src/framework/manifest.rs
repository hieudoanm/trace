use std::collections::BTreeMap;

/// Contents of a discovered package manifest file, keyed by repository-relative path.
#[derive(Debug, Clone, Default)]
pub struct ManifestContents {
    /// Path (repository-relative) -> file text.
    pub files: BTreeMap<String, String>,
}

impl ManifestContents {
    pub fn empty() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }
}
