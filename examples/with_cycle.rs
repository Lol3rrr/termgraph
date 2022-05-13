use termgraph::DirectedGraph;

fn main() {
    let graph = {
        let mut tmp = DirectedGraph::new();

        tmp.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
        tmp.add_edges([(0, 1), (1, 2), (1, 0)]);

        tmp
    };
    termgraph::display(&graph, 3);
}
