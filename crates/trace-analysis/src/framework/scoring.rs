use std::collections::BTreeSet;

use trace_core::Language;

use crate::framework::manifest::ManifestContents;
use crate::framework::types::{
    FrameworkConfidence, FrameworkFamily, FrameworkSignal, FrameworkSignalKind, FrameworkSignature,
};

/// Whether a language belongs to a framework's family.
pub fn language_matches(family: FrameworkFamily, language: Language) -> bool {
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
pub fn best_confidence(signals: &[FrameworkSignal]) -> FrameworkConfidence {
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
pub fn extract_version(
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
        let Some(text) = manifests.files.get(path.as_str()) else {
            continue;
        };
        if let Some(version) = version_from_text(text, sig.manifest_deps) {
            return Some(version);
        }
    }
    None
}

/// Naive version extraction: look for `name@version`, `"name": "version"`,
/// or `name==version` patterns.
pub fn version_from_text(text: &str, deps: &[&'static str]) -> Option<String> {
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
        // Handle `name==version` / `name>=version` (requirements.txt style).
        if let Some(start) = text.find(dep) {
            let after = &text[start + dep.len()..];
            let after = after.trim_start_matches(|c: char| c.is_whitespace());
            for sep in ["==", ">=", "<=", "~=", "!==", "!= ", "> ", "< "] {
                if let Some(rest) = after.strip_prefix(sep) {
                    let end = rest
                        .find(|c: char| {
                            !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '_'
                        })
                        .unwrap_or(rest.len());
                    let version = &rest[..end];
                    if !version.is_empty() {
                        return Some(version.to_string());
                    }
                }
            }
        }
    }
    None
}
