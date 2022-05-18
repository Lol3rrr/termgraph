use termgraph::{Config, DirectedGraph, IDFormatter};

fn main() {
    let mut graph = DirectedGraph::new();
    graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
    graph.add_edges([(0, 1), (1, 2)]);

    println!("Without Color:");
    let id_config = Config::new(IDFormatter::new(), 3);
    termgraph::display(&graph, &id_config);

    println!("With Color:");
    let value_config = Config::new(IDFormatter::new(), 3).default_colors();
    termgraph::display(&graph, &value_config);
}
