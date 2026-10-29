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
    path.push("../../fixtures");
    path
}

#[test]
fn scan_typescript_fixture() {
    let path = fixtures_dir().join("typescript");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    // Verify it's valid JSON
    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON output");

    // Check basic structure
    assert!(json["schemaVersion"].is_string());
    assert!(json["traceVersion"].is_string());
    assert!(json["repository"].is_object());
    assert!(json["nodes"].is_array());
    assert!(json["edges"].is_array());

    // Verify we found files
    assert!(json["repository"]["fileCount"].as_u64().unwrap() > 0);

    // Verify TypeScript nodes exist
    let nodes = json["nodes"].as_array().unwrap();
    assert!(nodes.iter().any(|n| n["language"] == "typescript"));
}

#[test]
fn scan_python_fixture() {
    let path = fixtures_dir().join("python");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON output");
    assert!(json["repository"]["fileCount"].as_u64().unwrap() > 0);

    let nodes = json["nodes"].as_array().unwrap();
    assert!(nodes.iter().any(|n| n["language"] == "python"));
}

#[test]
fn scan_go_fixture() {
    let path = fixtures_dir().join("go");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON output");
    assert!(json["repository"]["fileCount"].as_u64().unwrap() > 0);

    let nodes = json["nodes"].as_array().unwrap();
    assert!(nodes.iter().any(|n| n["language"] == "go"));
}

#[test]
fn scan_rust_fixture() {
    let path = fixtures_dir().join("rust");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON output");
    assert!(json["repository"]["fileCount"].as_u64().unwrap() > 0);

    let nodes = json["nodes"].as_array().unwrap();
    assert!(nodes.iter().any(|n| n["language"] == "rust"));
}

#[test]
fn scan_kotlin_fixture() {
    let path = fixtures_dir().join("kotlin");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON output");
    assert!(json["repository"]["fileCount"].as_u64().unwrap() > 0);

    let nodes = json["nodes"].as_array().unwrap();
    assert!(nodes.iter().any(|n| n["language"] == "kotlin"));
}

#[test]
fn scan_swift_fixture() {
    let path = fixtures_dir().join("swift");
    let output = run_trace(&["scan", path.to_str().unwrap()]);

    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON output");
    assert!(json["repository"]["fileCount"].as_u64().unwrap() > 0);

    let nodes = json["nodes"].as_array().unwrap();
    assert!(nodes.iter().any(|n| n["language"] == "swift"));
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

    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON output");
    let nodes = json["nodes"].as_array().unwrap();

    // All nodes should be TypeScript
    assert!(nodes
        .iter()
        .all(|n| n["language"] == "typescript" || n["kind"] == "repository"));
}

#[test]
fn scan_no_cache() {
    let path = fixtures_dir().join("typescript");
    let output = run_trace(&["scan", path.to_str().unwrap(), "--no-cache"]);

    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON output");
    assert_eq!(json["analysis"]["cacheEnabled"], false);
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
}
