use std::collections::HashMap;

use rand::Rng;
use termgraph::{Config, DirectedGraph, IDFormatter};

const NODE_COUNT: usize = 20;

fn main() {
    let config = Config::new(IDFormatter::new(), 3).default_colors();

    let nodes: Vec<_> = (0..NODE_COUNT)
        .map(|_| rand::thread_rng().gen_range(10..100))
        .collect();

    let edge_count = rand::thread_rng().gen_range((NODE_COUNT / 2)..(NODE_COUNT - 1));

    let edges: HashMap<_, _> = (0..edge_count)
        .map(|_| {
            let src = rand::thread_rng().gen_range(0..NODE_COUNT);
            let target = loop {
                let tmp = rand::thread_rng().gen_range(0..NODE_COUNT);
                if tmp != src {
                    break tmp;
                }
            };

            (nodes[src], nodes[target])
        })
        .into_iter()
        .collect();

    let graph = {
        let mut tmp = DirectedGraph::new();

        tmp.add_nodes(nodes.into_iter().map(|i| (i, i.to_string())));
        tmp.add_edges(edges);

        tmp
    };

    termgraph::display(&graph, &config);
}
