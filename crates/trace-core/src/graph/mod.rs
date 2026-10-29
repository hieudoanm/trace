//! In-memory directed attributed graph with lookup indexes.

pub mod filter;
pub mod lookup;
pub mod resolve;
pub mod stats;
pub mod storage;
pub mod validation;

#[cfg(test)]
mod tests;

pub use filter::NodeFilter;
pub use lookup::{
    find_by_kind, find_by_name, find_by_qualified_name, in_edges, node, out_edges, require_node,
};
pub use resolve::edge_kind_counts;
pub use stats::Stats;
pub use storage::{repository_node, Graph};
pub use validation::{validate, validation_issues};
