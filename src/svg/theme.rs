use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::term::Color;

pub const BUTTONS: [&str; 3] = ["#ff5f57", "#febc2e", "#28c840"];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Palette {
    pub fg: String,
    pub bg: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chrome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<String>>,
    pub ansi: Vec<String>,
}

impl Palette {
    pub fn cursor(&self) -> &str {
        self.cursor.as_deref().unwrap_or(&self.fg)
    }

    pub fn chrome(&self) -> &str {
        self.chrome.as_deref().unwrap_or(&self.bg)
    }

    pub fn border(&self) -> &str {
        self.border.as_deref().unwrap_or(&self.bg)
    }

    pub fn buttons(&self) -> Vec<&str> {
        match &self.buttons {
            Some(v) if v.len() == 3 => v.iter().map(String::as_str).collect(),
            _ => BUTTONS.to_vec(),
        }
    }

    pub fn contrast(&self) -> Option<f64> {
        contrast(&self.fg, &self.bg)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Theme {
    pub name: String,
    pub dark: Palette,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<Palette>,
}

const BUILTINS: &[(&str, &str)] = &[
    ("tokyonight", include_str!("../../themes/tokyonight.toml")),
    ("catppuccin", include_str!("../../themes/catppuccin.toml")),
    (
        "catppuccin-mocha",
        include_str!("../../themes/catppuccin-mocha.toml"),
    ),
    (
        "catppuccin-macchiato",
        include_str!("../../themes/catppuccin-macchiato.toml"),
    ),
    (
        "catppuccin-frappe",
        include_str!("../../themes/catppuccin-frappe.toml"),
    ),
    (
        "catppuccin-latte",
        include_str!("../../themes/catppuccin-latte.toml"),
    ),
    ("gruvbox", include_str!("../../themes/gruvbox.toml")),
    ("nord", include_str!("../../themes/nord.toml")),
    ("dracula", include_str!("../../themes/dracula.toml")),
    ("solarized", include_str!("../../themes/solarized.toml")),
    ("onedark", include_str!("../../themes/onedark.toml")),
    ("monokai", include_str!("../../themes/monokai.toml")),
    ("everforest", include_str!("../../themes/everforest.toml")),
    ("kanagawa", include_str!("../../themes/kanagawa.toml")),
    ("rose-pine", include_str!("../../themes/rose-pine.toml")),
    ("github", include_str!("../../themes/github.toml")),
];

pub fn user_dir() -> Option<PathBuf> {
    let root = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    };
    root.map(|r| r.join("ttysvg").join("themes"))
}

pub fn user_names() -> Vec<String> {
    let Some(dir) = user_dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<String> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .filter(|n| !BUILTINS.iter().any(|(b, _)| b == n))
        .collect();
    out.sort();
    out
}

impl Theme {
    pub fn load(spec: &str) -> Result<Theme> {
        if let Some((dark, light)) = split_pair(spec) {
            return Theme::pair(&dark, &light);
        }
        Theme::single(spec)
    }

    fn single(name_or_path: &str) -> Result<Theme> {
        if let Some((_, src)) = BUILTINS.iter().find(|(n, _)| *n == name_or_path) {
            return Ok(toml::from_str(src)?);
        }

        if let Some(path) = user_dir().map(|d| d.join(format!("{name_or_path}.toml"))) {
            if path.exists() {
                let src = std::fs::read_to_string(&path)?;
                return toml::from_str(&src).map_err(|e| anyhow!("theme {}: {e}", path.display()));
            }
        }

        let path = std::path::Path::new(name_or_path);
        if path.exists() {
            let src = std::fs::read_to_string(path)?;
            return toml::from_str(&src).map_err(|e| anyhow!("theme {}: {e}", path.display()));
        }

        Err(anyhow!(
            "unknown theme {:?}. built in: {}",
            name_or_path,
            Theme::names().join(", ")
        ))
    }

    fn pair(dark: &str, light: &str) -> Result<Theme> {
        let a = Theme::single(dark)?;
        let b = Theme::single(light)?;
        Ok(Theme {
            name: format!("{},{}", a.name, b.name),
            dark: a.dark,
            light: Some(b.light.unwrap_or(b.dark)),
        })
    }

    pub fn names() -> Vec<&'static str> {
        BUILTINS.iter().map(|(n, _)| *n).collect()
    }

    pub fn all_names() -> Vec<String> {
        let mut out: Vec<String> = Theme::names().into_iter().map(String::from).collect();
        out.extend(user_names());
        out
    }

    pub fn light(&self) -> &Palette {
        self.light.as_ref().unwrap_or(&self.dark)
    }

    pub fn has_light(&self) -> bool {
        self.light.is_some()
    }

    pub fn palettes(&self) -> Vec<&Palette> {
        match &self.light {
            Some(l) => vec![&self.dark, l],
            None => vec![&self.dark],
        }
    }

    pub fn has_cursor(&self) -> bool {
        self.palettes().iter().any(|p| p.cursor.is_some())
    }

