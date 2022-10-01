use termgraph::{Config, DirectedGraph, IDFormatter, LineGlyphs};

fn main() {
    let config = Config::new(IDFormatter::new(), 3)
        .default_colors()
        .line_glyphs(LineGlyphs::custom('v', 'h', 'c', 'd'));

    let cross_level_branch = {
        let mut tmp: DirectedGraph<usize, &str> = DirectedGraph::new();

        tmp.add_nodes([
            (0, "first"),
            (1, "second"),
            (2, "third"),
            (3, "fourth"),
            (4, "fourth"),
        ]);
        tmp.add_edges([(0, 1), (0, 2), (0, 3), (1, 3), (2, 3), (3, 4), (0, 4)]);

        tmp
    };
    termgraph::display(&cross_level_branch, &config);
}
