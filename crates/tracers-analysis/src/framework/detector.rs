use std::collections::BTreeMap;
use std::path::Path;

use tracers_parser::ParsedFile;

use crate::framework::lookup::{
    signatures_for_base_class, signatures_for_call, signatures_for_decorator, signatures_for_import,
};
use crate::framework::manifest::ManifestContents;
use crate::framework::result::FrameworkDetectionResult;
use crate::framework::scoring::{best_confidence, extract_version, language_matches};
use crate::framework::signatures::all_signatures;
use crate::framework::types::{
    Framework, FrameworkConfidence, FrameworkSignal, FrameworkSignalKind, FrameworkSignature,
};

/// Detects frameworks used in a repository.
#[derive(Debug, Clone, Default)]
pub struct FrameworkDetector {
    signatures: Vec<FrameworkSignature>,
}

impl FrameworkDetector {
    pub fn new() -> Self {
        Self {
            signatures: all_signatures().to_vec(),
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
                    if manifest_dep_present(text, dep) {
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

/// Whether a manifest declares `dep`. A bare substring search produces false
/// positives (`react` inside `preact`, `react-dom`, or `@types/react`), so the
/// dependency name must sit on identifier boundaries: the preceding character
/// may not be alphanumeric, `_`, `-`, `.`, `/` or `@`, and the following
/// character may not be alphanumeric, `_` or `-`.
fn manifest_dep_present(text: &str, dep: &str) -> bool {
    let bytes = text.as_bytes();
    let dep_bytes = dep.as_bytes();
    let is_boundary_before =
        |b: u8| !(b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'/' | b'@'));
    // Deps ending in '/' (e.g. `@vue/`) are prefix patterns matching a whole
    // package family, so no boundary is required after them.
    let prefix_dep = dep.ends_with('/');
    let is_boundary_after = |b: u8| !(b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'));
    bytes
        .windows(dep_bytes.len())
        .enumerate()
        .any(|(i, window)| {
            window == dep_bytes
                && (i == 0 || is_boundary_before(bytes[i - 1]))
                && (prefix_dep
                    || i + dep_bytes.len() == bytes.len()
                    || is_boundary_after(bytes[i + dep_bytes.len()]))
        })
}

#[cfg(test)]
mod manifest_dep_tests {
    use super::manifest_dep_present;

    #[test]
    fn matches_quoted_and_unquoted_declarations() {
        assert!(manifest_dep_present(
            r#"{"dependencies": {"react": "18"}}"#,
            "react"
        ));
        assert!(manifest_dep_present("react = \"18\"", "react"));
        assert!(manifest_dep_present("react==18.0", "react"));
        assert!(manifest_dep_present(
            "require github.com/gin-gonic/gin v1.9.0",
            "github.com/gin-gonic/gin"
        ));
        assert!(manifest_dep_present(
            r#"import "@vue/compiler-sfc""#,
            "@vue/"
        ));
    }

    #[test]
    fn rejects_substring_false_positives() {
        assert!(!manifest_dep_present(
            r#"{"dependencies": {"preact": "10"}}"#,
            "react"
        ));
        assert!(!manifest_dep_present(
            r#"{"dependencies": {"react-dom": "18"}}"#,
            "react"
        ));
        assert!(!manifest_dep_present(
            r#"{"devDependencies": {"@types/react": "18"}}"#,
            "react"
        ));
        assert!(!manifest_dep_present("# uses react-like patterns", "react"));
        assert!(!manifest_dep_present("express-session = \"1\"", "express"));
        assert!(!manifest_dep_present(
            r#"{"deps": {"my.react": "1"}}"#,
            "react"
        ));
    }
}
