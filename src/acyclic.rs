use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

#[derive(Debug)]
pub struct AcyclicDirectedGraph<'g, ID, T> {
    pub(crate) nodes: HashMap<&'g ID, &'g T>,
    edges: HashMap<&'g ID, HashSet<&'g ID>>,
}

impl<'g, ID, T> AcyclicDirectedGraph<'g, ID, T>
where
    ID: Hash + Eq,
{
    pub fn new(nodes: HashMap<&'g ID, &'g T>, edges: HashMap<&'g ID, HashSet<&'g ID>>) -> Self {
        Self { nodes, edges }
    }

    /// Performs a transitive reduction on the current acyclic graph. This means that all of the
    /// Edges `a -> c` are removed if the Edges `a -> b` and `b -> c` exist.
    pub fn transitive_reduction(&self) -> MinimalAcyclicDirectedGraph<'g, ID, T> {
        let reachable = {
            let mut reachable: HashMap<&ID, HashSet<&ID>> = HashMap::new();

            for id in self.nodes.keys() {
                if reachable.contains_key(id) {
                    continue;
                }

                let mut stack: Vec<&ID> = vec![*id];
                while let Some(id) = stack.pop() {
                    if reachable.contains_key(id) {
                        continue;
                    }

                    let succs = match self.edges.get(id) {
                        Some(s) => s,
                        None => {
                            reachable.insert(id, HashSet::new());
                            continue;
                        }
                    };
                    if succs.is_empty() {
                        reachable.insert(id, HashSet::new());
                        continue;
                    }

                    if succs.iter().all(|id| reachable.contains_key(id)) {
                        let others: HashSet<&ID> = succs
                            .iter()
                            .flat_map(|id| {
                                reachable
                                    .get(id)
                                    .expect("We previously check that it contains the Key")
                                    .iter()
                                    .copied()
                            })
                            .chain(succs.iter().copied())
                            .collect();

                        reachable.insert(id, others);

                        continue;
                    }

                    stack.push(id);
                    stack.extend(succs.iter());
                }
            }

            reachable
        };

        let mut remove_edges = HashMap::new();

        let empty_succs = HashSet::new();
        for node in self.nodes.keys() {
            let edges = self.edges.get(node).unwrap_or(&empty_succs);

            let succ_reachs: HashSet<_> = edges
                .iter()
                .flat_map(|id| {
                    reachable
                        .get(id)
                        .expect("There is an Entry in the reachable Map for every Node")
                })
                .collect();

            let unique_edges: HashSet<&ID> = edges
                .iter()
                .filter(|id| !succ_reachs.contains(id))
                .copied()
                .collect();

            let remove: HashSet<&ID> = edges.difference(&unique_edges).copied().collect();

            remove_edges.insert(*node, remove);
        }

        let n_edges: HashMap<&ID, HashSet<&ID>> = self
            .edges
            .iter()
            .map(|(from, to)| {
                let filter_targets = remove_edges.get(from).expect("");

                (
                    *from,
                    to.iter()
                        .filter(|t_id| !filter_targets.contains(*t_id))
                        .copied()
                        .collect(),
                )
            })
            .collect();

        MinimalAcyclicDirectedGraph {
            inner: AcyclicDirectedGraph {
                nodes: self.nodes.clone(),
                edges: n_edges,
            },
        }
    }

    pub fn successors(&self, node: &ID) -> Option<&HashSet<&'g ID>> {
        self.edges.get(node)
    }
}

impl<'g, ID, T> PartialEq for AcyclicDirectedGraph<'g, ID, T>
where
    ID: PartialEq + Hash + Eq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.nodes != other.nodes {
            return false;
        }
        if self.edges != other.edges {
            return false;
        }

        true
    }
}

/// This is an acyclic directed Graph that is transitively reduced so there should be no edges in
/// the form `a -> c` if the edges `a -> b` and `b -> c` exist.
///
/// This form makes the level generation easier as we can basically attempt to assign all the
/// successors of a node to the level below the node.
#[derive(Debug)]
pub struct MinimalAcyclicDirectedGraph<'g, ID, T> {
    pub(crate) inner: AcyclicDirectedGraph<'g, ID, T>,
}

