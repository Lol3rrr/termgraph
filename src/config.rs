use crate::NodeFormatter;

/// The Colors that can be displayed in the console
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Clone)]
pub enum Color {
    Black,
    White,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    /// This allows for custom ANSI Colors to be specified, the specified Color Code will not be
    /// valided and the User of this API needs to make sure that its a valid code
    Custom(usize),
}

impl From<Color> for usize {
    fn from(color: Color) -> Self {
        match color {
            Color::Black => 30,
            Color::White => 37,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::Custom(c) => c,
        }
    }
}

/// The Configuration to use for displaying a Graph
///
/// # Example
/// ```rust
/// use termgraph::{Config, IDFormatter};
///
/// let config: Config<usize, usize> = Config::new(IDFormatter::new(), 3).default_colors();
/// ```
pub struct Config<ID, T> {
    pub(crate) formatter: Box<dyn NodeFormatter<ID, T>>,
    pub(crate) color_palette: Option<Vec<Color>>,
    pub(crate) max_per_layer: usize,
}

impl<ID, T> Config<ID, T> {
    /// Creates a new Config with the given Formatter and maximum number of Nodes per Horizontal Layer
    ///
    /// # Default Values
    /// * Colors: disabled
    pub fn new<F>(nfmt: F, max_per_layer: usize) -> Self
    where
        F: NodeFormatter<ID, T> + 'static,
    {
        Self {
            formatter: Box::new(nfmt),
            color_palette: None,
            max_per_layer,
        }
    }

    /// Sets the Formatter of this Configuration to the provided one
    pub fn formatter<F>(mut self, nfmt: F) -> Self
    where
        F: NodeFormatter<ID, T> + 'static,
    {
        self.formatter = Box::new(nfmt);
        self
    }

    /// Updates the Number of Nodes that should be placed on a single horizontal Layer at most
    pub fn max_per_layer(mut self, count: usize) -> Self {
        self.max_per_layer = count;
        self
    }

    /// Sets the Color-Palette to the default Color-Palette
    pub fn default_colors(mut self) -> Self {
        self.color_palette = Some(vec![
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
        ]);
        self
    }

    /// Sets the Color-Palette to the given List of Colors
    pub fn custom_colors(mut self, colors: Vec<Color>) -> Self {
        self.color_palette = Some(colors);
        self
    }

    /// Disables the colors for the output
    pub fn disable_colors(mut self) -> Self {
        self.color_palette = None;
        self
    }
}
