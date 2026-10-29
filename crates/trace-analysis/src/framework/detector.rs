//! Framework detection engine.
//!
//! Examines parsed imports, decorators, heritage clauses, calls, and package
//! manifests to determine which frameworks are used in a repository. The
//! detector is conservative: a framework is reported only when an actual
//! signal (import, decorator, base class, call, manifest dependency) is
//! observed.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use trace_core::Language;
use trace_parser::ParsedFile;

use crate::framework::registry::{
    signatures_for_base_class, signatures_for_call, signatures_for_decorator, signatures_for_import,
};
use crate::framework::types::{
    Framework, FrameworkConfidence, FrameworkFamily, FrameworkSignal, FrameworkSignalKind,
    FrameworkSignature,
};

/// Contents of a discovered package manifest file, keyed by repository-relative path.
pub struct ManifestContents {
    /// Path (repository-relative) -> file text.
    pub files: BTreeMap<String, String>,
}

impl ManifestContents {
    pub fn empty() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }
}

/// Result of framework detection for one repository.
#[derive(Debug, Clone, Default)]
pub struct FrameworkDetectionResult {
    pub frameworks: Vec<Framework>,
    /// Signals grouped by framework name, used for diagnostics/reporting.
    pub signals_by_framework: BTreeMap<String, Vec<FrameworkSignal>>,
}

impl FrameworkDetectionResult {
    pub fn is_empty(&self) -> bool {
        self.frameworks.is_empty()
    }

    pub fn names(&self) -> Vec<&str> {
        self.frameworks.iter().map(|f| f.name.as_str()).collect()
    }
}

/// Detects frameworks used in a repository.
#[derive(Debug, Clone, Default)]
pub struct FrameworkDetector {
    signatures: Vec<FrameworkSignature>,
}

impl FrameworkDetector {
    pub fn new() -> Self {
        Self {
            signatures: crate::framework::registry::FRAMEWORK_SIGNATURES.to_vec(),
        }
    }

    /// Detect frameworks from parsed files, discovered imports/decorators/calls,
    /// and manifest contents.
    pub fn detect(
        &self,
        _root: &Path,
        files: &[ParsedFile],
        manifests: &ManifestContents,
    ) -> FrameworkDetectionResult {
        let mut signals: BTreeMap<String, Vec<FrameworkSignal>> = BTreeMap::new();

        for parsed in files {
            let path = &parsed.path;
            let language = parsed.language;

            // Import signals.
            for import in &parsed.imports {
                let sigs = signatures_for_import(&import.module);
                for sig in sigs {
                    let _confidence = if language_matches(sig.family, language) {
                        sig.import_confidence
                    } else {
                        FrameworkConfidence::Low
                    };
                    signals
                        .entry(sig.name.to_string())
                        .or_default()
                        .push(FrameworkSignal {
                            kind: FrameworkSignalKind::Import,
                            value: import.module.clone(),
                            file: Some(path.clone()),
                            line: Some(import.range.start.line as usize),
                        });
                }
            }

            // Decorator signals (from symbol decorators).
            for symbol in &parsed.symbols {
                for decorator in &symbol.decorators {
                    let sigs = signatures_for_decorator(decorator);
                    for sig in sigs {
                        let _confidence = if language_matches(sig.family, language) {
                            sig.pattern_confidence
                        } else {
                            FrameworkConfidence::Low
                        };
                        signals
                            .entry(sig.name.to_string())
                            .or_default()
                            .push(FrameworkSignal {
                                kind: FrameworkSignalKind::Decorator,
                                value: decorator.clone(),
                                file: Some(path.clone()),
                                line: Some(symbol.range.start.line as usize),
                            });
                    }
                }
            }

            // Heritage / base class signals.
            for heritage in &parsed.heritage {
                let sigs = signatures_for_base_class(&heritage.name);
                for sig in sigs {
                    let _confidence = if language_matches(sig.family, language) {
                        sig.pattern_confidence
                    } else {
                        FrameworkConfidence::Low
                    };
                    signals
                        .entry(sig.name.to_string())
                        .or_default()
                        .push(FrameworkSignal {
                            kind: FrameworkSignalKind::BaseClass,
                            value: heritage.name.clone(),
                            file: Some(path.clone()),
                            line: Some(heritage.range.start.line as usize),
                        });
                }
            }

            // Call signals.
            for call in &parsed.calls {
                let sigs = signatures_for_call(&call.name);
                for sig in sigs {
                    let _confidence = if language_matches(sig.family, language) {
                        sig.pattern_confidence
                    } else {
                        FrameworkConfidence::Low
                    };
                    signals
                        .entry(sig.name.to_string())
                        .or_default()
                        .push(FrameworkSignal {
                            kind: FrameworkSignalKind::Call,
                            value: call.name.clone(),
                            file: Some(path.clone()),
                            line: Some(call.range.start.line as usize),
                        });
                }
            }
        }

        // Manifest signals.
        for (path, text) in &manifests.files {
            for sig in &self.signatures {
                for dep in sig.manifest_deps {
                    if text.contains(dep) {
                        signals
                            .entry(sig.name.to_string())
                            .or_default()
                            .push(FrameworkSignal {
                                kind: FrameworkSignalKind::Manifest,
                                value: dep.to_string(),
                                file: Some(path.clone()),
                                line: None,
                            });
                    }
                }
            }
        }

        // Build framework entries from accumulated signals.
        let mut frameworks = Vec::new();
        for (name, signal_list) in &signals {
            let Some(sig) = self.signatures.iter().find(|s| s.name == name) else {
                continue;
            };
            let confidence = best_confidence(signal_list);
            let version = extract_version(manifests, sig, signal_list);
            frameworks.push(Framework {
                name: name.to_string(),
                display_name: sig.display_name.to_string(),
                family: sig.family,
                version,
                confidence,
                signals: signal_list.clone(),
            });
        }

        frameworks.sort_by(|a, b| a.name.cmp(&b.name));

        FrameworkDetectionResult {
            frameworks,
            signals_by_framework: signals,
        }
    }
}

