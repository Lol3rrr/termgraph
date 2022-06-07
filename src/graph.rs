use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use crate::acyclic::AcyclicDirectedGraph;

mod feedback_arc_set;
mod tarjan;

/// A Directed Graph that can be displayed using [`display`](crate::display)
///
/// This is a simple representation of a Graph based on adjacency lists.
/// In most cases you would want to convert your graph representation into this representation
/// for displaying purposes only.
///
/// # Example
/// ```rust
/// # use termgraph::DirectedGraph;
/// #
/// let mut graph = DirectedGraph::new();
/// graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
/// graph.add_edges([(0, 1), (1, 2)]);
/// ```
#[derive(Debug)]
pub struct DirectedGraph<ID, T> {
    nodes: HashMap<ID, T>,
    edges: HashMap<ID, HashSet<ID>>,
}

impl<ID, T> DirectedGraph<ID, T>
where
    ID: Hash + Eq,
{
    /// Creates a new empty Graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Adds the Nodes to the Graph
    pub fn add_nodes<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (ID, T)>,
    {
        for (id, e) in iter {
            self.nodes.insert(id, e);
        }
    }

    /// Adds the given Edges to the Graph
    ///
    /// # Input
    /// The Tuples returned by the Iterator should be in the Format (src, target)
    pub fn add_edges<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (ID, ID)>,
    {
        for (from, to) in iter {
            let entry = self.edges.entry(from);
            let value = entry.or_insert_with(|| HashSet::new());
            value.insert(to);
        }
    }

    /// Converts the DirectedGraph into an AcyclicDirectedGraph and also returns a List of edges
    /// that needed to be reversed to make the Graph acyclic.
    pub(crate) fn to_acyclic(&self) -> (AcyclicDirectedGraph<'_, ID, T>, Vec<(&ID, &ID)>) {
        let anodes: HashMap<_, _> = self.nodes.iter().collect();
        let mut aedges: HashMap<_, HashSet<_, _>> = self
            .edges
            .iter()
            .map(|(id, targets)| (id, targets.iter().collect()))
            .collect();

        let sccs = tarjan::sccs((&anodes, &aedges));

        // If the given Graph has no cycles, we can just return the same Nodes and Edges
        if sccs.iter().all(|s| s.len() == 1) {
            return (AcyclicDirectedGraph::new(anodes, aedges), Vec::new());
        }

        // A List of Edges that were reversed to make the Graph acyclic
        let mut reversed_edges: Vec<(&ID, &ID)> = Vec::new();

        // Break up the Cycles
        loop {
            // Determine the current Strongly-Connected-Components
            let current_sccs = tarjan::sccs((&anodes, &aedges));
            if current_sccs.iter().all(|s| s.len() == 1) {
                break;
            }

            // Get the first Strongly Connected Component with at least 2 Elements
            let mut first_scc = current_sccs.into_iter().find(|s| s.len() > 1).expect(
                "We just checked that there is at least one SCC with more than 1 Verticies",
            );

            // Get the last Entry of the Component
            let last_part = first_scc
                .pop()
                .copied()
                .expect("We know that the SCC has at least 2 Verticies");

            // The Vertices reachable from the last Component
            let last_targets = aedges.get(last_part).expect("");

            // Search for a Vertex in the Component that is reachable from the last Entry
            let first_part = first_scc
                .into_iter()
                .find(|target| last_targets.contains(*target))
                .copied()
                .expect("We know that the SCC has at least 1 more remaining Vertex");

            // Store the Edge as being reversed
            reversed_edges.push((last_part, first_part));

            // Reverse the Edges
            let last_targets = aedges.get_mut(last_part).expect("");
            last_targets.remove(first_part);
            let first_targets = aedges.get_mut(first_part).expect("");
            first_targets.insert(last_part);
        }

        (AcyclicDirectedGraph::new(anodes, aedges), reversed_edges)
    }
}

impl<ID, T> Default for DirectedGraph<ID, T>
where
    ID: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<ID, T> PartialEq for DirectedGraph<ID, T>
where
    ID: Hash + Eq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes && self.edges == other.edges
    }
}
impl<ID, T> Eq for DirectedGraph<ID, T>
where
    ID: Hash + Eq,
    T: Eq,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toacyclic_without_cycle() {
        let nodes = [(0, "first"), (1, "second"), (2, "third")];
        let edges = [(0, 1), (1, 2), (0, 2)];

        let normal = {
            let mut tmp = DirectedGraph::new();
            tmp.add_nodes(nodes);
            tmp.add_edges(edges);
            tmp
        };

        let (result_graph, reversed_edges) = normal.to_acyclic();
        let expected = AcyclicDirectedGraph::new(
            nodes.iter().map(|(id, c)| (id, c)).collect(),
            [
                (&0, [&1, &2].into_iter().collect()),
                (&1, [&2].into_iter().collect()),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(expected, result_graph);
        assert_eq!(Vec::<(&i32, &i32)>::new(), reversed_edges);
    }

    #[test]
    fn toacyclic_with_cycle() {
        let nodes = [(0, "first"), (1, "second"), (2, "third")];
        let edges = [(0, 1), (1, 2), (2, 0)];

        let normal = {
            let mut tmp = DirectedGraph::new();
            tmp.add_nodes(nodes);
            tmp.add_edges(edges);
            tmp
        };

        let (result_graph, reved_edges) = normal.to_acyclic();

        assert_eq!(1, reved_edges.len());

        // TODO
        // Determine a way to check if the Graph is truly acyclic
        let _ = result_graph;
    }
}
