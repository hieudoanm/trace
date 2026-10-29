use trace_core::Language;
use trace_parser::ImportData;

use crate::index::ModuleRef;

use crate::resolve::resolve_import_pass::ImportContext;
use crate::resolve::resolve_path_helpers::module_id_to_package;
use crate::resolve::resolve_types::Resolution;

pub(crate) fn resolve_go(
    ctx: &ImportContext<'_>,
    import: &ImportData,
) -> std::result::Result<ModuleRef, Resolution> {
    let path = import.module.trim();
    let mut directories = Vec::new();
    for module in &ctx.index.go_modules {
        if path == module.module {
            directories.push(if module.directory == "." {
                ".".to_string()
            } else {
                module.directory.clone()
            });
        } else if let Some(rest) = path.strip_prefix(&format!("{}/", module.module)) {
            let directory = if module.directory == "." {
                rest.to_string()
            } else {
                format!("{}/{}", module.directory, rest)
            };
            directories.push(directory);
        }
    }
    directories.sort();
    directories.dedup();
    directories.retain(|directory| ctx.index.by_directory.contains_key(directory));
    if directories.len() == 1 {
        let directory = directories.remove(0);
        return Ok(ModuleRef::Package(module_id_to_package(&directory)));
    }
    let suffix = format!("/{path}");
    for directory in ctx.index.by_directory.keys() {
        if *directory == path || directory.ends_with(&suffix) {
            directories.push(directory.clone());
        }
    }
    directories.sort();
    directories.dedup();
    directories.retain(|directory| {
        ctx.index.by_directory.get(directory).is_some_and(|files| {
            files.iter().any(|path| {
                ctx.index
                    .files
                    .get(path)
                    .is_some_and(|state| state.language == Language::Go)
            })
        })
    });
    match directories.len() {
        1 => Ok(ModuleRef::Package(module_id_to_package(&directories[0]))),
        0 => Ok(ModuleRef::External(path.to_string())),
        _ => Err(Resolution::Ambiguous {
            name: path.to_string(),
            candidates: directories,
        }),
    }
}
