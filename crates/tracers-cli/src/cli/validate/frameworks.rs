use std::path::Path;

use anyhow::{anyhow, Result};

use tracers_analysis::cache::SyntaxCache;
use tracers_analysis::config::TracersConfig;
use tracers_analysis::framework::FrameworkDetector;
use tracers_analysis::graph::graph_builder::discovery::discover_manifests;
use tracers_analysis::graph::graph_builder::parsing::parse_files;
use tracers_parser::discovery::discover;

use crate::cli::scan::discovery_options;

pub(crate) fn cmd_frameworks(
    path: &Path,
    include: &[String],
    exclude: &[String],
    language: Option<&str>,
) -> Result<()> {
    let (config, _source) = TracersConfig::load(path, None)
        .map_err(|e| anyhow!("failed to load configuration: {e}"))?;
    let options =
        discovery_options(&config, include, exclude, language).map_err(anyhow::Error::msg)?;
    let report = discover(path, &options).map_err(anyhow::Error::msg)?;

    let cache = SyntaxCache::from_config(path, &config, false);
    let jobs = config.analysis.jobs;
    if jobs == Some(0) {
        anyhow::bail!("analysis.jobs must be at least 1");
    }
    let (mut parsed_files, parse_diagnostics) = parse_files(
        &report.files,
        jobs,
        cache.enabled().then_some(&cache),
        options.max_file_size,
    )
    .map_err(|e| anyhow!("parser error: {e}"))?;
    for diagnostic in &parse_diagnostics {
        eprintln!(
            "warning: skipping {}: {}",
            diagnostic.file.as_deref().unwrap_or("?"),
            diagnostic.message
        );
    }
    if !parse_diagnostics.is_empty() {
        eprintln!(
            "warning: {} file(s) could not be analysed",
            parse_diagnostics.len()
        );
    }
    parsed_files.sort_by(|a, b| a.path.cmp(&b.path));

    let manifest_files = discover_manifests(path);

    let detector = FrameworkDetector::new();
    let detection = detector.detect(path, &parsed_files, &manifest_files);

    println!("Detected frameworks:");
    if detection.frameworks.is_empty() {
        println!("  No frameworks detected.");
    } else {
        for fw in &detection.frameworks {
            let version = fw
                .version
                .as_deref()
                .map(|v| format!(" {v}"))
                .unwrap_or_default();
            println!(
                "  {} {}{}\n    family: {}, confidence: {}",
                fw.name,
                fw.display_name,
                version,
                fw.family.as_str(),
                fw.confidence.as_str()
            );
            for signal in &fw.signals {
                let file_info = signal
                    .file
                    .as_deref()
                    .map(|f| format!(" in {f}"))
                    .unwrap_or_default();
                let line_info = signal.line.map(|l| format!(":{l}")).unwrap_or_default();
                println!(
                    "    - {}: {}{}{}",
                    signal.kind.as_str(),
                    signal.value,
                    file_info,
                    line_info
                );
            }
        }
    }

    Ok(())
}