/// Whether a language belongs to a framework's family.
fn language_matches(family: FrameworkFamily, language: Language) -> bool {
    match family {
        FrameworkFamily::EcmaScript => {
            matches!(
                language,
                Language::TypeScript | Language::Tsx | Language::JavaScript | Language::Jsx
            )
        }
        FrameworkFamily::Python => matches!(language, Language::Python),
        FrameworkFamily::Go => matches!(language, Language::Go),
        FrameworkFamily::Rust => matches!(language, Language::Rust),
        FrameworkFamily::Kotlin => matches!(language, Language::Kotlin),
        FrameworkFamily::Java => matches!(language, Language::Java),
        FrameworkFamily::CSharp => matches!(language, Language::CSharp),
        FrameworkFamily::Php => matches!(language, Language::Php),
        FrameworkFamily::Ruby => matches!(language, Language::Ruby),
    }
}

/// Compute the best confidence for a framework from its signals.
fn best_confidence(signals: &[FrameworkSignal]) -> FrameworkConfidence {
    let mut has_pattern = false;
    let mut has_import = false;
    let mut has_manifest = false;
    let mut distinct_kinds = BTreeSet::new();
    for signal in signals {
        distinct_kinds.insert(signal.kind);
        match signal.kind {
            FrameworkSignalKind::Decorator
            | FrameworkSignalKind::BaseClass
            | FrameworkSignalKind::Call => has_pattern = true,
            FrameworkSignalKind::Import => has_import = true,
            FrameworkSignalKind::Manifest => has_manifest = true,
        }
    }
    if distinct_kinds.len() >= 2 && (has_pattern || has_import || has_manifest) {
        return FrameworkConfidence::High;
    }
    if has_pattern {
        return FrameworkConfidence::High;
    }
    if has_import || has_manifest {
        return FrameworkConfidence::Medium;
    }
    FrameworkConfidence::Low
}

/// Extract a version hint from manifest files when available.
fn extract_version(
    manifests: &ManifestContents,
    sig: &FrameworkSignature,
    signals: &[FrameworkSignal],
) -> Option<String> {
    for signal in signals {
        if signal.kind != FrameworkSignalKind::Manifest {
            continue;
        }
        let Some(path) = &signal.file else {
            continue;
        };
        let Some(text) = manifests.files.get(path) else {
            continue;
        };
        if let Some(version) = version_from_text(text, sig.manifest_deps) {
            return Some(version);
        }
    }
    None
}

