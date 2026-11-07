//! Framework signatures for PHP family (Laravel, Symfony).

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const PHP_SIGNATURES: &[FrameworkSignature] = &[
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
];
