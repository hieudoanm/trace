use std::path::Path;

use anyhow::{anyhow, Result};

use trace_analysis::framework::{FrameworkDetector, ManifestContents};
use trace_parser::discovery::{discover, DiscoveryOptions};
use trace_parser::parser::SyntaxParser;

pub(crate) fn cmd_frameworks(path: &Path) -> Result<()> {
    let options = DiscoveryOptions::default();
    let report = discover(path, &options).map_err(anyhow::Error::msg)?;
    let mut parsed_files = Vec::new();
    for file in &report.files {
        let source = std::fs::read_to_string(&file.absolute)
            .map_err(|e| anyhow!("cannot read {}: {e}", file.absolute.display()))?;
        let mut parser = SyntaxParser::new().map_err(|e| anyhow!("parser error: {e}"))?;
        let parsed = parser
            .parse(&file.path, file.language, &source)
            .map_err(|e| anyhow!("parse error: {e}"))?;
        parsed_files.push(parsed);
    }
    parsed_files.sort_by(|a, b| a.path.cmp(&b.path));

    let manifest_names = [
        "package.json",
        "Cargo.toml",
        "go.mod",
        "requirements.txt",
        "pyproject.toml",
        "setup.py",
        "setup.cfg",
        "Gemfile",
        "composer.json",
        "pom.xml",
        "build.gradle",
        "build.gradle.kts",
        "settings.gradle",
        "angular.json",
        "next.config.js",
        "next.config.mjs",
        "config/application.rb",
    ];
    let mut manifest_files = ManifestContents::empty();
    for name in &manifest_names {
        let manifest_path = path.join(name);
        if manifest_path.is_file() {
            if let Ok(text) = std::fs::read_to_string(&manifest_path) {
                let relative = manifest_path
                    .strip_prefix(path)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| name.to_string());
                manifest_files.files.insert(relative, text);
            }
        }
    }

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
