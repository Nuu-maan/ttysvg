use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};

use crate::tape::{Config, Key, Op, Tape};

pub fn parse(src: &str) -> Result<Tape> {
    let mut config = Config::default();
    let mut ops = Vec::new();

    for (lineno, raw) in src.lines().enumerate() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        let toks = tokenize(line)
            .with_context(|| format!("line {}: {}", lineno + 1, raw.trim()))?;
        if toks.is_empty() {
            continue;
        }
        apply(&mut config, &mut ops, &toks)
            .with_context(|| format!("line {}: {}", lineno + 1, raw.trim()))?;
    }

    Ok(Tape { config, ops })
}

fn strip_comment(line: &str) -> &str {
    let mut in_quote = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' => in_quote = !in_quote,
            '#' if !in_quote => return &line[..i],
            _ => {}
        }
    }
    line
}

fn tokenize(line: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut started = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quote = !in_quote;
                started = true;
            }
            '\\' if in_quote => match chars.peek() {
                Some('"') | Some('\\') => {
                    cur.push(chars.next().unwrap());
                }
                _ => cur.push('\\'),
            },
            c if c.is_whitespace() && !in_quote => {
                if started || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    started = false;
                }
            }
            c => {
                cur.push(c);
                started = true;
            }
        }
    }

    if in_quote {
        bail!("unterminated string");
    }
    if started || !cur.is_empty() {
        out.push(cur);
    }
    Ok(out)
}

pub fn duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    let (num, mult) = if let Some(rest) = s.strip_suffix("ms") {
        (rest, 1.0)
    } else if let Some(rest) = s.strip_suffix('s') {
        (rest, 1000.0)
    } else if let Some(rest) = s.strip_suffix('m') {
        (rest, 60_000.0)
    } else {
        (s, 1.0)
    };
    let v: f64 = num
        .trim()
        .parse()
        .map_err(|_| anyhow!("bad duration {s:?}, try 500ms or 2s"))?;
    if v < 0.0 {
        bail!("negative duration {s:?}");
    }
    Ok(Duration::from_secs_f64(v * mult / 1000.0))
}

fn flag(v: &str) -> Result<bool> {
    match v.to_ascii_lowercase().as_str() {
        "on" | "true" | "yes" | "1" => Ok(true),
        "off" | "false" | "no" | "0" => Ok(false),
        other => Err(anyhow!("expected on or off, got {other:?}")),
    }
}

fn arg<'a>(toks: &'a [String], i: usize, what: &str) -> Result<&'a str> {
    toks.get(i)
        .map(String::as_str)
        .ok_or_else(|| anyhow!("{} expects {}", toks[0], what))
}

