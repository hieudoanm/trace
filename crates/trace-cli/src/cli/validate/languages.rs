use anyhow::Result;

pub(crate) fn cmd_languages() -> Result<()> {
    let capabilities = trace_analysis::capabilities::all_capabilities();
    let mut languages: Vec<&str> = capabilities
        .iter()
        .map(|cap| cap.language.display_name())
        .collect();
    languages.sort();
    languages.dedup();

    println!("Supported languages:");
    for name in &languages {
        println!("  {name}");
    }
    println!("\nCapability matrix:");
    for cap in &capabilities {
        println!(
            "  {} ({}): syntax={}, symbols={}, imports={}, calls={}, cross-file={}",
            cap.language.display_name(),
            cap.grammar,
            cap.syntax_parsing,
            cap.symbol_extraction,
            cap.import_resolution,
            cap.call_resolution,
            cap.cross_file_references,
        );
    }
    Ok(())
}
