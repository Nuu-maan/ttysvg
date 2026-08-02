use anyhow::{Context, Result};
use regex::{escape, Captures, Regex};

use crate::tape::Config;
use crate::term::Frame;

pub const MASK: char = '*';

#[derive(Clone, Debug)]
struct Rule {
    re: Regex,
    with: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Rules {
    rules: Vec<Rule>,
}

impl Rules {
    pub fn from_config(cfg: &Config) -> Result<Self> {
        let mut rules = Rules::default();
        for pattern in &cfg.redact {
            rules.pattern(pattern)?;
        }
        if cfg.sanitize {
            rules.sanitize();
        }
        Ok(rules)
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn pattern(&mut self, pattern: &str) -> Result<()> {
        let re = Regex::new(pattern).with_context(|| format!("bad redact pattern {pattern:?}"))?;
        self.rules.push(Rule { re, with: None });
        Ok(())
    }

    pub fn replace(&mut self, needle: &str, with: &str) {
        self.push_literal(&escape(needle), with);
    }

    pub fn replace_word(&mut self, needle: &str, with: &str) {
        self.push_literal(&format!(r"\b{}\b", escape(needle)), with);
    }

    fn push_literal(&mut self, pattern: &str, with: &str) {
        if let Ok(re) = Regex::new(pattern) {
            self.rules.push(Rule {
                re,
                with: Some(with.to_string()),
            });
        }
    }

    pub fn sanitize(&mut self) {
        for key in ["USERPROFILE", "HOME"] {
            if let Ok(home) = std::env::var(key) {
                if home.is_empty() {
                    continue;
                }
                self.replace(&home, "~");
                let slashed = home.replace('\\', "/");
                if slashed != home {
                    self.replace(&slashed, "~");
                }
            }
        }
        for key in ["USERNAME", "USER"] {
            if let Ok(user) = std::env::var(key) {
                if !user.is_empty() {
                    self.replace_word(&user, "user");
                }
            }
        }
        for key in ["COMPUTERNAME", "HOSTNAME"] {
            if let Ok(host) = std::env::var(key) {
                if !host.is_empty() {
                    self.replace_word(&host, "host");
                }
            }
        }
    }

    pub fn text(&self, input: &str) -> String {
        if self.rules.is_empty() {
            return input.to_string();
        }
        let mut out = input.to_string();
        for rule in &self.rules {
            out = rule
                .re
                .replace_all(&out, |caps: &Captures| {
                    let found = &caps[0];
                    let room = found.chars().count();
                    match &rule.with {
                        None => MASK.to_string().repeat(room),
                        Some(with) => with.chars().take(room).collect(),
                    }
                })
                .into_owned();
        }
        out
    }

    pub fn strings(&self, values: &[String]) -> Vec<String> {
        values.iter().map(|v| self.text(v)).collect()
    }

    pub fn frame(&self, frame: &mut Frame) {
        if self.rules.is_empty() {
            return;
        }
        for row in &mut frame.rows {
            for run in &mut row.runs {
                let cleaned = self.text(&run.text);
                if cleaned == run.text {
                    continue;
                }
                let before = run.text.chars().count() as u16;
                let after = cleaned.chars().count() as u16;
                run.width = run.width.saturating_sub(before.saturating_sub(after));
                run.text = cleaned;
            }
        }
    }
}
