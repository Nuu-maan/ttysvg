use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, List, ListItem, Padding, Paragraph, Wrap};
use ratatui::Frame as Ui;

use crate::emit::human_size;
use crate::svg::text::num;
use crate::svg::theme::Theme;
use crate::tape::write::dur;
use crate::ui::app::{App, Step, FIELDS};
use crate::ui::{preset, preview};

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;
const WARN: Color = Color::Yellow;
const GOOD: Color = Color::Green;

pub fn draw(f: &mut Ui, app: &App) {
    let area = f.area();
    let root = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM))
        .title(Span::styled(
            " ttysvg ",
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(hints(app).centered());
    let inner = root.inner(area);
    f.render_widget(root, area);

    let [crumbs, body, status] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(inner);

    f.render_widget(Paragraph::new(crumb_line(app)), crumbs);
    caret(f, status.x, status.y);

    match app.step {
        Step::Preset => preset_body(f, app, body),
        Step::Command => command_body(f, app, body),
        Step::Theme => theme_body(f, app, body),
        Step::Details => details_body(f, app, body),
        Step::Review => review_body(f, app, body),
        Step::Done => done_body(f, app, body),
    }

    f.render_widget(Paragraph::new(status_line(app)), status);
}

fn cap(area: Rect, rows: u16) -> Rect {
    Rect {
        height: area.height.min(rows),
        ..area
    }
}

fn caret(f: &mut Ui, x: u16, y: u16) {
    let area = f.area();
    if area.is_empty() {
        return;
    }
    f.set_cursor_position((
        x.min(area.right().saturating_sub(1)),
        y.min(area.bottom().saturating_sub(1)),
    ));
}

fn panel(title: &str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM))
        .title(Span::styled(format!(" {title} "), Style::new().fg(DIM)))
        .padding(Padding::horizontal(1))
}

fn crumb_line(app: &App) -> Line<'static> {
    let here = match app.step {
        Step::Done => usize::MAX,
        other => Step::FLOW.iter().position(|s| *s == other).unwrap_or(0),
    };

    let mut spans = vec![Span::raw(" ")];
    for (i, step) in Step::FLOW.iter().enumerate() {
        let style = if i == here {
            Style::new()
                .fg(Color::Black)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD)
        } else if i < here {
            Style::new().fg(GOOD)
        } else {
            Style::new().fg(DIM)
        };
        spans.push(Span::styled(format!(" {} {} ", i + 1, step.label()), style));
        if i + 1 < Step::FLOW.len() {
            spans.push(Span::styled("\u{2192}", Style::new().fg(DIM)));
        }
    }
    if app.step == Step::Done {
        spans.push(Span::styled(
            "  done",
            Style::new().fg(GOOD).add_modifier(Modifier::BOLD),
        ));
    }
    Line::from(spans)
}

fn hints(app: &App) -> Line<'static> {
    let keys: &[(&str, &str)] = match app.step {
        Step::Preset => &[("up down", "choose"), ("enter", "next"), ("q", "quit")],
        Step::Command => &[
            ("type", "edit"),
            ("enter", "new line"),
            ("enter twice", "next"),
            ("shift tab", "back"),
        ],
        Step::Theme => &[("up down", "choose"), ("enter", "next"), ("esc", "back")],
        Step::Details => &[
            ("up down", "field"),
            ("left right", "change"),
            ("enter", "next"),
            ("esc", "back"),
        ],
        Step::Review => &[("enter", "record"), ("esc", "back"), ("q", "quit")],
        Step::Done => &[
            ("left right", "theme"),
            ("w", "window"),
            ("o", "open"),
            ("r", "redo"),
            ("e", "edit"),
            ("q", "quit"),
        ],
    };

    let mut spans = vec![Span::raw(" ")];
    for (i, (key, what)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" \u{b7} ", Style::new().fg(DIM)));
        }
        spans.push(Span::styled(
            (*key).to_string(),
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(" {what}"), Style::new().fg(DIM)));
    }
    spans.push(Span::raw(" "));
    Line::from(spans)
}

fn status_line(app: &App) -> Line<'static> {
    match &app.status {
        Some(message) => Line::from(vec![
            Span::raw(" "),
            Span::styled(message.clone(), Style::new().fg(WARN)),
        ]),
        None => Line::raw(""),
    }
}

fn kv(key: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key:<15}"), Style::new().fg(DIM)),
        Span::raw(value),
    ])
}

fn onoff(v: bool) -> String {
    if v {
        "on".into()
    } else {
        "off".into()
    }
}

