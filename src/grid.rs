use std::{
    collections::{HashMap},
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::{acyclic::AcyclicDirectedGraph, levels::Level, Color, Config, LineGlyphs};

mod entry;
pub use entry::Entry;

mod grid_structure;
use grid_structure::*;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum InternalNode<'g, ID> {
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

/// A LevelEntry describes an entry in a given Level of the Graph
#[derive(Debug)]
pub enum LevelEntry<'g, ID> {
    /// A User Entry is an actual Node from the Users graph, that should be displayed
    User(&'g ID),
    /// A Dummy Entry is just a placeholder to easily support Edges that span multiple Levels
    Dummy { from: &'g ID, to: &'g ID },
}

impl<'g, ID> LevelEntry<'g, ID> {
    /// The ID of the Source
    pub fn id(&self) -> &'g ID {
        match &self {
            Self::User(s) => s,
            Self::Dummy { from, .. } => from,
        }
    }

    /// Whether or not the Entry is a User-Entry
    pub fn is_user(&self) -> bool {
        matches!(self, Self::User(_))
    }
}

impl<'g, ID> Clone for LevelEntry<'g, ID> {
    fn clone(&self) -> Self {
        match &self {
            Self::User(id) => Self::User(id),
            Self::Dummy { from, to } => Self::Dummy { from, to },
        }
    }
}

/// A Horizontal is used to connect from a single Source in the upper layer to one or multiple
/// Targets in the lower layer
#[derive(Debug)]
enum Horizontal<'g, ID> {
    TopBottom {
        /// The X-Coordinate of the Source in the upper Level
        src_x: GridCoordinate,
        /// The ID of the Source
        src: &'g ID,
        /// The X-Coordinates of the Targets in the lower Level
        targets: Vec<(GridCoordinate, bool)>,
        /// A touple of the smallest and largest x coordinates
        x_bounds: (GridCoordinate, GridCoordinate),
    },
    BottomTop {
        src_x: GridCoordinate,
        src: &'g ID,
        target: GridCoordinate,
        x_bounds: (GridCoordinate, GridCoordinate),
    },
    TopTop {
        src_x: GridCoordinate,
        src: &'g ID,
        target: GridCoordinate,
        x_bounds: (GridCoordinate, GridCoordinate),
    },
    BottomBottom {
        src_x: GridCoordinate,
        src: &'g ID,
        target: GridCoordinate,
        x_bounds: (GridCoordinate, GridCoordinate)
    }
}

impl<'g, ID> Horizontal<'g, ID> {
    pub fn x_bounds(&self) -> (GridCoordinate, GridCoordinate) {
        match self {
            Self::TopBottom { x_bounds , .. } => *x_bounds,
            Self::BottomTop { x_bounds, .. } => *x_bounds,
            Self::TopTop {  x_bounds, .. } => *x_bounds,
            Self::BottomBottom { x_bounds, .. } => *x_bounds,
        }
    }
}

impl<'g, ID> Clone for Horizontal<'g, ID> {
    fn clone(&self) -> Self {
        match self {
            Self::TopBottom { src_x, src, targets, x_bounds } => Self::TopBottom { src_x: *src_x, src, targets: targets.clone(), x_bounds: *x_bounds },
            Self::BottomTop { src_x, src, target, x_bounds } => Self::BottomTop { src_x: *src_x, src: *src, target: *target, x_bounds: *x_bounds },
            Self::TopTop { src_x, src, target, x_bounds } => Self::TopTop { src_x: *src_x, src: *src, target: *target, x_bounds: *x_bounds },
            Self::BottomBottom { src_x, src, target, x_bounds } => Self::BottomBottom { src_x: *src_x, src: *src, target: *target, x_bounds: *x_bounds },
        }
    }
}

/// The Grid which stores the generated Layout before displaying it to the User, which allows for
/// easier construction as well as modifiying already placed Entries
pub struct Grid<'g, ID>
where
    ID: Eq + Hash,
{
    /// The actual Grid Data-Structure
    inner: InnerGrid<'g, ID>,
    /// Maps from the IDs to the Names that should be displayed in the Graph
    names: HashMap<&'g ID, String>,
}

