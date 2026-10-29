pub mod builder;
pub mod edge_key;
pub mod graph_add;
pub mod graph_finalize;
pub mod graph_query;
pub mod graph_struct;

use crate::diagnostic::Diagnostic;
use crate::error::{CoreError, Result};
use crate::node::Node;

pub use self::builder::repository_node;
pub use self::graph_struct::Graph;

impl Graph {
    pub fn stats(&self, diagnostics: &[Diagnostic]) -> crate::graph::stats::Stats {
        crate::graph::stats::Stats::compute(self, diagnostics)
    }

    pub fn validate(&self) -> Result<()> {
        let issues = crate::graph::validation::validation_issues(self);
        if issues.is_empty() {
            Ok(())
        } else {
            Err(CoreError::InvalidGraph(issues.join("; ")))
        }
    }

    pub fn resolve_symbols(&self, selector: &str) -> Vec<&Node> {
        crate::graph::resolve::resolve_symbols(self, selector)
    }
}
