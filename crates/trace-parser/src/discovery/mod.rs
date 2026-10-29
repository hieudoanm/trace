pub mod constants;
pub mod filters;
pub mod single_file;
pub mod types;
pub mod walk;

#[cfg(test)]
mod tests;

use std::path::Path;

pub use constants::DEFAULT_EXCLUDED_DIRS;
pub use filters::{compile_globs, looks_generated, relative_path};
pub use types::{DiscoveredFile, DiscoveryOptions, DiscoveryReport, SkipReason, SkippedFile};

pub fn discover(root: &Path, options: &DiscoveryOptions) -> Result<DiscoveryReport, String> {
    if root.is_file() {
        return single_file::discover_single_file(root, options);
    }
    walk::discover_directory(root, options)
}
