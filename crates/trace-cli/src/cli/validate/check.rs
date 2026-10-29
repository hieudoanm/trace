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
    for cap in &capabilities {
        let status = if cap.syntax_parsing { "ok" } else { "missing" };
        println!(
            "  {} [{}]: {}",
            cap.language.display_name(),
            cap.grammar,
            status
        );
    }

    println!("\nCheck passed.");
    Ok(())
}
