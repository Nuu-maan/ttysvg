use ttysvg::svg::theme::{self, Theme};
use ttysvg::svg::{render, RenderOpts};

use std::time::Duration;
use ttysvg::optimize::Timeline;
use ttysvg::term::Frame;

fn opts(name: &str) -> RenderOpts {
    RenderOpts {
        theme: Theme::load(name).unwrap(),
        font_family: "monospace".into(),
        font_size: 14.0,
        advance: 0.0,
        line_height: 0.0,
        padding: 10.0,
        window: true,
        title: "demo".into(),
        cols: 10,
        rows: 2,
        loop_forever: true,
        literal: None,
    }
    .with_metrics()
}

fn one_frame() -> Timeline {
    Timeline {
        frames: vec![Frame::blank(2)],
        starts: vec![Duration::ZERO],
        total: Duration::from_millis(10),
    }
}

#[test]
fn every_built_in_theme_loads_and_validates() {
    for name in Theme::names() {
        let theme = Theme::load(name).unwrap_or_else(|e| panic!("theme {name}: {e}"));
        theme
            .validate()
            .unwrap_or_else(|e| panic!("theme {name}: {e}"));
        assert_eq!(theme.dark.ansi.len(), 16, "{name}");
    }
}

#[test]
fn every_built_in_theme_is_readable() {
    for name in Theme::names() {
        let theme = Theme::load(name).unwrap();
        let warnings = theme.warnings();
        assert!(
            warnings.is_empty(),
            "theme {name} would be hard to read: {warnings:?}"
        );
    }
}

#[test]
fn a_theme_without_a_light_variant_falls_back_to_dark() {
    let theme = Theme::load("dracula").unwrap();
    assert!(!theme.has_light());
    assert_eq!(theme.light().bg, theme.dark.bg);
}

#[test]
fn a_dark_only_theme_emits_no_light_switch() {
    let svg = render(&one_frame(), &opts("dracula"));
    assert!(
        !svg.contains("prefers-color-scheme"),
        "there is no light palette to switch to"
    );

    let paired = render(&one_frame(), &opts("github"));
    assert!(paired.contains("prefers-color-scheme"));
}

#[test]
fn two_themes_can_be_paired_by_name() {
    let theme = Theme::load("dracula,github").unwrap();
    assert_eq!(theme.dark.bg, Theme::load("dracula").unwrap().dark.bg);
    assert_eq!(theme.light().bg, Theme::load("github").unwrap().light().bg);
    assert!(theme.has_light());
}

#[test]
fn a_pair_can_name_its_slots_in_either_order() {
    let a = Theme::load("light=github,dark=dracula").unwrap();
    let b = Theme::load("dracula,github").unwrap();
    assert_eq!(a.dark.bg, b.dark.bg);
    assert_eq!(a.light().bg, b.light().bg);
}

#[test]
fn pairing_with_a_dark_only_theme_borrows_its_dark_palette() {
    let theme = Theme::load("github,monokai").unwrap();
    assert_eq!(theme.light().bg, Theme::load("monokai").unwrap().dark.bg);
}

#[test]
fn an_unknown_theme_says_what_is_available() {
    let err = Theme::load("nosuchtheme").unwrap_err().to_string();
    assert!(err.contains("nosuchtheme"));
    assert!(err.contains("tokyonight"));
}

#[test]
fn the_extra_palette_slots_reach_the_svg() {
    let svg = render(&one_frame(), &opts("dracula"));
    let theme = Theme::load("dracula").unwrap();

    assert!(svg.contains("--chrome:"), "chrome color never emitted");
    assert!(svg.contains("--border:"), "border color never emitted");
    assert!(svg.contains("--cur:"), "cursor color never emitted");
    assert!(svg.contains("var(--btn0)"), "buttons never themed");
    assert!(svg.contains(theme.dark.chrome()));
    assert!(svg.contains(theme.dark.border()));
}

#[test]
fn a_theme_without_the_extra_slots_renders_as_it_always_did() {
    let svg = render(&one_frame(), &opts("tokyonight"));
    assert!(!svg.contains("--chrome:"));
    assert!(!svg.contains("--border:"));
    assert!(!svg.contains("--cur:"));
    assert!(
        svg.contains("#ff5f57"),
        "the default buttons should survive"
    );
}

#[test]
fn a_literal_render_inlines_the_extra_slots_too() {
    let mut o = opts("dracula");
    let theme = o.theme.clone();
    o.literal = Some(theme.dark.clone());
    let svg = render(&one_frame(), &o);

    assert!(!svg.contains("var(--"), "resvg cannot resolve variables");
    assert!(svg.contains(theme.dark.chrome()));
    assert!(svg.contains(theme.dark.buttons()[0]));
}

#[test]
fn a_palette_falls_back_before_it_fails() {
    let theme = Theme::load("tokyonight").unwrap();
    assert_eq!(theme.dark.cursor(), theme.dark.fg);
    assert_eq!(theme.dark.chrome(), theme.dark.bg);
    assert_eq!(theme.dark.buttons(), theme::BUTTONS.to_vec());
}

#[test]
fn contrast_is_measured_the_way_the_web_measures_it() {
    let ratio = theme::contrast("#ffffff", "#000000").unwrap();
    assert!((ratio - 21.0).abs() < 0.01, "{ratio}");
    assert_eq!(theme::contrast("#000000", "#000000").unwrap(), 1.0);
    assert!(theme::contrast("not a color", "#000000").is_none());
}

#[test]
fn colors_must_look_like_colors() {
    assert_eq!(theme::rgb("#1a2b3c"), Some((0x1a, 0x2b, 0x3c)));
    assert_eq!(theme::rgb("1a2b3c"), None);
    assert_eq!(theme::rgb("#abc"), None);
    assert_eq!(theme::rgb("#gggggg"), None);
}

#[test]
fn a_broken_theme_is_rejected_with_a_reason() {
    let src = r##"
name = "broken"
[dark]
fg = "#ffffff"
bg = "#000000"
ansi = ["#ffffff"]
"##;
    let theme: Theme = toml::from_str(src).unwrap();
    let err = theme.validate().unwrap_err().to_string();
    assert!(err.contains("16 ansi colors"), "{err}");

    let src = src.replace(
        r##"ansi = ["#ffffff"]"##,
        &format!("ansi = [{}]", vec!["\"nope\""; 16].join(",")),
    );
    let theme: Theme = toml::from_str(&src).unwrap();
    let err = theme.validate().unwrap_err().to_string();
    assert!(err.contains("#rrggbb"), "{err}");
}

#[test]
fn the_catppuccin_flavours_are_all_different() {
    let names = [
        "catppuccin-mocha",
        "catppuccin-macchiato",
        "catppuccin-frappe",
        "catppuccin-latte",
    ];
    let mut seen = Vec::new();
    for name in names {
        let theme = Theme::load(name).unwrap();
        assert!(
            !seen.contains(&theme.dark.bg),
            "{name} repeats a background"
        );
        seen.push(theme.dark.bg);
    }
}
