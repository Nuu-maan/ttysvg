use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use ttysvg::capture::{self, driver};
use ttysvg::optimize::{self, Options};
use ttysvg::svg::theme::Theme;
use ttysvg::svg::{render, RenderOpts};
use ttysvg::tape::{self, Config, DEFAULT_FONT};
use ttysvg::term::Frame;

#[derive(Parser)]
#[command(
    name = "ttysvg",
    version,
    about = "Terminal recordings as self-contained animated SVG"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Record(RecordArgs),
    Build(BuildArgs),
    Themes,
}

#[derive(Args)]
struct StyleArgs {
    #[arg(long, default_value = "tokyonight")]
    theme: String,
    #[arg(long, default_value = DEFAULT_FONT)]
    font: String,
    #[arg(long = "font-size", default_value_t = 14.0)]
    font_size: f64,
    #[arg(long, default_value_t = 0.0)]
    advance: f64,
    #[arg(long = "line-height", default_value_t = 0.0)]
    line_height: f64,
    #[arg(long, default_value_t = 18.0)]
    padding: f64,
    #[arg(long)]
    window: bool,
    #[arg(long, default_value = "")]
    title: String,
    #[arg(long = "no-loop")]
    no_loop: bool,
}

#[derive(Args)]
struct RecordArgs {
    #[arg(short, long, default_value = "demo.svg")]
    out: PathBuf,
    #[arg(long)]
    cols: Option<u16>,
    #[arg(long)]
    rows: Option<u16>,
    #[arg(long = "trim-idle", default_value = "1s")]
    trim_idle: String,
    #[arg(long, default_value_t = 1.0)]
    speed: f64,
    #[arg(long, default_value = "2s")]
    tail: String,
    #[command(flatten)]
    style: StyleArgs,
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

#[derive(Args)]
struct BuildArgs {
    tape: PathBuf,
    #[arg(short, long)]
    out: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Record(a) => record(a),
        Cmd::Build(a) => build(a),
        Cmd::Themes => {
            for name in Theme::names() {
                println!("{name}");
            }
            Ok(())
        }
    }
}

fn trim_opt(s: &str) -> Result<Option<Duration>> {
    if s.eq_ignore_ascii_case("off") || s.eq_ignore_ascii_case("none") {
        Ok(None)
    } else {
        Ok(Some(tape::parse::duration(s)?))
    }
}

fn config_from(style: &StyleArgs) -> Config {
    Config {
        theme: style.theme.clone(),
        font_family: style.font.clone(),
        font_size: style.font_size,
        advance: style.advance,
        line_height: style.line_height,
        padding: style.padding,
        window: style.window,
        title: style.title.clone(),
        loop_forever: !style.no_loop,
        ..Config::default()
    }
}

fn record(a: RecordArgs) -> Result<()> {
    let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((90, 22));
    let cols = a.cols.unwrap_or(term_cols.clamp(20, 200));
    let rows = a.rows.unwrap_or(term_rows.clamp(6, 60));

    let mut cfg = config_from(&a.style);
    cfg.output = a.out.clone();
    cfg.cols = cols;
    cfg.rows = rows;
    cfg.trim_idle = trim_opt(&a.trim_idle)?;
    cfg.tail = tape::parse::duration(&a.tail)?;
    cfg.speed = a.speed;

    let theme = load_theme(&cfg.theme)?;

    eprintln!(
        "ttysvg: recording {} at {}x{}. exit the program to stop.",
        a.command.join(" "),
        cols,
        rows
    );

    let mut session = capture::spawn(&a.command, cols, rows, true)?;

    let raw_mode = crossterm::terminal::enable_raw_mode().is_ok();

    if let Some(mut writer) = session.take_writer() {
        std::thread::spawn(move || {
            let mut stdin = std::io::stdin();
            let mut buf = [0u8; 1024];
            loop {
                match stdin.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if writer.write_all(&buf[..n]).is_err() {
                            break;
                        }
                        let _ = writer.flush();
                    }
                }
            }
        });
    }

    session.wait_exit();

    if raw_mode {
        let _ = crossterm::terminal::disable_raw_mode();
    }

    let frames = session.collect();
    emit(&cfg, theme, &frames)
}

fn build(a: BuildArgs) -> Result<()> {
    let src = std::fs::read_to_string(&a.tape)
        .with_context(|| format!("reading tape {}", a.tape.display()))?;
    let parsed = tape::parse(&src)?;
    let mut cfg = parsed.config;
    if let Some(out) = a.out {
        cfg.output = out;
    }
    if cfg.output.is_relative() {
        if let Some(dir) = a.tape.parent() {
            if !dir.as_os_str().is_empty() {
                cfg.output = dir.join(&cfg.output);
            }
        }
    }

    let theme = load_theme(&cfg.theme)?;

    eprintln!(
        "ttysvg: building {} ({} ops) at {}x{}",
        a.tape.display(),
        parsed.ops.len(),
        cfg.cols,
        cfg.rows
    );

    let mut session = capture::spawn(&cfg.shell, cfg.cols, cfg.rows, false)?;
    let result = driver::run(&mut session, &parsed.ops, cfg.type_delay);
    let frames = session.finish(cfg.tail);
    result?;

    emit(&cfg, theme, &frames)
}

fn load_theme(name: &str) -> Result<Theme> {
    let theme = Theme::load(name)?;
    theme.validate()?;
    Ok(theme)
}

fn emit(cfg: &Config, theme: Theme, raw: &[(Duration, Frame)]) -> Result<()> {
    let opts = Options {
        trim_idle: cfg.trim_idle,
        tail: cfg.tail,
        speed: cfg.speed,
        ..Options::default()
    };
    let tl = optimize::optimize(raw, &opts);

    if tl.is_empty() {
        anyhow::bail!("captured no frames, nothing to render");
    }

    let render_opts = RenderOpts {
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
    }
    .with_metrics();

    let svg = render(&tl, &render_opts);

    if let Some(dir) = cfg.output.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }
    std::fs::write(&cfg.output, svg.as_bytes())
        .with_context(|| format!("writing {}", cfg.output.display()))?;

    eprintln!(
        "ttysvg: wrote {} ({} frames, {:.1}s, {})",
        cfg.output.display(),
        tl.len(),
        tl.total.as_secs_f64(),
        human_size(svg.len())
    );
    Ok(())
}

fn human_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
