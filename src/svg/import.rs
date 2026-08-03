use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::svg::theme::{luminance, rgb, Palette, Theme};

const BASE16_ANSI: [usize; 16] = [
    0x00, 0x08, 0x0b, 0x0a, 0x0d, 0x0e, 0x0c, 0x05, 0x03, 0x08, 0x0b, 0x0a, 0x0d, 0x0e, 0x0c, 0x07,
];

const WT_KEYS: [&str; 16] = [
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "purple",
    "cyan",
    "white",
    "brightBlack",
    "brightRed",
    "brightGreen",
    "brightYellow",
    "brightBlue",
    "brightPurple",
    "brightCyan",
    "brightWhite",
];

pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn hex(raw: &str) -> Result<String> {
    let v = raw.trim().trim_matches('"').trim_matches('\'').trim();
    let v = if v.starts_with('#') {
        v.to_ascii_lowercase()
    } else {
        format!("#{}", v.to_ascii_lowercase())
    };
    rgb(&v).ok_or_else(|| anyhow!("{raw:?} is not an #rrggbb color"))?;
    Ok(v)
}

pub fn read(path: &Path) -> Result<Vec<Theme>> {
    let src =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if src.trim_start().starts_with('{') {
        windows_terminal(&src)
    } else {
        base16(&src).map(|t| vec![t])
    }
}

pub fn base16(src: &str) -> Result<Theme> {
    let mut slots: [Option<String>; 16] = Default::default();
    let mut name = String::new();

    for line in src.lines() {
        let line = line.trim();
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match key {
            "scheme" | "name" => {
                if name.is_empty() {
                    name = value.trim_matches('"').trim_matches('\'').to_string();
                }
            }
            k if k.len() == 6 && k.starts_with("base") => {
                if let Ok(idx) = usize::from_str_radix(&k[4..], 16) {
                    slots[idx] = Some(hex(value)?);
                }
            }
            _ => {}
        }
    }

    let mut base = Vec::with_capacity(16);
    for (i, slot) in slots.into_iter().enumerate() {
        base.push(slot.ok_or_else(|| anyhow!("base16 scheme is missing base{i:02X}"))?);
    }

    if name.is_empty() {
        name = "imported".into();
    }

    let palette = Palette {
        fg: base[0x05].clone(),
        bg: base[0x00].clone(),
        cursor: Some(base[0x05].clone()),
        chrome: Some(base[0x01].clone()),
        border: Some(base[0x02].clone()),
        buttons: Some(vec![
            base[0x08].clone(),
            base[0x0a].clone(),
            base[0x0b].clone(),
        ]),
        ansi: BASE16_ANSI.iter().map(|i| base[*i].clone()).collect(),
    };

    Ok(Theme {
        name: slug(&name),
        dark: palette,
        light: None,
    })
}

pub fn windows_terminal(src: &str) -> Result<Vec<Theme>> {
    let root: serde_json::Value =
        serde_json::from_str(src).context("parsing the windows terminal json")?;

    let schemes: Vec<&serde_json::Value> = match root.get("schemes") {
        Some(serde_json::Value::Array(list)) => list.iter().collect(),
        _ => vec![&root],
    };

    let mut out = Vec::new();
    for scheme in schemes {
        out.push(one_windows_terminal(scheme)?);
    }
    if out.is_empty() {
        return Err(anyhow!("no color schemes in that file"));
    }
    Ok(out)
}

fn one_windows_terminal(scheme: &serde_json::Value) -> Result<Theme> {
    let pick = |key: &str| -> Result<String> {
        let raw = scheme
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("scheme is missing {key:?}"))?;
        hex(raw)
    };

    let name = scheme
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("imported");

    let fg = pick("foreground")?;
    let bg = pick("background")?;
    let mut ansi = Vec::with_capacity(16);
    for key in WT_KEYS {
        ansi.push(pick(key)?);
    }

    let cursor = scheme
        .get("cursorColor")
        .and_then(|v| v.as_str())
        .and_then(|v| hex(v).ok());

    let palette = Palette {
        fg,
        bg,
        cursor,
        chrome: Some(ansi[0].clone()),
        border: Some(ansi[8].clone()),
        buttons: Some(vec![ansi[1].clone(), ansi[3].clone(), ansi[2].clone()]),
        ansi,
    };

    Ok(Theme {
        name: slug(name),
        dark: palette,
        light: None,
    })
}

pub fn is_light(theme: &Theme) -> bool {
    luminance(&theme.dark.bg).unwrap_or(0.0) > 0.5
}
