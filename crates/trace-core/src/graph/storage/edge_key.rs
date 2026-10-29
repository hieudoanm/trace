use crate::edge::{Edge, EdgeKind, ResolutionStatus};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EdgeKey {
    pub(crate) source: String,
    pub(crate) kind: EdgeKind,
    pub(crate) target: String,
    pub(crate) status: ResolutionStatus,
    pub(crate) origin: crate::edge::ResolutionOrigin,
}

impl EdgeKey {
    pub(crate) fn from_edge(edge: &Edge) -> Self {
        Self {
            source: edge.source.clone(),
            kind: edge.kind,
            target: edge.target.clone(),
            status: edge.status,
            origin: edge.origin,
        }
    }
}
