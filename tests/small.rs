use termgraph::{DefaultFormatter, DirectedGraph};

#[test]
fn display_empty() {
    let graph: DirectedGraph<usize, &str> = DirectedGraph::new();
    let formatter = DefaultFormatter::new();

    termgraph::display(&graph, 10, &formatter);
}

#[test]
fn one_node() {
    let mut graph: DirectedGraph<usize, &str> = DirectedGraph::new();
    graph.add_nodes([(0, "test")]);

    let formatter = DefaultFormatter::new();

    termgraph::display(&graph, 10, &formatter);
}
