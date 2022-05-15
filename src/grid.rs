use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::acyclic::AcyclicDirectedGraph;

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
}

impl<'g, ID> Grid<'g, ID>
where
    ID: Hash + Eq + Debug,
{
    fn generate_horizontals<T>(
        agraph: &AcyclicDirectedGraph<'g, ID, T>,
        levels: &mut Vec<Vec<LevelEntry<'g, ID>>>,
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
                            .map(|id| format!("{:?}", id.id()).len().saturating_sub(1))
                            .sum();

                        (GridCoordinate(raw_x * 5 + 2 + offset), e)
                    })
                    .map(|(root, entry)| {
                        let succs = agraph.successors(entry.id()).unwrap();

                        let mut targets: Vec<GridCoordinate> = succs
                            .iter()
                            .filter_map(|succ| second_entries.get(*succ).map(|i| (succ, i)))
                            .map(|(_, index)| {
                                let offset: usize = second
                                    .iter()
                                    .take(*index)
                                    .filter(|id| id.is_user())
                                    .map(|id| format!("{:?}", id.id()).len().saturating_sub(1))
                                    .sum();

                                GridCoordinate(index * 5 + 2 + offset)
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
                                    .map(|id| format!("{:?}", id.id()).len().saturating_sub(1))
                                    .sum();

                                targets.push(GridCoordinate(x * 5 + 2 + offset));
                            }
                        }

                        Horizontal {
                            x_coord: root,
                            src: entry.id(),
                            targets,
                        }
                    })
                    .collect();

                temp_horizontal.sort_unstable_by(|x1, x2| x1.x_coord.cmp(&x2.x_coord));
                temp_horizontal
            })
            .collect()
    }

    fn connect_layer(
        y: &mut usize,
        level: &[LevelEntry<'g, ID>],
        result: &mut InnerGrid<'g, ID>,
        mut horizontals: Vec<Horizontal<'g, ID>>,
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
                    LevelEntry::User(_) => {
                        cursor.set(Entry::OpenParen);
                        cursor.set_node(entry.clone());
                        cursor.set(Entry::CloseParen);
                    }
                    LevelEntry::Dummy { .. } => {
                        cursor.set(Entry::Empty);
                        cursor.set_node(entry.clone());
                        cursor.set(Entry::Empty);
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
    ) -> Self {
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

        let horizontal = Self::generate_horizontals(agraph, &mut levels);

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
            Self::connect_layer(&mut y, level, &mut result, horizontals);
        }

        Self { inner: result }
    }

    pub fn display(&self) {
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
                entry.display(&mut get_color);
            }
            println!();
        }
    }
}
