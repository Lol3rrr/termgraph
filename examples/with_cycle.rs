use termgraph::{Config, DirectedGraph, IDFormatter};

fn main() {
    let config = Config::new(IDFormatter::new(), 3).default_colors();

    let graph = {
        let mut tmp = DirectedGraph::new();

        tmp.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
        tmp.add_edges([(0, 1), (1, 2), (1, 0)]);

        tmp
    };
    termgraph::display(&graph, &config);

    let graph2 = {
        let mut tmp = DirectedGraph::new();

        tmp.add_nodes([
            (0, "first"),
            (1, "second"),
            (2, "third"),
            (3, "fourth"),
            (4, "fifth"),
        ]);
        tmp.add_edges([(0, 1), (1, 2), (2, 3), (3, 4), (3, 2)]);

        tmp
    };
    termgraph::display(&graph2, &config);
}
