use trace_parser::ImportData;

use crate::index::ModuleRef;

use crate::resolve::resolve_import_pass::ImportContext;
use crate::resolve::resolve_types::Resolution;

pub(crate) fn resolve_rust(
    ctx: &ImportContext<'_>,
    import: &ImportData,
) -> std::result::Result<ModuleRef, Resolution> {
    let raw = import
        .module
        .as_str()
        .trim_end_matches('*')
        .trim_end_matches("::");
    if let Some(module) = resolve_module_path(ctx, raw) {
        return Ok(module);
    }
    let first = raw.split("::").next().unwrap_or(raw);
    if matches!(first, "crate" | "self" | "super") {
        return Err(Resolution::Unresolved {
            name: raw.to_string(),
        });
    }
    if raw.is_empty() {
        return Err(Resolution::Unresolved {
            name: raw.to_string(),
        });
    }
    Ok(ModuleRef::External(
        raw.split("::").next().unwrap_or(raw).to_string(),
    ))
}

pub(crate) fn resolve_module_path(ctx: &ImportContext<'_>, raw: &str) -> Option<ModuleRef> {
    let state = ctx.state;
    let mut segments: Vec<String> = raw
        .split("::")
        .filter(|segment| !segment.is_empty() && *segment != "*")
        .map(|segment| segment.to_string())
        .collect();
    if segments.is_empty() {
        return None;
    }
    let mut module_segments: Vec<String> = Vec::new();
    let first = segments.first()?.as_str();
    match first {
        "crate" => {
            segments.remove(0);
        }
        "self" => {
            segments.remove(0);
            module_segments = rust_current_module(ctx, state);
        }
        "super" => {
            let mut hops = 0;
            while segments.first().is_some_and(|segment| segment == "super") {
                segments.remove(0);
                hops += 1;
            }
            module_segments = rust_current_module(ctx, state);
            for _ in 0..hops {
                module_segments.pop();
            }
        }
        _ => {}
    }
    let crate_root = state.crate_root.clone().unwrap_or_default();

    let mut full = module_segments.clone();
    full.extend(segments.iter().cloned());
    if let Some(path) = rust_file_for(ctx, &crate_root, &full) {
        return Some(ModuleRef::File(path));
    }
    if segments.len() > 1 {
        let mut module = module_segments.clone();
        module.extend(segments[..segments.len() - 1].iter().cloned());
        if let Some(path) = rust_file_for(ctx, &crate_root, &module) {
            return Some(ModuleRef::File(path));
        }
    }
    None
}

pub(crate) fn rust_current_module(
    _ctx: &ImportContext<'_>,
    state: &crate::index::FileState,
) -> Vec<String> {
    state
        .module_qualified
        .strip_prefix("crate::")
        .unwrap_or("")
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect()
}

pub(crate) fn rust_file_for(
    ctx: &ImportContext<'_>,
    crate_root: &str,
    segments: &[String],
) -> Option<String> {
    let relative = segments.join("/");
    let mut candidates = Vec::new();
    let prefix = if crate_root.is_empty() || crate_root == "." {
        String::new()
    } else {
        format!("{crate_root}/")
    };
    if relative.is_empty() {
        candidates.push(format!("{prefix}src/lib.rs"));
        candidates.push(format!("{prefix}src/main.rs"));
        candidates.push(format!("{prefix}lib.rs"));
        candidates.push(format!("{prefix}main.rs"));
    } else {
        candidates.push(format!("{prefix}src/{relative}.rs"));
        candidates.push(format!("{prefix}src/{relative}/mod.rs"));
        candidates.push(format!("{prefix}{relative}.rs"));
        candidates.push(format!("{prefix}{relative}/mod.rs"));
    }
    candidates
        .into_iter()
        .find(|path| ctx.index.files.contains_key(path))
}

pub(crate) fn rust_module_tail(ctx: &ImportContext<'_>, path: &str) -> String {
    ctx.index
        .files
        .get(path)
        .map(|state| {
            state
                .module_qualified
                .rsplit("::")
                .next()
                .unwrap_or("")
                .to_string()
        })
        .unwrap_or_else(|| crate::index::stem(path).to_string())
}
