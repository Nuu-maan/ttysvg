pub mod driver;

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

use crate::redact::Rules;
use crate::term::{self, Frame};

const DRAIN_GRACE: Duration = Duration::from_millis(200);

pub struct Shared {
    parser: vt100::Parser,
    pub frames: Vec<(Duration, Frame)>,
}

impl Shared {
    pub fn screen_text(&self) -> String {
        self.parser.screen().contents()
    }
}

pub struct Session {
    pub shared: Arc<Mutex<Shared>>,
    writer: Option<Box<dyn Write + Send>>,
    master: Option<Box<dyn MasterPty + Send>>,
    child: Box<dyn Child + Send + Sync>,
    reader: Option<std::thread::JoinHandle<()>>,
}

pub fn spawn(argv: &[String], cols: u16, rows: u16, echo: bool, rules: &Rules) -> Result<Session> {
    let (program, args) = argv
        .split_first()
        .ok_or_else(|| anyhow!("no command given"))?;

    let pty = native_pty_system();
    let pair = pty.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new(program);
    for a in args {
        cmd.arg(a);
    }
    if let Ok(cwd) = std::env::current_dir() {
        cmd.cwd(cwd);
    }
    if let Some(path) = std::env::var_os("PATH") {
        cmd.env("PATH", path);
    }
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("CLICOLOR_FORCE", "1");
    cmd.env("FORCE_COLOR", "1");

    let child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;

    let shared = Arc::new(Mutex::new(Shared {
        parser: vt100::Parser::new(rows, cols, 0),
        frames: vec![(Duration::ZERO, Frame::blank(rows))],
    }));

    let start = Instant::now();
    let sink = Arc::clone(&shared);
    let rules = rules.clone();

    let handle = std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let at = start.elapsed();

                    if echo {
                        let mut out = std::io::stdout();
                        let _ = out.write_all(&buf[..n]);
                        let _ = out.flush();
                    }

                    let mut s = sink.lock().unwrap();
                    s.parser.process(&buf[..n]);
                    let mut frame = term::snapshot(s.parser.screen());
                    rules.frame(&mut frame);
                    if s.frames
                        .last()
                        .map(|(_, prev)| prev != &frame)
                        .unwrap_or(true)
                    {
                        s.frames.push((at, frame));
                    }
                }
            }
        }
    });

    Ok(Session {
        shared,
        writer: Some(writer),
        master: Some(pair.master),
        child,
        reader: Some(handle),
    })
}

impl Session {
    pub fn send(&mut self, bytes: &[u8]) -> Result<()> {
        let w = self
            .writer
            .as_mut()
            .ok_or_else(|| anyhow!("session writer already taken"))?;
        w.write_all(bytes)?;
        w.flush()?;
        Ok(())
    }

    pub fn take_writer(&mut self) -> Option<Box<dyn Write + Send>> {
        self.writer.take()
    }

    pub fn wait_for(&self, needle: &str, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        loop {
            {
                let s = self.shared.lock().unwrap();
                if s.screen_text().contains(needle) {
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                return Err(anyhow!(
                    "timed out after {:?} waiting for {:?} to appear on screen",
                    timeout,
                    needle
                ));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn wait_exit(&mut self) {
        let _ = self.child.wait();
        std::thread::sleep(DRAIN_GRACE);
    }

    pub fn finish(mut self, tail: Duration) -> Vec<(Duration, Frame)> {
        std::thread::sleep(tail);
        let _ = self.child.kill();
        let _ = self.child.wait();
        std::thread::sleep(DRAIN_GRACE);
        self.collect()
    }

    pub fn collect(mut self) -> Vec<(Duration, Frame)> {
        let frames = {
            let s = self.shared.lock().unwrap();
            s.frames.clone()
        };
        drop(self.writer.take());
        drop(self.master.take());
        drop(self.reader.take());
        frames
    }
}
