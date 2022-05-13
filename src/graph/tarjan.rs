//! Based on this Algorithm
//! https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm

use std::{
    collections::HashMap,
    hash::Hash,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::DirectedGraph;

struct NodeData {
    index: Option<usize>,
    lowlink: Option<usize>,
    onstack: bool,
}

pub fn sccs<ID, T>(graph: &DirectedGraph<ID, T>) -> Vec<Vec<&ID>>
where
    ID: Hash + Eq,
{
    let mut result = Vec::new();

    let index = AtomicUsize::new(0);
    let mut stack: Vec<&ID> = Vec::new();

    let mut nodes: HashMap<&ID, _> = graph
        .nodes
        .keys()
        .map(|id| {
            (
                id,
                NodeData {
                    index: None,
                    lowlink: None,
                    onstack: false,
                },
            )
        })
        .collect();

    for id in graph.nodes.keys() {
        let data = nodes.get(id).expect("");
        if data.index.is_none() {
            strongconnect(
                id,
                &mut nodes,
                graph,
                &mut stack,
                &|| index.fetch_add(1, Ordering::SeqCst),
                &mut |ids| {
                    result.push(ids);
                },
            );
        }
    }

    result
}

fn strongconnect<'g, ID, T, I, AS>(
    node: &'g ID,
    nodes: &mut HashMap<&ID, NodeData>,
    graph: &'g DirectedGraph<ID, T>,
    stack: &mut Vec<&'g ID>,
    index_fn: &I,
    add_scc: &mut AS,
) where
    ID: Hash + Eq,
    I: Fn() -> usize,
    AS: FnMut(Vec<&'g ID>),
{
    let index = index_fn();

    let v = nodes.get_mut(node).expect("");
    v.index = Some(index);
    v.lowlink = Some(index);

    stack.push(node);
    v.onstack = true;

    if let Some(succs) = graph.edges.get(node) {
        for succ_id in succs {
            let w = nodes.get(succ_id).expect("");
            if w.index.is_none() {
                strongconnect(succ_id, nodes, graph, stack, index_fn, add_scc);

                let w = nodes.get(succ_id).expect("");
                let w_lowlink = w.lowlink.expect("");

                let v = nodes.get_mut(node).expect("");
                v.lowlink = Some(std::cmp::min(v.lowlink.expect(""), w_lowlink));
            } else if w.onstack {
                let w_index = w.index.unwrap();

                let v = nodes.get_mut(node).expect("");
                v.lowlink = Some(std::cmp::min(v.lowlink.expect(""), w_index));
            }
        }
    }

    let v = nodes.get_mut(node).expect("");
    if v.lowlink == v.index {
        let mut scc = Vec::new();

        loop {
            let w_id = stack.pop().unwrap();
            scc.push(w_id);

            let w = nodes.get_mut(w_id).expect("");
            w.onstack = false;

            if node == w_id {
                break;
            }
        }

        add_scc(scc);
    }
}
