use std::{collections::HashMap, fmt::Display, hash::Hash};

use crate::acyclic::AcyclicDirectedGraph;

use super::{
    grid_structure::GridCoordinate, internalnode::InternalNode, Alignment, Horizontal, Index,
    NodeNameLength,
};

pub struct LevelConnection<'g, ID>(pub(super) Vec<Horizontal<'g, ID>>);

impl<'g, ID> LevelConnection<'g, ID>
where
    ID: Hash + Eq + Display,
{
    fn get_x_coord(
        target_idx: usize,
        nodes: &[InternalNode<'g, ID>],
        node_names: &HashMap<&ID, String>,
        user_id: Option<&ID>,
        max_x: usize,
        alignment: Alignment,
    ) -> usize {
        let offset: usize = nodes
            .iter()
            .take(target_idx)
            .map(|id| match id {
                InternalNode::User(id) => node_names.get(id).map_or(0, String::len),
                _ => 1,
            })
            .sum();

        let inner_align = match alignment {
            Alignment::Left => 0,
            Alignment::Center => {
                user_id.map_or(0, |id| node_names.get(id).map_or(0, |n| n.len() / 2))
            }
            Alignment::Right => {
                user_id.map_or(0, |id| node_names.get(id).map_or(0, |n| n.len() / 2))
            }
        };

        let raw_x = target_idx * 2 + offset + inner_align + 1;

        raw_x.min(max_x)
    }

    fn get_reverse_dummies(
        second: &[InternalNode<'g, ID>],
        node_names: &HashMap<&ID, String>,
        max_x: usize,
    ) -> Vec<Horizontal<'g, ID>> {
        // assert!(!second.is_empty());

        second
            .iter()
            .enumerate()
            .filter_map(|(i, n)| match n {
                InternalNode::ReverseDummy { src, target, .. } => Some((i, src, target)),
                _ => None,
            })
            .filter_map(|(src_index, src, target)| {
                let (target_index, target_user_id) =
                    second.iter().enumerate().find_map(|(i, n)| match n {
                        InternalNode::User(uid) if uid == target => Some((i, *uid)),
                        _ => None,
                    })?;

                // Calculate the Offset until the Target
                let target_x = Self::get_x_coord(
                    target_index,
                    second,
                    node_names,
                    Some(target_user_id),
                    max_x,
                    Alignment::Center,
                );

                // Calculate the Offset until the Target
                let src_x = Self::get_x_coord(
                    src_index,
                    second,
                    node_names,
                    None,
                    max_x,
                    Alignment::Center,
                );

                let sx = GridCoordinate(src_x.min(target_x));
                let tx = GridCoordinate(src_x.max(target_x));

                Some(Horizontal::BottomBottom {
                    src_x: GridCoordinate(src_x),
                    src: *src,
                    target: GridCoordinate(target_x),
                    x_bounds: (sx, tx),
                })
            })
            .collect()
    }

    fn calc_entries<'a>(
        first: &'a [InternalNode<'g, ID>],
        node_names: &HashMap<&ID, String>,
    ) -> HashMap<&'a InternalNode<'g, ID>, (Index, NodeNameLength)> {
        first
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let len = match id {
                    InternalNode::User(uid) => node_names.get(uid).map_or(0, String::len),
                    _ => 0,
                };

                (id, (Index(i), NodeNameLength(len)))
            })
            .collect()
    }

    /// Construct the connection between the two given Layers
    pub fn construct<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        first: &[InternalNode<'g, ID>],
        second: &[InternalNode<'g, ID>],
        node_names: &HashMap<&ID, String>,
        max_x: usize,
    ) -> Self {
        // Special case
        let base = Self::get_reverse_dummies(second, node_names, max_x);

        // The Entries in the second/lower level mapped to their respective X-Indices
        let first_entries: HashMap<_, (Index, NodeNameLength)> =
            Self::calc_entries(first, node_names);

        // The Entries in the second/lower level mapped to their respective X-Indices
        let second_entries: HashMap<_, (Index, NodeNameLength)> =
            Self::calc_entries(second, node_names);

        // An iterator over all the Source Entries and their respective coordinates in the first layer
        let first_src_coords = first.iter().enumerate().map(|(raw_x, e)| {
            // Calculate the Source Coordinates

            let cord = Self::get_x_coord(
                raw_x,
                first,
                node_names,
                match e {
                    InternalNode::User(id) => Some(id),
                    _ => None,
                },
                max_x,
                Alignment::Center,
            );

            (GridCoordinate(cord), e)
        });

        let mut temp_horizontal: Vec<_> = first_src_coords
            .filter_map(|(root, src_entry)| {
                // Connect the Source to its Targets in the lower Level

                // An Iterator over the Successors of the src_entry
                let succs: Box<dyn Iterator<Item = (&InternalNode<ID>, usize)>> = src_entry.successor_targets(agraph, first, second, &first_entries, &second_entries, node_names);

                let targets: Vec<_> = succs
                    .map(|(t_id, raw_x)| {
                        // Calculate the Coordinate of the Target
                        (
                            GridCoordinate(raw_x.min(max_x)),
                            matches!(t_id, InternalNode::Dummy { .. }),
                        )
                    })
                    .collect();

                if targets.is_empty() {
                    return None;
                }

                // Smallest x coordinate in the entire horizontal
                let sx = *std::iter::once(&root)
                    .chain(targets.iter().map(|t| &t.0))
                    .min()
                    .expect("We know that there is at least one item in the Iterator so there is always a min element");
                // Smallest x coordinate in the entire horizontal
                let tx = *std::iter::once(&root)
                    .chain(targets.iter().map(|t| &t.0))
                    .max()
                    .expect("We know that there is at least one item in the Iterator so there is always a max element");

                match src_entry {
                    InternalNode::User(src) | InternalNode::Dummy { src, .. } => {
                        Some(Horizontal::TopBottom {
                            src_x: root,
                            src: *src,
                            targets,
                            x_bounds: (sx, tx),
                        })
                    }
                    InternalNode::ReverseDummy { src, target, .. } => {
                        if first.iter().any(|n| match n {
                            InternalNode::User(uid) => uid == src,
                            _ => false,
                        }) {
                            let target = targets.into_iter().next().map(|(c, _)| c).expect("We previously checked that targets is not empty");
                            Some(Horizontal::TopTop { src_x: root, src: *src, target, x_bounds: (sx, tx) })
                        } else if let Some((_, _)) = second.iter().enumerate().find(|(_, n)| match n {
                            InternalNode::ReverseDummy { src: s_src, target: s_target, .. } => src == s_src && target == s_target,
                            _ => false,
                        }) {
                            let target = targets.into_iter().next().map(|(c, _)| c).expect("We previously checked that targets is not empty");

                            let sx = target.min(root);
                            let tx = target.max(root);

                            Some(Horizontal::BottomTop { src_x: target, src: *src, target: root, x_bounds: (sx, tx) })
                        } else {
                            // FIXME
                            // I have no idea why this todo is still here?

                            todo!()
                        }
                    }
                }

            })
            .collect();

        // Sorts them based on their source X-Coordinates
        // temp_horizontal.sort_unstable_by(|x1, x2| x1.src_x.cmp(&x2.src_x));

        // Sorts them based on their Targets average Coordinate, to try to avoid
        // unnecessary crossings in the Edges
        /*
        temp_horizontal.sort_by_cached_key(|hori| {
            let sum_targets: usize = hori.targets.iter().map(|cord| cord.0 .0).sum();
            let target_count = hori.targets.len().max(1);
            sum_targets / target_count
        });
        */
        temp_horizontal.extend(base);
        Self(temp_horizontal)
    }
}
