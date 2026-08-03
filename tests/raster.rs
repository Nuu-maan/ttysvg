use std::time::Duration;

use ttysvg::optimize::Timeline;
use ttysvg::raster::{self, MIN_DELAY_CS};
use ttysvg::svg::theme::Theme;
use ttysvg::svg::{render, RenderOpts};
use ttysvg::term::{Color, Frame, Row, Run, Style};

fn frame(text: &str, fg: Color) -> Frame {
    Frame {
        rows: vec![
            Row {
                runs: vec![Run {
                    col: 0,
                    width: text.chars().count() as u16,
                    text: text.to_string(),
                    style: Style {
                        fg,
                        ..Style::default()
                    },
                }],
            },
            Row::default(),
        ],
        cursor: Some((1, 0)),
    }
}

fn timeline() -> Timeline {
    Timeline {
        frames: vec![
            frame("building", Color::Idx(4)),
            frame("finished", Color::Idx(2)),
            frame("done", Color::Default),
        ],
        starts: vec![
            Duration::ZERO,
            Duration::from_millis(300),
            Duration::from_millis(900),
        ],
        total: Duration::from_millis(1500),
    }
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
        cols: 20,
        rows: 2,
        loop_forever: true,
        literal: None,
    }
    .with_metrics()
}

fn png_size(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "not a png");
    let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    (w, h)
}

#[test]
fn a_literal_palette_leaves_no_css_variables() {
    let plain = render(&timeline(), &opts());
    assert!(
        plain.contains("var(--"),
        "the normal path uses css variables"
    );

    let mut flat = opts();
    flat.literal = Some(flat.theme.dark.clone());
    let literal = render(&timeline(), &flat);

    assert!(
        !literal.contains("var(--"),
        "a literal render must not leave css variables, resvg cannot resolve them"
    );
    assert!(!literal.contains("prefers-color-scheme"));
    assert!(literal.contains(&opts().theme.dark.bg));
}

#[test]
fn light_and_dark_produce_different_pixels() {
    let tl = timeline();
    let dark = raster::png(&tl, &opts(), 1.0, false).unwrap();
    let light = raster::png(&tl, &opts(), 1.0, true).unwrap();

    assert_eq!(png_size(&dark), png_size(&light));
    assert_ne!(
        dark, light,
        "the light palette rendered identically to dark"
    );
}

#[test]
fn png_writes_a_bitmap_of_the_expected_size() {
    let bytes = raster::png(&timeline(), &opts(), 1.0, false).unwrap();
    let (w, h) = png_size(&bytes);

    let advance: f64 = 14.0 * 0.6;
    let line: f64 = 14.0 * 1.32;
    assert_eq!(w, (20.0 * advance + 20.0).round() as u32);
    assert_eq!(h, (2.0 * line + 20.0).round() as u32);
}

#[test]
fn scale_multiplies_the_bitmap() {
    let one = png_size(&raster::png(&timeline(), &opts(), 1.0, false).unwrap());
    let two = png_size(&raster::png(&timeline(), &opts(), 2.0, false).unwrap());

    assert_eq!(two.0, one.0 * 2);
    assert_eq!(two.1, one.1 * 2);
}

#[test]
fn gif_carries_every_frame_with_sane_delays() {
    let tl = timeline();
    let bytes = raster::gif(&tl, &opts(), 1.0, false).unwrap();
    assert_eq!(&bytes[..6], b"GIF89a", "not a gif");

    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    let mut reader = decoder.read_info(std::io::Cursor::new(&bytes)).unwrap();

    let expected = png_size(&raster::png(&tl, &opts(), 1.0, false).unwrap());
    assert_eq!(reader.width() as u32, expected.0);
    assert_eq!(reader.height() as u32, expected.1);

    let mut delays = Vec::new();
    while let Some(frame) = reader.read_next_frame().unwrap() {
        delays.push(frame.delay);
    }

    assert_eq!(delays.len(), tl.frames.len());
    assert!(
        delays.iter().all(|d| *d >= MIN_DELAY_CS),
        "a delay below {MIN_DELAY_CS} centiseconds is ignored by most viewers: {delays:?}"
    );

    let total: u32 = delays.iter().map(|d| *d as u32).sum();
    assert_eq!(total, 150, "the gif should last as long as the timeline");
}

fn chunks(bytes: &[u8], tag: &[u8; 4]) -> usize {
    bytes.windows(4).filter(|w| *w == tag).count()
}

