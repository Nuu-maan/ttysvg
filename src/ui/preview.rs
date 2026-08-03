use ratatui::style::{Color as UiColor, Modifier, Style as UiStyle};
use ratatui::text::{Line, Span};

use crate::svg::theme::{xterm256, Palette};
use crate::term::{Color, Frame, Row, Run, Style};

pub fn hex(value: &str) -> Option<UiColor> {
    let raw = value.strip_prefix('#')?;
    if raw.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&raw[0..2], 16).ok()?;
    let g = u8::from_str_radix(&raw[2..4], 16).ok()?;
    let b = u8::from_str_radix(&raw[4..6], 16).ok()?;
    Some(UiColor::Rgb(r, g, b))
}

fn resolve(color: Color, palette: &Palette, background: bool) -> Option<UiColor> {
    match color {
        Color::Default => hex(if background { &palette.bg } else { &palette.fg }),
        Color::Idx(i) if (i as usize) < palette.ansi.len() => hex(&palette.ansi[i as usize]),
        Color::Idx(i) => hex(&xterm256(i)),
        Color::Rgb(r, g, b) => Some(UiColor::Rgb(r, g, b)),
    }
}

fn style_of(style: &Style, palette: &Palette) -> UiStyle {
    let mut out = UiStyle::default();
    if let Some(fg) = resolve(style.fg, palette, false) {
        out = out.fg(fg);
    }
    if let Some(bg) = resolve(style.bg, palette, true) {
        out = out.bg(bg);
    }
    if style.bold {
        out = out.add_modifier(Modifier::BOLD);
    }
    if style.italic {
        out = out.add_modifier(Modifier::ITALIC);
    }
    if style.underline {
        out = out.add_modifier(Modifier::UNDERLINED);
    }
    out
}

fn clip(text: &str, room: usize) -> (String, usize) {
    let taken: String = text.chars().take(room).collect();
    let used = taken.chars().count();
    (taken, used)
}

pub fn lines(frame: &Frame, palette: &Palette, cols: usize, rows: usize) -> Vec<Line<'static>> {
    let blank = UiStyle::default().bg(resolve(Color::Default, palette, true).unwrap_or_default());
    let mut out = Vec::with_capacity(rows);

    for row in frame.rows.iter().take(rows) {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut at = 0usize;

        for run in &row.runs {
            let start = run.col as usize;
            if start >= cols {
                break;
            }
            if start > at {
                spans.push(Span::styled(" ".repeat(start - at), blank));
                at = start;
            }
            let (text, used) = clip(&run.text, cols - at);
            if used == 0 {
                continue;
            }
            spans.push(Span::styled(text, style_of(&run.style, palette)));
            at += used;
        }

        if at < cols {
            spans.push(Span::styled(" ".repeat(cols - at), blank));
        }
        out.push(Line::from(spans));
    }

    while out.len() < rows {
        out.push(Line::from(Span::styled(" ".repeat(cols), blank)));
    }
    out
}

pub fn swatch(palette: &Palette) -> Line<'static> {
    let mut spans = Vec::with_capacity(palette.ansi.len());
    for value in &palette.ansi {
        let style = match hex(value) {
            Some(c) => UiStyle::default().fg(c),
            None => UiStyle::default(),
        };
        spans.push(Span::styled("\u{2588}\u{2588}", style));
    }
    Line::from(spans)
}

fn run(col: u16, text: &str, fg: Color, bold: bool) -> Run {
    Run {
        col,
        width: text.chars().count() as u16,
        text: text.to_string(),
        style: Style {
            fg,
            bold,
            ..Style::default()
        },
    }
}

fn row(runs: Vec<Run>) -> Row {
    Row { runs }
}

pub fn sample() -> Frame {
    let dim = Color::Idx(8);
    let green = Color::Idx(2);
    let blue = Color::Idx(4);
    let cyan = Color::Idx(6);
    let yellow = Color::Idx(3);
    let magenta = Color::Idx(5);

    Frame {
        rows: vec![
            row(vec![
                run(0, "$", green, true),
                run(
                    2,
                    "ttysvg build examples/banner.tape",
                    Color::Default,
                    false,
                ),
            ]),
            row(vec![
                run(0, "ttysvg:", blue, true),
                run(8, "building examples/banner.tape", dim, false),
                run(38, "(14 ops)", magenta, false),
            ]),
            row(vec![
                run(0, "ttysvg:", blue, true),
                run(8, "wrote", dim, false),
                run(14, "banner.svg", cyan, true),
                run(25, "61 frames", yellow, false),
                run(35, "7.2s", yellow, false),
                run(40, "74.1 KB", yellow, false),
            ]),
            row(vec![]),
            row(vec![
                run(0, "$", green, true),
                run(2, "\u{2588}", Color::Default, false),
            ]),
        ],
        cursor: None,
    }
}
