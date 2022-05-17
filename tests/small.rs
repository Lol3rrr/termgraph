use termgraph::DirectedGraph;

#[test]
fn display_empty() {
    let graph: DirectedGraph<usize, &str> = DirectedGraph::new();

    termgraph::display(&graph, 10);
}

#[test]
fn one_node() {
    let mut graph: DirectedGraph<usize, &str> = DirectedGraph::new();
    graph.add_nodes([(0, "test")]);

    termgraph::display(&graph, 10);
}
