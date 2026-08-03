use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::svg::theme::Theme;
use crate::tape::{Config, Key as TapeKey, Op};
use crate::term::Frame;
use crate::ui::preset;

pub const WARMUP: Duration = Duration::from_millis(900);
pub const PAUSE_STEP: u64 = 250;
pub const PAUSE_MAX: u64 = 15_000;
pub const FIELDS: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    Preset,
    Command,
    Theme,
    Details,
    Review,
    Done,
}

impl Step {
    pub const FLOW: [Step; 5] = [
        Step::Preset,
        Step::Command,
        Step::Theme,
        Step::Details,
        Step::Review,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Step::Preset => "preset",
            Step::Command => "command",
            Step::Theme => "theme",
            Step::Details => "details",
            Step::Review => "record",
            Step::Done => "done",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Action {
    #[default]
    None,
    Record,
    Rerender,
    Open,
    Quit,
}

pub struct Done {
    pub raw: Vec<(Duration, Frame)>,
    pub frames: usize,
    pub secs: f64,
    pub bytes: usize,
    pub svg: PathBuf,
    pub tape: PathBuf,
    pub last: Frame,
}

pub struct App {
    pub step: Step,
    pub preset: usize,
    pub theme: usize,
    pub field: usize,
    pub commands: Vec<String>,
    pub line: usize,
    pub cfg: Config,
    pub themes: Vec<String>,
    pub output: String,
    pub pause: Duration,
    pub done: Option<Done>,
    pub status: Option<String>,
    pub action: Action,
    pub term_cols: u16,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut cfg = Config::default();
        preset::ALL[0].apply(&mut cfg);
        cfg.sanitize = true;
        cfg.title = "demo".into();

        App {
            step: Step::Preset,
            preset: 0,
            theme: 0,
            themes: Theme::all_names(),
            field: 0,
            commands: vec![String::new()],
            line: 0,
            cfg,
            output: "demo.svg".into(),
            pause: preset::ALL[0].pause,
            done: None,
            status: None,
            action: Action::None,
            term_cols: 0,
        }
    }

    pub fn themes(&self) -> &[String] {
        &self.themes
    }

    pub fn theme_name(&self) -> &str {
        &self.themes[self.theme.min(self.themes.len() - 1)]
    }

    pub fn preset(&self) -> &'static preset::Preset {
        &preset::ALL[self.preset.min(preset::ALL.len() - 1)]
    }

    pub fn config(&self) -> Config {
        let mut cfg = self.cfg.clone();
        cfg.theme = self.theme_name().to_string();
        cfg.output = PathBuf::from(self.output.trim());
        cfg
    }

    pub fn tape_path(&self) -> PathBuf {
        self.config().output.with_extension("tape")
    }

    pub fn lines(&self) -> Vec<&str> {
        self.commands
            .iter()
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .collect()
    }

    pub fn ops(&self) -> Vec<Op> {
        let mut ops = vec![Op::Sleep(WARMUP)];
        for command in self.lines() {
            ops.push(Op::Type(command.to_string()));
            ops.push(Op::Key(TapeKey::Enter));
            ops.push(Op::Sleep(self.pause));
        }
        ops
    }

    pub fn problem(&self) -> Option<String> {
        if self.lines().is_empty() {
            return Some("add at least one command to record".into());
        }
        let out = self.output.trim();
        if out.is_empty() {
            return Some("the output needs a file name".into());
        }
        if !out.to_ascii_lowercase().ends_with(".svg") {
            return Some("the output should end in .svg".into());
        }
        None
    }

    pub fn too_wide(&self) -> bool {
        self.term_cols > 0 && self.cfg.cols > self.term_cols
    }

    pub fn key(&mut self, key: KeyEvent) {
        self.status = None;

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char('c') | KeyCode::Char('q') = key.code {
                self.action = Action::Quit;
                return;
            }
        }

        match key.code {
            KeyCode::Esc => {
                if self.step == Step::Preset {
                    self.action = Action::Quit;
                } else {
                    self.back();
                }
                return;
            }
            KeyCode::Tab => {
                self.next();
                return;
            }
            KeyCode::BackTab => {
                self.back();
                return;
            }
            _ => {}
        }

