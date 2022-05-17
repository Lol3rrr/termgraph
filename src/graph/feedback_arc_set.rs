use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

/// Based on this Paper: https://www.sciencedirect.com/science/article/pii/002001909390079O
// TODO
// This is only needed because im not yet really using it
#[allow(unused)]
pub fn calulate<'g, ID>(
    mut nodes: HashSet<&'g ID>,
    mut edges: HashMap<&'g ID, HashSet<&'g ID>>,
) -> Vec<&'g ID>
where
    ID: Eq + Hash + Debug,
{
    let mut s1: Vec<&ID> = Vec::new();
    let mut s2: Vec<&ID> = Vec::new();

    while !nodes.is_empty() {
        // Find Sink
        {
            loop {
                let node_targeted_count: HashMap<_, _> = nodes.iter().map(|id| (*id, 0)).collect();
                let pot_sink = edges
                    .iter()
                    .flat_map(|(_, targets)| targets.iter())
                    .fold(node_targeted_count, |mut acc, elem| {
                        let entry = acc.entry(*elem);
                        let value = entry.or_insert(0);
                        *value += 1;
                        acc
                    })
                    .into_iter()
                    .find(|(_, m)| *m == 0)
                    .map(|(id, _)| id);

                let sink = match pot_sink {
                    Some(s) => s,
                    None => break,
                };

                s2.insert(0, sink);
                nodes.remove(sink);
                edges.remove(sink);

                for (_, targets) in edges.iter_mut() {
                    targets.retain(|target| *target != sink);
                }
            }
        }

        // Find Source
        {
            loop {
                let pot_source = nodes
                    .iter()
                    .map(|id| (id, edges.get(id).map(|e| e.len()).unwrap_or(0)))
                    .find(|(_, es)| *es == 0)
                    .map(|(id, _)| id)
                    .copied();

                let source = match pot_source {
                    Some(s) => s,
                    None => break,
                };

                s1.push(source);
                nodes.remove(source);
                edges.remove(source);

                for (_, targets) in edges.iter_mut() {
                    targets.retain(|target| *target != source);
                }
            }
        }

        {
            if !nodes.is_empty() {
                let node_inputs: HashMap<&ID, usize> = edges
                    .iter()
                    .flat_map(|(_, targets)| targets.iter())
                    .fold(HashMap::new(), |mut acc, elem| {
                        let entry = acc.entry(*elem);
                        let value = entry.or_default();
                        *value += 1;
                        acc
                    });
                let u = nodes
                    .iter()
                    .copied()
                    .map(|id| (id, edges.get(id).map(|targets| targets.len()).unwrap_or(0)))
                    .map(|(id, out)| {
                        (
                            id,
                            out as isize,
                            node_inputs.get(id).copied().unwrap_or(0) as isize,
                        )
                    })
                    .map(|(id, out, in_)| (id, out - in_))
                    .max_by_key(|(_, v)| *v)
                    .map(|(id, _)| id)
                    .expect("");

                s1.push(u);

                nodes.remove(u);
                edges.remove(u);

                for (_, targets) in edges.iter_mut() {
                    targets.retain(|target| *target != u);
                }
            }
        }
    }

    s1.extend(s2);
    s1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_cycle() {
        let nodes: HashSet<&usize> = [&0, &1, &2].into_iter().collect();
        let edges: HashMap<&usize, HashSet<&usize>> = [
            (&0, [&1].into_iter().collect()),
            (&1, [&2].into_iter().collect()),
        ]
        .into_iter()
        .collect();

        let feedback_set = calulate(nodes, edges);

        let expected = vec![&2, &1, &0];

        assert_eq!(expected, feedback_set);
    }

    #[test]
    fn with_cycle() {
        let nodes: HashSet<&usize> = [&0, &1, &2, &3].into_iter().collect();
        let edges: HashMap<&usize, HashSet<&usize>> = [
            (&0, [&1].into_iter().collect()),
            (&1, [&2].into_iter().collect()),
            (&2, [&1, &3].into_iter().collect()),
            (&3, [].into_iter().collect()),
        ]
        .into_iter()
        .collect();

        let feedback_set = calulate(nodes, edges);
        dbg!(&feedback_set);

        assert_eq!(Some(&&3), feedback_set.first());
        assert_eq!(Some(&&0), feedback_set.last());
    }
}
