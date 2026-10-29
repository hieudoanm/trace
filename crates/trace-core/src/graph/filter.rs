use crate::language::Language;
use crate::node::{Node, NodeKind};

/// Filter used by `trace graph` and the query API.
#[derive(Debug, Clone, Default)]
pub struct NodeFilter {
    pub languages: Vec<Language>,
    pub kinds: Vec<NodeKind>,
    pub file_prefix: Option<String>,
    pub name: Option<String>,
    pub exported_only: bool,
    pub include_external: bool,
}

impl NodeFilter {
    pub fn matches(&self, node: &Node) -> bool {
        if !self.include_external && node.kind == NodeKind::ExternalSymbol {
            return false;
        }
        if !self.languages.is_empty() {
            match node.language {
                Some(language) if self.languages.contains(&language) => {}
                _ => return false,
            }
        }
        if !self.kinds.is_empty() && !self.kinds.contains(&node.kind) {
            return false;
        }
        if let Some(prefix) = &self.file_prefix {
            match &node.file {
                Some(file) if file.starts_with(prefix) => {}
                _ => return false,
            }
        }
        if let Some(name) = &self.name {
            let matches_name = node.name == *name
                || node.qualified_name.as_deref() == Some(name.as_str())
                || node.id == *name;
            if !matches_name {
                return false;
            }
        }
        if self.exported_only && node.exported != Some(true) {
            return false;
        }
        true
    }
}
