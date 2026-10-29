use crate::error::CoreError;
use crate::graph::storage::Graph;

pub fn validation_issues(graph: &Graph) -> Vec<String> {
    let mut issues = Vec::new();
    let ids: std::collections::HashSet<&str> =
        graph.nodes().iter().map(|n| n.id.as_str()).collect();
    if ids.len() != graph.node_count() {
        issues.push("duplicate node ids detected".to_string());
    }
    for node in graph.nodes() {
        if node.id.is_empty() {
            issues.push("node with empty id".to_string());
        }
        if node.name.is_empty() {
            issues.push(format!("node `{}` has an empty name", node.id));
        }
        if let Some(file) = &node.file {
            if file.starts_with('/') || file.contains(":\\") {
                issues.push(format!("node `{}` has an absolute file path", node.id));
            }
        }
        if let Some(range) = &node.range {
            if !range.is_valid() {
                issues.push(format!("node `{}` has an invalid source range", node.id));
            }
        }
        if let Some(parent) = &node.parent {
            if !ids.contains(parent.as_str()) {
                issues.push(format!(
                    "node `{}` references missing parent `{}`",
                    node.id, parent
                ));
            }
        }
    }
    for edge in graph.edges() {
        if !ids.contains(edge.source.as_str()) {
            issues.push(format!("edge references missing source `{}`", edge.source));
        }
        if !ids.contains(edge.target.as_str()) {
            issues.push(format!("edge references missing target `{}`", edge.target));
        }
        for range in &edge.sites {
            if !range.is_valid() {
                issues.push(format!(
                    "edge {} -{}-> {} has an invalid site range",
                    edge.source,
                    edge.kind.as_str(),
                    edge.target
                ));
            }
        }
    }
    issues
}

pub fn validate(graph: &Graph) -> Result<(), CoreError> {
    let issues = validation_issues(graph);
    if issues.is_empty() {
        Ok(())
    } else {
        Err(CoreError::InvalidGraph(issues.join("; ")))
    }
}
