use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::redact::Rules;
use crate::tape::Config;
use crate::term::Frame;

pub const FORMAT: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shot {
    pub at_ms: u64,
    pub frame: Frame,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capture {
    pub format: u32,
    pub command: Vec<String>,
    pub config: Config,
    pub shots: Vec<Shot>,
}

impl Capture {
    pub fn new(
        command: &[String],
        config: &Config,
        raw: &[(Duration, Frame)],
        rules: &Rules,
    ) -> Self {
        let mut shots: Vec<Shot> = Vec::with_capacity(raw.len());
        for (at, frame) in raw {
            let mut frame = frame.clone();
            rules.frame(&mut frame);
            if shots.last().map(|s| s.frame != frame).unwrap_or(true) {
                shots.push(Shot {
                    at_ms: at.as_millis() as u64,
                    frame,
                });
            }
        }

        let mut config = config.clone();
        config.shell = rules.strings(&config.shell);
        config.title = rules.text(&config.title);
        config.output = PathBuf::from(rules.text(&config.output.to_string_lossy()));

        Capture {
            format: FORMAT,
            command: rules.strings(command),
            config,
            shots,
        }
    }

    pub fn frames(&self) -> Vec<(Duration, Frame)> {
        self.shots
            .iter()
            .map(|s| (Duration::from_millis(s.at_ms), s.frame.clone()))
            .collect()
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).context("encoding capture")
    }

    pub fn from_json(text: &str) -> Result<Self> {
        let capture: Capture = serde_json::from_str(text).context("decoding capture")?;
        if capture.format != FORMAT {
            bail!(
                "capture is format {}, this build reads format {}",
                capture.format,
                FORMAT
            );
        }
        if capture.shots.is_empty() {
            bail!("capture contains no frames");
        }
        Ok(capture)
    }

    pub fn save(&self, path: &Path) -> Result<usize> {
        let json = self.to_json()?;
        if let Some(dir) = path.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir)?;
            }
        }
        std::fs::write(path, json.as_bytes())
            .with_context(|| format!("writing {}", path.display()))?;
        Ok(json.len())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Capture::from_json(&text).with_context(|| format!("in {}", path.display()))
    }
}
