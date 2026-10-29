use std::path::PathBuf;
use std::process::Command;

fn trace_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../target/debug/trace");
    path
}

fn run_trace(args: &[&str]) -> String {
    let output = Command::new(trace_bin())
        .args(args)
        .output()
        .expect("Failed to execute trace");

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn fixtures_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../fixtures/languages");
    path
}

#[test]
fn scan_typescript_fixture() {
    let path = fixtures_dir().join("typescript");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    // Verify summary output
    assert!(output.contains("=== Trace scan summary ==="));
    assert!(output.contains("Nodes:"));
    assert!(output.contains("Edges:"));
    assert!(output.contains("Languages detected:"));
    assert!(output.contains("typescript"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_python_fixture() {
    let path = fixtures_dir().join("python");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Trace scan summary ==="));
    assert!(output.contains("python"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_go_fixture() {
    let path = fixtures_dir().join("go");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Trace scan summary ==="));
    assert!(output.contains("go"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_rust_fixture() {
    let path = fixtures_dir().join("rust");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Trace scan summary ==="));
    assert!(output.contains("rust"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_kotlin_fixture() {
    let path = fixtures_dir().join("kotlin");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Trace scan summary ==="));
    assert!(output.contains("kotlin"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_swift_fixture() {
    let path = fixtures_dir().join("swift");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Trace scan summary ==="));
    assert!(output.contains("swift"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn stats_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_trace(&["stats", path.to_str().unwrap()]);

    assert!(output.contains("Graph statistics:"));
    assert!(output.contains("files:"));
    assert!(output.contains("nodes:"));
    assert!(output.contains("edges:"));
}

#[test]
fn languages_command() {
    let output = run_trace(&["languages"]);

    assert!(output.contains("Supported languages:"));
    assert!(output.contains("TypeScript"));
    assert!(output.contains("Python"));
    assert!(output.contains("Go"));
    assert!(output.contains("Rust"));
    assert!(output.contains("Kotlin"));
    assert!(output.contains("Swift"));
    assert!(output.contains("Java"));
    assert!(output.contains("C#"));
    assert!(output.contains("PHP"));
    assert!(output.contains("Ruby"));
}

#[test]
fn check_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_trace(&["check", path.to_str().unwrap()]);

    assert!(output.contains("Configuration: valid"));
    assert!(output.contains("Discovery:"));
    assert!(output.contains("Languages:"));
    assert!(output.contains("Check passed."));
}

#[test]
fn scan_with_language_filter() {
    let path = fixtures_dir().join("typescript");
    let output = run_trace(&["scan", path.to_str().unwrap(), "--language", "typescript"]);

    assert!(output.contains("=== Trace scan summary ==="));
    assert!(output.contains("typescript"));
    let languages_section = output
        .split("Languages detected:")
        .nth(1)
        .unwrap_or("")
        .split("\n\n")
        .next()
        .unwrap_or("");
    assert!(!languages_section.contains("python"));
    assert!(!languages_section.contains("go"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_no_cache() {
    let path = fixtures_dir().join("typescript");
    let output = run_trace(&["scan", path.to_str().unwrap(), "--no-cache"]);

    assert!(output.contains("=== Trace scan summary ==="));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn cycles_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_trace(&["cycles", path.to_str().unwrap()]);

    // Should either show cycles or report none
    assert!(output.contains("Dependency cycles:") || output.contains("No dependency cycles"));
}

#[test]
fn scan_output_to_file() {
    let path = fixtures_dir().join("typescript");
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.json");

    run_trace(&[
        "scan",
        path.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
    ]);

    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");
    assert!(json["nodes"].is_array());

    // Verify all file types (.json, .dot, .html) were generated
    let dot_file = output_file.with_extension("dot");
    assert!(dot_file.exists());
    let dot_content = std::fs::read_to_string(&dot_file).unwrap();
    assert!(dot_content.contains("digraph trace"));

    let html_file = output_file.with_extension("html");
    assert!(html_file.exists());
    let html_content = std::fs::read_to_string(&html_file).unwrap();
    assert!(html_content.contains("Trace Graph Explorer"));
}

#[test]
fn scan_outputs_all_files_to_trace_dir() {
    let temp_repo = tempfile::tempdir().unwrap();
    let src_file = temp_repo.path().join("main.ts");
    std::fs::write(
        &src_file,
        "export function add(a: number, b: number) { return a + b; }",
    )
    .unwrap();

    run_trace(&["scan", temp_repo.path().to_str().unwrap()]);

    let trace_dir = temp_repo.path().join(".trace");
    assert!(trace_dir.exists());

    let json_file = trace_dir.join("graph.json");
    assert!(json_file.exists());
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json_file).unwrap()).expect("Invalid JSON");
    assert!(json["nodes"].is_array());

    let dot_file = trace_dir.join("graph.dot");
    assert!(dot_file.exists());
    let dot = std::fs::read_to_string(&dot_file).unwrap();
    assert!(dot.contains("digraph trace"));

    let html_file = trace_dir.join("graph.html");
    assert!(html_file.exists());
    let html = std::fs::read_to_string(&html_file).unwrap();
    assert!(html.contains("Trace Graph Explorer"));
}
