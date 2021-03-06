use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::{acyclic::AcyclicDirectedGraph, Color, LineGlyphs, NodeFormat};

mod entry;
pub use entry::Entry;

mod grid_structure;
use grid_structure::*;

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
            Self::User(s) => *s,
            Self::Dummy { from, .. } => *from,
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
    x_coord: GridCoordinate,
    /// The ID of the Source
    src: &'g ID,
    /// The X-Coordinates of the Targets in the lower Level
    targets: Vec<GridCoordinate>,
}

impl<'g, ID> Clone for Horizontal<'g, ID> {
    fn clone(&self) -> Self {
        Self {
            x_coord: self.x_coord,
            src: self.src,
            targets: self.targets.clone(),
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
    /// This is responsible for generating all the Horizontals needed for each Layer
    fn generate_horizontals<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        levels: &mut Vec<Vec<LevelEntry<'g, ID>>>,
        node_names: &HashMap<&ID, String>,
    ) -> Vec<Vec<Horizontal<'g, ID>>> {
        (0..(levels.len() - 1))
            .map(|index| {
                let levels_slice = levels.as_mut_slice();
                let (first_half, second_half) = levels_slice.split_at_mut(index + 1);

                // The upper and lower level that need to be connected
                let first = first_half.get_mut(index).expect("");
                let second = second_half.get_mut(0).unwrap();

                // The Entries in the second/lower level mapped to their respective X-Indices
                let second_entries: HashMap<&ID, usize> = second
                    .iter()
                    .enumerate()
                    .map(|(i, id)| (id.id(), i))
                    .collect();

                let mut temp_horizontal: Vec<_> = first
                    .iter()
                    .enumerate()
                    .map(|(raw_x, e)| {
                        // Calculate the Source Coordinates

                        // Calculate the Offset "generated" by the preceding Entries at the Level
                        let offset: usize = first
                            .iter()
                            .take(raw_x)
                            .map(|id| {
                                if id.is_user() {
                                    node_names.get(id.id()).map(|n| n.len()).unwrap_or(0)
                                } else {
                                    1
                                }
                            })
                            .sum();

                        // Caclulate the actual Coordinate based on the Entry itself as User and Dummy entries have slightly different behaviour
                        let cord = if e.is_user() {
                            let in_node_offset =
                                node_names.get(e.id()).map(|s| s.len()).unwrap_or(0);
                            raw_x * 2 + offset + in_node_offset / 2 + 1
                        } else {
                            raw_x * 2 + offset + 1
                        };

                        (GridCoordinate(cord), e)
                    })
                    .map(|(root, src_entry)| {
                        // Connect the Source to its Targets in the lower Level

                        // An Iterator over the Successors of the src_entry
                        let succs: Box<dyn Iterator<Item = &ID>> = match src_entry {
                            LevelEntry::User(src_id) => {
                                Box::new(agraph.successors(src_id).unwrap().iter().copied())
                            }
                            LevelEntry::Dummy { to, .. } => Box::new(std::iter::once(*to)),
                        };

                        let targets = succs
                            .map(|t_id| {
                                let (index, in_node_offset) =
                                    match second_entries.get(t_id).copied() {
                                        Some(i) => {
                                            // If the successor is in the next Layer, we already know its index and
                                            // then also calculate the offset to point at the middle of the Target-Node
                                            let in_node_offset =
                                                node_names.get(t_id).map(|s| s.len()).unwrap_or(0);
                                            (i, in_node_offset)
                                        }
                                        None => {
                                            // The Successor is not in the next Layer, so we need to add a Dummy Node to
                                            // target instead
                                            second.push(LevelEntry::Dummy {
                                                from: src_entry.id(),
                                                to: t_id,
                                            });

                                            (second.len() - 1, 0)
                                        }
                                    };

                                // Calculate the Offset until the Target
                                let offset: usize = second
                                    .iter()
                                    .take(index)
                                    .map(|id| {
                                        if id.is_user() {
                                            node_names.get(id.id()).map(|n| n.len()).unwrap_or(0)
                                        } else {
                                            1
                                        }
                                    })
                                    .sum();

                                // Calculate the Coordinate of the Target
                                GridCoordinate(index * 2 + offset + in_node_offset / 2 + 1)
                            })
                            .collect();

                        Horizontal {
                            x_coord: root,
                            src: src_entry.id(),
                            targets,
                        }
                    })
                    .collect();

                // Sorts them based on their source X-Coordinates
                temp_horizontal.sort_unstable_by(|x1, x2| x1.x_coord.cmp(&x2.x_coord));

                // Sorts them based on their Targets average Coordinate, to try to avoid
                // unnecessary crossings in the Edges
                temp_horizontal.sort_by_cached_key(|hori| {
                    let sum_targets: usize = hori.targets.iter().map(|cord| cord.0).sum();
                    let target_count = hori.targets.len();
                    sum_targets / target_count
                });
                temp_horizontal
            })
            .collect()
    }

