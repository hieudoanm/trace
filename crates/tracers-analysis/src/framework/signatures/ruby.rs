//! Framework signatures for Ruby family (Ruby on Rails).

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const RUBY_SIGNATURES: &[FrameworkSignature] = &[FrameworkSignature {
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
}];
