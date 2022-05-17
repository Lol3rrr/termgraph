use std::fmt::Display;

/// Specifies how the Nodes of the Graph should be formatted
pub trait NodeFormatter<ID, T> {
    /// Formats the given Node, the returned Value will be displayed in the Graph itself
    fn format_node(&self, id: &ID, value: &T) -> String;
}

/// The Default Formatter for Nodes in the Graph
pub struct DefaultFormatter {}

impl DefaultFormatter {
    /// Creates a new Instance of the DefaultFormatter
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
