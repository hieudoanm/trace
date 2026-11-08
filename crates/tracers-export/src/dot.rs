use tracers_core::edge::EdgeKind;
use tracers_core::graph::Graph;
use tracers_core::node::NodeKind;

/// Export a graph to DOT format for Graphviz visualization.
pub fn to_dot(graph: &Graph) -> String {
    let mut output = String::new();
    output.push_str("digraph tracers {\n");
    output.push_str("  rankdir=LR;\n");
    output.push_str("  node [shape=box];\n");
    output.push_str("  edge [dir=forward];\n\n");

    // Add nodes
    for node in graph.nodes() {
        let label = format!("{}\\n{}", dot_escape(&node.name), node.kind.as_str());
        let color = node_kind_color(&node.kind);
        output.push_str(&format!(
            "  \"{}\" [label=\"{}\", color=\"{}\"];\n",
            dot_escape(&node.id),
            label,
            color
        ));
    }

    output.push('\n');

    // Add edges
    for edge in graph.edges() {
        let style = edge_style(&edge.kind);
        output.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\", style=\"{}\"];\n",
            dot_escape(&edge.source),
            dot_escape(&edge.target),
            edge.kind.as_str(),
            style
        ));
    }

    output.push_str("}\n");
    output
}

/// Escape a string for use inside a DOT double-quoted string.
fn dot_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Get a color for a node kind in DOT output.
fn node_kind_color(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Repository => "purple",
        NodeKind::File => "blue",
        NodeKind::Module | NodeKind::Package | NodeKind::Namespace => "cyan",
        NodeKind::Class | NodeKind::Struct | NodeKind::Interface | NodeKind::Trait => "green",
        NodeKind::Enum => "orange",
        NodeKind::Function | NodeKind::Method | NodeKind::Constructor => "red",
        NodeKind::Variable | NodeKind::Constant | NodeKind::Property | NodeKind::Parameter => {
            "gray"
        }
        NodeKind::Type => "yellow",
        NodeKind::ExternalSymbol => "lightgray",
        NodeKind::Framework => "magenta",
        NodeKind::Unknown => "black",
    }
}

/// Get a style for an edge kind in DOT output.
fn edge_style(kind: &EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "solid",
        EdgeKind::Imports | EdgeKind::Exports => "dashed",
        EdgeKind::Calls => "bold",
        EdgeKind::References => "dotted",
        EdgeKind::Inherits => "solid",
        EdgeKind::Implements => "dashed",
        EdgeKind::UsesType => "dotted",
        EdgeKind::Defines => "solid",
        EdgeKind::Overrides => "dashed",
        EdgeKind::DependsOn => "dotted",
        EdgeKind::UsesFramework => "dashed",
        EdgeKind::ImplementsPattern => "dotted",
        EdgeKind::ExtendsFramework => "dotted",
        EdgeKind::RoutesTo => "bold",
        EdgeKind::Injects => "dotted",
    }
}
