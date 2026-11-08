use std::path::PathBuf;
use std::process::Command;

fn tracers_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../target/debug/tracers");
    path
}

fn run_tracers(args: &[&str]) -> String {
    let output = Command::new(tracers_bin())
        .args(args)
        .output()
        .expect("Failed to execute tracers");

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
    let output = run_tracers(&["scan", path.to_str().unwrap()]);

    // Verify summary output
    assert!(output.contains("=== Tracers scan summary ==="));
    assert!(output.contains("Nodes:"));
    assert!(output.contains("Edges:"));
    assert!(output.contains("Languages detected:"));
    assert!(output.contains("typescript"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_python_fixture() {
    let path = fixtures_dir().join("python");
    let output = run_tracers(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Tracers scan summary ==="));
    assert!(output.contains("python"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_go_fixture() {
    let path = fixtures_dir().join("go");
    let output = run_tracers(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Tracers scan summary ==="));
    assert!(output.contains("go"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_rust_fixture() {
    let path = fixtures_dir().join("rust");
    let output = run_tracers(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Tracers scan summary ==="));
    assert!(output.contains("rust"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_kotlin_fixture() {
    let path = fixtures_dir().join("kotlin");
    let output = run_tracers(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Tracers scan summary ==="));
    assert!(output.contains("kotlin"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn scan_swift_fixture() {
    let path = fixtures_dir().join("swift");
    let output = run_tracers(&["scan", path.to_str().unwrap()]);

    assert!(output.contains("=== Tracers scan summary ==="));
    assert!(output.contains("swift"));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn stats_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["stats", path.to_str().unwrap()]);

    assert!(output.contains("Graph statistics:"));
    assert!(output.contains("files:"));
    assert!(output.contains("nodes:"));
    assert!(output.contains("edges:"));
}

#[test]
fn languages_command() {
    let output = run_tracers(&["languages"]);

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
    let output = run_tracers(&["check", path.to_str().unwrap()]);

    assert!(output.contains("Configuration: valid"));
    assert!(output.contains("Discovery:"));
    assert!(output.contains("Languages:"));
    assert!(output.contains("Check passed."));
}

#[test]
fn scan_with_language_filter() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["scan", path.to_str().unwrap(), "--language", "typescript"]);

    assert!(output.contains("=== Tracers scan summary ==="));
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
    let output = run_tracers(&["scan", path.to_str().unwrap(), "--no-cache"]);

    assert!(output.contains("=== Tracers scan summary ==="));
    assert!(output.contains("=== Scan complete ==="));
}

#[test]
fn cycles_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["cycles", path.to_str().unwrap()]);

    // Should either show cycles or report none
    assert!(output.contains("Dependency cycles:") || output.contains("No dependency cycles"));
}

#[test]
fn scan_output_to_file() {
    let path = fixtures_dir().join("typescript");
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.json");

    run_tracers(&[
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
    assert!(dot_content.contains("digraph tracers"));

    let html_file = output_file.with_extension("html");
    assert!(html_file.exists());
    let html_content = std::fs::read_to_string(&html_file).unwrap();
    assert!(html_content.contains("Tracers Graph Explorer"));
}

#[test]
fn scan_outputs_all_files_to_tracers_dir() {
    let temp_repo = tempfile::tempdir().unwrap();
    let src_file = temp_repo.path().join("main.ts");
    std::fs::write(
        &src_file,
        "export function add(a: number, b: number) { return a + b; }",
    )
    .unwrap();

    run_tracers(&["scan", temp_repo.path().to_str().unwrap()]);

    let tracers_dir = temp_repo.path().join(".tracers");
    assert!(tracers_dir.exists());

    let json_file = tracers_dir.join("graph.json");
    assert!(json_file.exists());
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json_file).unwrap()).expect("Invalid JSON");
    assert!(json["nodes"].is_array());

    let dot_file = tracers_dir.join("graph.dot");
    assert!(dot_file.exists());
    let dot = std::fs::read_to_string(&dot_file).unwrap();
    assert!(dot.contains("digraph tracers"));

    let html_file = tracers_dir.join("graph.html");
    assert!(html_file.exists());
    let html = std::fs::read_to_string(&html_file).unwrap();
    assert!(html.contains("Tracers Graph Explorer"));
}

#[test]
fn callers_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["callers", "twice", path.to_str().unwrap()]);

    assert!(output.contains("Callers of `twice`:"));
    assert!(output.contains("run"));
    assert!(output.contains("via calls"));
}

#[test]
fn callees_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["callees", "run", path.to_str().unwrap()]);

    assert!(output.contains("Callees of `run`:"));
    assert!(output.contains("twice"));
}

#[test]
fn refs_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["refs", "twice", path.to_str().unwrap()]);

    assert!(output.contains("References of `twice`:"));
    assert!(output.contains("run"));
}

#[test]
fn deps_command_accepts_file_target() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["deps", "src/math.ts", path.to_str().unwrap()]);

    assert!(output.contains("Dependencies of `src/math.ts`"));
}

#[test]
fn impact_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["impact", "twice", path.to_str().unwrap()]);

    assert!(output.contains("Impact of `twice`"));
    assert!(output.contains("run"));
}

#[test]
fn path_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["path", "run", "twice", path.to_str().unwrap()]);

    assert!(output.contains("Path from `run` to `twice`:"));
    assert!(output.contains("--calls"));
}

#[test]
fn roots_command() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["roots", path.to_str().unwrap()]);

    assert!(output.contains("entry-point candidates"));
    assert!(output.contains("run"));
    assert!(output.contains("not a dead-code report"));
}

