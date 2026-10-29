//! Framework signatures for Java family (Spring Boot, Jakarta EE).

use crate::framework::types::{FrameworkConfidence, FrameworkFamily, FrameworkSignature};

pub const JAVA_SIGNATURES: &[FrameworkSignature] = &[
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
];
