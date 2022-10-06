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
struct Horizontal<'g, ID> {
    /// The X-Coordinate of the Source in the upper Level
    src_x: GridCoordinate,
    /// The ID of the Source
    src: &'g ID,
    /// The X-Coordinates of the Targets in the lower Level
    targets: Vec<(GridCoordinate, bool)>,
    /// A touple of the smallest and largest x coordinates
    x_bounds: (GridCoordinate, GridCoordinate),
}

impl<'g, ID> Clone for Horizontal<'g, ID> {
    fn clone(&self) -> Self {
        Self {
            src_x: self.src_x,
            src: self.src,
            targets: self.targets.clone(),
            x_bounds: self.x_bounds,
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
    ) -> Vec<Horizontal<'g, ID>> {
        #[derive(Clone, Copy)]
        struct NodeNameLength(usize);

        #[derive(Clone, Copy)]
        struct Index(usize);

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
                })
                .sum();

            // Caclulate the actual Coordinate based on the Entry itself as User and Dummy entries have slightly different behaviour
            let cord = match e {
                InternalNode::User(id) => {
                    let in_node_offset = node_names.get(id).map(|s| s.len()).unwrap_or(0);
                    raw_x * 2 + offset + in_node_offset / 2 + 1
                }
                InternalNode::Dummy { .. } => raw_x * 2 + offset + 1,
            };

            (GridCoordinate(cord), e)
        });

        let mut temp_horizontal: Vec<_> = first_src_coords
            .filter_map(|(root, src_entry)| {
                // Connect the Source to its Targets in the lower Level

                // An Iterator over the Successors of the src_entry
                let succs = match src_entry {
                    InternalNode::User(id) => {
                        let raw_succs = agraph.successors(id).cloned().unwrap_or_default();
                        
                        Box::new(raw_succs.into_iter().map(|succ_id| {
                            second.iter().find(|second_id| {
                                match second_id {
                                    InternalNode::User(uid) => *uid == succ_id,
                                    InternalNode::Dummy { src, target, .. } => *src == *id && *target == succ_id,
                                }
                            }).expect("")
                        })) as Box<dyn Iterator<Item = &InternalNode<'g, ID>>>
                    }
                    InternalNode::Dummy { src, target, .. } => {
                        Box::new(core::iter::once(second.iter().find(|second_id| {
                            match second_id {
                                InternalNode::User(uid) => uid == target,
                                InternalNode::Dummy { src: s_src, target: s_target, .. } => src == s_src && target == s_target,
                            }
                        }).unwrap())) as Box<dyn Iterator<Item = &InternalNode<'g, ID>>>
                    }
                };

                let targets: Vec<_> = succs
                    .map(|t_id| {
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

                        // Calculate the Coordinate of the Target
                        (
                            GridCoordinate(index * 2 + offset + in_node_offset / 2 + 1),
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

                Some(Horizontal {
                    src_x: root,
                    src: match src_entry {
                        InternalNode::User(uid) => *uid,
                        InternalNode::Dummy { src, .. } => *src,
                    },
                    targets,
                    x_bounds: (sx, tx),
                })
            })
            .collect();

        // Sorts them based on their source X-Coordinates
        temp_horizontal.sort_unstable_by(|x1, x2| x1.src_x.cmp(&x2.src_x));

        // Sorts them based on their Targets average Coordinate, to try to avoid
        // unnecessary crossings in the Edges
        temp_horizontal.sort_by_cached_key(|hori| {
            let sum_targets: usize = hori.targets.iter().map(|cord| cord.0 .0).sum();
            let target_count = hori.targets.len().max(1);
            sum_targets / target_count
        });
        temp_horizontal
    }

    /// This is responsible for generating all the Horizontals needed for each Layer
    fn generate_horizontals<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        levels: &[Vec<InternalNode<'g, ID>>],
        node_names: &HashMap<&ID, String>,
    ) -> impl Iterator<Item = Vec<Horizontal<'g, ID>>> {
        levels
            .windows(2)
            .map(|window| {
                // The upper and lower level that need to be connected
                let first = &window[0];
                let second = &window[1];

                Self::generate_horizontal(agraph, first, second, node_names)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn insert_nodes(
        y: usize,
        result: &mut InnerGrid<'g, ID>,
        level: &[InternalNode<'g, ID>],
        node_names: &HashMap<&ID, String>,
    ) {
        let row = result.row_mut(y);
        let mut cursor = row.into_cursor();
        for entry in level.iter() {
            cursor.set(Entry::Empty);
            match &entry {
                InternalNode::User(id) => {
                    let name = node_names.get(id).expect("");
                    cursor.set_node(LevelEntry::User(id), name);
                }
                InternalNode::Dummy { src, target, .. } => {
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
                    if i < horis.len() - 1 && h.x_bounds.0 != h.x_bounds.1 {
                        1 + horizontal_spacer
                    } else {
                        usize::from(i == horis.len() - 1 && h.x_bounds.0 != h.x_bounds.1)
                    }
                })
                .sum::<usize>()
            + 4;

        (
            horis.iter().cloned().map(move |hori| {
                let hy = y;
                if hori.x_bounds.0 != hori.x_bounds.1 {
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
        Self::insert_nodes(*y, result, level, node_names);
        *y += 1;

        // Insert the Vertical Row below every Node
        for hori in horizontals.iter() {
            result.set(hori.src_x, *y, Entry::Veritcal(Some(hori.src)));
        }
        *y += 1;

        let (hori_iter, lowest_y) =
            Self::determine_ys(*y - 2, &horizontals, config.vertical_edge_spacing);
        for (hori, y_height) in hori_iter {
            // Draw the horizontal line
            if hori.x_bounds.0 != hori.x_bounds.1 {
                for x in hori.x_bounds.0.between(&(hori.x_bounds.1 + 1)) {
                    result.set(x, y_height, Entry::Horizontal(hori.src));
                }
            }

            // Connect the src node to the horizontal line being drawn
            for vy in (*y - 1)..=y_height {
                result.set(hori.src_x, vy, Entry::Veritcal(Some(hori.src)));
            }

            for target in hori.targets {
                for y in y_height..(lowest_y - 1) {
                    result.set(target.0, y, Entry::Veritcal(Some(hori.src)));
                }

                for py in y_height..(*y - 1) {
                    result.set(target.0, py, Entry::Veritcal(Some(hori.src)));
                }

                let ent = if target.1 {
                    Entry::Veritcal(Some(hori.src))
                } else {
                    Entry::ArrowDown(Some(hori.src))
                };
                result.set(target.0, lowest_y - 1, ent);
            }
        }

        *y = lowest_y;
    }

    fn generate_levels<T>(
        levels: Vec<Level<'g, ID>>,
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
    ) -> Vec<Vec<InternalNode<'g, ID>>> {
        if levels.is_empty() {
            return Vec::new();
        }

        let mut dummy_id = 0;

        let mut internal_levels = levels.iter().map(|level| {
            level.nodes.iter().map(|n| InternalNode::User(*n)).collect::<Vec<_>>()
        }).collect::<Vec<_>>();

        for index in 0..levels.len()-1 {
            let split = internal_levels.split_at_mut(index+1);
            let first = split.0.last_mut().unwrap();
            let second = split.1.first_mut().unwrap();


            for fnode in first.iter() {
                match fnode {
                    InternalNode::User(uid) => {
                        let graph_succs = agraph.successors(uid).cloned().unwrap_or_default();

                        for gsucc in graph_succs {
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
                };
            }
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
        // TODO
        // Figure out how to correctly incorporate the reversed Edges into the generated Grid
        let _ = reved_edges;

        // Convert all the previously generated Levels into the Levels we need for this step
        /*
        let levels: Vec<Vec<LevelEntry<'g, ID>>> = levels
            .into_iter()
            .map(|inner_level| {
                inner_level
                    .nodes
                    .into_iter()
                    .map(|l| LevelEntry::User(l))
                    .collect()
            })
            .collect();
            */

        // let levels = Self::insert_dummies(agraph, levels.clone());

        let internal_levels = Self::generate_levels(levels, agraph);

        // We first generate all the horizontals to connect all the Levels
        let horizontal = Self::generate_horizontals(agraph, &internal_levels, &names);

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
        let horizontals = [Horizontal {
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
        let horizontals = [Horizontal {
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