#[test]
fn query_unknown_symbol_errors() {
    let path = fixtures_dir().join("typescript");
    let output = Command::new(tracers_bin())
        .args(["callers", "definitely_not_a_symbol", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute tracers");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no symbol matches"));
}

#[test]
fn graph_kind_filter() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["graph", path.to_str().unwrap(), "--kind", "function"]);
    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON");
    let nodes = json["nodes"].as_array().expect("nodes array");
    assert!(!nodes.is_empty());
    assert!(nodes.iter().all(|n| n["kind"] == "function"));
}

#[test]
fn graph_rejects_unknown_format() {
    let path = fixtures_dir().join("typescript");
    let output = Command::new(tracers_bin())
        .args(["graph", path.to_str().unwrap(), "-f", "xml"])
        .output()
        .expect("Failed to execute tracers");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported graph format"));
}

#[test]
fn scan_reports_tracers_dir_write() {
    let temp_repo = tempfile::tempdir().unwrap();
    std::fs::write(
        temp_repo.path().join("main.ts"),
        "export function add(a: number, b: number) { return a + b; }",
    )
    .unwrap();

    let output = run_tracers(&["scan", temp_repo.path().to_str().unwrap()]);
    assert!(output.contains("Wrote graph export to"));
    assert!(output.contains(".tracers"));
}

#[test]
fn query_commands_accept_exported_graph_file() {
    let path = fixtures_dir().join("typescript");
    let temp_dir = tempfile::tempdir().unwrap();
    let graph_file = temp_dir.path().join("graph.json");

    // Produce an export once, then answer queries from it without rescanning.
    run_tracers(&[
        "graph",
        path.to_str().unwrap(),
        "-o",
        graph_file.to_str().unwrap(),
    ]);
    assert!(graph_file.exists());

    let output = run_tracers(&[
        "callers",
        "twice",
        path.to_str().unwrap(),
        "--graph",
        graph_file.to_str().unwrap(),
    ]);
    assert!(output.contains("Callers of `twice`:"));
    assert!(output.contains("run"));

    let stats = run_tracers(&[
        "stats",
        path.to_str().unwrap(),
        "--graph",
        graph_file.to_str().unwrap(),
    ]);
    assert!(stats.contains("Graph statistics:"));
}

#[test]
fn query_graph_file_rejects_bad_schema() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.json");
    std::fs::write(&bad, "{}").unwrap();
    let path = fixtures_dir().join("typescript");

    let output = Command::new(tracers_bin())
        .args([
            "callers",
            "twice",
            path.to_str().unwrap(),
            "--graph",
            bad.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute tracers");

    assert!(!output.status.success());
}

#[test]
fn deps_max_results_bounds_output() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&[
        "deps",
        "src/service.ts",
        path.to_str().unwrap(),
        "--transitive",
        "--max-results",
        "1",
    ]);

    assert!(output.contains("Dependencies of `src/service.ts`"));
    assert!(output.contains("result limit reached"));
}

#[test]
fn dependents_command_shows_reverse_dependencies() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["dependents", "src/math.ts", path.to_str().unwrap()]);

    assert!(output.contains("Dependents of `src/math.ts`:"));
    assert!(output.contains("src/service.ts"));
    assert!(output.contains("via imports"));
}

#[test]
fn reach_command_lists_reachable_nodes() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["reach", "run", path.to_str().unwrap(), "--max-results", "5"]);

    assert!(output.contains("Reachable nodes of `run`:"));
    assert!(output.contains("twice"));
}

#[test]
fn diagnostics_command_lists_findings() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["diagnostics", path.to_str().unwrap()]);

    assert!(output.contains("Analysis diagnostics"));
    assert!(output.contains("unresolved_import"));
}

#[test]
fn graph_from_exported_file_applies_filters() {
    let path = fixtures_dir().join("typescript");
    let temp_dir = tempfile::tempdir().unwrap();
    let graph_file = temp_dir.path().join("graph.json");

    run_tracers(&[
        "graph",
        path.to_str().unwrap(),
        "-o",
        graph_file.to_str().unwrap(),
    ]);

    let output = run_tracers(&[
        "graph",
        path.to_str().unwrap(),
        "--graph",
        graph_file.to_str().unwrap(),
        "--kind",
        "function",
    ]);
    let json: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON");
    let nodes = json["nodes"].as_array().expect("nodes array");
    assert!(!nodes.is_empty());
    assert!(nodes.iter().all(|n| n["kind"] == "function"));
}

