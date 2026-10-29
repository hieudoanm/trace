//! Framework signatures for Go family (Gin, Echo, Cobra).

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const GO_SIGNATURES: &[FrameworkSignature] = &[
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
        name: "cobra",
        display_name: "Cobra",
        family: FrameworkFamily::Go,
        import_prefixes: &["github.com/spf13/cobra"],
        decorators: &[],
        base_classes: &[],
        calls: &["cobra.Command"],
        manifest_files: &["go.mod"],
        manifest_deps: &["github.com/spf13/cobra"],
        import_confidence: FrameworkConfidence::Medium,
        pattern_confidence: FrameworkConfidence::High,
        manifest_confidence: FrameworkConfidence::Medium,
    },
];
