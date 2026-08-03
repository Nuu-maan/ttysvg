use std::fmt::Write as _;
use std::time::Duration;

use crate::svg::text::num;
use crate::tape::{default_shell, Config, Key, Op, DEFAULT_FONT};

pub fn source(cfg: &Config, ops: &[Op]) -> String {
    let d = Config::default();
    let mut s = String::with_capacity(512);

    let _ = writeln!(s, "output {}", quoted(&cfg.output.display().to_string()));
    let _ = writeln!(s, "theme {}", quoted(&cfg.theme));
    let _ = writeln!(s, "width {}", cfg.cols);
    let _ = writeln!(s, "height {}", cfg.rows);
    let _ = writeln!(s, "padding {}", num(cfg.padding));

    if cfg.font_family != DEFAULT_FONT {
        let _ = writeln!(s, "font {}", quoted(&cfg.font_family));
    }
    if cfg.font_size != d.font_size {
        let _ = writeln!(s, "font-size {}", num(cfg.font_size));
    }
    if cfg.advance != d.advance {
        let _ = writeln!(s, "advance {}", num(cfg.advance));
    }
    if cfg.line_height != d.line_height {
        let _ = writeln!(s, "line-height {}", num(cfg.line_height));
    }
    if cfg.shell != default_shell() {
        let words: Vec<String> = cfg.shell.iter().map(|w| quoted(w)).collect();
        let _ = writeln!(s, "shell {}", words.join(" "));
    }
    if cfg.window != d.window {
        let _ = writeln!(s, "window {}", onoff(cfg.window));
    }
    if !cfg.title.is_empty() {
        let _ = writeln!(s, "title {}", quoted(&cfg.title));
    }
    if cfg.type_delay != d.type_delay {
        let _ = writeln!(s, "type-delay {}", dur(cfg.type_delay));
    }
    if cfg.trim_idle != d.trim_idle {
        let _ = match cfg.trim_idle {
            Some(v) => writeln!(s, "trim-idle {}", dur(v)),
            None => writeln!(s, "trim-idle off"),
        };
    }
    if cfg.tail != d.tail {
        let _ = writeln!(s, "tail {}", dur(cfg.tail));
    }
    if cfg.speed != d.speed {
        let _ = writeln!(s, "speed {}", num(cfg.speed));
    }
    if cfg.loop_forever != d.loop_forever {
        let _ = writeln!(s, "loop {}", onoff(cfg.loop_forever));
    }
    for pattern in &cfg.redact {
        let _ = writeln!(s, "redact {}", quoted(pattern));
    }
    if cfg.sanitize != d.sanitize {
        let _ = writeln!(s, "sanitize {}", onoff(cfg.sanitize));
    }

    if !ops.is_empty() {
        s.push('\n');
    }
    for op in ops {
        let _ = writeln!(s, "{}", op_line(op));
    }

    s
}

fn op_line(op: &Op) -> String {
    match op {
        Op::Type(text) => format!("type {}", quoted(text)),
        Op::Sleep(d) => format!("sleep {}", dur(*d)),
        Op::Wait { text, timeout } => format!("wait {} {}", quoted(text), dur(*timeout)),
        Op::Key(key) => key_line(*key),
    }
}

fn key_line(key: Key) -> String {
    match key {
        Key::Enter => "enter".into(),
        Key::Tab => "tab".into(),
        Key::Backspace => "backspace".into(),
        Key::Escape => "escape".into(),
        Key::Space => "space".into(),
        Key::Up => "up".into(),
        Key::Down => "down".into(),
        Key::Left => "left".into(),
        Key::Right => "right".into(),
        Key::Ctrl(c) => format!("ctrl {c}"),
    }
}

fn onoff(v: bool) -> &'static str {
    if v {
        "on"
    } else {
        "off"
    }
}

pub fn dur(d: Duration) -> String {
    let ms = d.as_millis();
    if ms >= 1000 && ms.is_multiple_of(1000) {
        format!("{}s", ms / 1000)
    } else {
        format!("{ms}ms")
    }
}

pub fn quoted(v: &str) -> String {
    let mut out = String::with_capacity(v.len() + 2);
    out.push('"');
    for ch in v.chars() {
        if ch == '"' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('"');
    out
}
