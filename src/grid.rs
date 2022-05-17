use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::{acyclic::AcyclicDirectedGraph, NodeFormatter};

mod entry;
pub use entry::Entry;

mod grid_structure;
use grid_structure::*;

#[derive(Debug)]
pub enum LevelEntry<'g, ID> {
    User(&'g ID),
    Dummy { from: &'g ID, to: &'g ID },
}

impl<'g, ID> LevelEntry<'g, ID> {
    pub fn id(&self) -> &'g ID {
        match &self {
            Self::User(s) => *s,
            Self::Dummy { from, .. } => *from,
        }
    }

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

#[derive(Debug)]
struct Horizontal<'g, ID> {
    x_coord: GridCoordinate,
    src: &'g ID,
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

pub struct Grid<'g, ID>
where
    ID: Eq + Hash,
{
    inner: InnerGrid<'g, ID>,
    names: HashMap<&'g ID, String>,
}

impl<'g, ID> Grid<'g, ID>
where
    ID: Hash + Eq,
{
    fn generate_horizontals<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        levels: &mut Vec<Vec<LevelEntry<'g, ID>>>,
        node_names: &HashMap<&ID, String>,
    ) -> Vec<Vec<Horizontal<'g, ID>>> {
        (0..(levels.len() - 1))
            .map(|index| {
                let levels_slice = levels.as_mut_slice();
                let (first_half, second_half) = levels_slice.split_at_mut(index + 1);

                let first = first_half.get_mut(index).expect("");
                let second = second_half.get_mut(0).unwrap();

                let second_entries: HashMap<&ID, usize> = second
                    .iter()
                    .enumerate()
                    .map(|(i, id)| (id.id(), i))
                    .collect();

                let mut temp_horizontal: Vec<_> = first
                    .iter()
                    .enumerate()
                    .map(|(raw_x, e)| {
                        let offset: usize = first
                            .iter()
                            .take(raw_x)
                            .filter(|id| id.is_user())
                            .map(|id| node_names.get(id.id()).map(|n| n.len()).unwrap_or(0))
                            .sum();

                        let in_node_offset = node_names.get(e.id()).map(|s| s.len()).unwrap_or(0);

                        let cord = if e.is_user() {
                            raw_x * 2 + offset + in_node_offset / 2 + 1
                        } else {
                            raw_x * 2 + offset + 1
                        };

                        (GridCoordinate(cord), e)
                    })
                    .map(|(root, entry)| {
                        let succs = agraph.successors(entry.id()).unwrap();

                        let mut targets: Vec<GridCoordinate> = succs
                            .iter()
                            .filter_map(|succ| second_entries.get(*succ).map(|i| (succ, i)))
                            .map(|(t_id, index)| {
                                let offset: usize = second
                                    .iter()
                                    .take(*index)
                                    .filter(|id| id.is_user())
                                    .map(|id| node_names.get(id.id()).map(|n| n.len()).unwrap_or(0))
                                    .sum();

                                let in_node_offset =
                                    node_names.get(t_id).map(|s| s.len()).unwrap_or(0);

                                GridCoordinate(index * 2 + offset + in_node_offset / 2 + 1)
                            })
                            .collect();

                        let max_target = second.len() - 1;

                        let succ_count = match &entry {
                            LevelEntry::User(_) => succs.len(),
                            LevelEntry::Dummy { to, .. } => {
                                succs.iter().filter(|id| **id == *to).count()
                            }
                        };

                        if succ_count != targets.len() {
                            let succ_targets: Box<dyn Iterator<Item = &ID>> = match &entry {
                                LevelEntry::User(_) => Box::new(
                                    succs
                                        .iter()
                                        .filter(|id| !second_entries.contains_key(*id))
                                        .copied(),
                                ),
                                LevelEntry::Dummy { to, .. } => Box::new(std::iter::once(*to)),
                            };

                            let mut current_x = max_target;
                            let x_pos_iter = succ_targets.map(|id| {
                                current_x += 1;
                                (id, current_x)
                            });
                            for (id, x) in x_pos_iter {
                                second.push(LevelEntry::Dummy {
                                    from: entry.id(),
                                    to: id,
                                });
                                let offset: usize = second
                                    .iter()
                                    .take(x)
                                    .filter(|id| id.is_user())
                                    .map(|id| node_names.get(id.id()).map(|n| n.len()).unwrap_or(0))
                                    .sum();

                                targets.push(GridCoordinate(x * 2 + offset + 1));
                            }
                        }

                        Horizontal {
                            x_coord: root,
                            src: entry.id(),
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

    pub fn construct<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        levels: Vec<Vec<&'g ID>>,
        reved_edges: Vec<(&'g ID, &'g ID)>,
        nfmt: &dyn NodeFormatter<ID, T>,
    ) -> Self {
        let names: HashMap<&'g ID, String> = agraph
            .nodes
            .iter()
            .map(|(id, value)| (*id, nfmt.format_node(*id, value)))
            .collect();

        // TODO
        // Figure out how to correctly incorporate the reversed Edges into the generated Grid
        let _ = reved_edges;

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

        let horizontal = Self::generate_horizontals(agraph, &mut levels, &names);

        let mut y = 0;
        for (level, horizontals) in levels.iter().enumerate().map(|(y, l)| {
            (
                l,
                horizontal
                    .get(y)
                    .map(|h| h.to_owned())
                    .unwrap_or_else(Vec::new),
            )
        }) {
            Self::connect_layer(&mut y, level, &mut result, horizontals, &names);
        }

        Self {
            inner: result,
            names,
        }
    }

    pub fn display<T>(&self, nfmt: &dyn NodeFormatter<ID, T>) {
        let mut colors = HashMap::new();
        let mut current_color = 30;

        let mut get_color = |id: &'g ID| {
            let entry = colors.entry(id);
            let color = entry.or_insert_with(|| {
                current_color += 1;
                current_color = ((current_color - 30) % 8) + 31;
                current_color
            });

            *color
        };

        for row in &self.inner.inner {
            for entry in row {
                entry.display(&mut get_color, |id| self.names.get(id).unwrap().clone());
            }
            println!();
        }
    }
}
