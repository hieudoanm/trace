pub mod builder;
pub mod builtins;
pub mod strings;
pub mod tree_sitter;

#[cfg(test)]
mod tests;

pub use builder::SyntaxBuilder;
pub use builtins::is_builtin_type;
pub use strings::{is_upper_camel, last_segment, signature, strip_quotes};
pub use tree_sitter::{
    child_field, children, find_descendant, first_child_of_kind, first_named, named_children,
    range, text, walk,
};
