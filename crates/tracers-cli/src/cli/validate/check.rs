use std::path::Path;

use anyhow::{Context, Result};
use tracers_core::Language;

use tracers_analysis::config::TracersConfig;
use tracers_parser::discovery::discover;

use crate::cli::scan::discovery_options;

pub(crate) fn cmd_check(
    path: &Path,
    include: &[String],
    exclude: &[String],
    language: Option<&str>,
) -> Result<()> {
    let (config, config_source) =
        TracersConfig::load(path, None).context("failed to load configuration")?;

    println!("Configuration: valid");
    if let Some(source) = &config_source {
        println!("  source: {source}");
    }

    println!("\nDiscovery:");
    let options = discovery_options(&config, include, exclude, language)
        .context("invalid discovery options")?;
    let report = discover(path, &options).map_err(anyhow::Error::msg)?;
    println!("  root:        {}", report.root.display());
    println!("  files:       {}", report.files.len());
    println!("  skipped:     {}", report.skipped.len());
    println!("  unsupported: {}", report.unsupported_files);
    println!("  gitignored:  {}", report.ignored_files);
    let skip_counts = report.skip_counts();
    if !skip_counts.is_empty() {
        let breakdown = skip_counts
            .iter()
            .map(|(reason, count)| format!("{reason}={count}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("  skip reasons: {breakdown}");
    }

    let cache_dir = config.cache_directory(path);
    println!(
        "\nCache: {} ({})",
        if config.cache.enabled {
            "enabled"
        } else {
            "disabled"
        },
        cache_dir.display()
    );

    let mut languages: Vec<Language> = report.files.iter().map(|file| file.language).collect();
    languages.sort();
    languages.dedup();

    println!("\nLanguages:");
    if languages.is_empty() {
        println!("  (none detected)");
    } else {
        for language in &languages {
            let count = report
                .files
                .iter()
                .filter(|f| f.language == *language)
                .count();
            println!("  {}: {} file(s)", language.display_name(), count);
        }
    }

    let capabilities = tracers_analysis::capabilities::all_capabilities();
    println!("\nParser availability:");
    println!("  All tree-sitter grammars are compiled in; syntax parsing is always available.");

    let mut tools: Vec<&str> = capabilities
        .iter()
        .filter_map(|cap| cap.semantic_tool.as_deref())
        .collect();
    tools.sort_unstable();
    tools.dedup();
    if tools.is_empty() {
        println!("\nExternal semantic tools: none declared.");
    } else {
        println!("\nExternal semantic tools (optional; not required for syntax analysis):");
        for tool in tools {
            let status = if tool_on_path(tool) {
                "found"
            } else {
                "not found"
            };
            println!("  {tool}: {status}");
        }
    }

    if !config.tools.is_empty() {
        println!("\nConfigured tools ([tools] in tracers.toml):");
        for (name, tool_path) in &config.tools {
            let candidate = Path::new(tool_path);
            let resolved = if candidate.is_absolute() {
                candidate.to_path_buf()
            } else {
                // Relative paths resolve against the repository root.
                path.join(candidate)
            };
            let status = if is_executable(&resolved) {
                "found"
            } else {
                "not executable"
            };
            println!("  {name}: {tool_path} ({status})");
        }
    }

    println!("\nCheck passed.");
    Ok(())
}

/// Whether `tool` resolves to an executable file on `PATH`.
fn tool_on_path(tool: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| {
        let candidate = dir.join(tool);
        candidate.is_file() && is_executable(&candidate)
    })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}
