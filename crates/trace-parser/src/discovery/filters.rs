use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::util::strip_quotes;

pub fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn compile_globs(patterns: &[String], flag: &str) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let pattern = strip_quotes(pattern);
        let glob = Glob::new(&pattern)
            .map_err(|error| format!("invalid {flag} pattern `{pattern}`: {error}"))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|error| format!("cannot compile {flag} patterns: {error}"))
}

pub fn looks_generated(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    name.contains(".min.")
        || name.contains(".generated.")
        || name.contains(".designer.")
        || name.ends_with("_generated.rs")
        || name.ends_with("_generated.go")
        || name.ends_with(".pb.go")
        || name.ends_with(".pb.swift")
        || name.ends_with(".g.swift")
        || name == "bindings.rs"
}
