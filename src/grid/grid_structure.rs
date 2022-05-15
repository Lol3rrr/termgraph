use std::{fmt::Debug, ops::Add};

use super::{Entry, LevelEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridCoordinate(pub usize);

impl GridCoordinate {
    pub fn between(&self, other: &Self) -> impl Iterator<Item = Self> {
        (self.0..other.0).map(GridCoordinate)
    }
}
impl Add<usize> for &GridCoordinate {
    type Output = GridCoordinate;

    fn add(self, rhs: usize) -> Self::Output {
        GridCoordinate(self.0 + rhs)
    }
}

pub struct InnerGrid<'g, ID> {
    pub inner: Vec<Vec<Entry<'g, ID>>>,
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

    pub fn set(&mut self, x: GridCoordinate, y: usize, entry: Entry<'g, ID>) {
        let mut row = self.row_mut(y);
        row.set(x.0, entry);
    }
}

pub struct Row<'r, 'g, ID> {
    y: usize,
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

pub struct Cursor<'r, 'g, ID> {
    y: usize,
    x: usize,
    row: &'r mut Vec<Entry<'g, ID>>,
}

impl<'r, 'g, ID> Cursor<'r, 'g, ID>
where
    ID: PartialEq + Debug,
{
    /// Returns the Middle Index of the Node
    pub fn set_node(&mut self, entry: LevelEntry<'g, ID>) -> GridCoordinate {
        let length = format!("{:?}", entry.id()).len();

        let last_x = self.x + length;
        while self.row.len() <= last_x {
            self.row.push(Entry::Empty);
        }

        for part in 0..length {
            let target = self.row.get_mut(self.x).unwrap();
            *target = &target + Entry::Node(entry.clone(), part);

            self.x += 1;
        }

        GridCoordinate(self.x - ((length + 1) / 2))
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
