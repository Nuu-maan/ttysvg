use ratatui::style::Color as UiColor;

use ttysvg::svg::theme::Theme;
use ttysvg::term::{Frame, Row, Run, Style};
use ttysvg::ui::preview;

fn palette() -> ttysvg::svg::theme::Palette {
    Theme::load("tokyonight").unwrap().dark
}

#[test]
fn hex_colors_become_true_colors() {
    assert_eq!(
        preview::hex("#1a2b3c"),
        Some(UiColor::Rgb(0x1a, 0x2b, 0x3c))
    );
    assert_eq!(preview::hex("1a2b3c"), None);
    assert_eq!(preview::hex("#abc"), None);
    assert_eq!(preview::hex("#zzzzzz"), None);
}

#[test]
fn every_preview_line_fills_the_box() {
    let frame = preview::sample();
    let lines = preview::lines(&frame, &palette(), 40, 8);

    assert_eq!(lines.len(), 8);
    for line in &lines {
        assert_eq!(line.width(), 40);
    }
}

#[test]
fn a_short_frame_is_padded_out_to_the_full_height() {
    let frame = Frame::blank(2);
    let lines = preview::lines(&frame, &palette(), 12, 6);

    assert_eq!(lines.len(), 6);
    for line in &lines {
        assert_eq!(line.width(), 12);
    }
}

#[test]
fn text_wider_than_the_box_is_cut_not_wrapped() {
    let frame = Frame {
        rows: vec![Row {
            runs: vec![Run {
                col: 0,
                width: 20,
                text: "abcdefghijklmnopqrst".into(),
                style: Style::default(),
            }],
        }],
        cursor: None,
    };

    let lines = preview::lines(&frame, &palette(), 6, 1);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].width(), 6);
    assert_eq!(lines[0].to_string(), "abcdef");
}

#[test]
fn a_run_that_starts_past_the_edge_is_dropped() {
    let frame = Frame {
        rows: vec![Row {
            runs: vec![
                Run {
                    col: 0,
                    width: 2,
                    text: "hi".into(),
                    style: Style::default(),
                },
                Run {
                    col: 40,
                    width: 4,
                    text: "gone".into(),
                    style: Style::default(),
                },
            ],
        }],
        cursor: None,
    };

    let lines = preview::lines(&frame, &palette(), 10, 1);
    assert_eq!(lines[0].to_string(), "hi        ");
}

#[test]
fn a_gap_between_runs_is_filled_with_spaces() {
    let frame = Frame {
        rows: vec![Row {
            runs: vec![
                Run {
                    col: 0,
                    width: 1,
                    text: "$".into(),
                    style: Style::default(),
                },
                Run {
                    col: 4,
                    width: 2,
                    text: "ls".into(),
                    style: Style::default(),
                },
            ],
        }],
        cursor: None,
    };

    let lines = preview::lines(&frame, &palette(), 8, 1);
    assert_eq!(lines[0].to_string(), "$   ls  ");
}

#[test]
fn the_swatch_shows_every_ansi_color() {
    let p = palette();
    let swatch = preview::swatch(&p);
    assert_eq!(swatch.spans.len(), p.ansi.len());
    assert_eq!(swatch.width(), p.ansi.len() * 2);
}

#[test]
fn the_sample_frame_renders_under_every_theme() {
    let frame = preview::sample();
    for name in Theme::names() {
        let theme = Theme::load(name).unwrap();
        for p in [&theme.dark, theme.light()] {
            let lines = preview::lines(&frame, p, 60, 6);
            assert_eq!(lines.len(), 6);
            assert!(lines.iter().all(|l| l.width() == 60));
        }
    }
}
