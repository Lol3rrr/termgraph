use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use crate::acyclic::AcyclicDirectedGraph;

mod tarjan;

/// A Directed Graph that can be displayed using [`display`](crate::display)
#[derive(Debug)]
pub struct DirectedGraph<ID, T> {
    nodes: HashMap<ID, T>,
    edges: HashMap<ID, HashSet<ID>>,
}

impl<ID, T> DirectedGraph<ID, T>
where
    ID: Hash + Eq + Debug,
{
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_nodes<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (ID, T)>,
    {
        for (id, e) in iter {
            self.nodes.insert(id, e);
        }
    }

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

    pub(crate) fn to_acyclic(&self) -> AcyclicDirectedGraph<'_, ID, T> {
        let sccs = tarjan::sccs(self);

        if sccs.iter().all(|s| s.len() == 1) {
            let nodes = self.nodes.iter().collect();
            let edges = self
                .edges
                .iter()
                .map(|(id, targets)| (id, targets.iter().collect()))
                .collect();
            return AcyclicDirectedGraph::new(nodes, edges);
        }

        let mut current_sccs = sccs;
        while !current_sccs.iter().all(|s| s.len() == 1) {
            dbg!(&current_sccs);
            let first_scc = current_sccs.swap_remove(0);
            dbg!(&first_scc);

            todo!("Break Cycle in Component")
        }

        todo!("Breakup Cycle")
    }
}

impl<ID, T> Default for DirectedGraph<ID, T>
where
    ID: Hash + Eq + Debug,
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

        let result = normal.to_acyclic();
        let expected = AcyclicDirectedGraph::new(
            nodes.iter().map(|(id, c)| (id, c)).collect(),
            [
                (&0, [&1, &2].into_iter().collect()),
                (&1, [&2].into_iter().collect()),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(expected, result);
    }

    #[test]
    #[ignore = "Braking up cycles is not yet supported"]
    fn toacyclic_with_cycle() {
        let nodes = [(0, "first"), (1, "second"), (2, "third")];
        let edges = [(0, 1), (1, 2), (2, 0)];

        let normal = {
            let mut tmp = DirectedGraph::new();
            tmp.add_nodes(nodes);
            tmp.add_edges(edges);
            tmp
        };

        let result = normal.to_acyclic();

        let _ = result;
        unreachable!("Test not finished")
    }
}