impl<'g, ID, T> PartialEq for MinimalAcyclicDirectedGraph<'g, ID, T>
where
    ID: PartialEq + Hash + Eq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<'g, ID, T> MinimalAcyclicDirectedGraph<'g, ID, T>
where
    ID: Hash + Eq,
{
    /// Generates a Mapping for each Vertex to Vertices that are leading to it
    pub fn incoming_mapping(&self) -> HashMap<&'g ID, HashSet<&'g ID>> {
        let mut result: HashMap<&ID, HashSet<&ID>> = HashMap::with_capacity(self.inner.nodes.len());
        for node in self.inner.nodes.keys() {
            result.insert(*node, HashSet::new());
        }

        for (from, to) in self.inner.edges.iter() {
            for target in to {
                let entry = result.entry(target);
                let value = entry.or_default();
                value.insert(*from);
            }
        }

        result
    }

    pub fn outgoing(&self, node: &ID) -> Option<impl Iterator<Item = &'g ID> + '_> {
        let targets = self.inner.edges.get(node)?;
        Some(targets.iter().copied())
    }

    pub fn topological_sort(&self) -> Vec<&'g ID>
    where
        ID: Hash + Eq,
    {
        let incoming = self.incoming_mapping();

        let mut ordering: Vec<&ID> = Vec::new();

        let mut nodes: Vec<_> = self.inner.nodes.keys().copied().collect();

        while !nodes.is_empty() {
            let mut potential: Vec<(usize, &ID)> = nodes
                .iter()
                .enumerate()
                .filter(|(_, id)| match incoming.get(*id) {
                    Some(in_edges) => in_edges.iter().all(|id| ordering.contains(id)),
                    None => true,
                })
                .map(|(i, id)| (i, *id))
                .collect();

            // TODO
            // The Second part of the Ordering Condition is not really used/implemented
            // and may even be outright wrong

            if potential.len() == 1 {
                let (index, entry) = potential
                    .pop()
                    .expect("We previously checked that there is at least one item in it");
                ordering.push(entry);
                nodes.remove(index);
                continue;
            }

            potential.sort_by(|(_, a), (_, b)| {
                let a_incoming = match incoming.get(a) {
                    Some(i) => i,
                    None => return std::cmp::Ordering::Less,
                };
                let a_first_index = ordering
                    .iter()
                    .enumerate()
                    .find(|(_, id)| a_incoming.contains(*id))
                    .map(|(i, _)| i);

                let b_incoming = match incoming.get(b) {
                    Some(i) => i,
                    None => return std::cmp::Ordering::Greater,
                };
                let b_first_index = ordering
                    .iter()
                    .enumerate()
                    .find(|(_, id)| b_incoming.contains(*id))
                    .map(|(i, _)| i);

                a_first_index.cmp(&b_first_index)
            });

            let (_, entry) = potential.remove(0);
            let index = nodes
                .iter()
                .enumerate()
                .find(|(_, id)| **id == entry)
                .map(|(i, _)| i)
                .expect("We know that the there is at least one potential entry, so we can assume that we find that entry");
            ordering.push(entry);
            nodes.remove(index);
        }

        ordering
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_with_changes() {
        let nodes: HashMap<&i32, &&str> = [(&0, &"first"), (&1, &"second"), (&2, &"third")]
            .into_iter()
            .collect();
        let graph = AcyclicDirectedGraph::new(
            nodes.clone(),
            [
                (&0, [&1, &2].into_iter().collect()),
                (&1, [&2].into_iter().collect()),
                (&2, [].into_iter().collect()),
            ]
            .into_iter()
            .collect(),
        );

        let result = graph.transitive_reduction();

        let expected = MinimalAcyclicDirectedGraph {
            inner: AcyclicDirectedGraph::new(
                nodes,
                [
                    (&0, [&1].into_iter().collect()),
                    (&1, [&2].into_iter().collect()),
                    (&2, [].into_iter().collect()),
                ]
                .into_iter()
                .collect(),
            ),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn incoming_mapping_linear() {
        let graph = MinimalAcyclicDirectedGraph {
            inner: AcyclicDirectedGraph::new(
                [
                    (&0, &"test"),
                    (&1, &"test"),
                    (&2, &"test"),
                    (&3, &"test"),
                    (&4, &"test"),
                ]
                .into_iter()
                .collect(),
                [
                    (&0, [&1].into_iter().collect()),
                    (&1, [&2].into_iter().collect()),
                    (&2, [&3].into_iter().collect()),
                    (&3, [&4].into_iter().collect()),
                ]
                .into_iter()
                .collect(),
            ),
        };

        let mapping = graph.incoming_mapping();
        dbg!(&mapping);

        let expected: HashMap<_, HashSet<_>> = [
            (&0, [].into_iter().collect()),
            (&1, [&0].into_iter().collect()),
            (&2, [&1].into_iter().collect()),
            (&3, [&2].into_iter().collect()),
            (&4, [&3].into_iter().collect()),
        ]
        .into_iter()
        .collect();

        assert_eq!(expected, mapping);
    }

    #[test]
    fn incoming_mapping_branched() {
        let graph = MinimalAcyclicDirectedGraph {
            inner: AcyclicDirectedGraph::new(
                [
                    (&0, &"test"),
                    (&1, &"test"),
                    (&2, &"test"),
                    (&3, &"test"),
                    (&4, &"test"),
                ]
                .into_iter()
                .collect(),
                [
                    (&0, [&1, &2].into_iter().collect()),
                    (&1, [&3].into_iter().collect()),
                    (&2, [&4].into_iter().collect()),
                ]
                .into_iter()
                .collect(),
            ),
        };

        let mapping = graph.incoming_mapping();

        let expected: HashMap<_, HashSet<_>> = [
            (&0, [].into_iter().collect()),
            (&1, [&0].into_iter().collect()),
            (&2, [&0].into_iter().collect()),
            (&3, [&1].into_iter().collect()),
            (&4, [&2].into_iter().collect()),
        ]
        .into_iter()
        .collect();

        assert_eq!(expected, mapping);
    }

    #[test]
    fn topsort_linear() {
        let graphs = MinimalAcyclicDirectedGraph {
            inner: AcyclicDirectedGraph::new(
                [
                    (&0, &"test"),
                    (&1, &"test"),
                    (&2, &"test"),
                    (&3, &"test"),
                    (&4, &"test"),
                ]
                .into_iter()
                .collect(),
                [
                    (&0, [&1].into_iter().collect()),
                    (&1, [&2].into_iter().collect()),
                    (&2, [&3].into_iter().collect()),
                    (&3, [&4].into_iter().collect()),
                ]
                .into_iter()
                .collect(),
            ),
        };

        let sort = graphs.topological_sort();
        dbg!(&sort);

        let expected = vec![&0, &1, &2, &3, &4];

        assert_eq!(expected, sort);
    }

    #[test]
    fn topsort_branched() {
        let graphs = MinimalAcyclicDirectedGraph {
            inner: AcyclicDirectedGraph::new(
                [
                    (&0, &"test"),
                    (&1, &"test"),
                    (&2, &"test"),
                    (&3, &"test"),
                    (&4, &"test"),
                ]
                .into_iter()
                .collect(),
                [
                    (&0, [&1, &2].into_iter().collect()),
                    (&1, [&3].into_iter().collect()),
                    (&2, [&4].into_iter().collect()),
                ]
                .into_iter()
                .collect(),
            ),
        };

        let sort = graphs.topological_sort();
        dbg!(&sort);

        let expected1 = vec![&0, &1, &2, &3, &4];
        let expected2 = vec![&0, &2, &1, &4, &3];

        assert!(sort == expected1 || sort == expected2);
    }
}