fn preset_body(f: &mut Ui, app: &App, area: Rect) {
    let area = cap(area, 13);
    let [left, right] =
        Layout::horizontal([Constraint::Length(24), Constraint::Min(24)]).areas(area);
    caret(f, left.x + 2, left.y + 1 + app.preset as u16);

    let items: Vec<ListItem> = preset::ALL
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let chosen = i == app.preset;
            ListItem::new(Line::from(vec![
                Span::styled(
                    if chosen { "\u{25b8} " } else { "  " },
                    Style::new().fg(ACCENT),
                ),
                Span::styled(
                    p.name,
                    if chosen {
                        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
                    } else {
                        Style::new()
                    },
                ),
            ]))
        })
        .collect();
    f.render_widget(List::new(items).block(panel("presets")), left);

    let p = app.preset();
    let text = Text::from(vec![
        Line::from(Span::styled(
            p.about,
            Style::new().add_modifier(Modifier::ITALIC),
        )),
        Line::raw(""),
        kv("size", format!("{} x {} cells", p.cols, p.rows)),
        kv("padding", num(p.padding)),
        kv("font size", num(p.font_size)),
        kv("window chrome", onoff(p.window)),
        kv(
            "trim idle",
            match p.trim_idle {
                Some(v) => dur(v),
                None => "off".into(),
            },
        ),
        kv("tail", dur(p.tail)),
        kv("pause", format!("{} between commands", dur(p.pause))),
    ]);
    f.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .block(panel("what it sets")),
        right,
    );
}

fn command_body(f: &mut Ui, app: &App, area: Rect) {
    let area = cap(area, app.commands.len() as u16 + 8);
    let [top, help] = Layout::vertical([Constraint::Min(3), Constraint::Length(6)]).areas(area);
    caret(
        f,
        top.x + 7 + app.commands[app.line].chars().count() as u16,
        top.y + 1 + app.line as u16,
    );

    let lines: Vec<Line> = app
        .commands
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let here = i == app.line;
            let mut spans = vec![Span::styled(
                format!("{:>3}  ", i + 1),
                Style::new().fg(DIM),
            )];
            spans.push(Span::styled(
                text.clone(),
                if here {
                    Style::new().add_modifier(Modifier::BOLD)
                } else {
                    Style::new()
                },
            ));
            Line::from(spans)
        })
        .collect();

    f.render_widget(
        Paragraph::new(Text::from(lines)).block(panel("commands, run in order")),
        top,
    );

    let text = Text::from(vec![
        Line::from(vec![
            Span::styled("Each line is typed into ", Style::new().fg(DIM)),
            Span::raw(app.cfg.shell.first().cloned().unwrap_or_default()),
            Span::styled(" and followed by enter.", Style::new().fg(DIM)),
        ]),
        Line::from(Span::styled(
            format!(
                "The recording waits {} after each one before moving on.",
                dur(app.pause)
            ),
            Style::new().fg(DIM),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "Press enter on an empty last line to move to the theme.",
            Style::new().fg(DIM),
        )),
    ]);
    f.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .block(panel("how this runs")),
        help,
    );
}

fn theme_body(f: &mut Ui, app: &App, area: Rect) {
    let area = cap(area, 12);
    let [left, right] =
        Layout::horizontal([Constraint::Length(20), Constraint::Min(24)]).areas(area);
    caret(f, left.x + 2, left.y + 1 + app.theme as u16);

    let items: Vec<ListItem> = app
        .themes()
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let chosen = i == app.theme;
            ListItem::new(Line::from(vec![
                Span::styled(
                    if chosen { "\u{25b8} " } else { "  " },
                    Style::new().fg(ACCENT),
                ),
                Span::styled(
                    (*name).to_string(),
                    if chosen {
                        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
                    } else {
                        Style::new()
                    },
                ),
            ]))
        })
        .collect();
    f.render_widget(List::new(items).block(panel("themes")), left);

    let [swatches, shot] =
        Layout::vertical([Constraint::Length(4), Constraint::Min(3)]).areas(right);

    let theme = Theme::load(app.theme_name()).ok();

    f.render_widget(
        Paragraph::new(match &theme {
            Some(t) => Text::from(vec![preview::swatch(&t.dark), preview::swatch(t.light())]),
            None => Text::raw("theme could not be loaded"),
        })
        .block(panel("palette, dark then light")),
        swatches,
    );

    let block = panel("preview");
    let area_inner = block.inner(shot);
    f.render_widget(block, shot);

    if let Some(t) = &theme {
        let sample = preview::sample();
        let lines = preview::lines(
            &sample,
            &t.dark,
            area_inner.width as usize,
            area_inner.height as usize,
        );
        f.render_widget(Paragraph::new(Text::from(lines)), area_inner);
    }
}

