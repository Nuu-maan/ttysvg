use std::path::PathBuf;
use std::time::Duration;

use ttysvg::tape::write::{dur, quoted, source};
use ttysvg::tape::{parse, Config, Key, Op};

fn roundtrip(cfg: &Config, ops: &[Op]) -> (Config, Vec<Op>) {
    let src = source(cfg, ops);
    let tape = parse(&src).unwrap_or_else(|e| panic!("could not reparse\n{src}\n{e}"));
    (tape.config, tape.ops)
}

#[test]
fn a_default_config_survives_a_round_trip() {
    let cfg = Config::default();
    let (back, ops) = roundtrip(&cfg, &[]);

    assert_eq!(back.cols, cfg.cols);
    assert_eq!(back.rows, cfg.rows);
    assert_eq!(back.theme, cfg.theme);
    assert_eq!(back.padding, cfg.padding);
    assert_eq!(back.output, cfg.output);
    assert!(ops.is_empty());
}

#[test]
fn every_setting_survives_a_round_trip() {
    let cfg = Config {
        output: PathBuf::from("docs/out.svg"),
        theme: "nord".into(),
        cols: 72,
        rows: 18,
        font_size: 15.0,
        advance: 8.5,
        line_height: 20.0,
        padding: 26.0,
        shell: vec!["bash".into(), "-i".into()],
        trim_idle: Some(Duration::from_millis(600)),
        tail: Duration::from_millis(2500),
        type_delay: Duration::from_millis(45),
        speed: 1.6,
        window: true,
        title: "demo".into(),
        loop_forever: false,
        redact: vec!["sk-[A-Za-z0-9]+".into()],
        sanitize: true,
        ..Config::default()
    };

    let (back, _) = roundtrip(&cfg, &[]);

    assert_eq!(back.output, cfg.output);
    assert_eq!(back.theme, cfg.theme);
    assert_eq!(back.cols, cfg.cols);
    assert_eq!(back.rows, cfg.rows);
    assert_eq!(back.font_size, cfg.font_size);
    assert_eq!(back.advance, cfg.advance);
    assert_eq!(back.line_height, cfg.line_height);
    assert_eq!(back.padding, cfg.padding);
    assert_eq!(back.shell, cfg.shell);
    assert_eq!(back.trim_idle, cfg.trim_idle);
    assert_eq!(back.tail, cfg.tail);
    assert_eq!(back.type_delay, cfg.type_delay);
    assert_eq!(back.speed, cfg.speed);
    assert_eq!(back.window, cfg.window);
    assert_eq!(back.title, cfg.title);
    assert_eq!(back.loop_forever, cfg.loop_forever);
    assert_eq!(back.redact, cfg.redact);
    assert_eq!(back.sanitize, cfg.sanitize);
}

#[test]
fn trim_idle_off_survives_a_round_trip() {
    let cfg = Config {
        trim_idle: None,
        ..Config::default()
    };
    let (back, _) = roundtrip(&cfg, &[]);
    assert_eq!(back.trim_idle, None);
}

#[test]
fn every_op_survives_a_round_trip() {
    let ops = vec![
        Op::Sleep(Duration::from_millis(900)),
        Op::Type("git status --short".into()),
        Op::Key(Key::Enter),
        Op::Sleep(Duration::from_secs(2)),
        Op::Key(Key::Tab),
        Op::Key(Key::Backspace),
        Op::Key(Key::Escape),
        Op::Key(Key::Space),
        Op::Key(Key::Up),
        Op::Key(Key::Down),
        Op::Key(Key::Left),
        Op::Key(Key::Right),
        Op::Key(Key::Ctrl('c')),
        Op::Wait {
            text: "done".into(),
            timeout: Duration::from_secs(30),
        },
    ];

    let (_, back) = roundtrip(&Config::default(), &ops);
    assert_eq!(back, ops);
}

#[test]
fn windows_paths_and_quotes_survive_a_round_trip() {
    let ops = vec![
        Op::Type(r".\scripts\showcase.ps1".into()),
        Op::Type(r#"echo "hi there""#.into()),
        Op::Type(r"C:\temp\new\report.txt".into()),
        Op::Type("echo # not a comment".into()),
    ];
    let cfg = Config {
        output: PathBuf::from(r"docs\out.svg"),
        title: r#"say "hi""#.into(),
        ..Config::default()
    };

    let (back_cfg, back_ops) = roundtrip(&cfg, &ops);
    assert_eq!(back_ops, ops);
    assert_eq!(back_cfg.output, cfg.output);
    assert_eq!(back_cfg.title, cfg.title);
}

#[test]
fn durations_read_the_way_a_person_would_write_them() {
    assert_eq!(dur(Duration::from_millis(900)), "900ms");
    assert_eq!(dur(Duration::from_millis(2000)), "2s");
    assert_eq!(dur(Duration::from_millis(2500)), "2500ms");
    assert_eq!(dur(Duration::ZERO), "0ms");
}

#[test]
fn quoting_escapes_only_quotes_and_backslashes() {
    assert_eq!(quoted("plain"), r#""plain""#);
    assert_eq!(quoted(r"a\b"), r#""a\\b""#);
    assert_eq!(quoted(r#"say "hi""#), r#""say \"hi\"""#);
}

#[test]
fn the_written_tape_is_readable_by_a_person() {
    let cfg = Config {
        cols: 90,
        rows: 20,
        window: true,
        title: "demo".into(),
        sanitize: true,
        ..Config::default()
    };
    let ops = vec![
        Op::Sleep(Duration::from_millis(900)),
        Op::Type("cargo test".into()),
        Op::Key(Key::Enter),
        Op::Sleep(Duration::from_secs(2)),
    ];

    let src = source(&cfg, &ops);
    let mut lines = src.lines();

    assert_eq!(lines.next(), Some(r#"output "demo.svg""#));
    assert_eq!(lines.next(), Some(r#"theme "tokyonight""#));
    assert_eq!(lines.next(), Some("width 90"));
    assert_eq!(lines.next(), Some("height 20"));
    assert!(src.contains("sanitize on"));
    assert!(src.contains("\n\nsleep 900ms\n"));
    assert!(src.contains(r#"type "cargo test""#));
}
