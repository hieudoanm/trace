use std::collections::BTreeMap;

use crate::framework::types::{Framework, FrameworkSignal};

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
