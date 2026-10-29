use crate::language::Language;
use crate::location::SourceRange;

use super::kind::NodeKind;

impl crate::node::Node {
    pub fn new(id: impl Into<String>, kind: NodeKind, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            name: name.into(),
            qualified_name: None,
            file: None,
            range: None,
            language: None,
            parent: None,
            signature: None,
            visibility: None,
            exported: None,
            metadata: std::collections::BTreeMap::new(),
        }
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_range(mut self, range: SourceRange) -> Self {
        self.range = Some(range);
        self
    }

    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn with_qualified_name(mut self, qualified: impl Into<String>) -> Self {
        self.qualified_name = Some(qualified.into());
        self
    }

    pub fn with_exported(mut self, exported: bool) -> Self {
        self.exported = Some(exported);
        self
    }

    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }

    pub fn display_label(&self) -> String {
        match &self.qualified_name {
            Some(q) if q != &self.name => format!("{} ({})", self.name, self.kind),
            _ => format!("{} ({})", self.name, self.kind),
        }
    }
}
