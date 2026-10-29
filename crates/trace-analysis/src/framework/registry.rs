//! Framework signature database.
//!
//! Each entry describes how to recognise one framework from imports,
//! decorators, base classes, calls, and package manifests. Matches are
//! deliberately conservative: substring prefix matching on imports and exact
//! matching on decorator/base-class/call names.

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

/// All known framework signatures, ordered for deterministic iteration.
pub const FRAMEWORK_SIGNATURES: &[FrameworkSignature] = &[
    FrameworkSignature {
        name: "react",
        display_name: "React",
        family: FrameworkFamily::EcmaScript,
        import_prefixes: &["react", "react-dom"],
        decorators: &[],
        base_classes: &["React.Component", "React.PureComponent", "React.FC"],
        calls: &[],
        manifest_files: &["package.json"],
        manifest_deps: &["react", "react-dom"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "vue",
        display_name: "Vue",
        family: FrameworkFamily::EcmaScript,
        import_prefixes: &["vue", "@vue/"],
        decorators: &[],
        base_classes: &[],
        calls: &["defineComponent", "createApp", "defineAsyncComponent"],
        manifest_files: &["package.json"],
        manifest_deps: &["vue", "@vue/"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "angular",
        display_name: "Angular",
        family: FrameworkFamily::EcmaScript,
        import_prefixes: &["@angular/"],
        decorators: &[
            "Component",
            "Injectable",
            "NgModule",
            "Directive",
            "Pipe",
            "Input",
            "Output",
        ],
        base_classes: &[],
        calls: &[],
        manifest_files: &["package.json", "angular.json"],
        manifest_deps: &["@angular/core", "@angular/common"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "express",
        display_name: "Express",
        family: FrameworkFamily::EcmaScript,
        import_prefixes: &["express"],
        decorators: &[],
        base_classes: &[],
        calls: &["express", "Router", "application"],
        manifest_files: &["package.json"],
        manifest_deps: &["express"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "nextjs",
        display_name: "Next.js",
        family: FrameworkFamily::EcmaScript,
        import_prefixes: &["next/"],
        decorators: &[],
        base_classes: &[],
        calls: &[],
        manifest_files: &["package.json", "next.config.js", "next.config.mjs"],
        manifest_deps: &["next"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "django",
        display_name: "Django",
        family: FrameworkFamily::Python,
        import_prefixes: &["django"],
        decorators: &[],
        base_classes: &["Model", "View", "TemplateView"],
        calls: &[],
        manifest_files: &[
            "requirements.txt",
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
        ],
        manifest_deps: &["django"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "flask",
        display_name: "Flask",
        family: FrameworkFamily::Python,
        import_prefixes: &["flask"],
        decorators: &[],
        base_classes: &[],
        calls: &["Flask"],
        manifest_files: &[
            "requirements.txt",
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
        ],
        manifest_deps: &["flask"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "fastapi",
        display_name: "FastAPI",
        family: FrameworkFamily::Python,
        import_prefixes: &["fastapi"],
        decorators: &[],
        base_classes: &[],
        calls: &["FastAPI"],
        manifest_files: &[
            "requirements.txt",
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
        ],
        manifest_deps: &["fastapi"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "gin",
        display_name: "Gin",
        family: FrameworkFamily::Go,
        import_prefixes: &["github.com/gin-gonic/gin"],
        decorators: &[],
        base_classes: &[],
        calls: &["Default", "New"],
        manifest_files: &["go.mod"],
        manifest_deps: &["github.com/gin-gonic/gin"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "echo",
        display_name: "Echo",
        family: FrameworkFamily::Go,
        import_prefixes: &["github.com/labstack/echo"],
        decorators: &[],
        base_classes: &[],
        calls: &["New"],
        manifest_files: &["go.mod"],
        manifest_deps: &["github.com/labstack/echo/v4", "github.com/labstack/echo"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "actix-web",
        display_name: "Actix Web",
        family: FrameworkFamily::Rust,
        import_prefixes: &["actix_web"],
        decorators: &["get", "post", "put", "delete", "patch"],
        base_classes: &[],
        calls: &["HttpServer", "App"],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["actix-web"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "axum",
        display_name: "Axum",
        family: FrameworkFamily::Rust,
        import_prefixes: &["axum"],
        decorators: &[],
        base_classes: &[],
        calls: &["Router", "routing"],
        manifest_files: &["Cargo.toml"],
        manifest_deps: &["axum"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "spring-boot",
        display_name: "Spring Boot",
        family: FrameworkFamily::Java,
        import_prefixes: &["org.springframework"],
        decorators: &[
            "SpringBootApplication",
            "RestController",
            "Controller",
            "Service",
            "Component",
            "Autowired",
            "Entity",
        ],
        base_classes: &[],
        calls: &[],
        manifest_files: &[
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
        ],
        manifest_deps: &["spring-boot-starter", "spring-boot"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "jakarta-ee",
        display_name: "Jakarta EE",
        family: FrameworkFamily::Java,
        import_prefixes: &["jakarta."],
        decorators: &["WebServlet", "Stateless", "Singleton", "ApplicationScoped"],
        base_classes: &["HttpServlet"],
        calls: &[],
        manifest_files: &["pom.xml", "build.gradle", "build.gradle.kts"],
        manifest_deps: &["jakarta"],
        import_confidence: FrameworkConfidence::Low,
        pattern_confidence: FrameworkConfidence::Medium,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "aspnetcore",
        display_name: "ASP.NET Core",
        family: FrameworkFamily::CSharp,
        import_prefixes: &["Microsoft.AspNetCore"],
        decorators: &[
            "ApiController",
            "Controller",
            "Route",
            "HttpGet",
            "HttpPost",
        ],
        base_classes: &["ControllerBase", "Controller"],
        calls: &[],
        manifest_files: &["*.csproj", "*.sln"],
        manifest_deps: &["Microsoft.AspNetCore"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "laravel",
        display_name: "Laravel",
        family: FrameworkFamily::Php,
        import_prefixes: &["Illuminate"],
        decorators: &[],
        base_classes: &["Controller", "Model"],
        calls: &["Route::get", "Route::post", "Route::resource"],
        manifest_files: &["composer.json"],
        manifest_deps: &["laravel/framework", "illuminate/"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "symfony",
        display_name: "Symfony",
        family: FrameworkFamily::Php,
        import_prefixes: &["symfony", "Symfony"],
        decorators: &["Route", "Controller"],
        base_classes: &["AbstractController"],
        calls: &[],
        manifest_files: &["composer.json"],
        manifest_deps: &["symfony/"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
    FrameworkSignature {
        name: "rails",
        display_name: "Ruby on Rails",
        family: FrameworkFamily::Ruby,
        import_prefixes: &["rails"],
        decorators: &[],
        base_classes: &[
            "ApplicationController",
            "ActiveRecord::Base",
            "ActionMailer::Base",
        ],
        calls: &[],
        manifest_files: &["Gemfile", "Gemfile.lock", "config/application.rb"],
        manifest_deps: &["rails"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];

/// Look up all signatures whose import prefixes match a given import path.
///
/// A prefix matches only at a package boundary: the import path must equal
/// the prefix, or be followed by `/` (a subpath import). Prefixes ending in
/// `/` (scoped package scopes like `@angular/`) also match any package within
/// that scope. This prevents `react-native` from matching the `react` prefix.
pub fn signatures_for_import(import_path: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| {
            sig.import_prefixes.iter().any(|prefix| {
                if import_path == *prefix {
                    return true;
                }
                let Some(rest) = import_path.strip_prefix(prefix) else {
                    return false;
                };
                rest.is_empty() || rest.starts_with('/') || prefix.ends_with('/')
            })
        })
        .collect()
}

/// Look up all signatures whose decorators contain a given name.
pub fn signatures_for_decorator(decorator: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| sig.decorators.iter().any(|d| d == &decorator))
        .collect()
}

/// Look up all signatures whose base classes contain a given name.
pub fn signatures_for_base_class(base: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| sig.base_classes.iter().any(|b| b == &base))
        .collect()
}

/// Look up all signatures whose calls contain a given name.
pub fn signatures_for_call(call: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| sig.calls.iter().any(|c| c == &call))
        .collect()
}

/// Look up all signatures whose manifest dependencies contain a given dep.
pub fn signatures_for_manifest_dep(dep: &str) -> Vec<&'static FrameworkSignature> {
    FRAMEWORK_SIGNATURES
        .iter()
        .filter(|sig| sig.manifest_deps.iter().any(|d| dep.contains(*d)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_matches_react() {
        let sigs = signatures_for_import("react");
        assert_eq!(sigs.len(), 1);
        assert_eq!(sigs[0].name, "react");
    }

    #[test]
    fn import_matches_angular() {
        let sigs = signatures_for_import("@angular/core");
        assert_eq!(sigs.len(), 1);
        assert_eq!(sigs[0].name, "angular");
    }

    #[test]
    fn decorator_matches_spring() {
        let sigs = signatures_for_decorator("RestController");
        assert_eq!(sigs.len(), 1);
        assert_eq!(sigs[0].name, "spring-boot");
    }

    #[test]
    fn base_class_matches_django() {
        let sigs = signatures_for_base_class("Model");
        assert!(sigs.iter().any(|s| s.name == "django"));
        assert!(sigs.iter().any(|s| s.name == "laravel"));
    }

    #[test]
    fn manifest_dep_matches_rails() {
        let sigs = signatures_for_manifest_dep("rails 7.1");
        assert_eq!(sigs.len(), 1);
        assert_eq!(sigs[0].name, "rails");
    }

    #[test]
    fn no_false_positive_for_similar_imports() {
        let sigs = signatures_for_import("react-native");
        assert_eq!(sigs.len(), 0);
        let sigs = signatures_for_import("react-query");
        assert_eq!(sigs.len(), 0);
        // Subpath imports still match.
        let sigs = signatures_for_import("react/jsx-runtime");
        assert_eq!(sigs.len(), 1);
        assert_eq!(sigs[0].name, "react");
    }
}
