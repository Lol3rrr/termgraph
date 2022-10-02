use std::{fmt::Debug, ops::Add};

use crate::LineGlyphs;

use super::LevelEntry;

pub enum Entry<'g, ID> {
    Empty,
    Horizontal(&'g ID),
    Veritcal(Option<&'g ID>),
    Cross(Option<&'g ID>),
    ArrowDown(Option<&'g ID>),
    Node(LevelEntry<'g, ID>, usize),
    OpenParen,
    CloseParen,
}

impl<'g, ID> Debug for Entry<'g, ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.debug_struct("Empty").finish(),
            Self::Horizontal(_) => f.debug_struct("Horizontal").finish(),
            Self::Veritcal(_) => f.debug_struct("Veritcal").finish(),
            Self::Cross(_) => f.debug_struct("Cross").finish(),
            Self::ArrowDown(_) => f.debug_struct("ArrowDown").finish(),
            Self::Node(_, _) => f.debug_struct("Node").finish(),
            Self::OpenParen => f.debug_struct("OpenParen").finish(),
            Self::CloseParen => f.debug_struct("CloseParen").finish(),
        }
    }
}

impl<'g, ID> Add<Entry<'g, ID>> for &&mut Entry<'g, ID>
where
    ID: PartialEq,
{
    type Output = Entry<'g, ID>;

    fn add(self, rhs: Entry<'g, ID>) -> Self::Output {
        match (self, rhs) {
            (Entry::Empty, other) => other,
            // Something being added to an existing Horizontal Line
            (Entry::Horizontal(og), Entry::Horizontal(n)) if *og == n => Entry::Horizontal(n),
            (Entry::Horizontal(_), Entry::Horizontal(_)) => {
                panic!("Overlapping Horizontals with different SRC's")
            }
            (Entry::Horizontal(n), Entry::Empty) => Entry::Horizontal(*n),
            (Entry::Horizontal(hsrc), Entry::Veritcal(Some(vsrc))) if *hsrc == vsrc => {
                Entry::Cross(Some(vsrc))
            }
            (Entry::Horizontal(_), Entry::Veritcal(_)) => Entry::Cross(None),
            // Something being added to an existing Vertical Line
            (Entry::Veritcal(og), Entry::Veritcal(n)) if *og == n => Entry::Veritcal(n),
            (Entry::Veritcal(_), Entry::Veritcal(_)) => Entry::Veritcal(None),
            (Entry::Veritcal(n), Entry::Empty) => Entry::Veritcal(*n),
            (Entry::Veritcal(Some(vsrc)), Entry::Horizontal(hsrc)) if *vsrc == hsrc => {
                Entry::Cross(Some(hsrc))
            }
            (Entry::Veritcal(_), Entry::Horizontal(_)) => Entry::Cross(None),
            // Something being added to an existing arrow-down
            (Entry::ArrowDown(og), Entry::ArrowDown(n)) if *og == n => Entry::ArrowDown(n),
            (Entry::ArrowDown(_), Entry::ArrowDown(_)) => Entry::ArrowDown(None),
            (Entry::Veritcal(og), Entry::ArrowDown(n)) if *og == n => Entry::ArrowDown(n),
            (Entry::Veritcal(_), Entry::ArrowDown(_)) => Entry::ArrowDown(None),
            // Something being added to an existing Cross
            (Entry::Cross(n), Entry::Empty) => Entry::Cross(*n),
            (Entry::Cross(None), _) => Entry::Cross(None),
            (Entry::Cross(Some(csrc)), Entry::Horizontal(hsrc)) if *csrc == hsrc => {
                Entry::Cross(Some(hsrc))
            }
            (Entry::Cross(Some(_)), Entry::Horizontal(_)) => Entry::Cross(None),
            (Entry::Cross(Some(csrc)), Entry::Veritcal(Some(vsrc))) if *csrc == vsrc => {
                Entry::Cross(Some(vsrc))
            }
            (Entry::Cross(Some(_)), Entry::Veritcal(_)) => Entry::Cross(None),
            (s, o) => {
                dbg!(s, o);

                dbg!(
                    std::mem::discriminant(&Entry::<'g, ID>::Empty),
                    std::mem::discriminant(&Entry::<'g, ID>::Horizontal),
                    std::mem::discriminant(&Entry::<'g, ID>::Veritcal),
                    std::mem::discriminant(&Entry::<'g, ID>::Cross),
                );
                todo!()
            }
        }
    }
}

impl<'g, ID> Entry<'g, ID> {
    pub fn display<C, N>(&self, get_color: &mut C, get_name: N, glyphs: &LineGlyphs)
    where
        C: FnMut(&'g ID) -> Option<usize>,
        N: FnOnce(&'g ID) -> String,
    {
        match self {
            Entry::Empty => print!(" "),
            Entry::OpenParen => print!("("),
            Entry::CloseParen => print!(")"),
            Entry::Horizontal(src) => match get_color(*src) {
                Some(c) => print!("\x1b[{}m{}\x1b[0m", c, glyphs.horizontal),
                None => print!("{}", glyphs.horizontal),
            },
            Entry::Veritcal(src) => match src {
                Some(src) => match get_color(*src) {
                    Some(c) => print!("\x1b[{}m{}\x1b[0m", c, glyphs.vertical),
                    None => print!("{}", glyphs.vertical),
                },
                None => print!("{}", glyphs.vertical),
            },
            Entry::Cross(src) => match src {
                Some(src) => match get_color(*src) {
                    Some(c) => print!("\x1b[{}m{}\x1b[0m", c, glyphs.crossing),
                    None => print!("{}", glyphs.crossing),
                },
                None => print!("{}", glyphs.crossing),
            },
            Entry::ArrowDown(src) => match src {
                Some(src) => match get_color(*src) {
                    Some(c) => print!("\x1b[{}m{}\x1b[0m", c, glyphs.arrow_down),
                    None => print!("{}", glyphs.arrow_down),
                },
                None => print!("{}", glyphs.arrow_down),
            },
            Entry::Node(_, part) if *part > 0 => {}
            Entry::Node(id, _) => match id {
                LevelEntry::User(id) => print!("{}", get_name(id)),
                LevelEntry::Dummy { from, .. } => match get_color(*from) {
                    Some(c) => print!("\x1b[{}m|\x1b[0m", c),
                    None => print!("|"),
                },
            },
        };
    }
}
