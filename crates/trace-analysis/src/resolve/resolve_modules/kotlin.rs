use trace_parser::ImportData;

use crate::index::ModuleRef;

use crate::resolve::resolve_import_pass::ImportContext;
use crate::resolve::resolve_path_helpers::kotlin_package_id;
use crate::resolve::resolve_types::Resolution;

pub(crate) fn resolve_kotlin(
    ctx: &ImportContext<'_>,
    import: &ImportData,
) -> std::result::Result<ModuleRef, Resolution> {
    let raw = import.module.trim();
    if let Some(package) = raw.strip_suffix(".*") {
        let id = kotlin_package_id(package);
        if ctx.index.packages.contains_key(&id) {
            return Ok(ModuleRef::Package(id));
        }
        return Err(Resolution::Unresolved {
            name: package.to_string(),
        });
    }
    let (package, name) = match raw.rsplit_once('.') {
        Some((package, name)) => (package, name),
        None => ("", raw),
    };
    let candidates: Vec<String> = ctx
        .index
        .by_kotlin_package
        .get(package)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|path| {
            ctx.index.files.get(path).is_some_and(|state| {
                state
                    .top_level
                    .get(name)
                    .is_some_and(|indexes| !indexes.is_empty())
            })
        })
        .collect();
    match candidates.len() {
        1 => Ok(ModuleRef::File(candidates[0].clone())),
        0 => {
            if ctx.index.by_kotlin_package.contains_key(package) {
                Ok(ModuleRef::Package(kotlin_package_id(package)))
            } else {
                Ok(ModuleRef::External(raw.to_string()))
            }
        }
        _ => Err(Resolution::Ambiguous {
            name: raw.to_string(),
            candidates,
        }),
    }
}
