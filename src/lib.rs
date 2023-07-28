//! A Crate to output graphs on the Terminal
//!
//! # Intended Use-Case
//! This is mostly intended to help in developing other Software that uses Graphs and needs a way
//! to easily display them, either during Debugging or as Output to display to the User.
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

pub use graph::DirectedGraph;

mod acyclic;

mod grid;

mod formatter;
pub use formatter::{IDFormatter, NodeFormat, ValueFormatter};

mod config;
pub use config::{Color, Config, LineGlyphBuilder, LineGlyphs};

mod levels;

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
    fdisplay(graph, config, std::io::stdout().lock())
}

/// This function is essentially the same as [`display`], but allows you to specify the Output
/// Target.
///
/// # Example
/// Write the visual to a Vec
/// ```rust
/// use termgraph::{DirectedGraph, IDFormatter, Config};
///
/// let config = Config::new(IDFormatter::new(), 3);
/// let mut graph = DirectedGraph::new();
/// graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
/// graph.add_edges([(0, 1), (0,2), (1, 2)]);
///
/// let mut target = Vec::new();
/// termgraph::fdisplay(&graph, &config, &mut target);
/// ```
pub fn fdisplay<ID, T, W>(graph: &DirectedGraph<ID, T>, config: &Config<ID, T>, mut dest: W)
where
    ID: Hash + Eq + Display,
    W: std::io::Write,
{
    if graph.is_empty() {
        return;
    }

    let (agraph, reved_edges) = graph.to_acyclic();

    let names: HashMap<&ID, String> = agraph
        .nodes
        .iter()
        .map(|(id, value)| (*id, config.formatter.format_node(*id, value)))
        .collect();

    let levels = levels::GraphLevels::construct(&agraph, config, &names);

    let grid = grid::Grid::construct(&agraph, levels.0.clone(), reved_edges, config, names);
    grid.fdisplay(
        config.color_palette.as_ref(),
        &config.line_glyphs,
        &mut dest,
    );
    let _ = writeln!(dest);
}
