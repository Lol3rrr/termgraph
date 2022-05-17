use termgraph::{DefaultFormatter, DirectedGraph};

fn main() {
    let formatter = DefaultFormatter::new();

    let display_graph = {
        let mut tmp: DirectedGraph<usize, String> = DirectedGraph::new();

        tmp.add_nodes([
            (0, "first".to_string()),
            (1, "second".to_string()),
            (2, "third".to_string()),
        ]);
        tmp.add_edges([(0, 1), (1, 2)]);

        tmp
    };
    termgraph::display(&display_graph, 3, &formatter);

    let branched_graph = {
        let mut tmp: DirectedGraph<usize, &str> = DirectedGraph::new();

        tmp.add_nodes([(0, "first"), (1, "second"), (2, "third"), (3, "fourth")]);
        tmp.add_edges([(0, 1), (0, 2), (2, 3), (1, 3)]);

        tmp
    };
    termgraph::display(&branched_graph, 3, &formatter);

    let square_graph = {
        let mut tmp: DirectedGraph<usize, &str> = DirectedGraph::new();

        tmp.add_nodes([(0, "first"), (1, "second"), (2, "third"), (3, "fourth")]);
        tmp.add_edges([(0, 1), (0, 2), (3, 1), (3, 2)]);

        tmp
    };
    termgraph::display(&square_graph, 3, &formatter);

    let cross_level_branch = {
        let mut tmp: DirectedGraph<usize, &str> = DirectedGraph::new();

        tmp.add_nodes([
            (0, "first"),
            (1, "second"),
            (2, "third"),
            (3, "fourth"),
            (4, "fourth"),
        ]);
        tmp.add_edges([(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (0, 4)]);

        tmp
    };
    termgraph::display(&cross_level_branch, 3, &formatter);
}
