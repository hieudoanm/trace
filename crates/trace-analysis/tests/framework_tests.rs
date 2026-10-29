//! Integration tests for framework detection on fixture repositories.

use std::path::PathBuf;

use trace_analysis::framework::{FrameworkDetector, ManifestContents};
use trace_analysis::graph::GraphBuilder;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/frameworks")
}

#[test]
fn detects_react_on_fixture() {
    let path = fixtures_dir().join("react");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();
    let names: Vec<&str> = result.frameworks.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"react"), "expected react, got {names:?}");
    let react = result
        .frameworks
        .iter()
        .find(|f| f.name == "react")
        .unwrap();
    assert_eq!(react.display_name, "React");
    assert!(!react.signals.is_empty());
}

#[test]
fn detects_django_on_fixture() {
    let path = fixtures_dir().join("django");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();
    let names: Vec<&str> = result.frameworks.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"django"), "expected django, got {names:?}");
    let django = result
        .frameworks
        .iter()
        .find(|f| f.name == "django")
        .unwrap();
    assert_eq!(django.display_name, "Django");
    assert_eq!(django.version.as_deref(), Some("6.1.2"));
}

#[test]
fn detects_express_on_fixture() {
    let path = fixtures_dir().join("express");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();
    let names: Vec<&str> = result.frameworks.iter().map(|f| f.name.as_str()).collect();
    assert!(
        names.contains(&"express"),
        "expected express, got {names:?}"
    );
    let express = result
        .frameworks
        .iter()
        .find(|f| f.name == "express")
        .unwrap();
    assert_eq!(express.display_name, "Express");
}

#[test]
fn detects_actix_web_on_fixture() {
    let path = fixtures_dir().join("actix-web");
    let builder = GraphBuilder::new();
    let result = builder.build_repository(&path).unwrap();
    let names: Vec<&str> = result.frameworks.iter().map(|f| f.name.as_str()).collect();
    assert!(
        names.contains(&"actix-web"),
        "expected actix-web, got {names:?}"
    );
    let actix = result
        .frameworks
        .iter()
        .find(|f| f.name == "actix-web")
        .unwrap();
    assert_eq!(actix.display_name, "Actix Web");
}

#[test]
fn detector_manifest_only_detection() {
    let mut manifests = ManifestContents::empty();
    manifests.files.insert(
        "package.json".to_string(),
        r#"{"dependencies": {"vue": "^3.3.0"}}"#.to_string(),
    );
    let detector = FrameworkDetector::new();
    let result = detector.detect(std::path::Path::new("."), &[], &manifests);
    let names: Vec<&str> = result.frameworks.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"vue"), "expected vue, got {names:?}");
}
