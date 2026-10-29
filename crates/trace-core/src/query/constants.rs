use crate::edge::EdgeKind;

pub const DEPENDENCY_EDGES: &[EdgeKind] = &[
    EdgeKind::Imports,
    EdgeKind::DependsOn,
    EdgeKind::UsesType,
    EdgeKind::Inherits,
    EdgeKind::Implements,
];

pub const IMPACT_EDGES: &[EdgeKind] = &[
    EdgeKind::Calls,
    EdgeKind::References,
    EdgeKind::UsesType,
    EdgeKind::Imports,
    EdgeKind::DependsOn,
    EdgeKind::Inherits,
    EdgeKind::Implements,
    EdgeKind::Overrides,
];

pub const PATH_EDGES: &[EdgeKind] = &[
    EdgeKind::Calls,
    EdgeKind::References,
    EdgeKind::UsesType,
    EdgeKind::Imports,
    EdgeKind::DependsOn,
    EdgeKind::Inherits,
    EdgeKind::Implements,
];