impl<'g, ID> Grid<'g, ID>
where
    ID: Hash + Eq + Display,
{
    fn generate_horizontal<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        first: &[InternalNode<'g, ID>],
        second: &[InternalNode<'g, ID>],
        node_names: &HashMap<&ID, String>,
        max_x: usize,
    ) -> Vec<Horizontal<'g, ID>> {
        // Special case
        let base: Vec<_> = {
            // assert!(!second.is_empty());

            let reverse_dummies = second.iter().enumerate().filter_map(|(i, n)| match n {
                InternalNode::ReverseDummy { d_id, src, target } => Some((i, src, target)),
                _ => None,
            });

            reverse_dummies.filter_map(|(src_index, src, target)| {
                let (target_index, target_user_id) = second.iter().enumerate().find_map(|(i, n)| {
                    match n {
                        InternalNode::User(uid) if uid == target => Some((i, *uid)),
                        _ => None,
                    }
                })?;


                // Calculate the Offset until the Target
                let offset: usize = second
                    .iter()
                    .take(target_index)
                    .map(|id| {
                        match id {
                            InternalNode::User(id) => {
                                node_names.get(id).map(|n| n.len()).unwrap_or(0)
                            }
                            _ => 1,
                        }
                    })
                    .sum();

                let raw_x = target_index * 2 + offset + node_names.get(target_user_id).map(|n| n.len()/2).unwrap_or(0) + 1;
                let target_x = raw_x.min(max_x);

                // Calculate the Offset until the Target
                let offset: usize = second
                    .iter()
                    .take(src_index)
                    .map(|id| {
                        match id {
                            InternalNode::User(id) => {
                                node_names.get(id).map(|n| n.len()).unwrap_or(0)
                            }
                            _ => 1,
                        }
                    })
                    .sum();

                let raw_x = src_index * 2 + offset + 1;
                let src_x = raw_x.min(max_x);

                let sx = GridCoordinate(src_x.min(target_x));
                let tx = GridCoordinate(src_x.max(target_x));

                Some(Horizontal::BottomBottom { src_x: GridCoordinate(src_x), src: *src, target: GridCoordinate(target_x), x_bounds: (sx, tx) })
            }).collect()
        };

        #[derive(Clone, Copy)]
        struct NodeNameLength(usize);

        #[derive(Clone, Copy)]
        struct Index(usize);

        // The Entries in the second/lower level mapped to their respective X-Indices
        let first_entries: HashMap<_, (Index, NodeNameLength)> = first
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let len = match id {
                    InternalNode::User(uid) => node_names.get(uid).map(|n| n.len()).unwrap_or(0),
                    _ => 0,
                };

                (id, (Index(i), NodeNameLength(len)))
            })
            .collect();

        // The Entries in the second/lower level mapped to their respective X-Indices
        let second_entries: HashMap<_, (Index, NodeNameLength)> = second
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let len = match id {
                    InternalNode::User(uid) => node_names.get(uid).map(|n| n.len()).unwrap_or(0),
                    _ => 0,
                };

                (id, (Index(i), NodeNameLength(len)))
            })
            .collect();

        // An iterator over all the Source Entries and their respective coordinates in the first layer
        let first_src_coords = first.iter().enumerate().map(|(raw_x, e)| {
            // Calculate the Source Coordinates

            // Calculate the Offset "generated" by the preceding Entries at the Level
            let offset: usize = first
                .iter()
                .take(raw_x)
                .map(|id| match id {
                    InternalNode::User(id) => node_names.get(id).map(|n| n.len()).unwrap_or(0),
                    InternalNode::Dummy { .. } => 1,
                    InternalNode::ReverseDummy { .. } => 1,
                })
                .sum();

            // Caclulate the actual Coordinate based on the Entry itself as User and Dummy entries have slightly different behaviour
            let cord = match e {
                InternalNode::User(id) => {
                    let in_node_offset = node_names.get(id).map(|s| s.len()).unwrap_or(0);
                    raw_x * 2 + offset + in_node_offset / 2 + 1
                }
                InternalNode::Dummy { .. } => raw_x * 2 + offset + 1,
                InternalNode::ReverseDummy { .. } => raw_x * 2 + offset + 1,
            }.min(max_x);

            (GridCoordinate(cord), e)
        });

        let mut temp_horizontal: Vec<_> = first_src_coords
            .filter_map(|(root, src_entry)| {
                // Connect the Source to its Targets in the lower Level

                // An Iterator over the Successors of the src_entry
                let succs: Box<dyn Iterator<Item = (&InternalNode<ID>, usize)>> = match src_entry {
                    InternalNode::User(id) => {
                        let raw_succs = agraph.successors(id).cloned().unwrap_or_default();
                        
                        Box::new(raw_succs.into_iter().map(|succ_id| {
                            second.iter().find(|second_id| {
                                match second_id {
                                    InternalNode::User(uid) => *uid == succ_id,
                                    InternalNode::Dummy { src, target, .. } => *src == *id && *target == succ_id,
                                    InternalNode::ReverseDummy { src, target, .. } => *src == *id && *target == succ_id,
                                }
                            }).expect("")
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
                                            node_names.get(id).map(|n| n.len()).unwrap_or(0)
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
                                InternalNode::ReverseDummy { d_id, src, target } => todo!(),
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
                                            node_names.get(id).map(|n| n.len()).unwrap_or(0)
                                        }
                                        _ => 1,
                                    }
                                })
                                .sum();
    
                            let raw_x = index * 2 + offset + in_node_offset / 2 + 1;

                            (t_id, raw_x)
                        }))
                    }
                    InternalNode::ReverseDummy { d_id, src, target } => {
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
                                                node_names.get(id).map(|n| n.len()).unwrap_or(0)
                                            }
                                            _ => 1,
                                        }
                                    })
                                    .sum();
        
                                let raw_x = index * 2 + offset + in_node_offset / 2 + 1;
    
                                (t_id, raw_x)
                            }))
                        } else {
                            let following_layer_iter = core::iter::once(second.iter().find(|second_id| {
                                match second_id {
                                    InternalNode::ReverseDummy { src: s_src, target: s_target, .. } => src == s_src && target == s_target,
                                    _ => false,
                                }
                            }).unwrap());

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
                                                node_names.get(id).map(|n| n.len()).unwrap_or(0)
                                            }
                                            _ => 1,
                                        }
                                    })
                                    .sum();
        
                                let raw_x = index * 2 + offset + in_node_offset / 2 + 1;
    
                                (t_id, raw_x)
                            }))
                        }
                    },
                };

                let targets: Vec<_> = succs
                    .map(|(t_id, raw_x)| {
                        // Calculate the Coordinate of the Target
                        (
                            GridCoordinate(raw_x.min(max_x)),
                            matches!(t_id, InternalNode::Dummy { .. }),
                        )
                    })
                    .collect();

                // Smallest x coordinate in the entire horizontal
                let sx = *std::iter::once(&root)
                    .chain(targets.iter().map(|t| &t.0))
                    .min()
                    .unwrap();
                // Smallest x coordinate in the entire horizontal
                let tx = *std::iter::once(&root)
                    .chain(targets.iter().map(|t| &t.0))
                    .max()
                    .unwrap();

                if targets.is_empty() {
                    return None;
                }

                match src_entry {
                    InternalNode::User(src) | InternalNode::Dummy { src, .. } => {
                        Some(Horizontal::TopBottom {
                            src_x: root,
                            src: *src,
                            targets,
                            x_bounds: (sx, tx),
                        })
                    }
                    InternalNode::ReverseDummy { d_id, src, target } => {
                        if first.iter().any(|n| match n {
                            InternalNode::User(uid) => uid == src,
                            _ => false,
                        }) {
                            Some(Horizontal::TopTop { src_x: root, src: *src, target: targets.into_iter().next().map(|(c, _)| c).unwrap(), x_bounds: (sx, tx) })
                        } else if let Some((index, node)) = second.iter().enumerate().find(|(_, n)| match n {
                            InternalNode::ReverseDummy { d_id, src: s_src, target: s_target } => src == s_src && target == s_target,
                            _ => false,
                        }) {
                            let target = targets.into_iter().next().map(|(c, _)| c).unwrap();

                            let sx = target.min(root);
                            let tx = target.max(root);

                            Some(Horizontal::BottomTop { src_x: target, src: *src, target: root, x_bounds: (sx, tx) })
                        } else {
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
        temp_horizontal
    }

    /// This is responsible for generating all the Horizontals needed for each Layer
    fn generate_horizontals<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        levels: &[Vec<InternalNode<'g, ID>>],
        node_names: &HashMap<&ID, String>,
        max_x: usize
    ) -> impl Iterator<Item = Vec<Horizontal<'g, ID>>> {
        levels
            .windows(2)
            .map(|window| {
                // The upper and lower level that need to be connected
                let first = &window[0];
                let second = &window[1];

                Self::generate_horizontal(agraph, first, second, node_names, max_x)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn insert_nodes(
        y: usize,
        result: &mut InnerGrid<'g, ID>,
        level: &[InternalNode<'g, ID>],
        node_names: &HashMap<&ID, String>,
        max_x: usize
    ) {
        let row = result.row_mut(y);
        let mut cursor = row.into_cursor();
        for entry in level.iter() {
            if cursor.next_x() > max_x {
                cursor.set_x(max_x);

                match &entry {
                    InternalNode::User(_) => {
                        unreachable!("");
                    }
                    InternalNode::Dummy { src, target, .. } => {
                        cursor.set_node(LevelEntry::Dummy { from: src, to: target }, "");
                    }
                    InternalNode::ReverseDummy { d_id, src, target } => {
                        cursor.set_node(LevelEntry::Dummy { from: src, to: target }, "");
                    }
                };


                continue;
            }

            cursor.set(Entry::Empty);
            match &entry {
                InternalNode::User(id) => {
                    let name = node_names.get(id).expect("");
                    cursor.set_node(LevelEntry::User(id), name);
                }
                InternalNode::Dummy { src, target, .. } => {
                    cursor.set_node(LevelEntry::Dummy { from: src, to: target }, "");
                }
                InternalNode::ReverseDummy { d_id, src, target } => {
                    cursor.set_node(LevelEntry::Dummy { from: src, to: target }, "");
                }
            };

            cursor.set(Entry::Empty);
        }
    }

    /// # Params:
    /// * `src_y`: The y-coordinate for the src nodes
    /// * `horis`: An Iterator over all the Horizontals in this Connection Layer
    /// * `horizontal_spacer`: Determines how much space should be left between each horizontal line
    ///
    /// # Returns
    /// An iterator over the Horizontals and their respective y-coordinate for the horizontal part, as well as the max y-coordinate
    fn determine_ys<'h>(
        src_y: usize,
        horis: &'h [Horizontal<'g, ID>],
        horizontal_spacer: usize,
    ) -> (
        impl Iterator<Item = (Horizontal<'g, ID>, usize)> + 'h,
        usize,
    ) {
        let mut y = src_y + 2;

        let final_y = src_y
            + horis
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    let x_bounds = h.x_bounds();
                    if i < horis.len() - 1 && x_bounds.0 != x_bounds.1 {
                        1 + horizontal_spacer
                    } else {
                        usize::from(i == horis.len() - 1 && x_bounds.0 != x_bounds.1)
                    }
                    
                })
                .sum::<usize>()
            + 4;

        (
            horis.iter().cloned().map(move |hori| {
                let hy = y;
                let x_bounds = hori.x_bounds();
                if x_bounds.0 != x_bounds.1 {
                    y += 1 + horizontal_spacer;
                }
                

                (hori, hy)
            }),
            final_y,
        )
    }

    /// This is used to actually "draw" the lines between two layers
    fn connect_layer<T>(
        y: &mut usize,
        level: &[InternalNode<'g, ID>],
        result: &mut InnerGrid<'g, ID>,
        horizontals: Vec<Horizontal<'g, ID>>,
        node_names: &HashMap<&ID, String>,
        config: &Config<ID, T>,
    ) {
        // Inserts the Nodes at the current y-Level
        Self::insert_nodes(*y, result, level, node_names, config.glyph_width() - 1);
        *y += 1;

        // Insert the Vertical Row below every Node
        for hori in horizontals.iter() {
            match hori {
                Horizontal::TopBottom { src_x, src, .. } => {
                    result.set(*src_x, *y, Entry::Veritcal(Some(src)));
                }
                Horizontal::BottomTop { target, src, .. } => {
                    result.set(*target, *y, Entry::Veritcal(Some(src)));
                }
                Horizontal::TopTop { src_x, src, .. } => {
                    result.set(*src_x, *y, Entry::Veritcal(Some(src)));
                }
                Horizontal::BottomBottom { src_x, src, target, x_bounds } => {
                    // Do nothing
                }
            };
            
        }
        *y += 1;

        let (hori_iter, lowest_y) =
            Self::determine_ys(*y - 2, &horizontals, config.vertical_edge_spacing);
        for (hori, y_height) in hori_iter {
            match hori {
                Horizontal::TopBottom { src_x, src, targets, x_bounds } => {
                    // Draw the horizontal line
                    if x_bounds.0 != x_bounds.1 {
                        for x in x_bounds.0.between(&(x_bounds.1 + 1)) {
                            result.set(x, y_height, Entry::Horizontal(src));
                        }
                    }

                    // Connect the src node to the horizontal line being drawn
                    for vy in (*y - 1)..=y_height {
                        result.set(src_x, vy, Entry::Veritcal(Some(src)));
                    }

                    for target in targets {
                        for y in y_height..(lowest_y - 1) {
                            result.set(target.0, y, Entry::Veritcal(Some(src)));
                        }

                        for py in y_height..(*y - 1) {
                            result.set(target.0, py, Entry::Veritcal(Some(src)));
                        }

                        let ent = if target.1 {
                            Entry::Veritcal(Some(src))
                        } else {
                            Entry::ArrowDown(Some(src))
                        };
                        result.set(target.0, lowest_y - 1, ent);
                    }
                }
                Horizontal::BottomTop {src_x, src, x_bounds, target } => {
                    // Draw the horizontal line
                    if x_bounds.0 != x_bounds.1 {
                        for x in x_bounds.0.between(&(x_bounds.1 + 1)) {
                            result.set(x, y_height, Entry::Horizontal(src));
                        }
                    }

                    // Connect the src node to the horizontal line being drawn
                    for vy in (*y - 1)..=y_height {
                        result.set(target, vy, Entry::Veritcal(Some(src)));
                    }

                    
                    for y in y_height..=(lowest_y - 1) {
                        result.set(src_x, y, Entry::Veritcal(Some(src)));
                    }

                    
                    for py in y_height..(*y - 1) {
                        result.set(src_x, py, Entry::Veritcal(Some(src)));
                    }

                    
                }
                Horizontal::TopTop {src_x, src, x_bounds, target } => {
                    // Draw the horizontal line
                    if x_bounds.0 != x_bounds.1 {
                        for x in x_bounds.0.between(&(x_bounds.1 + 1)) {
                            result.set(x, y_height, Entry::Horizontal(src));
                        }
                    }

                    // Connect the src node to the horizontal line being drawn
                    for vy in (*y - 1)..=y_height {
                        result.set(src_x, vy, Entry::Veritcal(Some(src)));
                    }

                    for vy in (*y - 1)..=y_height {
                        result.set(target, vy, Entry::Veritcal(Some(src)));
                    }
                }
                Horizontal::BottomBottom { src_x, src, target, x_bounds } => {
                    // Draw the horizontal line
                    if x_bounds.0 != x_bounds.1 {
                        for x in x_bounds.0.between(&(x_bounds.1 + 1)) {
                            result.set(x, y_height, Entry::Horizontal(src));
                        }
                    }

                    // Connect the src node to the horizontal line being drawn
                    for vy in y_height..(lowest_y ) {
                        result.set(src_x, vy, Entry::Veritcal(Some(src)));
                    }
                    for vy in y_height..(lowest_y-1) {
                        result.set(target, vy, Entry::Veritcal(Some(src)));
                    }
                    result.set(target, lowest_y-1, Entry::ArrowDown(Some(src)));
                }
            };
            
        }

        *y = lowest_y;
    }

    fn generate_levels<T>(
        levels: Vec<Level<'g, ID>>,
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        reved_edges: &[(&'g ID, &'g ID)],
    ) -> Vec<Vec<InternalNode<'g, ID>>> {
        if levels.is_empty() {
            return Vec::new();
        }

        let mut dummy_id = 0;

        let mut internal_levels = levels.iter().map(|level| {
            level.nodes.iter().map(|n| InternalNode::User(*n)).collect::<Vec<_>>()
        }).collect::<Vec<_>>();
        
        if !reved_edges.is_empty() {
            internal_levels.insert(0, Vec::new());
            internal_levels.push(Vec::new());
        }

        let level_index_iter = if !reved_edges.is_empty() {
            1..internal_levels.len()-2
        } else {
            0..internal_levels.len()-1
        };

        

        for index in level_index_iter {
            let split = internal_levels.split_at_mut(index+1);
            let first = split.0.last_mut().unwrap();
            let second = split.1.first_mut().unwrap();

            let mut first_rev = Vec::new();

            for fnode in first.iter() {
                match fnode {
                    InternalNode::User(uid) => {
                        let graph_succs = agraph.successors(uid).cloned().unwrap_or_default();

                        for gsucc in graph_succs {
                            if reved_edges.iter().any(|re| re.0 == gsucc) {
                                let id = dummy_id;
                                dummy_id += 1;
                                first_rev.push(InternalNode::ReverseDummy { d_id: id, src: gsucc, target: uid });
                                
                                let id = dummy_id;
                                dummy_id += 1;
                                second.push(InternalNode::ReverseDummy { d_id: id, src: gsucc, target: uid });
                            }

                            if !second.iter().any(|sid| match sid {
                                InternalNode::User(uid) => gsucc == *uid,
                                _ => false,
                            }) {
                                let id = dummy_id;
                                dummy_id += 1;
                                second.push(InternalNode::Dummy { d_id: id, src: uid, target: gsucc });
                            }
                        }
                    }
                    InternalNode::Dummy {  src, target, .. } => {
                        if !second.iter().any(|sid| match sid {
                            InternalNode::User(uid) => target == uid,
                            _ => false,
                        }) {
                            let id = dummy_id;
                            dummy_id += 1;
                            second.push(InternalNode::Dummy { d_id: id, src: *src, target: *target });
                        }
                    }
                    InternalNode::ReverseDummy { d_id, src, target } => {
                        if !first.iter().any(|n| match n {
                            InternalNode::User(uid) => uid == src,
                            _ => false,
                        }) {
                            let id = dummy_id;
                            dummy_id += 1;
                            second.push(InternalNode::ReverseDummy { d_id: id, src: *src, target: *target });
                        }
                    }
                };
            }

            first.extend(first_rev);
        }

        internal_levels
    }

    /// Construct the Grid based on the given information about the levels and overall structure
    pub fn construct<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        levels: Vec<Level<'g, ID>>,
        reved_edges: Vec<(&'g ID, &'g ID)>,
        config: &Config<ID, T>,
        names: HashMap<&'g ID, String>,
    ) -> Self {
        // Convert all the previously generated Levels into the Levels we need for this step
        let internal_levels = Self::generate_levels(levels, agraph, &reved_edges);

        // We first generate all the horizontals to connect all the Levels
        let horizontal = Self::generate_horizontals(agraph, &internal_levels, &names, config.glyph_width()-1);

        // An Iterator over all the Layers and the Horizontal connecting it to the Layer below
        let level_horizontal_iter = internal_levels.into_iter().zip(
            horizontal
                .into_iter()
                .chain(std::iter::repeat_with(Vec::new)),
        );

        let mut result = InnerGrid::new();

        // Connect all the layers
        let mut y = 0;
        for (level, horizontals) in level_horizontal_iter {
            Self::connect_layer(&mut y, &level, &mut result, horizontals, &names, config);
        }

        Self {
            inner: result,
            names,
        }
    }

    pub fn display(&self, color_palette: Option<&Vec<Color>>, glyphs: &LineGlyphs) {
        let mut colors = HashMap::new();
        let mut current_color = 0;

        let mut get_color = |id: &'g ID| {
            let color_p = color_palette.as_ref()?;

            let entry = colors.entry(id);
            let color = entry.or_insert_with(|| {
                current_color += 1;
                color_p[current_color % color_p.len()].clone()
            });

            Some(usize::from(color.clone()))
        };

        for row in &self.inner.inner {
            for entry in row {
                entry.display(
                    &mut get_color,
                    |id| self.names.get(id).unwrap().clone(),
                    glyphs,
                );
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_ys_nogap_0hori() {
        let horizontals = [];
        let (mut result_iter, result_y) = Grid::<usize>::determine_ys(0, &horizontals, 0);

        assert_eq!(4, result_y);
        assert!(result_iter.next().is_none());
    }

    #[test]
    fn determine_ys_nogap_1hori_straight() {
        let horizontals = [Horizontal::TopBottom {
            src: &0,
            src_x: GridCoordinate(0),
            targets: vec![(GridCoordinate(0), false)],
            x_bounds: (GridCoordinate(0), GridCoordinate(0)),
        }];
        let (mut result_iter, result_y) = Grid::<usize>::determine_ys(0, &horizontals, 0);

        assert_eq!(4, result_y);

        let first_res = result_iter.next().expect("Should return 1 result");
        assert_eq!(2, first_res.1);

        assert!(result_iter.next().is_none());
    }

    #[test]
    fn determine_ys_nogap_1hori_notstraight() {
        let horizontals = [Horizontal::TopBottom {
            src: &0,
            src_x: GridCoordinate(0),
            targets: vec![(GridCoordinate(2), false)],
            x_bounds: (GridCoordinate(0), GridCoordinate(2)),
        }];
        let (mut result_iter, result_y) = Grid::<usize>::determine_ys(0, &horizontals, 0);

        assert_eq!(5, result_y);

        let first_res = result_iter.next().expect("Should return 1 result");
        assert_eq!(2, first_res.1);

        assert!(result_iter.next().is_none());
    }
}