    /// This is used to actually "draw" the lines between two layers
    fn connect_layer(
        y: &mut usize,
        level: &[LevelEntry<'g, ID>],
        result: &mut InnerGrid<'g, ID>,
        mut horizontals: Vec<Horizontal<'g, ID>>,
        node_names: &HashMap<&ID, String>,
    ) {
        horizontals.sort_by_key(|h| h.targets.len());

        let level_y = *y;

        // Inserts the Nodes at the current y-Level
        {
            let row = result.row_mut(*y);
            let mut cursor = row.into_cursor();
            for entry in level.iter() {
                cursor.set(Entry::Empty);
                match &entry {
                    LevelEntry::User(id) => {
                        let name = node_names.get(id).expect("");
                        cursor.set_node(entry.clone(), name);
                    }
                    LevelEntry::Dummy { .. } => {
                        cursor.set_node(entry.clone(), "");
                    }
                };

                cursor.set(Entry::Empty);
            }
            *y += 1;
        }

        // Insert the Vertical Row below every Node
        {
            for hori in horizontals.iter() {
                result.set(hori.x_coord, *y, Entry::Veritcal(Some(hori.src)));
            }
            *y += 1;
        }

        let horizontal_iter: Vec<_> = horizontals
            .iter()
            .flat_map(|hori| {
                let sx = std::iter::once(&hori.x_coord)
                    .chain(hori.targets.iter())
                    .min()
                    .unwrap();
                let tx = std::iter::once(&hori.x_coord)
                    .chain(hori.targets.iter())
                    .max()
                    .unwrap();

                let horizontal_y = *y;
                {
                    for vy in (level_y + 2)..=*y {
                        result.set(hori.x_coord, vy, Entry::Veritcal(Some(hori.src)));
                    }

                    if sx != tx {
                        for x in sx.between(&(tx + 1)) {
                            result.set(x, horizontal_y, Entry::Horizontal(hori.src));
                        }
                    }
                }

                if sx != tx {
                    *y += 1;

                    let into_coords = {
                        let mut targets = hori.targets.clone();
                        targets.sort_unstable();
                        targets.dedup();
                        targets
                    };

                    for x in into_coords.iter() {
                        result.set(*x, *y - 1, Entry::Veritcal(Some(hori.src)));
                        result.set(*x, *y, Entry::Veritcal(Some(hori.src)));
                    }
                }
                *y += 1;

                Box::new(
                    hori.targets
                        .iter()
                        .map(move |x_targ| (hori.src, horizontal_y, *x_targ)),
                )
            })
            .collect();

        for (src, target_y, target_x) in horizontal_iter {
            for py in target_y..(*y) {
                result.set(target_x, py, Entry::Veritcal(Some(src)));
            }
        }
    }

    /// Construct the Grid based on the given information about the levels and overall structure
    pub fn construct<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        levels: Vec<Vec<&'g ID>>,
        reved_edges: Vec<(&'g ID, &'g ID)>,
        nfmt: &dyn NodeFormat<ID, T>,
    ) -> Self {
        let names: HashMap<&'g ID, String> = agraph
            .nodes
            .iter()
            .map(|(id, value)| (*id, nfmt.format_node(*id, value)))
            .collect();

        // TODO
        // Figure out how to correctly incorporate the reversed Edges into the generated Grid
        let _ = reved_edges;

        // Convert all the previously generated Levels into the Levels we need for this step
        let mut levels: Vec<Vec<LevelEntry<'g, ID>>> = levels
            .into_iter()
            .map(|inner_level| {
                inner_level
                    .into_iter()
                    .map(|l| LevelEntry::User(l))
                    .collect()
            })
            .collect();

        let mut result = InnerGrid::new();

        // We first generate all the horizontals to connect all the Levels
        let horizontal = Self::generate_horizontals(agraph, &mut levels, &names);

        // An Iterator over all the Layers and the Horizontal connecting it to the Layer below
        let level_horizontal_iter = levels.into_iter().zip(
            horizontal
                .into_iter()
                .chain(std::iter::repeat_with(Vec::new)),
        );

        // Connect all the layers
        let mut y = 0;
        for (level, horizontals) in level_horizontal_iter {
            Self::connect_layer(&mut y, &level, &mut result, horizontals, &names);
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
