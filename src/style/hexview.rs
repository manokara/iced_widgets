//! Style for the [`Hexview`] widget.
//!
//! [`Hexview`]: ../native/hexview/struct.Hexview.html
use iced_native::{Color, Font};
//use crate::style::default_colors;

/// The apperance of an [`Hexview`].
///
/// [`Hexview`]: ../../native/hexview/struct.Hexview.html
#[derive(Debug, Clone)]
pub struct Style {
    /// Background color
    pub background_color: Color,
    /// Color for vertical separators
    pub line_color: Color,
    /// Color for offsets
    pub offset_color: Color,
    /// Color of bytes and text representation
    pub data_color: Color,
    /// Color for non-printable bytes.
    pub non_printable_color: Option<Color>,
    /// Color for the cursor
    pub cursor_color: Color,
    /// Font for column headers and offsets
    pub header_font: Font,
    /// Font for bytes and ascii
    pub data_font: Font,
}

/// A set of styles for an [`Hexview`]
///
/// [`Hexview`]: ../../native/hexview/struct.Hexview.html
pub trait StyleSheet {
    fn active(&self) -> Style;
}

pub struct Light;
pub struct Dark;

impl Light {
    const ACTIVE_STYLE: Style = Style {
        background_color: Color::from_rgb(1.0, 1.0, 1.0),
        line_color: Color::from_rgb(0.75, 0.75, 0.75),
        offset_color: Color::from_rgb(0.33, 0.33, 0.33),
        data_color: Color::from_rgb(0.196, 0.196, 0.196),
        non_printable_color: Some(Color::from_rgb(0.64, 0.64, 0.64)),
        cursor_color: Color::from_rgb(0.63, 0.63, 0.63),
        header_font: Font::Default,
        data_font: Font::Default,
    };
}

impl Dark {
    const ACTIVE_STYLE: Style = Style {
        background_color: Color::from_rgb(0.18, 0.21, 0.22),
        line_color: Color::from_rgb(0.278, 0.33, 0.345),
        offset_color: Color::from_rgb(0.294, 0.372, 0.372),
        data_color: Color::from_rgb(0.44, 0.53, 0.53),
        non_printable_color: Some(Color::from_rgb(0.27, 0.368, 0.368)),
        cursor_color: Color::from_rgb(0.15, 0.38, 0.44),
        header_font: Font::Default,
        data_font: Font::Default,
    };
}

impl StyleSheet for Light {
    fn active(&self) -> Style {
        Self::ACTIVE_STYLE
    }
}

impl StyleSheet for Dark {
    fn active(&self) -> Style {
        Self::ACTIVE_STYLE
    }
}

impl std::default::Default for Box<dyn StyleSheet> {
    fn default() -> Self {
        Box::new(Light)
    }
}

impl<T> From<T> for Box<dyn StyleSheet>
where
    T: 'static + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
