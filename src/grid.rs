use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::acyclic::AcyclicDirectedGraph;

mod entry;
pub use entry::Entry;

struct InnerGrid<'g, ID> {
    inner: Vec<Vec<Entry<'g, ID>>>,
}

pub struct Row<'r, 'g, ID> {
    y: usize,
    row: &'r mut Vec<Entry<'g, ID>>,
}

pub struct Cursor<'r, 'g, ID> {
    y: usize,
    x: usize,
    row: &'r mut Vec<Entry<'g, ID>>,
}

impl<'r, 'g, ID> From<Cursor<'r, 'g, ID>> for Row<'r, 'g, ID> {
    fn from(cur: Cursor<'r, 'g, ID>) -> Self {
        Self {
            y: cur.y,
            row: cur.row,
        }
    }
}

impl<'g, ID> InnerGrid<'g, ID>
where
    ID: PartialEq,
{
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn row_mut(&mut self, y: usize) -> Row<'_, 'g, ID> {
        while self.inner.len() <= y {
            self.inner.push(Vec::new());
        }

        Row {
            y,
            row: self.inner.get_mut(y).unwrap(),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, entry: Entry<'g, ID>) {
        let mut row = self.row_mut(y);
        row.set(x, entry);
    }
}

impl<'r, 'g, ID> Row<'r, 'g, ID>
where
    ID: PartialEq,
{
    pub fn set(&mut self, x: usize, entry: Entry<'g, ID>) {
        while self.row.len() <= x {
            self.row.push(Entry::Empty);
        }

        let target = self.row.get_mut(x).unwrap();
        *target = &target + entry;
    }

    pub fn into_cursor(self) -> Cursor<'r, 'g, ID> {
        Cursor {
            y: self.y,
            x: 0,
            row: self.row,
        }
    }
}

impl<'r, 'g, ID> Cursor<'r, 'g, ID>
where
    ID: PartialEq,
{
    pub fn move_to(&mut self, x: usize) {
        self.x = x;
    }

    pub fn set(&mut self, entry: Entry<'g, ID>) -> usize {
        while self.row.len() <= self.x {
            self.row.push(Entry::Empty);
        }

        let target = self.row.get_mut(self.x).unwrap();
        *target = &target + entry;

        self.x += 1;
        self.x - 1
    }
}

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
    ) -> Vec<Vec<(usize, Vec<usize>)>> {
        let mut horizontal = Vec::new();
        for index in 0..(levels.len() - 1) {
            let levels_slice = levels.as_mut_slice();
            let (first_half, second_half) = levels_slice.split_at_mut(index + 1);

            let first = first_half.get_mut(index).expect("");
            let second = second_half.get_mut(0).unwrap();

            let mut temp_horizontal = Vec::new();
            for (firstx, entry) in first.iter().enumerate() {
                let succs = agraph.successors(entry.id()).unwrap();

                let second_entries: HashMap<&ID, usize> = second
                    .iter()
                    .enumerate()
                    .map(|(i, id)| (id.id(), i))
                    .collect();

                let mut targets: Vec<usize> = succs
                    .iter()
                    .filter_map(|succ| second_entries.get(*succ))
                    .copied()
                    .collect();

                let max_target = second.len() - 1;

                let succ_count = match &entry {
                    LevelEntry::User(_) => succs.len(),
                    LevelEntry::Dummy { to, .. } => succs.iter().filter(|id| **id == *to).count(),
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
                        targets.push(x);
                    }
                }

                temp_horizontal.push((firstx, targets));
            }

            temp_horizontal.sort_unstable_by(|(x, _), (x2, _)| x.cmp(x2));
            horizontal.push(temp_horizontal);
        }

        horizontal
    }

    fn connect_layer(
        y: &mut usize,
        level: &[LevelEntry<'g, ID>],
        result: &mut InnerGrid<'g, ID>,
        mut horizontals: Vec<(usize, Vec<usize>)>,
    ) {
        horizontals.sort_by_key(|(_, ts)| ts.len());

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
                        cursor.set(Entry::Node(entry.clone()));
                        cursor.set(Entry::CloseParen);
                    }
                    LevelEntry::Dummy { .. } => {
                        cursor.set(Entry::Empty);
                        cursor.set(Entry::Node(entry.clone()));
                        cursor.set(Entry::Empty);
                    }
                };

                cursor.set(Entry::Empty);
            }
            *y += 1;
        }

        // Inserts the Vertical-Lines below every Node
        {
            let row = result.row_mut(*y);
            let mut cursor = row.into_cursor();

            for (x_i, src) in level
                .iter()
                .enumerate()
                .filter(|(x, _)| horizontals.iter().any(|(hx, _)| hx == x))
                .map(|(x, src)| (x * 5, src))
            {
                cursor.move_to(x_i);
                cursor.set(Entry::Empty);
                cursor.set(Entry::Empty);
                cursor.set(Entry::Veritcal(Some(src.id())));
                cursor.set(Entry::Empty);
                cursor.set(Entry::Empty);
            }
            *y += 1;
        }

        let horizontal_iter: Vec<_> = horizontals
            .iter()
            .flat_map(|(x1, x_targets)| {
                let src = level
                    .iter()
                    .enumerate()
                    .find(|(hx, _)| hx == x1)
                    .map(|(_, id)| id.id())
                    .unwrap();

                let sx = std::iter::once(x1).chain(x_targets.iter()).min().unwrap();
                let tx = std::iter::once(x1).chain(x_targets.iter()).max().unwrap();

                let horizontal_y = *y;
                {
                    for vy in (level_y + 2)..=*y {
                        result.set(x1 * 5 + 2, vy, Entry::Veritcal(Some(src)));
                    }

                    if sx != tx {
                        for x in ((sx * 5) + 2)..((tx * 5) + 3) {
                            result.set(x, horizontal_y, Entry::Horizontal(src));
                        }
                    }
                }

                if sx != tx {
                    *y += 1;

                    let into_coords = {
                        let mut targets = x_targets.clone();
                        targets.sort_unstable();
                        targets.dedup();
                        targets
                    };

                    for x in into_coords.iter() {
                        result.set(x * 5 + 2, *y - 1, Entry::Veritcal(Some(src)));
                        result.set(x * 5 + 2, *y, Entry::Veritcal(Some(src)));
                    }
                }
                *y += 1;

                Box::new(
                    x_targets
                        .iter()
                        .map(move |x_targ| (src, horizontal_y, x_targ * 5 + 2)),
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
                match entry {
                    Entry::Empty => print!(" "),
                    Entry::OpenParen => print!("("),
                    Entry::CloseParen => print!(")"),
                    Entry::Horizontal(src) => {
                        let color = get_color(*src);
                        print!("\x1b[{}m-\x1b[0m", color)
                    }
                    Entry::Veritcal(src) => match src {
                        Some(src) => {
                            let color = get_color(*src);
                            print!("\x1b[{}m|\x1b[0m", color)
                        }
                        None => print!("|"),
                    },
                    Entry::Cross(src) => match src {
                        Some(src) => {
                            let color = get_color(*src);
                            print!("\x1b[{}m+\x1b[0m", color)
                        }
                        None => print!("+"),
                    },
                    Entry::Node(id) => match id {
                        LevelEntry::User(id) => print!("{:?}", id),
                        LevelEntry::Dummy { from, .. } => {
                            let color = get_color(*from);
                            print!("\x1b[{}m|\x1b[0m", color)
                        }
                    },
                };
            }
            println!();
        }
    }
}
