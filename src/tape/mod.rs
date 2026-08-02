pub mod parse;

pub use parse::parse;

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Enter,
    Tab,
    Backspace,
    Escape,
    Space,
    Up,
    Down,
    Left,
    Right,
    Ctrl(char),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Op {
    Type(String),
    Key(Key),
    Sleep(Duration),
    Wait { text: String, timeout: Duration },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub output: PathBuf,
    pub theme: String,
    pub cols: u16,
    pub rows: u16,
    pub font_family: String,
    pub font_size: f64,
    pub advance: f64,
    pub line_height: f64,
    pub padding: f64,
    pub shell: Vec<String>,
    pub trim_idle: Option<Duration>,
    pub tail: Duration,
    pub type_delay: Duration,
    pub speed: f64,
    pub window: bool,
    pub title: String,
    pub loop_forever: bool,
}

pub fn default_shell() -> Vec<String> {
    if cfg!(windows) {
        vec![
            "powershell.exe".into(),
            "-NoLogo".into(),
            "-NoProfile".into(),
        ]
    } else {
        let sh = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        vec![sh, "-i".into()]
    }
}

pub const DEFAULT_FONT: &str =
    "ui-monospace, SFMono-Regular, 'JetBrains Mono', Menlo, Consolas, 'Cascadia Mono', monospace";

impl Default for Config {
    fn default() -> Self {
        Config {
            output: PathBuf::from("demo.svg"),
            theme: "tokyonight".into(),
            cols: 90,
            rows: 22,
            font_family: DEFAULT_FONT.into(),
            font_size: 14.0,
            advance: 0.0,
            line_height: 0.0,
            padding: 18.0,
            shell: default_shell(),
            trim_idle: Some(Duration::from_millis(1000)),
            tail: Duration::from_millis(2000),
            type_delay: Duration::from_millis(55),
            speed: 1.0,
            window: false,
            title: String::new(),
            loop_forever: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tape {
    pub config: Config,
    pub ops: Vec<Op>,
}
