use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::term::Color;

#[derive(Clone, Debug, Deserialize)]
pub struct Palette {
    pub fg: String,
    pub bg: String,
    pub ansi: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Theme {
    pub name: String,
    pub dark: Palette,
    pub light: Palette,
}

const BUILTINS: &[(&str, &str)] = &[
    ("tokyonight", include_str!("../../themes/tokyonight.toml")),
    ("catppuccin", include_str!("../../themes/catppuccin.toml")),
    ("gruvbox", include_str!("../../themes/gruvbox.toml")),
    ("nord", include_str!("../../themes/nord.toml")),
];

impl Theme {
    pub fn load(name_or_path: &str) -> Result<Theme> {
        if let Some((_, src)) = BUILTINS.iter().find(|(n, _)| *n == name_or_path) {
            return Ok(toml::from_str(src)?);
        }

        let path = std::path::Path::new(name_or_path);
        if path.exists() {
            let src = std::fs::read_to_string(path)?;
            return Ok(toml::from_str(&src)?);
        }

        Err(anyhow!(
            "unknown theme {:?}. built in: {}",
            name_or_path,
            Theme::names().join(", ")
        ))
    }

    pub fn names() -> Vec<&'static str> {
        BUILTINS.iter().map(|(n, _)| *n).collect()
    }

    pub fn validate(&self) -> Result<()> {
        for (label, p) in [("dark", &self.dark), ("light", &self.light)] {
            if p.ansi.len() != 16 {
                return Err(anyhow!(
                    "theme {:?}: {} palette needs 16 ansi colors, found {}",
                    self.name,
                    label,
                    p.ansi.len()
                ));
            }
        }
        Ok(())
    }
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

pub fn paint(color: Color, fallback: &str) -> String {
    match color {
        Color::Default => format!("var(--{fallback})"),
        Color::Idx(i) if i < 16 => format!("var(--c{i})"),
        Color::Idx(i) => xterm256(i),
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
    }
}
