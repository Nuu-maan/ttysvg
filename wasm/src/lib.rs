use std::time::Duration;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use ttysvg::emit;
use ttysvg::session::Capture;
use ttysvg::svg;
use ttysvg::tape::Config;

#[derive(Serialize)]
struct Info {
    cols: u16,
    rows: u16,
    shots: usize,
    duration_ms: u64,
    command: Vec<String>,
    theme: String,
    title: String,
    window: bool,
    padding: f64,
    font_size: f64,
    advance: f64,
    line_height: f64,
    speed: f64,
    trim_idle_ms: u64,
    tail_ms: u64,
    loop_forever: bool,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Patch {
    theme: Option<String>,
    title: Option<String>,
    window: Option<bool>,
    padding: Option<f64>,
    font_size: Option<f64>,
    advance: Option<f64>,
    line_height: Option<f64>,
    speed: Option<f64>,
    trim_idle_ms: Option<u64>,
    tail_ms: Option<u64>,
    loop_forever: Option<bool>,
}

impl Patch {
    fn apply(self, cfg: &mut Config) {
        if let Some(v) = self.theme {
            cfg.theme = v;
        }
        if let Some(v) = self.title {
            cfg.title = v;
        }
        if let Some(v) = self.window {
            cfg.window = v;
        }
        if let Some(v) = self.padding {
            cfg.padding = v;
        }
        if let Some(v) = self.font_size {
            cfg.font_size = v;
        }
        if let Some(v) = self.advance {
            cfg.advance = v;
        }
        if let Some(v) = self.line_height {
            cfg.line_height = v;
        }
        if let Some(v) = self.speed {
            cfg.speed = v.max(0.05);
        }
        if let Some(v) = self.trim_idle_ms {
            cfg.trim_idle = if v == 0 {
                None
            } else {
                Some(Duration::from_millis(v))
            };
        }
        if let Some(v) = self.tail_ms {
            cfg.tail = Duration::from_millis(v);
        }
        if let Some(v) = self.loop_forever {
            cfg.loop_forever = v;
        }
    }
}

fn parse(capture_json: &str) -> Result<Capture, JsError> {
    Capture::from_json(capture_json).map_err(|e| JsError::new(&format!("{e:#}")))
}

#[wasm_bindgen]
pub fn themes() -> Vec<String> {
    svg::theme::Theme::names()
        .into_iter()
        .map(String::from)
        .collect()
}

#[wasm_bindgen]
pub fn inspect(capture_json: &str) -> Result<String, JsError> {
    let capture = parse(capture_json)?;
    let cfg = &capture.config;
    let info = Info {
        cols: cfg.cols,
        rows: cfg.rows,
        shots: capture.shots.len(),
        duration_ms: capture.shots.last().map(|s| s.at_ms).unwrap_or(0),
        command: capture.command.clone(),
        theme: cfg.theme.clone(),
        title: cfg.title.clone(),
        window: cfg.window,
        padding: cfg.padding,
        font_size: cfg.font_size,
        advance: cfg.advance,
        line_height: cfg.line_height,
        speed: cfg.speed,
        trim_idle_ms: cfg.trim_idle.map(|d| d.as_millis() as u64).unwrap_or(0),
        tail_ms: cfg.tail.as_millis() as u64,
        loop_forever: cfg.loop_forever,
    };
    serde_json::to_string(&info).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub fn render(capture_json: &str, patch_json: &str) -> Result<String, JsError> {
    let capture = parse(capture_json)?;

    let patch: Patch =
        serde_json::from_str(patch_json).map_err(|e| JsError::new(&format!("options: {e}")))?;
    let mut cfg = capture.config.clone();
    patch.apply(&mut cfg);

    let theme = emit::load_theme(&cfg.theme).map_err(|e| JsError::new(&format!("{e:#}")))?;
    let timeline = emit::timeline(&capture.frames(), &cfg);
    Ok(svg::render(&timeline, &emit::opts(&cfg, theme)))
}
