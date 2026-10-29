//! Framework signatures for CSharp family (ASP.NET Core).

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const CSHARP_SIGNATURES: &[FrameworkSignature] = &[FrameworkSignature {
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
}];
