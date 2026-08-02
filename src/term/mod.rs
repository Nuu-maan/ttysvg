pub mod frame;

pub use frame::{Color, Frame, Row, Run, Style};

fn conv_color(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => Color::Default,
        vt100::Color::Idx(i) => Color::Idx(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

pub fn snapshot(screen: &vt100::Screen) -> Frame {
    let (rows, cols) = screen.size();
    let mut out = Vec::with_capacity(rows as usize);

    for r in 0..rows {
        let mut runs: Vec<Run> = Vec::new();
        let mut col = 0u16;

        while col < cols {
            let Some(cell) = screen.cell(r, col) else {
                col += 1;
                continue;
            };

            if cell.is_wide_continuation() {
                col += 1;
                continue;
            }

            let width = if cell.is_wide() { 2 } else { 1 };

            let mut style = Style {
                fg: conv_color(cell.fgcolor()),
                bg: conv_color(cell.bgcolor()),
                bold: cell.bold(),
                italic: cell.italic(),
                underline: cell.underline(),
            };
            if cell.inverse() {
                std::mem::swap(&mut style.fg, &mut style.bg);
            }

            let contents = cell.contents();
            let text = if contents.is_empty() {
                " ".to_string()
            } else {
                contents
            };

            match runs.last_mut() {
                Some(last) if last.style == style && last.col + last.width == col => {
                    last.text.push_str(&text);
                    last.width += width;
                }
                _ => runs.push(Run {
                    col,
                    width,
                    text,
                    style,
                }),
            }

            col += width;
        }

        runs.retain(|run| !(run.style.is_blank() && run.text.trim().is_empty()));

        out.push(Row { runs });
    }

    let cursor = if screen.hide_cursor() {
        None
    } else {
        let (cr, cc) = screen.cursor_position();
        (cr < rows && cc < cols).then_some((cr, cc))
    };

    Frame { rows: out, cursor }
}