#[test]
fn selector_accepts_dot_slash_prefix() {
    let path = fixtures_dir().join("typescript");
    let output = run_tracers(&["deps", "./src/math.ts", path.to_str().unwrap()]);

    assert!(output.contains("Dependencies of `./src/math.ts`"));
}

#[test]
fn scan_reports_cache_activity() {
    let temp_repo = tempfile::tempdir().unwrap();
    std::fs::write(
        temp_repo.path().join("main.ts"),
        "export function add(a: number, b: number) { return a + b; }",
    )
    .unwrap();

    // First scan populates the cache; the second reports hits.
    run_tracers(&["scan", temp_repo.path().to_str().unwrap()]);
    let output = run_tracers(&["scan", temp_repo.path().to_str().unwrap()]);
    assert!(output.contains("Cache:"));
    assert!(output.contains("hits"));
}

#[test]
fn export_records_effective_languages() {
    let path = fixtures_dir().join("typescript");
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("out.json");

    run_tracers(&[
        "scan",
        path.to_str().unwrap(),
        "--language",
        "typescript",
        "-o",
        output_file.to_str().unwrap(),
        "--no-tracers-dir",
    ]);

    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output_file).unwrap()).unwrap();
    assert_eq!(
        json["analysis"]["languages"],
        serde_json::json!(["typescript"])
    );
}

#[test]
fn scan_rejects_zero_jobs() {
    let path = fixtures_dir().join("typescript");
    let output = Command::new(tracers_bin())
        .args(["scan", path.to_str().unwrap(), "--jobs", "0"])
        .output()
        .expect("Failed to execute tracers");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--jobs must be at least 1"));
}

#[test]
fn check_reports_skip_reasons_and_cache() {
    let temp_repo = tempfile::tempdir().unwrap();
    std::fs::write(
        temp_repo.path().join("main.ts"),
        "export function add(a: number, b: number) { return a + b; }",
    )
    .unwrap();
    // A generated-looking file is skipped, producing a skip-reason breakdown.
    std::fs::write(temp_repo.path().join("app.min.js"), "var x=1;").unwrap();

    let output = run_tracers(&["check", temp_repo.path().to_str().unwrap()]);

    assert!(output.contains("skip reasons:"));
    assert!(output.contains("generated"));
    assert!(output.contains("Cache:"));
}

#[test]
fn graph_html_embeds_graph_data() {
    let temp_repo = tempfile::tempdir().unwrap();
    std::fs::write(
        temp_repo.path().join("main.ts"),
        "export function add(a: number, b: number) { return a + b; }",
    )
    .unwrap();

    run_tracers(&["scan", temp_repo.path().to_str().unwrap()]);

    let html = std::fs::read_to_string(temp_repo.path().join(".tracers/graph.html")).unwrap();
    assert!(html.contains("window.__TRACERS_DATA__"));
    assert!(html.contains("function:main.ts::add") || html.contains("main.ts"));
}

#[test]
fn scan_accepts_repeated_include_and_exclude() {
    let temp_repo = tempfile::tempdir().unwrap();
    std::fs::write(temp_repo.path().join("a.ts"), "export const a = 1;").unwrap();
    std::fs::write(temp_repo.path().join("b.ts"), "export const b = 2;").unwrap();
    std::fs::write(temp_repo.path().join("c.py"), "x = 1").unwrap();

    let output = run_tracers(&[
        "scan",
        temp_repo.path().to_str().unwrap(),
        "--no-tracers-dir",
        "--include",
        "*.ts",
        "--exclude",
        "b.ts",
    ]);

    assert!(output.contains("Files scanned: 1"));
    assert!(output.contains("typescript"));
    assert!(!output.contains("python:"));
}

#[test]
fn check_honours_language_filter() {
    let temp_repo = tempfile::tempdir().unwrap();
    std::fs::write(temp_repo.path().join("a.ts"), "export const a = 1;").unwrap();
    std::fs::write(temp_repo.path().join("b.py"), "x = 1").unwrap();

    let output = run_tracers(&[
        "check",
        temp_repo.path().to_str().unwrap(),
        "--language",
        "python",
    ]);

    assert!(output.contains("files:       1"));
    assert!(output.contains("Python: 1 file(s)"));
    assert!(!output.contains("TypeScript"));
}

#[test]
fn scan_survives_unreadable_source_files() {
    let temp_repo = tempfile::tempdir().unwrap();
    std::fs::write(
        temp_repo.path().join("good.ts"),
        "export function add(a: number, b: number) { return a + b; }",
    )
    .unwrap();
    // Invalid UTF-8 with NUL bytes: previously aborted the whole scan.
    std::fs::write(temp_repo.path().join("bad.ts"), b"\xff\xfe\x00bad").unwrap();

    let output = run_tracers(&[
        "scan",
        temp_repo.path().to_str().unwrap(),
        "--no-tracers-dir",
    ]);

    assert!(output.contains("Files scanned: 1"));
    assert!(output.contains("typescript: 1 nodes") || output.contains("typescript"));
}