fn apply(cfg: &mut Config, ops: &mut Vec<Op>, toks: &[String]) -> Result<()> {
    let cmd = toks[0].to_ascii_lowercase();
    match cmd.as_str() {
        "output" => cfg.output = PathBuf::from(arg(toks, 1, "a path")?),
        "theme" => cfg.theme = arg(toks, 1, "a theme name or path")?.to_string(),
        "width" | "cols" => cfg.cols = arg(toks, 1, "a number")?.parse()?,
        "height" | "rows" => cfg.rows = arg(toks, 1, "a number")?.parse()?,
        "font" => {
            cfg.font_family = arg(toks, 1, "a font family")?.to_string();
            if let Some(size) = toks.get(2) {
                cfg.font_size = size.parse()?;
            }
        }
        "font-size" => cfg.font_size = arg(toks, 1, "a number")?.parse()?,
        "advance" => cfg.advance = arg(toks, 1, "a number")?.parse()?,
        "line-height" => cfg.line_height = arg(toks, 1, "a number")?.parse()?,
        "padding" => cfg.padding = arg(toks, 1, "a number")?.parse()?,
        "shell" => {
            let rest = &toks[1..];
            if rest.is_empty() {
                bail!("shell expects a command");
            }
            cfg.shell = if rest.len() == 1 {
                rest[0].split_whitespace().map(str::to_string).collect()
            } else {
                rest.to_vec()
            };
        }
        "trim-idle" => {
            let v = arg(toks, 1, "a duration or off")?;
            cfg.trim_idle = if v.eq_ignore_ascii_case("off") {
                None
            } else {
                Some(duration(v)?)
            };
        }
        "tail" => cfg.tail = duration(arg(toks, 1, "a duration")?)?,
        "type-delay" => cfg.type_delay = duration(arg(toks, 1, "a duration")?)?,
        "speed" => cfg.speed = arg(toks, 1, "a number")?.parse()?,
        "window" => cfg.window = flag(arg(toks, 1, "on or off")?)?,
        "title" => cfg.title = arg(toks, 1, "a string")?.to_string(),
        "loop" => cfg.loop_forever = flag(arg(toks, 1, "on or off")?)?,

        "type" => ops.push(Op::Type(arg(toks, 1, "a string")?.to_string())),
        "sleep" => ops.push(Op::Sleep(duration(arg(toks, 1, "a duration")?)?)),
        "wait" => {
            let text = arg(toks, 1, "a string to wait for")?.to_string();
            let timeout = match toks.get(2) {
                Some(t) => duration(t)?,
                None => Duration::from_secs(15),
            };
            ops.push(Op::Wait { text, timeout });
        }
        "enter" => push_repeat(ops, toks, Op::Key(Key::Enter))?,
        "tab" => push_repeat(ops, toks, Op::Key(Key::Tab))?,
        "backspace" => push_repeat(ops, toks, Op::Key(Key::Backspace))?,
        "escape" | "esc" => push_repeat(ops, toks, Op::Key(Key::Escape))?,
        "space" => push_repeat(ops, toks, Op::Key(Key::Space))?,
        "up" => push_repeat(ops, toks, Op::Key(Key::Up))?,
        "down" => push_repeat(ops, toks, Op::Key(Key::Down))?,
        "left" => push_repeat(ops, toks, Op::Key(Key::Left))?,
        "right" => push_repeat(ops, toks, Op::Key(Key::Right))?,
        "ctrl" => {
            let c = arg(toks, 1, "a letter")?
                .chars()
                .next()
                .ok_or_else(|| anyhow!("ctrl expects a letter"))?;
            ops.push(Op::Key(Key::Ctrl(c)));
        }
        other => bail!("unknown directive {other:?}"),
    }
    Ok(())
}

fn push_repeat(ops: &mut Vec<Op>, toks: &[String], op: Op) -> Result<()> {
    let n: usize = match toks.get(1) {
        Some(v) => v.parse().unwrap_or(1),
        None => 1,
    };
    for _ in 0..n.max(1) {
        ops.push(op.clone());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(duration("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(duration("2s").unwrap(), Duration::from_secs(2));
        assert_eq!(duration("1.5s").unwrap(), Duration::from_millis(1500));
        assert_eq!(duration("250").unwrap(), Duration::from_millis(250));
        assert!(duration("nope").is_err());
    }

    #[test]
    fn quoted_tokens_survive_spaces_and_hashes() {
        let toks = tokenize(r#"type "echo # not a comment""#).unwrap();
        assert_eq!(toks, vec!["type", "echo # not a comment"]);
    }

    #[test]
    fn comments_outside_quotes_are_stripped() {
        assert_eq!(strip_comment("width 90 # cols").trim(), "width 90");
        assert_eq!(strip_comment(r#"type "a#b""#).trim(), r#"type "a#b""#);
    }

    #[test]
    fn windows_paths_survive_intact() {
        let toks = tokenize(r#"type ".\scripts\showcase.ps1""#).unwrap();
        assert_eq!(toks, vec!["type", r".\scripts\showcase.ps1"]);

        let toks = tokenize(r#"type "C:\temp\new\report.txt""#).unwrap();
        assert_eq!(toks, vec!["type", r"C:\temp\new\report.txt"]);
    }

    #[test]
    fn quotes_and_backslashes_can_still_be_escaped() {
        let toks = tokenize(r#"type "say \"hi\" now""#).unwrap();
        assert_eq!(toks, vec!["type", r#"say "hi" now"#]);

        let toks = tokenize(r#"type "a\\b""#).unwrap();
        assert_eq!(toks, vec!["type", r"a\b"]);
    }

    #[test]
    fn empty_string_is_a_token() {
        let toks = tokenize(r#"title """#).unwrap();
        assert_eq!(toks, vec!["title", ""]);
    }

    #[test]
    fn parses_a_small_tape() {
        let tape = parse(
            r#"
            output "docs/demo.svg"
            width 80
            type "ls"
            enter
            wait "$" 5s
            sleep 1s
            "#,
        )
        .unwrap();
        assert_eq!(tape.config.cols, 80);
        assert_eq!(tape.config.output, PathBuf::from("docs/demo.svg"));
        assert_eq!(tape.ops.len(), 4);
    }
}
