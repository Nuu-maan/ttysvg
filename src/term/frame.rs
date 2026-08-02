#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Style {
    pub fn is_blank(&self) -> bool {
        self.bg.is_default() && !self.underline
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Run {
    pub col: u16,
    pub width: u16,
    pub text: String,
    pub style: Style,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Row {
    pub runs: Vec<Run>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Frame {
    pub rows: Vec<Row>,
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
