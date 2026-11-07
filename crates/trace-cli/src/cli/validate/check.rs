use std::path::Path;

use anyhow::{Context, Result};
use trace_core::Language;

use trace_analysis::config::TraceConfig;
use trace_parser::discovery::{discover, DiscoveryOptions};

pub(crate) fn cmd_check(path: &Path) -> Result<()> {
    let (config, config_source) =
        TraceConfig::load(path, None).context("failed to load configuration")?;

    println!("Configuration: valid");
    if let Some(source) = &config_source {
        println!("  source: {source}");
    }

    println!("\nDiscovery:");
    let options = DiscoveryOptions {
        max_file_size: config.analysis.max_file_size,
        follow_symlinks: config.analysis.follow_symlinks,
        include_generated: config.analysis.include_generated,
        languages: config.languages(),
        ..DiscoveryOptions::default()
    };
    let report = discover(path, &options).map_err(anyhow::Error::msg)?;
    println!("  root:        {}", report.root.display());
    println!("  files:       {}", report.files.len());
    println!("  skipped:     {}", report.skipped.len());
    println!("  unsupported: {}", report.unsupported_files);

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

    let capabilities = trace_analysis::capabilities::all_capabilities();
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