fn details_body(f: &mut Ui, app: &App, area: Rect) {
    let area = cap(area, FIELDS as u16 + 7);
    let [top, help] =
        Layout::vertical([Constraint::Length(FIELDS as u16 + 2), Constraint::Min(3)]).areas(area);

    let values = [
        ("output", app.output.clone()),
        ("title", app.cfg.title.clone()),
        ("window chrome", onoff(app.cfg.window)),
        ("sanitize", onoff(app.cfg.sanitize)),
        ("pause", dur(app.pause)),
    ];

    let lines: Vec<Line> = values
        .iter()
        .enumerate()
        .map(|(i, (label, value))| {
            let here = i == app.field;
            let mark = if here { "\u{25b8} " } else { "  " };
            let value_style = if here {
                Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::new()
            };
            Line::from(vec![
                Span::styled(mark, Style::new().fg(ACCENT)),
                Span::styled(format!("{label:<15}"), Style::new().fg(DIM)),
                Span::styled(value.clone(), value_style),
            ])
        })
        .collect();

    caret(
        f,
        top.x + 19 + values[app.field.min(FIELDS - 1)].1.chars().count() as u16,
        top.y + 1 + app.field as u16,
    );

    f.render_widget(
        Paragraph::new(Text::from(lines)).block(panel("details")),
        top,
    );

    let about = match app.field {
        0 => "Where the SVG is written. A tape with the same name lands next to it, so the recording can be rebuilt without opening this screen again.",
        1 => "Shown in the window chrome. Ignored when the chrome is off.",
        2 => "Draws a rounded window with three dots around the terminal. Good for social posts, usually noise inside a README.",
        3 => "Masks your home directory, user name and machine name everywhere they appear, before any frame is written to disk.",
        _ => "How long the recording holds after each command before typing the next one. Long enough to read, short enough to keep the file small.",
    };
    f.render_widget(
        Paragraph::new(Span::styled(about, Style::new().fg(DIM)))
            .wrap(Wrap { trim: true })
            .block(panel("about this field")),
        help,
    );
}

fn review_body(f: &mut Ui, app: &App, area: Rect) {
    let area = cap(area, 16 + app.lines().len() as u16);
    let cfg = app.config();
    let mut lines = vec![
        kv("preset", app.preset().name.to_string()),
        kv("size", format!("{} x {} cells", cfg.cols, cfg.rows)),
        kv("theme", cfg.theme.clone()),
        kv("window chrome", onoff(cfg.window)),
        kv("sanitize", onoff(cfg.sanitize)),
        kv("output", cfg.output.display().to_string()),
        kv("tape", app.tape_path().display().to_string()),
        Line::raw(""),
        Line::from(Span::styled("commands", Style::new().fg(DIM))),
    ];
    for command in app.lines() {
        lines.push(Line::from(vec![
            Span::styled("  $ ", Style::new().fg(GOOD)),
            Span::raw(command.to_string()),
        ]));
    }
    if app.lines().is_empty() {
        lines.push(Line::from(Span::styled(
            "  nothing yet, go back to step 2",
            Style::new().fg(WARN),
        )));
    }

    lines.push(Line::raw(""));
    if app.too_wide() {
        lines.push(Line::from(Span::styled(
            format!(
                "This terminal is {} columns and the recording is {}. The SVG is fine, the live preview below will wrap.",
                app.term_cols, cfg.cols
            ),
            Style::new().fg(WARN),
        )));
        lines.push(Line::raw(""));
    }
    lines.push(Line::from(Span::styled(
        "Press enter to record. The session takes over this terminal so you can watch it run, then comes back here.",
        Style::new().add_modifier(Modifier::BOLD),
    )));

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: true })
            .block(panel("about to record")),
        area,
    );
}

fn done_body(f: &mut Ui, app: &App, area: Rect) {
    let Some(done) = &app.done else {
        return;
    };

    let area = cap(area, 19);
    let [left, right] =
        Layout::horizontal([Constraint::Length(40), Constraint::Min(24)]).areas(area);

    let lines = vec![
        kv("svg", done.svg.display().to_string()),
        kv("tape", done.tape.display().to_string()),
        kv("frames", done.frames.to_string()),
        kv("runtime", format!("{:.1}s", done.secs)),
        kv("size", human_size(done.bytes)),
        Line::raw(""),
        kv("theme", app.theme_name().to_string()),
        kv("window chrome", onoff(app.cfg.window)),
        Line::raw(""),
        Line::from(Span::styled(
            "Arrow keys restyle the file from the frames already captured. Nothing runs a second time.",
            Style::new().fg(DIM),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "Rebuild it any time with",
            Style::new().fg(DIM),
        )),
        Line::from(Span::styled(
            format!("ttysvg build {}", done.tape.display()),
            Style::new().fg(ACCENT),
        )),
    ];

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: true })
            .block(panel("recorded")),
        left,
    );

    let block = panel("last frame");
    let inner = block.inner(right);
    f.render_widget(block, right);

    if let Ok(theme) = Theme::load(app.theme_name()) {
        let lines = preview::lines(
            &done.last,
            &theme.dark,
            inner.width as usize,
            inner.height as usize,
        );
        f.render_widget(Paragraph::new(Text::from(lines)), inner);
    }
}