#[test]
fn webp_is_animated_and_carries_every_frame() {
    let tl = timeline();
    let bytes = raster::webp(&tl, &opts(), 1.0, false).unwrap();

    assert_eq!(&bytes[..4], b"RIFF", "not a riff container");
    assert_eq!(&bytes[8..12], b"WEBP", "not a webp");
    assert_eq!(chunks(&bytes, b"ANIM"), 1, "webp is not animated");
    assert_eq!(chunks(&bytes, b"ANMF"), tl.frames.len());
}

#[test]
fn webp_beats_the_gif_on_size() {
    let tl = timeline();
    let gif = raster::gif(&tl, &opts(), 1.0, false).unwrap();
    let webp = raster::webp(&tl, &opts(), 1.0, false).unwrap();
    assert!(
        webp.len() < gif.len(),
        "webp {} bytes is not smaller than gif {} bytes",
        webp.len(),
        gif.len()
    );
}

#[test]
fn apng_is_animated_and_carries_every_frame() {
    let tl = timeline();
    let bytes = raster::apng(&tl, &opts(), 1.0, false).unwrap();

    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "not a png");
    assert_eq!(chunks(&bytes, b"acTL"), 1, "png is not animated");
    assert_eq!(chunks(&bytes, b"fcTL"), tl.frames.len());
    assert_eq!(
        png_size(&bytes),
        png_size(&raster::png(&tl, &opts(), 1.0, false).unwrap())
    );
}

#[test]
fn apng_only_stores_the_part_of_the_screen_that_changed() {
    let tl = timeline();
    let full = tl.frames.len() * (270 * 57 * 4);
    let bytes = raster::apng(&tl, &opts(), 1.0, false).unwrap();
    assert!(
        bytes.len() < full / 4,
        "apng kept {} bytes, close to {} raw frames",
        bytes.len(),
        tl.frames.len()
    );
}

#[test]
fn the_changed_box_is_the_smallest_one_that_covers_the_difference() {
    let (w, h) = (4u32, 3u32);
    let old = vec![0u8; (w * h * 4) as usize];

    let mut new = old.clone();
    let touch = |buf: &mut Vec<u8>, x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        buf[i] = 255;
    };
    touch(&mut new, 1, 1);
    touch(&mut new, 2, 2);

    assert_eq!(
        raster::dirty(&old, &new, w, h),
        raster::Rect {
            x: 1,
            y: 1,
            w: 2,
            h: 2
        }
    );
}

#[test]
fn an_unchanged_frame_still_writes_something_legal() {
    let (w, h) = (4u32, 3u32);
    let same = vec![7u8; (w * h * 4) as usize];
    let patch = raster::dirty(&same, &same, w, h);
    assert_eq!(patch.w, 1, "apng forbids a zero sized frame");
    assert_eq!(patch.h, 1);
}

#[test]
fn a_still_can_be_taken_from_any_moment() {
    let tl = timeline();
    let first = raster::png_at(&tl, &opts(), 1.0, false, Some(Duration::ZERO)).unwrap();
    let middle =
        raster::png_at(&tl, &opts(), 1.0, false, Some(Duration::from_millis(400))).unwrap();
    let last = raster::png(&tl, &opts(), 1.0, false).unwrap();

    assert_ne!(first, middle, "the two moments rendered the same");
    assert_ne!(middle, last);
    assert_eq!(
        last,
        raster::png_at(&tl, &opts(), 1.0, false, Some(Duration::from_secs(99))).unwrap(),
        "a time past the end should land on the last frame"
    );
}

#[test]
fn a_moment_lands_on_the_frame_showing_at_that_time() {
    let tl = timeline();
    assert_eq!(raster::frame_at(&tl, Duration::ZERO), 0);
    assert_eq!(raster::frame_at(&tl, Duration::from_millis(299)), 0);
    assert_eq!(raster::frame_at(&tl, Duration::from_millis(300)), 1);
    assert_eq!(raster::frame_at(&tl, Duration::from_millis(899)), 1);
    assert_eq!(raster::frame_at(&tl, Duration::from_millis(900)), 2);
}

#[test]
fn delays_follow_the_gaps_between_frames() {
    let tl = timeline();
    assert_eq!(raster::delay_cs(&tl, 0), 30);
    assert_eq!(raster::delay_cs(&tl, 1), 60);
    assert_eq!(raster::delay_cs(&tl, 2), 60);
}

#[test]
fn a_zero_length_gap_still_gets_a_visible_delay() {
    let tl = Timeline {
        frames: vec![frame("a", Color::Default), frame("b", Color::Default)],
        starts: vec![Duration::ZERO, Duration::ZERO],
        total: Duration::from_millis(20),
    };
    assert_eq!(raster::delay_cs(&tl, 0), MIN_DELAY_CS);
}
