use crate::NodeFormat;

/// The Colors that can be displayed in the console
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// This builder is used to construct a [`LineGlyphs`] instance
pub struct LineGlyphBuilder {
    vertical: char,
    horizontal: char,
    crossing: char,
    arrow_down: char,
}

impl LineGlyphBuilder {
    /// Creates the base Builder using the default ASCII symbols
    pub const fn ascii() -> Self {
        Self {
            vertical: '|',
            horizontal: '-',
            crossing: '+',
            arrow_down: 'V',
        }
    }

    /// Set the Glyph for vertical lines
    pub const fn vertical(mut self, glyph: char) -> Self {
        self.vertical = glyph;
        self
    }
    /// Set the Glyph for horizontal lines
    pub const fn horizontal(mut self, glyph: char) -> Self {
        self.horizontal = glyph;
        self
    }
    /// Set the Glyph for the crossings of two lines
    pub const fn crossing(mut self, glyph: char) -> Self {
        self.crossing = glyph;
        self
    }
    /// Set the Glyph for arrow heads at the end of the edges
    pub const fn arrow_down(mut self, glyph: char) -> Self {
        self.arrow_down = glyph;
        self
    }

    /// Should be called, once the configuration is done to obtain the final [`LineGlyphs`] instance
    pub const fn finish(self) -> LineGlyphs {
        LineGlyphs {
            vertical: self.vertical,
            horizontal: self.horizontal,
            crossing: self.crossing,
            arrow_down: self.arrow_down,
        }
    }
}

/// Describes the Glyphs that should be used to display the lines in the Graph.
///
/// This can't be constructed directly, but instead is constructed using [`LineGlyphBuilder`]
pub struct LineGlyphs {
    pub(crate) vertical: char,
    pub(crate) horizontal: char,
    pub(crate) crossing: char,
    pub(crate) arrow_down: char,
}

impl From<LineGlyphBuilder> for LineGlyphs {
    fn from(builder: LineGlyphBuilder) -> Self {
        builder.finish()
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
    pub(crate) formatter: Box<dyn NodeFormat<ID, T>>,
    pub(crate) color_palette: Option<Vec<Color>>,
    pub(crate) max_per_layer: usize,
    max_glyphs_per_layer: usize,
    pub(crate) vertical_edge_spacing: usize,
    pub(crate) line_glyphs: LineGlyphs,
}

impl<ID, T> Config<ID, T> {
    /// Creates a new Config with the given Formatter and maximum number of Nodes per Horizontal Layer
    ///
    /// # Default Values
    /// * Colors: disabled
    /// * Vertical-Edge-Spacing: 1
    pub fn new<F>(nfmt: F, max_per_layer: usize) -> Self
    where
        F: NodeFormat<ID, T> + 'static,
    {
        Self {
            formatter: Box::new(nfmt),
            color_palette: None,
            max_per_layer,
            max_glyphs_per_layer: usize::MAX,
            vertical_edge_spacing: 1,
            line_glyphs: LineGlyphBuilder::ascii().finish(),
        }
    }

    /// Sets the vertical spacing between the horizontal connecting edges
    pub fn vertical_edge_spacing(mut self, n_spacing: usize) -> Self {
        self.vertical_edge_spacing = n_spacing;
        self
    }

    /// Sets the Formatter of this Configuration to the provided one
    pub fn formatter<F>(mut self, nfmt: F) -> Self
    where
        F: NodeFormat<ID, T> + 'static,
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

    /// Sets the Glyphs to use for the Lines in the Graph
    pub fn line_glyphs<L>(mut self, glyphs: L) -> Self
    where
        L: Into<LineGlyphs>,
    {
        self.line_glyphs = glyphs.into();
        self
    }

    /// Set the max glyph width per layer
    pub fn max_glyphs_per_layer(mut self, max: usize) -> Self {
        self.max_glyphs_per_layer = max;
        self
    }

    /// Get the number of Glyphs that can be placed
    pub(crate) fn glyph_width(&self) -> usize {
        self.max_glyphs_per_layer
    }
}
