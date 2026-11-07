use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Protected,
    Private,
    Internal,
    Package,
    Unknown,
}

impl Visibility {
    pub fn parse(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "public" | "pub" | "open" => Visibility::Public,
            "protected" => Visibility::Protected,
            "private" | "priv" => Visibility::Private,
            "internal" | "fileprivate" => Visibility::Internal,
            "package" => Visibility::Package,
            _ => Visibility::Unknown,
        }
    }
}
