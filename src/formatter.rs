use std::fmt::Display;

/// Specifies how the Nodes of the Graph should be formatted
pub trait NodeFormat<ID, T> {
    /// Formats the given Node, the returned Value will be displayed in the Graph itself
    fn format_node(&self, id: &ID, value: &T) -> String;
}

/// Returns the ID for Formatting
pub struct IDFormatter {}

impl IDFormatter {
    /// Creates a new Instance of the Formatter
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for IDFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl<ID, T> NodeFormat<ID, T> for IDFormatter
where
    ID: Display,
{
    fn format_node(&self, id: &ID, _: &T) -> String {
        format!("({id})")
    }
}

/// Returns the Value for Formatting
pub struct ValueFormatter {}

impl ValueFormatter {
    /// Creates a new Instance of the Formatter
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ValueFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl<ID, T> NodeFormat<ID, T> for ValueFormatter
where
    T: Display,
{
    fn format_node(&self, _: &ID, value: &T) -> String {
        format!("({value})")
    }
}
