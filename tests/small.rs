use termgraph::{Config, DirectedGraph, IDFormatter};

#[test]
fn display_empty() {
    let graph: DirectedGraph<usize, &str> = DirectedGraph::new();

    let config = Config::new(IDFormatter::new(), 10);

    termgraph::display(&graph, &config);
}

#[test]
fn one_node() {
    let mut graph: DirectedGraph<usize, &str> = DirectedGraph::new();
    graph.add_nodes([(0, "test")]);

    let config = Config::new(IDFormatter::new(), 10);

    termgraph::display(&graph, &config);
}
