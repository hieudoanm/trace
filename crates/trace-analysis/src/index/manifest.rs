use std::collections::BTreeSet;
use std::path::Path;

use trace_core::Language;

use crate::index::repository::GoModule;

use trace_parser::ParsedFile;

pub(crate) fn nearest_manifest(root: &Path, path: &str, manifest: &str) -> Option<String> {
    let mut directory = super::helpers::directory_key(path);
    loop {
        let relative = if directory == "." {
            manifest.to_string()
        } else {
            format!("{directory}/{manifest}")
        };
        if root.join(&relative).is_file() {
            return Some(if directory == "." {
                String::new()
            } else {
                directory
            });
        }
        if directory == "." {
            return None;
        }
        directory = super::helpers::parent_directory(&directory).to_string();
        if directory.is_empty() {
            directory = ".".to_string();
        }
    }
}

pub(crate) fn discover_go_modules(root: &Path, parsed_files: &[ParsedFile]) -> Vec<GoModule> {
    let mut manifests = BTreeSet::new();
    for parsed in parsed_files {
        if parsed.language != Language::Go {
            continue;
        }
        let mut directory = super::helpers::directory_key(&parsed.path);
        loop {
            let relative = if directory == "." {
                "go.mod".to_string()
            } else {
                format!("{directory}/go.mod")
            };
            if root.join(&relative).is_file() {
                manifests.insert(relative);
                break;
            }
            if directory == "." {
                break;
            }
            directory = super::helpers::parent_directory(&directory).to_string();
            if directory.is_empty() {
                directory = ".".to_string();
            }
        }
    }
    let mut modules = Vec::new();
    for manifest in manifests {
        let Ok(text) = std::fs::read_to_string(root.join(&manifest)) else {
            continue;
        };
        let module = text.lines().find_map(|line| {
            line.trim()
                .strip_prefix("module ")
                .map(|value| value.trim().trim_matches('"').to_string())
        });
        let Some(module) = module else {
            continue;
        };
        let directory = super::helpers::parent_directory(&manifest).to_string();
        modules.push(GoModule {
            directory: if directory.is_empty() {
                ".".to_string()
            } else {
                directory
            },
            module,
        });
    }
    modules.sort_by(|a, b| a.directory.cmp(&b.directory));
    modules
}
