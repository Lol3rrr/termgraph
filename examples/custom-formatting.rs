use std::fmt::Display;

use termgraph::{DirectedGraph, IDFormatter, NodeFormatter, ValueFormatter};

struct BareFormatter {}

impl<ID, T> NodeFormatter<ID, T> for BareFormatter
where
    ID: Display,
{
    fn format_node(&self, id: &ID, _: &T) -> String {
        format!("{}", id)
    }
}

fn main() {
    let mut graph = DirectedGraph::new();
    graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
    graph.add_edges([(0, 1), (1, 2)]);

    println!("ID Formatter:");
    termgraph::display(&graph, 3, &IDFormatter::new());

    println!("Value Formatter");
    termgraph::display(&graph, 3, &ValueFormatter::new());

    println!("Bare Formatter:");
    termgraph::display(&graph, 3, &BareFormatter {});
}