/// Naive version extraction: look for `name@version`, `"name": "version"`,
/// or `"name", "version"` patterns.
fn version_from_text(text: &str, deps: &[&'static str]) -> Option<String> {
    for dep in deps {
        // Handle `name@version` (yarn/pnpm style).
        if let Some(start) = text.find(dep) {
            let after = &text[start + dep.len()..];
            if let Some(at) = after.find('@') {
                let rest = &after[at + 1..];
                let end = rest
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '_')
                    .unwrap_or(rest.len());
                let version = &rest[..end];
                if !version.is_empty() {
                    return Some(version.to_string());
                }
            }
        }
        // Handle `"name": "version"` (package.json / composer.json style).
        let pattern = format!("\"{}\"", dep);
        if let Some(start) = text.find(&pattern) {
            let after = &text[start + pattern.len()..];
            // Skip whitespace and the separator (: or ,) after the dep name.
            let after =
                after.trim_start_matches(|c: char| c.is_whitespace() || c == ',' || c == ':');
            if let Some(rest) = after.strip_prefix('"') {
                let end = rest.find('"').unwrap_or(rest.len());
                let version = &rest[..end];
                if !version.is_empty() {
                    return Some(version.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use trace_core::{language::Language, location::Position};
    use trace_parser::{
        CallData, CallKind, HeritageData, HeritageRelation, ImportData, ImportItem, ImportKind,
        ParsedFile,
    };

    fn range(line: u32) -> trace_core::SourceRange {
        trace_core::SourceRange::new(Position::new(line, 1, 0), Position::new(line, 10, 9))
    }

    fn parsed_ts(path: &str) -> ParsedFile {
        ParsedFile {
            path: path.to_string(),
            language: Language::TypeScript,
            source_hash: "test".to_string(),
            module_name: None,
            symbols: Vec::new(),
            imports: Vec::new(),
            calls: Vec::new(),
            type_refs: Vec::new(),
            heritage: Vec::new(),
            locals: Vec::new(),
            diagnostics: Vec::new(),
            parse_error_count: 0,
        }
    }

    #[test]
    fn detects_react_from_import() {
        let mut parsed = parsed_ts("src/App.tsx");
        parsed.imports.push(ImportData {
            module: "react".to_string(),
            alias: None,
            items: vec![ImportItem {
                name: "Component".to_string(),
                alias: None,
            }],
            kind: ImportKind::Es,
            range: range(1),
            is_export: false,
        });
        let detector = FrameworkDetector::new();
        let result = detector.detect(
            std::path::Path::new("."),
            &[parsed],
            &ManifestContents::empty(),
        );
        assert!(!result.is_empty());
        let react = result
            .frameworks
            .iter()
            .find(|f| f.name == "react")
            .unwrap();
        assert_eq!(react.display_name, "React");
        assert_eq!(react.family, FrameworkFamily::EcmaScript);
    }

    #[test]
    fn detects_django_from_base_class() {
        let mut parsed = parsed_ts("models.py");
        parsed.language = Language::Python;
        parsed.heritage.push(HeritageData {
            from: 0,
            name: "Model".to_string(),
            relation: HeritageRelation::Inherits,
            range: range(3),
        });
        let detector = FrameworkDetector::new();
        let result = detector.detect(
            std::path::Path::new("."),
            &[parsed],
            &ManifestContents::empty(),
        );
        let django = result
            .frameworks
            .iter()
            .find(|f| f.name == "django")
            .unwrap();
        assert_eq!(django.confidence, FrameworkConfidence::High);
    }

    #[test]
    fn detects_express_from_call() {
        let mut parsed = parsed_ts("server.js");
        parsed.language = Language::JavaScript;
        parsed.calls.push(CallData {
            caller: None,
            receiver: None,
            name: "express".to_string(),
            full: "express()".to_string(),
            kind: CallKind::Function,
            range: range(1),
        });
        let detector = FrameworkDetector::new();
        let result = detector.detect(
            std::path::Path::new("."),
            &[parsed],
            &ManifestContents::empty(),
        );
        let express = result
            .frameworks
            .iter()
            .find(|f| f.name == "express")
            .unwrap();
        assert_eq!(express.confidence, FrameworkConfidence::High);
    }

    #[test]
    fn manifest_detects_rails() {
        let mut manifests = ManifestContents::empty();
        manifests.files.insert(
            "Gemfile".to_string(),
            "gem \"rails\", \"7.1.0\"".to_string(),
        );
        let detector = FrameworkDetector::new();
        let result = detector.detect(std::path::Path::new("."), &[], &manifests);
        let rails = result
            .frameworks
            .iter()
            .find(|f| f.name == "rails")
            .unwrap();
        assert_eq!(rails.version.as_deref(), Some("7.1.0"));
    }

    #[test]
    fn no_frameworks_when_no_signals() {
        let parsed = parsed_ts("src/plain.ts");
        let detector = FrameworkDetector::new();
        let result = detector.detect(
            std::path::Path::new("."),
            &[parsed],
            &ManifestContents::empty(),
        );
        assert!(result.is_empty());
    }

    #[test]
    fn frameworks_are_sorted_by_name() {
        let mut parsed = parsed_ts("a.ts");
        parsed.imports.push(ImportData {
            module: "vue".to_string(),
            alias: None,
            items: Vec::new(),
            kind: ImportKind::Es,
            range: range(1),
            is_export: false,
        });
        let mut parsed2 = parsed_ts("b.ts");
        parsed2.imports.push(ImportData {
            module: "react".to_string(),
            alias: None,
            items: Vec::new(),
            kind: ImportKind::Es,
            range: range(1),
            is_export: false,
        });
        let detector = FrameworkDetector::new();
        let result = detector.detect(
            std::path::Path::new("."),
            &[parsed, parsed2],
            &ManifestContents::empty(),
        );
        let names: Vec<&str> = result.frameworks.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["react", "vue"]);
    }
}
