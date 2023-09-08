use std::{collections::HashMap, fmt::Display, hash::Hash};

use crate::acyclic::AcyclicDirectedGraph;

use super::{Index, NodeNameLength};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum InternalNode<'g, ID> {
    User(&'g ID),
    Dummy {
        d_id: usize,
        src: &'g ID,
        target: &'g ID,
    },
    /// Src and Target are already in their original orientation and dont need to be flipped again
    ReverseDummy {
        d_id: usize,
        src: &'g ID,
        target: &'g ID,
    },
}

impl<'g, ID> InternalNode<'g, ID>
where
    ID: Hash + Eq + Display,
{
    pub fn successor_targets<'a, T>(
        &'a self,
        agraph: &'a AcyclicDirectedGraph<'g, ID, T>,
        first: &'a [InternalNode<'g, ID>],
        second: &'a [InternalNode<'g, ID>],
        first_entries: &'a HashMap<&InternalNode<'g, ID>, (Index, NodeNameLength)>,
        second_entries: &'a HashMap<&'a InternalNode<'g, ID>, (Index, NodeNameLength)>,
        node_names: &'a HashMap<&'a ID, String>,
    ) -> Box<dyn Iterator<Item = (&'a InternalNode<ID>, usize)> + 'a> {
        match self {
            InternalNode::User(id) => {
                let raw_succs = agraph.successors(id).cloned().unwrap_or_default();

                Box::new(raw_succs.into_iter().map(|succ_id| {
                            match second.iter().find(|second_id| {
                                match second_id {
                                    InternalNode::User(uid) => *uid == succ_id,
                                    InternalNode::Dummy { src, target, .. } => *src == *id && *target == succ_id,
                                    InternalNode::ReverseDummy { src, target, .. } => *src == *id && *target == succ_id,
                                }
                            }) {
                                Some(s) => s,
                                None => {
                                    panic!("Could not find successor Node in second {}", succ_id)
                                }
                            }
                        }).map(|t_id| {
                            let (index, in_node_offset) = match second_entries.get(t_id).copied() {
                                Some((i, len)) => {
                                    (i.0, len.0)
                                },
                                None => {
                                    unreachable!("We previously checked and inserted all missing Entries/Dummy Nodes")
                                }
                            };

                            // Calculate the Offset until the Target
                            let offset: usize = second
                                .iter()
                                .take(index)
                                .map(|id| {
                                    match id {
                                        InternalNode::User(id) => {
                                            node_names.get(id).map_or(0, String::len)
                                        }
                                        _ => 1,
                                    }
                                })
                                .sum();

                                let raw_x = index * 2 + offset + in_node_offset / 2 + 1;

                            (t_id, raw_x)
                        }))
            }
            InternalNode::Dummy { src, target, .. } => {
                Box::new(core::iter::once(second.iter().find(|second_id| {
                            match second_id {
                                InternalNode::User(uid) => uid == target,
                                InternalNode::Dummy { src: s_src, target: s_target, .. } => src == s_src && target == s_target,
                                InternalNode::ReverseDummy { .. } => false,
                            }
                        }).unwrap()).map(|t_id| {
                            let (index, in_node_offset) = match second_entries.get(t_id).copied() {
                                Some((i, len)) => {
                                    (i.0, len.0)
                                },
                                None => {
                                    unreachable!("We previously checked and inserted all missing Entries/Dummy Nodes")
                                }
                            };

                            // Calculate the Offset until the Target
                            let offset: usize = second
                                .iter()
                                .take(index)
                                .map(|id| {
                                    match id {
                                        InternalNode::User(id) => {
                                            node_names.get(id).map_or(0, String::len)
                                        }
                                        _ => 1,
                                    }
                                })
                                .sum();

                                let raw_x = index * 2 + offset + in_node_offset / 2 + 1;

                            (t_id, raw_x)
                        }))
            }
            InternalNode::ReverseDummy { src, target, .. } => {
                if let Some(same_layer) = first.iter().find(|id| match id {
                    InternalNode::User(uid) => uid == src,
                    _ => false,
                }) {
                    Box::new(core::iter::once(same_layer).map(|t_id| {
                                let (index, in_node_offset) = match first_entries.get(t_id).copied() {
                                    Some((i, len)) => {
                                        (i.0, len.0)
                                    },
                                    None => {
                                        unreachable!("We previously checked and inserted all missing Entries/Dummy Nodes")
                                    }
                                };

                                // Calculate the Offset until the Target
                                let offset: usize = first
                                    .iter()
                                    .take(index)
                                    .map(|id| {
                                        match id {
                                            InternalNode::User(id) => {
                                                node_names.get(id).map_or(0, String::len)
                                            }
                                            _ => 1,
                                        }
                                    })
                                    .sum();
                                    let raw_x = index * 2 + offset + in_node_offset / 2 + 1;

                                (t_id, raw_x)
                            }))
                } else {
                    let following_layer_iter = core::iter::once(
                        second
                            .iter()
                            .find(|second_id| match second_id {
                                InternalNode::ReverseDummy {
                                    src: s_src,
                                    target: s_target,
                                    ..
                                } => src == s_src && target == s_target,
                                _ => false,
                            })
                            .unwrap(),
                    );

                    Box::new(following_layer_iter.map(|t_id| {
                                let (index, in_node_offset) = match second_entries.get(t_id).copied() {
                                    Some((i, len)) => {
                                        (i.0, len.0)
                                    },
                                    None => {
                                        unreachable!("We previously checked and inserted all missing Entries/Dummy Nodes")
                                    }
                                };

                                // Calculate the Offset until the Target
                                let offset: usize = second
                                    .iter()
                                    .take(index)
                                    .map(|id| {
                                        match id {
                                            InternalNode::User(id) => {
                                                node_names.get(id).map_or(0, String::len)
                                            }
                                            _ => 1,
                                        }
                                    })
                                    .sum();

                                let raw_x = index * 2 + offset + in_node_offset / 2 + 1;

                                (t_id, raw_x)
                            }))
                }
            }
        }
    }
}
