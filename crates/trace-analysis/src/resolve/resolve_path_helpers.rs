//! Path and module ID helper functions.

use crate::index::parent_directory;

/// Join a relative specifier to a directory path.
pub fn join_relative(from_path: &str, specifier: &str) -> String {
    let directory = parent_directory(from_path);
    let raw = if directory.is_empty() {
        specifier.to_string()
    } else {
        format!("{directory}/{specifier}")
    };
    trace_core::id::normalize(&raw)
}

/// Join a directory and a name with a slash.
pub fn join_dir(directory: &str, name: &str) -> String {
    if directory == "." || directory.is_empty() {
        name.to_string()
    } else {
        format!("{directory}/{name}")
    }
}

/// Generate candidate file paths for an ES import specifier.
pub fn es_candidates(base: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if base.ends_with(".ts")
        || base.ends_with(".tsx")
        || base.ends_with(".js")
        || base.ends_with(".jsx")
        || base.ends_with(".mjs")
        || base.ends_with(".cjs")
        || base.ends_with(".mts")
        || base.ends_with(".cts")
    {
        candidates.push(base.to_string());
        if base.ends_with(".js") || base.ends_with(".jsx") || base.ends_with(".mjs") {
            let stem = base
                .trim_end_matches(".js")
                .trim_end_matches(".jsx")
                .trim_end_matches(".mjs");
            candidates.push(format!("{stem}.ts"));
            candidates.push(format!("{stem}.tsx"));
        }
    }
    for extension in [
        ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".d.ts",
    ] {
        candidates.push(format!("{base}{extension}"));
    }
    for extension in [
        "/index.ts",
        "/index.tsx",
        "/index.js",
        "/index.jsx",
        "/index.mjs",
    ] {
        candidates.push(format!("{base}{extension}"));
    }
    candidates
}

/// Convert a module directory to a package ID.
pub fn module_id_to_package(directory: &str) -> String {
    trace_core::package_id(if directory == "." { "" } else { directory })
}

/// Convert a Kotlin package name to a package ID.
pub fn kotlin_package_id(package: &str) -> String {
    trace_core::package_id(&format!("kotlin/{}", package.replace('.', "/")))
}

/// Get the last path segment of a path.
pub fn last_path_segment(path: &str) -> &str {
    path.rsplit([':', '.'])
        .find(|segment| !segment.is_empty() && *segment != "*")
        .unwrap_or("")
}
