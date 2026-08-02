use std::time::Duration;

use ttysvg::optimize::{optimize, Options};
use ttysvg::svg::theme::Theme;
use ttysvg::svg::{render, RenderOpts};
use ttysvg::term::{self, Frame};

const COLS: u16 = 24;
const ROWS: u16 = 4;

fn capture(chunks: &[(u64, &str)]) -> Vec<(Duration, Frame)> {
    let mut parser = vt100::Parser::new(ROWS, COLS, 0);
    let mut out = vec![(Duration::ZERO, Frame::blank(ROWS))];
    for (ms, text) in chunks {
        parser.process(text.as_bytes());
        out.push((
            Duration::from_millis(*ms),
            term::snapshot(parser.screen()),
        ));
    }
    out
}

fn opts() -> RenderOpts {
    RenderOpts {
        theme: Theme::load("tokyonight").unwrap(),
        font_family: "monospace".into(),
        font_size: 14.0,
        advance: 0.0,
        line_height: 0.0,
        padding: 10.0,
        window: false,
        title: String::new(),
        cols: COLS,
        rows: ROWS,
        loop_forever: true,
    }
    .with_metrics()
}

#[test]
fn renders_colors_attributes_and_escapes() {
    let raw = capture(&[
        (100, "\x1b[32mgreen\x1b[0m plain\r\n"),
        (200, "\x1b[1;31mbold red\x1b[0m\r\n"),
        (300, "a & b < c > d\r\n"),
        (400, "\x1b[44mon blue\x1b[0m"),
    ]);
    let tl = optimize(&raw, &Options::default());
    insta::assert_snapshot!(render(&tl, &opts()));
}

#[test]
fn single_frame_emits_no_animation() {
    let raw = capture(&[(100, "static")]);
    let tl = optimize(
        &raw,
        &Options {
            min_frame: Duration::from_secs(10),
            ..Options::default()
        },
    );
    let svg = render(&tl, &opts());
    assert_eq!(tl.len(), 1);
    assert!(!svg.contains("@keyframes"));
}

#[test]
fn identical_frames_collapse() {
    let raw = capture(&[(100, "same"), (2000, ""), (4000, "")]);
    let tl = optimize(&raw, &Options::default());
    assert_eq!(tl.len(), 2);
}

#[test]
fn idle_gaps_are_clamped() {
    let raw = capture(&[(100, "a"), (30_000, "b")]);
    let tl = optimize(
        &raw,
        &Options {
            trim_idle: Some(Duration::from_millis(500)),
            tail: Duration::ZERO,
            ..Options::default()
        },
    );
    assert!(tl.total < Duration::from_millis(1200), "{:?}", tl.total);
}

#[test]
fn speed_scales_the_timeline() {
    let raw = capture(&[(100, "a"), (1000, "b")]);
    let base = optimize(
        &raw,
        &Options {
            trim_idle: None,
            tail: Duration::ZERO,
            ..Options::default()
        },
    );
    let fast = optimize(
        &raw,
        &Options {
            trim_idle: None,
            tail: Duration::ZERO,
            speed: 2.0,
            ..Options::default()
        },
    );
    assert!(fast.total < base.total);
}

#[test]
fn trailing_padding_is_not_emitted() {
    let raw = capture(&[(100, "hi")]);
    let svg = render(&optimize(&raw, &Options::default()), &opts());
    assert!(!svg.contains("hi   "), "trailing spaces leaked into output");
}
