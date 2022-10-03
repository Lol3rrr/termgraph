use std::{collections::HashMap, hash::Hash};

use crate::{
    acyclic::{AcyclicDirectedGraph, MinimalAcyclicDirectedGraph},
    Config,
};

pub fn levels<'g, ID, T>(
    agraph: &AcyclicDirectedGraph<'g, ID, T>,
    config: &Config<ID, T>,
) -> Vec<Vec<&'g ID>>
where
    ID: Hash + Eq,
{
    let reduced = agraph.transitive_reduction();

    let ordering = reduced.topological_sort();

    assign_levels(ordering, &reduced, config.max_per_layer)
}

fn assign_levels<'g, ID, T>(
    ordering: Vec<&'g ID>,
    graph: &MinimalAcyclicDirectedGraph<'g, ID, T>,
    max_per_level: usize,
) -> Vec<Vec<&'g ID>>
where
    ID: Hash + Eq,
{
    let mut levels: Vec<Vec<&'g ID>> = Vec::with_capacity(graph.inner.nodes.len() / max_per_level);
    let mut vertex_levels: HashMap<&'g ID, usize> = HashMap::new();

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
                        (0..(v_level + 1).saturating_sub(levels.len())).map(|_| Vec::new()),
                    );

                    levels.get_mut(v_level).expect("")
                }
            };

            if level.len() == max_per_level {
                continue;
            }

            level.push(v);
            vertex_levels.insert(v, v_level);

            break;
        }
    }

    levels.reverse();
    levels
}

#[cfg(test)]
mod tests {
    use crate::{DirectedGraph, IDFormatter};

    use super::*;

    #[test]
    fn assign_levels_spillover() {
        let config = Config::new(IDFormatter::new(), 1);
        let mut graph = DirectedGraph::new();
        graph.add_nodes([(0, "first"), (1, "second"), (2, "third")]);
        graph.add_edges([(0, 1), (0, 2)]);

        let (agraph, _) = graph.to_acyclic();
        let result_levels = levels(&agraph, &config);

        assert_eq!(3, result_levels.len());
        assert_eq!(1, result_levels[0].len());
        assert_eq!(1, result_levels[1].len());
        assert_eq!(1, result_levels[2].len());
    }
}
