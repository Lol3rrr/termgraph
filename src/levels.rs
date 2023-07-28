use std::{collections::HashMap, hash::Hash};

use crate::{
    acyclic::{AcyclicDirectedGraph, MinimalAcyclicDirectedGraph},
    Config,
};

/// A Level contains a list of all the Nodes that should be displayed on the same logical y-level
#[derive(Debug)]
pub struct Level<'g, ID> {
    pub(crate) nodes: Vec<&'g ID>,
}

impl<'g, ID> Clone for Level<'g, ID> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
        }
    }
}

pub struct GraphLevels<'g, ID>(pub Vec<Level<'g, ID>>);

impl<'g, ID> GraphLevels<'g, ID> {
    /// Constructs the GraphLevels from the provided Graph and Config
    pub fn construct<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        config: &Config<ID, T>,
        node_names: &HashMap<&'g ID, String>,
    ) -> GraphLevels<'g, ID>
    where
        ID: Hash + Eq,
    {
        // Reduce the Graph to remove transitive Edges
        let reduced = agraph.transitive_reduction();

        // Sort the Nodes in the Graph for a better distribution across the levels
        let ordering = reduced.topological_sort();

        Self::distribute_nodes(ordering, &reduced, config, node_names)
    }

    fn distribute_nodes<T>(
        ordering: Vec<&'g ID>,
        graph: &MinimalAcyclicDirectedGraph<'g, ID, T>,
        config: &Config<ID, T>,
        node_names: &HashMap<&'g ID, String>,
    ) -> GraphLevels<'g, ID>
    where
        ID: Hash + Eq,
    {
        // The size we use here is just a rough guess as to how many levels we might need and is just
        // there to hopefully reduce the number of reallocations needed
        let mut levels: Vec<Level<'g, ID>> =
            Vec::with_capacity(graph.inner.nodes.len() / config.max_per_layer);
        // We know that every Node will be in this map, so we can preallocate the exact space needed
        let mut vertex_levels: HashMap<&'g ID, usize> =
            HashMap::with_capacity(graph.inner.nodes.len());

        for v in ordering.into_iter().rev() {
            let initial_level = match graph.outgoing(v) {
                Some(out) => out
                    .map(|id| vertex_levels.get(id).unwrap_or(&0))
                    .max()
                    .map(|m| m + 1)
                    .unwrap_or(0),
                None => 0,
            };

            for v_level in initial_level..usize::MAX {
                let level = match levels.get_mut(v_level) {
                    Some(l) => l,
                    None => {
                        levels.extend(
                            (0..(v_level + 1).saturating_sub(levels.len()))
                                .map(|_| Level { nodes: Vec::new() }),
                        );

                        levels.get_mut(v_level).expect(
                            "We previously made sure that there are enough entries in the list",
                        )
                    }
                };

                // Check for max nodes per layer
                if level.nodes.len() >= config.max_per_layer {
                    continue;
                }

                // Check for max glyphs per layer
                let current_glyph_width: usize = level
                    .nodes
                    .iter()
                    .map(|n| node_names.get(n).map(|name| name.len()).unwrap_or(0) + 2)
                    .sum();
                let current_node_width = node_names.get(v).map(|name| name.len()).unwrap_or(0);
                let upper_bound = config.glyph_width().saturating_sub(current_node_width + 3);
                if current_glyph_width >= upper_bound {
                    debug_assert!(current_node_width < config.glyph_width() - 3);
                    continue;
                }

                level.nodes.push(v);
                vertex_levels.insert(v, v_level);

                break;
            }
        }

        levels.reverse();
        GraphLevels(levels)
    }
}

#[cfg(test)]
mod tests {
    use crate::{DirectedGraph, IDFormatter};

    use super::*;

    #[test]
    fn assign_levels_spillover_maxnodes() {
        let config = Config::new(IDFormatter::new(), 1);
        let mut graph = DirectedGraph::new();
        graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
        graph.add_edges([(0, 1), (0, 2)]);

        let names: HashMap<_, _> = [].into_iter().collect();

        let (agraph, _) = graph.to_acyclic();
        let result_levels = GraphLevels::construct(&agraph, &config, &names).0;

        assert_eq!(3, result_levels.len());
        assert_eq!(1, result_levels[0].nodes.len());
        assert_eq!(1, result_levels[1].nodes.len());
        assert_eq!(1, result_levels[2].nodes.len());
    }

    #[test]
    fn assign_levels_spillover_maxwidth() {
        let config = Config::new(IDFormatter::new(), 3).max_glyphs_per_layer(14);
        let mut graph = DirectedGraph::new();
        graph.add_nodes([(0, "first"), (1, "second"), (2, "third"), (3, "fourth")]);
        graph.add_edges([(0, 1), (0, 2), (0, 3)]);

        let names: HashMap<_, _> = [
            (&0, "(0)".to_string()),
            (&1, "(1)".to_string()),
            (&2, "(2)".to_string()),
            (&3, "(3)".to_string()),
        ]
        .into_iter()
        .collect();

        let (agraph, _) = graph.to_acyclic();
        let result_levels = GraphLevels::construct(&agraph, &config, &names).0;

        assert_eq!(3, result_levels.len());
        assert_eq!(1, result_levels[0].nodes.len());
        assert_eq!(1, result_levels[1].nodes.len());
        assert_eq!(2, result_levels[2].nodes.len());
    }
}
