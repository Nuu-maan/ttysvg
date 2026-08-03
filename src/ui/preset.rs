use std::time::Duration;

use crate::tape::Config;

pub struct Preset {
    pub name: &'static str,
    pub about: &'static str,
    pub cols: u16,
    pub rows: u16,
    pub padding: f64,
    pub font_size: f64,
    pub window: bool,
    pub trim_idle: Option<Duration>,
    pub tail: Duration,
    pub pause: Duration,
}

impl Preset {
    pub fn apply(&self, cfg: &mut Config) {
        cfg.cols = self.cols;
        cfg.rows = self.rows;
        cfg.padding = self.padding;
        cfg.font_size = self.font_size;
        cfg.window = self.window;
        cfg.trim_idle = self.trim_idle;
        cfg.tail = self.tail;
    }
}

pub const ALL: &[Preset] = &[
    Preset {
        name: "readme banner",
        about: "Wide and short, so it sits at the top of a README without pushing the text off screen. No window chrome, because a README already has a frame around it.",
        cols: 90,
        rows: 20,
        padding: 18.0,
        font_size: 14.0,
        window: false,
        trim_idle: Some(Duration::from_millis(1000)),
        tail: Duration::from_millis(2000),
        pause: Duration::from_millis(1200),
    },
    Preset {
        name: "full screen app",
        about: "Tall and roomy for a TUI that paints the whole screen. Idle trimming is gentle, because a full screen app looks still while it is really waiting for you.",
        cols: 100,
        rows: 30,
        padding: 16.0,
        font_size: 13.0,
        window: true,
        trim_idle: Some(Duration::from_millis(1500)),
        tail: Duration::from_millis(2500),
        pause: Duration::from_millis(2500),
    },
    Preset {
        name: "quick one liner",
        about: "One command, one answer, nothing else. Aggressive idle trimming keeps it under a few seconds so it reads as a screenshot that happens to move.",
        cols: 76,
        rows: 12,
        padding: 16.0,
        font_size: 14.0,
        window: false,
        trim_idle: Some(Duration::from_millis(600)),
        tail: Duration::from_millis(1500),
        pause: Duration::from_millis(1000),
    },
    Preset {
        name: "social preview",
        about: "Fewer columns and larger text, because timelines scale images down hard. Window chrome on, so it reads as a terminal at thumbnail size.",
        cols: 72,
        rows: 18,
        padding: 26.0,
        font_size: 15.0,
        window: true,
        trim_idle: Some(Duration::from_millis(800)),
        tail: Duration::from_millis(2500),
        pause: Duration::from_millis(1500),
    },
];
