use std::time::Duration;

use ttysvg::optimize::{self, Options};
use ttysvg::redact::Rules;
use ttysvg::session::Capture;
use ttysvg::svg::theme::Theme;
use ttysvg::svg::{render, RenderOpts};
use ttysvg::tape::Config;
use ttysvg::term::{Color, Frame, Row, Run, Style};

fn run(text: &str, style: Style) -> Run {
    Run {
        col: 0,
        width: text.chars().count() as u16,
        text: text.to_string(),
        style,
    }
}

fn frame(text: &str, style: Style) -> Frame {
    Frame {
        rows: vec![Row {
            runs: vec![run(text, style)],
        }],
        cursor: Some((0, 4)),
    }
}

fn styled() -> Style {
    Style {
        fg: Color::Idx(4),
        bg: Color::Rgb(10, 20, 30),
        bold: true,
        italic: false,
        underline: true,
    }
}

fn raw() -> Vec<(Duration, Frame)> {
    vec![
        (Duration::from_millis(0), frame("boot", Style::default())),
        (Duration::from_millis(120), frame("boot", Style::default())),
        (Duration::from_millis(400), frame("done", styled())),
    ]
}

fn config() -> Config {
    Config {
        cols: 12,
        rows: 1,
        theme: "nord".into(),
        title: "capture".into(),
        window: true,
        ..Config::default()
    }
}

fn svg_of(cfg: &Config, frames: &[(Duration, Frame)]) -> String {
    let opts = Options {
        trim_idle: cfg.trim_idle,
        tail: cfg.tail,
        speed: cfg.speed,
        ..Options::default()
    };
    let tl = optimize::optimize(frames, &opts);
    let theme = Theme::load(&cfg.theme).unwrap();
    let render_opts = RenderOpts {
        theme,
        font_family: cfg.font_family.clone(),
        font_size: cfg.font_size,
        advance: cfg.advance,
        line_height: cfg.line_height,
        padding: cfg.padding,
        window: cfg.window,
        title: cfg.title.clone(),
        cols: cfg.cols,
        rows: cfg.rows,
        loop_forever: cfg.loop_forever,
    }
    .with_metrics();
    render(&tl, &render_opts)
}

#[test]
fn capture_round_trips_through_json() {
    let cfg = config();
    let capture = Capture::new(&["glowfetch".to_string()], &cfg, &raw(), &Rules::default());
    let back = Capture::from_json(&capture.to_json().unwrap()).unwrap();

    assert_eq!(back.command, vec!["glowfetch"]);
    assert_eq!(back.config.cols, 12);
    assert_eq!(back.config.theme, "nord");
    assert!(back.config.window);
    assert_eq!(back.shots.len(), capture.shots.len());
    assert_eq!(back.frames(), capture.frames());
}

#[test]
fn identical_neighbours_are_dropped_on_save() {
    let capture = Capture::new(&[], &config(), &raw(), &Rules::default());
    assert_eq!(capture.shots.len(), 2);
    assert_eq!(capture.shots[0].at_ms, 0);
    assert_eq!(capture.shots[1].at_ms, 400);
}

#[test]
fn styles_survive_the_trip() {
    let capture = Capture::new(&[], &config(), &raw(), &Rules::default());
    let back = Capture::from_json(&capture.to_json().unwrap()).unwrap();
    let last = &back.shots[1].frame.rows[0].runs[0];

    assert_eq!(last.text, "done");
    assert_eq!(last.style, styled());
    assert_eq!(back.shots[1].frame.cursor, Some((0, 4)));
}

#[test]
fn re_rendering_a_capture_matches_rendering_it_live() {
    let cfg = config();
    let live = svg_of(&cfg, &raw());

    let capture = Capture::new(&[], &cfg, &raw(), &Rules::default());
    let back = Capture::from_json(&capture.to_json().unwrap()).unwrap();
    let rerendered = svg_of(&back.config, &back.frames());

    assert_eq!(live, rerendered);
}

#[test]
fn overriding_the_theme_changes_only_the_palette() {
    let capture = Capture::new(&[], &config(), &raw(), &Rules::default());
    let mut swapped = capture.config.clone();
    swapped.theme = "gruvbox".into();

    let before = svg_of(&capture.config, &capture.frames());
    let after = svg_of(&swapped, &capture.frames());

    assert_ne!(before, after);
    assert_eq!(
        before.matches("<text").count(),
        after.matches("<text").count()
    );
}

#[test]
fn a_wrong_format_is_rejected() {
    let capture = Capture::new(&[], &config(), &raw(), &Rules::default());
    let bumped = capture
        .to_json()
        .unwrap()
        .replace("\"format\":1", "\"format\":99");
    let err = Capture::from_json(&bumped).unwrap_err().to_string();
    assert!(err.contains("format 99"), "{err}");
}

#[test]
fn an_empty_capture_is_rejected() {
    let capture = Capture::new(&[], &config(), &[], &Rules::default());
    let err = Capture::from_json(&capture.to_json().unwrap())
        .unwrap_err()
        .to_string();
    assert!(err.contains("no frames"), "{err}");
}
