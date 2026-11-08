#[allow(unused_imports)]
use super::*;

#[test]
fn defaults_are_syntax_only() {
    let config = TracersConfig::default();
    assert_eq!(config.analysis.semantic, SemanticMode::Off);
    assert!(config.cache.enabled);
    assert!(config.languages().is_empty());
    config.validate().unwrap();
}

#[test]
fn rejects_unknown_keys_and_bad_languages() {
    let error = toml::from_str::<TracersConfig>("[analysis]\nlangauges = []\n").unwrap_err();
    assert!(error.to_string().contains("langauges"));

    let config = TracersConfig {
        analysis: AnalysisConfig {
            languages: vec!["cobol".to_string()],
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn loads_optional_file() {
    let dir = tempfile::tempdir().unwrap();
    let (config, source) = TracersConfig::load(dir.path(), None).unwrap();
    assert!(source.is_none());
    assert!(config.cache.enabled);

    std::fs::write(
        dir.path().join(CONFIG_FILE_NAME),
        "[analysis]\nlanguages = [\"rust\"]\n",
    )
    .unwrap();
    let (config, source) = TracersConfig::load(dir.path(), None).unwrap();
    assert_eq!(source.as_deref(), Some(CONFIG_FILE_NAME));
    assert_eq!(config.languages(), vec![Language::Rust]);
}

#[test]
fn loads_discovery_jobs_and_cache_pruning_options() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(CONFIG_FILE_NAME),
        r#"
[analysis]
include = ["src/**"]
exclude = ["src/gen/**"]
jobs = 2

[cache]
max_age_days = 7
"#,
    )
    .unwrap();
    let (config, _) = TracersConfig::load(dir.path(), None).unwrap();
    config.validate().unwrap();
    assert_eq!(config.analysis.include, vec!["src/**".to_string()]);
    assert_eq!(config.analysis.exclude, vec!["src/gen/**".to_string()]);
    assert_eq!(config.analysis.jobs, Some(2));
    assert_eq!(config.cache.max_age_days, 7);
}
