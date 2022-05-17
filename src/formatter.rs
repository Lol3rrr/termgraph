use std::fmt::Display;

/// Specifies how the Nodes of the Graph should be formatted
pub trait NodeFormatter<ID, T> {
    fn format_node(&self, id: &ID, value: &T) -> String;
}

pub struct DefaultFormatter {}

impl DefaultFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DefaultFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl<ID, T> NodeFormatter<ID, T> for DefaultFormatter
where
    ID: Display,
{
    fn format_node(&self, id: &ID, _: &T) -> String {
        format!("({})", id)
    }
}
