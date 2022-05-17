use termgraph::{DirectedGraph, IDFormatter};

#[test]
fn display_empty() {
    let graph: DirectedGraph<usize, &str> = DirectedGraph::new();
    let formatter = IDFormatter::new();

    termgraph::display(&graph, 10, &formatter);
}

#[test]
fn one_node() {
    let mut graph: DirectedGraph<usize, &str> = DirectedGraph::new();
    graph.add_nodes([(0, "test")]);

    let formatter = IDFormatter::new();

    termgraph::display(&graph, 10, &formatter);
}
