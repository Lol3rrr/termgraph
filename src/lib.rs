//! A Crate to output graphs on the Terminal
//!
//! # Intended Use-Case
//! This is mostly intended to help in developing other Software that uses Graphs and needs a way
//! to easily display them, either during Debugging or as Output to display to the User.
//!
//! # Limitations
//! ## Cycles
//! Although it currently accepts cycles in the Graph, it does not really handle cycles all to well
//! and it can lead to rather confusing output. This should be improved in future releases and is
//! a known pain-point. However even with this limitation, this library should still have plenty of
//! useful use-cases.
//!
//! # Example
//! ```rust
//! use termgraph::DirectedGraph;
//!
//! let mut graph = DirectedGraph::new();
//! graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
//! graph.add_edges([(0, 1), (0,2), (1, 2)]);
//!
//! termgraph::display(&graph, 2);
//! ```

mod graph;
use std::{collections::HashMap, fmt::Debug, hash::Hash};

use acyclic::MinimalAcyclicDirectedGraph;
pub use graph::DirectedGraph;

mod acyclic;
pub(crate) use acyclic::AcyclicDirectedGraph;

mod grid;

/// This is used to output the given Graph to the Terminal
///
/// # Usage
/// 1. Construct a [`DirectedGraph`] from your own Data-Structure
/// 2. Pass the Graph to this function, along with the maximum number of Nodes
/// that should be displayed on a single line
///
/// # Example
/// ```rust
/// use termgraph::DirectedGraph;
///
/// let mut graph = DirectedGraph::new();
/// graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
/// graph.add_edges([(0, 1), (0,2), (1, 2)]);
///
/// termgraph::display(&graph, 2);
/// ```
pub fn display<ID, T>(graph: &DirectedGraph<ID, T>, max_per_level: usize)
where
    ID: Hash + Eq + Debug,
{
    if graph.is_empty() {
        return;
    }

    let (agraph, reved_edges) = graph.to_acyclic();
    let levels = levels(&agraph, max_per_level);

    // TODO
    // Perform permutations on each Level to reduce the crossings of different Paths

    let grid = grid::Grid::construct(&agraph, levels, reved_edges);
    grid.display();
    println!();
}

fn levels<'g, ID, T>(
    agraph: &AcyclicDirectedGraph<'g, ID, T>,
    max_per_level: usize,
) -> Vec<Vec<&'g ID>>
where
    ID: Hash + Eq + Debug,
{
    let reduced = agraph.transitive_reduction();

    let ordering = reduced.topological_sort();

    assign_levels(ordering, &reduced, max_per_level)
}

fn assign_levels<'g, ID, T>(
    ordering: Vec<&'g ID>,
    graph: &MinimalAcyclicDirectedGraph<'g, ID, T>,
    max_per_level: usize,
) -> Vec<Vec<&'g ID>>
where
    ID: Hash + Eq + Debug,
{
    let mut levels: Vec<Vec<&'g ID>> = Vec::new();
    let mut vertex_levels: HashMap<&'g ID, usize> = HashMap::new();

    for v in ordering.into_iter().rev() {
        let v_level = match graph.outgoing(v) {
            Some(out) => out
                .map(|id| vertex_levels.get(id).unwrap_or(&0))
                .max()
                .map(|m| m + 1)
                .unwrap_or(0),
            None => 0,
        };

        let level = match levels.get_mut(v_level) {
            Some(l) => l,
            None => {
                while levels.len() < v_level + 1 {
                    levels.push(Vec::new());
                }

                levels.get_mut(v_level).expect("")
            }
        };

        if level.len() == max_per_level {
            todo!("Max per Level reached");
        }

        level.push(v);
        vertex_levels.insert(v, v_level);
    }

    levels.reverse();
    levels
}
