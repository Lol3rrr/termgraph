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
    let agraph = graph.to_acyclic();
    let levels = levels(&agraph, max_per_level);

    // TODO
    // Perform permutations on each Level to reduce the crossings of different Paths

    let grid = grid::Grid::construct(&agraph, levels);
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
            Some(out) => {
                out.map(|id| vertex_levels.get(id).unwrap_or(&0))
                    .max()
                    .unwrap()
                    + 1
            }
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