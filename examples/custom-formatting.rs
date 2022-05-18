use std::fmt::Display;

use termgraph::{Config, DirectedGraph, IDFormatter, NodeFormatter, ValueFormatter};

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
    let id_config = Config::new(IDFormatter::new(), 3).default_colors();
    termgraph::display(&graph, &id_config);

    println!("Value Formatter");
    let value_config = Config::new(ValueFormatter::new(), 3).default_colors();
    termgraph::display(&graph, &value_config);

    println!("Bare Formatter:");
    let bare_config = Config::new(BareFormatter {}, 3).default_colors();
    termgraph::display(&graph, &bare_config);
}
