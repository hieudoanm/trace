use std::fs;
use std::path::Path;

use super::*;
use trace_core::Language;

#[test]
fn discovers_supported_files_and_respects_excludes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "src/a.ts", "export function a() {}");
    write(root, "src/skip.py", "def b(): pass");
    write(root, "node_modules/dep/index.js", "module.exports = {}");
    write(root, "README.md", "# hi");
    write(root, "src/generated.generated.ts", "export const x = 1;");
    let options = DiscoveryOptions {
        excludes: vec!["src/skip.py".into()],
        ..Default::default()
    };
    let report = discover(root, &options).unwrap();
    let paths: Vec<&str> = report.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["README.md", "src/a.ts"]);
    assert_eq!(report.unsupported_files, 0);
    assert!(report.skipped.iter().any(|s| s.path == "src/skip.py"));
    assert!(report
        .skipped
        .iter()
        .any(|s| s.path == "src/generated.generated.ts"));
}

#[test]
fn includes_act_as_a_whitelist() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "src/a.ts", "export function a() {}");
    write(root, "src/b.ts", "export function b() {}");
    let options = DiscoveryOptions {
        includes: vec!["src/a.ts".into()],
        ..Default::default()
    };
    let report = discover(root, &options).unwrap();
    assert_eq!(report.files.len(), 1);
    assert_eq!(report.files[0].path, "src/a.ts");
}

#[test]
fn language_filter_is_applied() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "a.ts", "export const a = 1;");
    write(root, "b.py", "b = 1");
    let options = DiscoveryOptions {
        languages: vec![Language::Python],
        ..Default::default()
    };
    let report = discover(root, &options).unwrap();
    assert_eq!(report.files.len(), 1);
    assert_eq!(report.files[0].language, Language::Python);
}

#[test]
fn generated_detection_is_conservative() {
    assert!(looks_generated("src/api.pb.go"));
    assert!(looks_generated("web/app.min.js"));
    assert!(!looks_generated("src/generate.rs"));
}

fn write(root: &Path, path: &str, content: &str) {
    let full = root.join(path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}
