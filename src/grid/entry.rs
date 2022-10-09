use std::{fmt::Debug, ops::Add};

use crate::LineGlyphs;

use super::LevelEntry;

pub enum EntryNode<'g, ID> {
    User(&'g ID),
    SingleSrc(&'g ID),
    MultiSrc,
}

impl<'g, ID> From<LevelEntry<'g, ID>> for EntryNode<'g, ID> {
    fn from(src: LevelEntry<'g, ID>) -> Self {
        match src {
            LevelEntry::User(id) => EntryNode::User(id),
            LevelEntry::Dummy { from, .. } => EntryNode::SingleSrc(from),
        }
    }
}

pub enum Entry<'g, ID> {
    Empty,
    Horizontal(&'g ID),
    Veritcal(Option<&'g ID>),
    Cross(Option<&'g ID>),
    ArrowDown(Option<&'g ID>),
    Node(EntryNode<'g, ID>, usize),
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
            (
                Entry::Node(EntryNode::SingleSrc(fid), _),
                Entry::Node(EntryNode::SingleSrc(sid), _),
            ) if sid == *fid => Entry::Node(EntryNode::SingleSrc(sid), 0),
            (Entry::Node(EntryNode::SingleSrc(_), _), Entry::Node(EntryNode::SingleSrc(_), _)) => {
                Entry::Node(EntryNode::MultiSrc, 0)
            }
            (Entry::Node(EntryNode::MultiSrc, _), Entry::Node(EntryNode::SingleSrc(_), _)) => {
                Entry::Node(EntryNode::MultiSrc, 0)
            }
            (s, o) => {
                dbg!(s, o);

                unreachable!("")
            }
        }
    }
}

impl<'g, ID> Entry<'g, ID> {
    pub fn fdisplay<C, N, W>(
        &self,
        get_color: &mut C,
        get_name: N,
        glyphs: &LineGlyphs,
        dest: &mut W,
    ) where
        C: FnMut(&'g ID) -> Option<usize>,
        N: FnOnce(&'g ID) -> String,
        W: std::io::Write,
    {
        let _ = match self {
            Entry::Empty => write!(dest, " "),
            Entry::OpenParen => write!(dest, "("),
            Entry::CloseParen => write!(dest, ")"),
            Entry::Horizontal(src) => match get_color(*src) {
                Some(c) => write!(dest, "\x1b[{}m{}\x1b[0m", c, glyphs.horizontal),
                None => write!(dest, "{}", glyphs.horizontal),
            },
            Entry::Veritcal(src) => match src {
                Some(src) => match get_color(*src) {
                    Some(c) => write!(dest, "\x1b[{}m{}\x1b[0m", c, glyphs.vertical),
                    None => write!(dest, "{}", glyphs.vertical),
                },
                None => write!(dest, "{}", glyphs.vertical),
            },
            Entry::Cross(src) => match src {
                Some(src) => match get_color(*src) {
                    Some(c) => write!(dest, "\x1b[{}m{}\x1b[0m", c, glyphs.crossing),
                    None => write!(dest, "{}", glyphs.crossing),
                },
                None => write!(dest, "{}", glyphs.crossing),
            },
            Entry::ArrowDown(src) => match src {
                Some(src) => match get_color(*src) {
                    Some(c) => write!(dest, "\x1b[{}m{}\x1b[0m", c, glyphs.arrow_down),
                    None => write!(dest, "{}", glyphs.arrow_down),
                },
                None => write!(dest, "{}", glyphs.arrow_down),
            },
            Entry::Node(_, part) if *part > 0 => Ok(()),
            Entry::Node(id, _) => match id {
                EntryNode::User(id) => write!(dest, "{}", get_name(id)),
                EntryNode::SingleSrc(from) => match get_color(*from) {
                    Some(c) => write!(dest, "\x1b[{}m|\x1b[0m", c),
                    None => write!(dest, "|"),
                },
                EntryNode::MultiSrc => write!(dest, "|"),
            },
        };
    }
}
