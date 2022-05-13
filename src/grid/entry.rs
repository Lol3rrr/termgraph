use std::ops::Add;

use super::LevelEntry;

pub enum Entry<'g, ID> {
    Empty,
    Horizontal(&'g ID),
    Veritcal(Option<&'g ID>),
    Cross(Option<&'g ID>),
    Node(LevelEntry<'g, ID>, usize),
    OpenParen,
    CloseParen,
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
                dbg!(std::mem::discriminant(*s), std::mem::discriminant(&o));

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
