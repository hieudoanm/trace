use trace_parser::ImportData;

use crate::index::{directory_key, parent_directory, ModuleRef};

use crate::resolve::resolve_import_pass::ImportContext;
use crate::resolve::resolve_path_helpers::join_dir;
use crate::resolve::resolve_types::Resolution;

pub(crate) fn resolve_python(
    ctx: &ImportContext<'_>,
    import: &ImportData,
) -> std::result::Result<ModuleRef, Resolution> {
    let specifier = import.module.trim();
    if specifier.starts_with('.') {
        let dots = specifier.chars().take_while(|c| *c == '.').count();
        let rest = specifier[dots..].trim_matches('.');
        let mut directory = directory_key(&ctx.path);
        for _ in 1..dots {
            directory = parent_directory(&directory).to_string();
            if directory.is_empty() {
                directory = ".".to_string();
            }
        }
        if rest.is_empty() {
            let init = join_dir(&directory, "__init__.py");
            if ctx.index.files.contains_key(&init) {
                return Ok(ModuleRef::File(init));
            }
            return Err(Resolution::Unresolved {
                name: specifier.to_string(),
            });
        }
        let relative = rest.replace('.', "/");
        let direct = join_dir(&directory, &format!("{relative}.py"));
        if ctx.index.files.contains_key(&direct) {
            return Ok(ModuleRef::File(direct));
        }
        let package_init = join_dir(&directory, &format!("{relative}/__init__.py"));
        if ctx.index.files.contains_key(&package_init) {
            return Ok(ModuleRef::File(package_init));
        }
        return Err(Resolution::Unresolved {
            name: specifier.to_string(),
        });
    }
    python_module(ctx, specifier, &ctx.path).ok_or_else(|| Resolution::Unresolved {
        name: specifier.to_string(),
    })
}

pub(crate) fn python_module(
    ctx: &ImportContext<'_>,
    dotted: &str,
    from_path: &str,
) -> Option<ModuleRef> {
    let dotted = dotted.trim_matches('.');
    if dotted.is_empty() {
        return None;
    }
    if let Some(paths) = ctx.index.by_python_module.get(dotted) {
        if paths.len() == 1 {
            return Some(ModuleRef::File(paths[0].clone()));
        }
        if paths.len() > 1 {
            return None;
        }
    }
    let suffix = format!(".{dotted}");
    let matches: Vec<String> = ctx
        .index
        .by_python_module
        .iter()
        .filter(|(module, _)| module.ends_with(&suffix))
        .flat_map(|(_, paths)| paths.clone())
        .collect();
    if matches.len() == 1 {
        return Some(ModuleRef::File(matches[0].clone()));
    }
    let directory = parent_directory(from_path);
    let direct = if directory.is_empty() {
        format!("{}.py", dotted.replace('.', "/"))
    } else {
        format!("{directory}/{}.py", dotted.replace('.', "/"))
    };
    ctx.index
        .files
        .contains_key(&direct)
        .then_some(ModuleRef::File(direct))
}
