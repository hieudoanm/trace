//! Reusable query API over a graph.

pub mod constants;
pub mod convenience;
pub mod cycles;
pub mod direction;
pub mod distances;
pub mod impact;
pub mod options;
pub mod path;
pub mod tests;
pub mod traversal;
pub mod types;

pub use constants::{DEPENDENCY_EDGES, IMPACT_EDGES, PATH_EDGES};
pub use convenience::{callable_kinds, resolve_unique};
pub use options::QueryOptions;
pub use traversal::Query;
pub use types::{ImpactNode, Neighbor, PathEdge, PathResult};
