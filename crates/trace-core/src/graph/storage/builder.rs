use crate::id;
use crate::node::{Node, NodeKind};

pub fn repository_node(name: &str) -> Node {
    Node::new(id::repository_id(), NodeKind::Repository, name)
}