        match self.step {
            Step::Preset => self.preset_key(key),
            Step::Command => self.command_key(key),
            Step::Theme => self.theme_key(key),
            Step::Details => self.details_key(key),
            Step::Review => self.review_key(key),
            Step::Done => self.done_key(key),
        }
    }

    pub fn next(&mut self) {
        self.step = match self.step {
            Step::Preset => Step::Command,
            Step::Command => Step::Theme,
            Step::Theme => Step::Details,
            Step::Details => Step::Review,
            Step::Review => Step::Review,
            Step::Done => Step::Done,
        };
    }

    pub fn back(&mut self) {
        self.step = match self.step {
            Step::Preset => Step::Preset,
            Step::Command => Step::Preset,
            Step::Theme => Step::Command,
            Step::Details => Step::Theme,
            Step::Review => Step::Details,
            Step::Done => Step::Details,
        };
    }

    fn select_preset(&mut self, index: usize) {
        if index == self.preset {
            return;
        }
        self.preset = index;
        let chosen = &preset::ALL[index];
        chosen.apply(&mut self.cfg);
        self.pause = chosen.pause;
    }

    fn preset_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let last = preset::ALL.len() - 1;
                let next = if self.preset == 0 {
                    last
                } else {
                    self.preset - 1
                };
                self.select_preset(next);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let next = (self.preset + 1) % preset::ALL.len();
                self.select_preset(next);
            }
            KeyCode::Enter => self.next(),
            KeyCode::Char('q') => self.action = Action::Quit,
            _ => {}
        }
    }

    fn command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => self.commands[self.line].push(c),
            KeyCode::Backspace => {
                if self.commands[self.line].pop().is_none() && self.commands.len() > 1 {
                    self.commands.remove(self.line);
                    self.line = self.line.saturating_sub(1);
                }
            }
            KeyCode::Up => self.line = self.line.saturating_sub(1),
            KeyCode::Down => self.line = (self.line + 1).min(self.commands.len() - 1),
            KeyCode::Enter => {
                let last = self.line + 1 == self.commands.len();
                if last && self.commands[self.line].trim().is_empty() {
                    if self.commands.len() > 1 {
                        self.commands.pop();
                        self.line = self.commands.len() - 1;
                    }
                    self.next();
                } else {
                    self.commands.insert(self.line + 1, String::new());
                    self.line += 1;
                }
            }
            _ => {}
        }
    }

    fn theme_key(&mut self, key: KeyEvent) {
        let count = self.themes.len();
        match key.code {
            KeyCode::Up | KeyCode::Left | KeyCode::Char('k') => {
                self.theme = if self.theme == 0 {
                    count - 1
                } else {
                    self.theme - 1
                };
            }
            KeyCode::Down | KeyCode::Right | KeyCode::Char('j') => {
                self.theme = (self.theme + 1) % count;
            }
            KeyCode::Enter => self.next(),
            KeyCode::Char('q') => self.action = Action::Quit,
            _ => {}
        }
    }

    fn details_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                self.field = if self.field == 0 {
                    FIELDS - 1
                } else {
                    self.field - 1
                }
            }
            KeyCode::Down => self.field = (self.field + 1) % FIELDS,
            KeyCode::Enter => self.next(),
            KeyCode::Char(' ') if self.field >= 2 => self.toggle(),
            KeyCode::Left => self.nudge(false),
            KeyCode::Right => self.nudge(true),
            KeyCode::Char(c) => match self.field {
                0 => self.output.push(c),
                1 => self.cfg.title.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.field {
                0 => {
                    self.output.pop();
                }
                1 => {
                    self.cfg.title.pop();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn toggle(&mut self) {
        match self.field {
            2 => self.cfg.window = !self.cfg.window,
            3 => self.cfg.sanitize = !self.cfg.sanitize,
            _ => {}
        }
    }

    fn nudge(&mut self, up: bool) {
        match self.field {
            2 | 3 => self.toggle(),
            4 => {
                let ms = self.pause.as_millis() as u64;
                let next = if up {
                    (ms + PAUSE_STEP).min(PAUSE_MAX)
                } else {
                    ms.saturating_sub(PAUSE_STEP)
                };
                self.pause = Duration::from_millis(next);
            }
            _ => {}
        }
    }

    fn review_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => match self.problem() {
                Some(why) => self.status = Some(why),
                None => self.action = Action::Record,
            },
            KeyCode::Char('q') => self.action = Action::Quit,
            _ => {}
        }
    }

    fn done_key(&mut self, key: KeyEvent) {
        let count = self.themes.len();
        match key.code {
            KeyCode::Up | KeyCode::Left => {
                self.theme = if self.theme == 0 {
                    count - 1
                } else {
                    self.theme - 1
                };
                self.action = Action::Rerender;
            }
            KeyCode::Down | KeyCode::Right => {
                self.theme = (self.theme + 1) % count;
                self.action = Action::Rerender;
            }
            KeyCode::Char('w') => {
                self.cfg.window = !self.cfg.window;
                self.action = Action::Rerender;
            }
            KeyCode::Char('r') => {
                self.step = Step::Review;
                self.action = Action::Record;
            }
            KeyCode::Char('e') => self.step = Step::Preset,
            KeyCode::Char('o') => self.action = Action::Open,
            KeyCode::Char('q') => self.action = Action::Quit,
            _ => {}
        }
    }
}
