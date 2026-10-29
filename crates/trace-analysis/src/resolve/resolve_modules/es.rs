use trace_parser::ImportData;

use crate::index::ModuleRef;

use crate::resolve::resolve_import_pass::ImportContext;
use crate::resolve::resolve_path_helpers::{es_candidates, join_relative};
use crate::resolve::resolve_types::Resolution;

pub(crate) fn resolve_es(
    ctx: &ImportContext<'_>,
    import: &ImportData,
) -> std::result::Result<ModuleRef, Resolution> {
    let specifier = import.module.trim();
    if specifier.starts_with('.') {
        let base = join_relative(&ctx.path, specifier);
        for candidate in es_candidates(&base) {
            if ctx.index.files.contains_key(&candidate) {
                return Ok(ModuleRef::File(candidate));
            }
        }
        return Err(Resolution::Unresolved {
            name: specifier.to_string(),
        });
    }
    if specifier.starts_with('/') || specifier.is_empty() {
        return Err(Resolution::Unresolved {
            name: specifier.to_string(),
        });
    }
    Ok(ModuleRef::External(specifier.to_string()))
}
