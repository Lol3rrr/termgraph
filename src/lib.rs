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
//! use termgraph::{DirectedGraph, IDFormatter, Config};
//!
//! let config = Config::new(IDFormatter::new(), 3);
//! let mut graph = DirectedGraph::new();
//! graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
//! graph.add_edges([(0, 1), (0,2), (1, 2)]);
//!
//! termgraph::display(&graph, &config);
//! ```
#![warn(missing_docs)]

mod graph;
use std::{collections::HashMap, fmt::Display, hash::Hash};

use acyclic::MinimalAcyclicDirectedGraph;
pub use graph::DirectedGraph;

mod acyclic;
pub(crate) use acyclic::AcyclicDirectedGraph;

mod grid;

mod formatter;
pub use formatter::{IDFormatter, NodeFormat, ValueFormatter};

mod config;
pub use config::{Color, Config, LineGlyphBuilder, LineGlyphs};

/// This is used to output the given Graph to the Terminal
///
/// # Usage
/// 1. Construct a [`DirectedGraph`] from your own Data-Structure
/// 2. Pass the Graph to this function along with a Configuration specifying how it looks
///
/// # Example
/// ```rust
/// use termgraph::{DirectedGraph, IDFormatter, Config};
///
/// let config = Config::new(IDFormatter::new(), 3);
/// let mut graph = DirectedGraph::new();
/// graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
/// graph.add_edges([(0, 1), (0,2), (1, 2)]);
///
/// termgraph::display(&graph, &config);
/// ```
pub fn display<ID, T>(graph: &DirectedGraph<ID, T>, config: &Config<ID, T>)
where
    ID: Hash + Eq + Display,
{
    if graph.is_empty() {
        return;
    }

    let (agraph, reved_edges) = graph.to_acyclic();
    let levels = levels(&agraph, config.max_per_layer);

    // TODO
    // Perform permutations on each Level to reduce the crossings of different Paths

    let grid = grid::Grid::construct(&agraph, levels, reved_edges, config.formatter.as_ref());
    grid.display(config.color_palette.as_ref(), &config.line_glyphs);
    println!();
}

fn levels<'g, ID, T>(
    agraph: &AcyclicDirectedGraph<'g, ID, T>,
    max_per_level: usize,
) -> Vec<Vec<&'g ID>>
where
    ID: Hash + Eq,
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
    ID: Hash + Eq,
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
