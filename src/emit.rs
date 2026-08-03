use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::optimize::{self, Options, Timeline};
use crate::svg::theme::Theme;
use crate::svg::RenderOpts;
use crate::tape::Config;
use crate::term::Frame;

pub fn timeline(raw: &[(Duration, Frame)], cfg: &Config) -> Timeline {
    optimize::optimize(
        raw,
        &Options {
            trim_idle: cfg.trim_idle,
            tail: cfg.tail,
            speed: cfg.speed,
            ..Options::default()
        },
    )
}

pub fn opts(cfg: &Config, theme: Theme) -> RenderOpts {
    RenderOpts {
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
        literal: None,
    }
    .with_metrics()
}

pub fn load_theme(name: &str) -> Result<Theme> {
    let theme = Theme::load(name)?;
    theme.validate()?;
    Ok(theme)
}

pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }
    std::fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))
}

pub fn human_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
