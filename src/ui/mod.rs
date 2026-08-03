pub mod app;
pub mod draw;
pub mod preset;
pub mod preview;

use std::io::{IsTerminal, Stdout, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Result};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::capture::{self, driver};
use crate::emit;
use crate::redact::Rules;
use crate::svg::render;
use crate::tape;
use crate::ui::app::{Action, App, Done, Step};

pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

const POLL: Duration = Duration::from_millis(200);

type Screen = Terminal<CrosstermBackend<Stdout>>;

pub fn run() -> Result<()> {
    if !std::io::stdout().is_terminal() || !std::io::stdin().is_terminal() {
        bail!("interactive mode needs a terminal, try ttysvg record --help");
    }

    let mut app = App::new();
    app.term_cols = crossterm::terminal::size().map(|(c, _)| c).unwrap_or(0);

    let mut screen = enter()?;
    let outcome = loop_over(&mut screen, &mut app);
    leave()?;

    if let Some(done) = &app.done {
        eprintln!(
            "ttysvg: wrote {} and {}",
            done.svg.display(),
            done.tape.display()
        );
    }

    outcome
}

fn loop_over(screen: &mut Screen, app: &mut App) -> Result<()> {
    loop {
        screen.draw(|f| draw::draw(f, app))?;

        if !event::poll(POLL)? {
            continue;
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => app.key(key),
            _ => continue,
        }

        match std::mem::take(&mut app.action) {
            Action::None => {}
            Action::Quit => return Ok(()),
            Action::Open => {
                if let Some(done) = &app.done {
                    if let Err(e) = open(&done.svg) {
                        app.status = Some(format!("could not open it, {e}"));
                    }
                }
            }
            Action::Rerender => match write_files(app) {
                Ok(done) => {
                    let theme = app.theme_name().to_string();
                    app.done = Some(done);
                    app.status = Some(format!("restyled with {theme}, nothing re-ran"));
                }
                Err(e) => app.status = Some(format!("could not restyle, {e}")),
            },
            Action::Record => {
                leave()?;
                let outcome = record(app);
                *screen = enter()?;
                screen.clear()?;
                match outcome {
                    Ok(done) => {
                        app.done = Some(done);
                        app.step = Step::Done;
                        app.status = None;
                    }
                    Err(e) => {
                        app.step = Step::Review;
                        app.status = Some(format!("recording failed, {e}"));
                    }
                }
            }
        }
    }
}

fn enter() -> Result<Screen> {
    enable_raw_mode()?;
    let mut out = std::io::stdout();
    crossterm::execute!(out, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(out))?)
}

fn leave() -> Result<()> {
    let mut out = std::io::stdout();
    crossterm::execute!(out, crossterm::cursor::Show)?;
    crossterm::execute!(out, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    out.flush()?;
    Ok(())
}

pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = leave();
        previous(info);
    }));
}

fn record(app: &App) -> Result<Done> {
    let cfg = app.config();
    let rules = Rules::from_config(&cfg)?;
    emit::load_theme(&cfg.theme)?;

    eprintln!(
        "ttysvg: recording {} command(s) at {}x{}, watch it run",
        app.lines().len(),
        cfg.cols,
        cfg.rows
    );

    let mut session = capture::spawn(&cfg.shell, cfg.cols, cfg.rows, true, &rules)?;
    let outcome = driver::run(&mut session, &app.ops(), cfg.type_delay);
    let raw = session.finish(cfg.tail);
    outcome?;

    eprint!("\x1b[0m");
    let _ = std::io::stderr().flush();

    if emit::timeline(&raw, &cfg).is_empty() {
        bail!("captured no frames");
    }

    write_out(app, raw)
}

fn write_files(app: &App) -> Result<Done> {
    let raw = match &app.done {
        Some(done) => done.raw.clone(),
        None => bail!("nothing has been recorded yet"),
    };
    write_out(app, raw)
}

fn write_out(app: &App, raw: Vec<(Duration, crate::term::Frame)>) -> Result<Done> {
    let cfg = app.config();
    let theme = emit::load_theme(&cfg.theme)?;
    let timeline = emit::timeline(&raw, &cfg);
    if timeline.is_empty() {
        bail!("the recording has no frames to draw");
    }

    let svg = render(&timeline, &emit::opts(&cfg, theme));
    emit::write(&cfg.output, svg.as_bytes())?;

    let tape_path = app.tape_path();
    let source = tape::write::source(&cfg, &app.ops());
    emit::write(&tape_path, source.as_bytes())?;

    let last = timeline.frames.last().cloned().unwrap_or_default();

    Ok(Done {
        raw,
        frames: timeline.len(),
        secs: timeline.total.as_secs_f64(),
        bytes: svg.len(),
        svg: cfg.output,
        tape: tape_path,
        last,
    })
}

fn open(path: &Path) -> Result<()> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]);
        c
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
    } else {
        std::process::Command::new("xdg-open")
    };
    cmd.arg(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}