    pub fn has_chrome(&self) -> bool {
        self.palettes().iter().any(|p| p.chrome.is_some())
    }

    pub fn has_border(&self) -> bool {
        self.palettes().iter().any(|p| p.border.is_some())
    }

    pub fn has_buttons(&self) -> bool {
        self.palettes().iter().any(|p| p.buttons.is_some())
    }

    pub fn validate(&self) -> Result<()> {
        for (label, p) in self.labelled() {
            if p.ansi.len() != 16 {
                return Err(anyhow!(
                    "theme {:?}: {} palette needs 16 ansi colors, found {}",
                    self.name,
                    label,
                    p.ansi.len()
                ));
            }
            for c in std::iter::once(&p.fg)
                .chain(std::iter::once(&p.bg))
                .chain(p.ansi.iter())
            {
                if rgb(c).is_none() {
                    return Err(anyhow!(
                        "theme {:?}: {} palette has {:?}, expected #rrggbb",
                        self.name,
                        label,
                        c
                    ));
                }
            }
            if let Some(b) = &p.buttons {
                if b.len() != 3 {
                    return Err(anyhow!(
                        "theme {:?}: {} palette needs 3 buttons, found {}",
                        self.name,
                        label,
                        b.len()
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn warnings(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (label, p) in self.labelled() {
            if let Some(ratio) = p.contrast() {
                if ratio < 4.5 {
                    out.push(format!(
                        "{label} text on background is {ratio:.1} to 1, below the 4.5 readable threshold"
                    ));
                }
            }
            for (i, c) in p.ansi.iter().enumerate().take(7).skip(1) {
                if let Some(ratio) = contrast(c, &p.bg) {
                    if ratio < 1.6 {
                        out.push(format!(
                            "{label} color {i} is nearly invisible against the background"
                        ));
                    }
                }
            }
        }
        out
    }

    fn labelled(&self) -> Vec<(&'static str, &Palette)> {
        match &self.light {
            Some(l) => vec![("dark", &self.dark), ("light", l)],
            None => vec![("dark", &self.dark)],
        }
    }
}

pub fn rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.strip_prefix('#')?;
    if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some((
        u8::from_str_radix(&h[0..2], 16).ok()?,
        u8::from_str_radix(&h[2..4], 16).ok()?,
        u8::from_str_radix(&h[4..6], 16).ok()?,
    ))
}

fn channel(v: u8) -> f64 {
    let c = v as f64 / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn luminance(hex: &str) -> Option<f64> {
    let (r, g, b) = rgb(hex)?;
    Some(0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b))
}

pub fn contrast(a: &str, b: &str) -> Option<f64> {
    let (x, y) = (luminance(a)?, luminance(b)?);
    let (hi, lo) = if x > y { (x, y) } else { (y, x) };
    Some((hi + 0.05) / (lo + 0.05))
}

fn split_pair(spec: &str) -> Option<(String, String)> {
    if !spec.contains(',') {
        return None;
    }
    let mut dark = None;
    let mut light = None;
    let parts: Vec<&str> = spec.split(',').map(str::trim).collect();
    if parts.len() != 2 {
        return None;
    }
    for (i, part) in parts.iter().enumerate() {
        let (slot, name) = match part.split_once('=') {
            Some((k, v)) => (k.trim().to_ascii_lowercase(), v.trim()),
            None => (if i == 0 { "dark" } else { "light" }.to_string(), *part),
        };
        match slot.as_str() {
            "dark" => dark = Some(name.to_string()),
            "light" => light = Some(name.to_string()),
            _ => return None,
        }
    }
    Some((dark?, light?))
}

fn cube(v: u8) -> u8 {
    if v == 0 {
        0
    } else {
        55 + v * 40
    }
}

pub fn xterm256(idx: u8) -> String {
    match idx {
        16..=231 => {
            let i = idx - 16;
            let r = cube(i / 36);
            let g = cube((i % 36) / 6);
            let b = cube(i % 6);
            format!("#{r:02x}{g:02x}{b:02x}")
        }
        232..=255 => {
            let v = 8 + (idx - 232) * 10;
            format!("#{v:02x}{v:02x}{v:02x}")
        }
        _ => format!("var(--c{idx})"),
    }
}

pub fn paint(color: Color, fallback: &str, literal: Option<&Palette>) -> String {
    if let Some(p) = literal {
        return match color {
            Color::Default => {
                if fallback == "bg" {
                    p.bg.clone()
                } else {
                    p.fg.clone()
                }
            }
            Color::Idx(i) if (i as usize) < p.ansi.len() => p.ansi[i as usize].clone(),
            Color::Idx(i) => xterm256(i),
            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        };
    }
    match color {
        Color::Default => format!("var(--{fallback})"),
        Color::Idx(i) if i < 16 => format!("var(--c{i})"),
        Color::Idx(i) => xterm256(i),
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
    }
}
