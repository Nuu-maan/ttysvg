use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Color {
    #[default]
    Default,
    Idx(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    pub fn is_default(self) -> bool {
        matches!(self, Color::Default)
    }
}

fn color_is_default(c: &Color) -> bool {
    c.is_default()
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn style_is_default(s: &Style) -> bool {
    *s == Style::default()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Style {
    #[serde(skip_serializing_if = "color_is_default")]
    pub fg: Color,
    #[serde(skip_serializing_if = "color_is_default")]
    pub bg: Color,
    #[serde(skip_serializing_if = "is_false")]
    pub bold: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub italic: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub underline: bool,
}

impl Style {
    pub fn is_blank(&self) -> bool {
        self.bg.is_default() && !self.underline
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Run {
    pub col: u16,
    pub width: u16,
    pub text: String,
    #[serde(default, skip_serializing_if = "style_is_default")]
    pub style: Style,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Row {
    pub runs: Vec<Run>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Frame {
    pub rows: Vec<Row>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<(u16, u16)>,
}

impl Frame {
    pub fn blank(rows: u16) -> Self {
        Frame {
            rows: vec![Row::default(); rows as usize],
            cursor: None,
        }
    }
}
