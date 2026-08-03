use std::path::PathBuf;
use std::time::Duration;

use ttysvg::tape::{parse, write, Key as TapeKey, Op};
use ttysvg::ui::app::{Action, App, Step, PAUSE_STEP, WARMUP};
use ttysvg::ui::preset;
use ttysvg::ui::{KeyCode, KeyEvent, KeyModifiers};

fn press(app: &mut App, code: KeyCode) {
    app.key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn write_text(app: &mut App, text: &str) {
    for ch in text.chars() {
        press(app, KeyCode::Char(ch));
    }
}

fn ready(commands: &[&str]) -> App {
    let mut app = App::new();
    press(&mut app, KeyCode::Enter);
    for (i, command) in commands.iter().enumerate() {
        if i > 0 {
            press(&mut app, KeyCode::Enter);
        }
        write_text(&mut app, command);
    }
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    app
}

#[test]
fn the_flow_walks_forward_and_back() {
    let mut app = App::new();
    assert_eq!(app.step, Step::Preset);

    press(&mut app, KeyCode::Tab);
    assert_eq!(app.step, Step::Command);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.step, Step::Theme);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.step, Step::Details);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.step, Step::Review);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.step, Step::Review);

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.step, Step::Details);
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.step, Step::Theme);
}

#[test]
fn escape_only_quits_from_the_first_step() {
    let mut app = App::new();
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.action, Action::None);
    assert_eq!(app.step, Step::Preset);

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.action, Action::Quit);
}

#[test]
fn control_c_quits_from_anywhere() {
    let mut app = App::new();
    press(&mut app, KeyCode::Tab);
    write_text(&mut app, "ls");
    app.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert_eq!(app.action, Action::Quit);
}

#[test]
fn choosing_a_preset_rewrites_the_settings() {
    let mut app = App::new();
    assert_eq!(app.cfg.cols, preset::ALL[0].cols);

    press(&mut app, KeyCode::Down);
    assert_eq!(app.preset, 1);
    assert_eq!(app.cfg.cols, preset::ALL[1].cols);
    assert_eq!(app.cfg.rows, preset::ALL[1].rows);
    assert_eq!(app.cfg.window, preset::ALL[1].window);
    assert_eq!(app.pause, preset::ALL[1].pause);

    press(&mut app, KeyCode::Up);
    assert_eq!(app.preset, 0);
    assert_eq!(app.cfg.cols, preset::ALL[0].cols);
}

#[test]
fn the_preset_list_wraps_at_both_ends() {
    let mut app = App::new();
    press(&mut app, KeyCode::Up);
    assert_eq!(app.preset, preset::ALL.len() - 1);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.preset, 0);
}

#[test]
fn commands_are_edited_line_by_line() {
    let mut app = App::new();
    press(&mut app, KeyCode::Enter);

    write_text(&mut app, "cargo test");
    press(&mut app, KeyCode::Enter);
    write_text(&mut app, "cargo clippy");
    assert_eq!(app.commands, vec!["cargo test", "cargo clippy"]);
    assert_eq!(app.line, 1);

    press(&mut app, KeyCode::Backspace);
    assert_eq!(app.commands[1], "cargo clipp");

    press(&mut app, KeyCode::Up);
    assert_eq!(app.line, 0);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.line, 1);
}

#[test]
fn backspace_on_an_empty_line_removes_it() {
    let mut app = App::new();
    press(&mut app, KeyCode::Enter);
    write_text(&mut app, "ls");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.commands.len(), 2);

    press(&mut app, KeyCode::Backspace);
    assert_eq!(app.commands, vec!["ls"]);
    assert_eq!(app.line, 0);
}

#[test]
fn an_empty_last_line_moves_on_instead_of_growing() {
    let mut app = App::new();
    press(&mut app, KeyCode::Enter);
    write_text(&mut app, "ls");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.step, Step::Theme);
    assert_eq!(app.commands, vec!["ls"]);
}

#[test]
fn themes_cycle_and_land_in_the_config() {
    let mut app = App::new();
    let names = app.themes().to_vec();

    app.step = Step::Theme;
    press(&mut app, KeyCode::Down);
    assert_eq!(app.theme_name(), names[1]);
    assert_eq!(app.config().theme, names[1]);

    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Up);
    assert_eq!(app.theme_name(), names[names.len() - 1]);
}

#[test]
fn details_edit_text_toggle_flags_and_nudge_the_pause() {
    let mut app = App::new();
    app.step = Step::Details;

    press(&mut app, KeyCode::Backspace);
    assert_eq!(app.output, "demo.sv");
    write_text(&mut app, "g");
    assert_eq!(app.output, "demo.svg");

    press(&mut app, KeyCode::Down);
    write_text(&mut app, "!");
    assert!(app.cfg.title.ends_with('!'));

    press(&mut app, KeyCode::Down);
    let window = app.cfg.window;
    press(&mut app, KeyCode::Char(' '));
    assert_eq!(app.cfg.window, !window);

    press(&mut app, KeyCode::Down);
    assert!(app.cfg.sanitize);
    press(&mut app, KeyCode::Right);
    assert!(!app.cfg.sanitize);

    press(&mut app, KeyCode::Down);
    let before = app.pause;
    press(&mut app, KeyCode::Right);
    assert_eq!(app.pause, before + Duration::from_millis(PAUSE_STEP));
    press(&mut app, KeyCode::Left);
    assert_eq!(app.pause, before);
}

#[test]
fn sanitizing_is_on_before_anyone_asks() {
    assert!(App::new().cfg.sanitize);
}

#[test]
fn recording_is_refused_until_there_is_something_to_record() {
    let mut app = App::new();
    app.step = Step::Review;
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.action, Action::None);
    assert!(app.status.is_some());
}

#[test]
fn an_output_that_is_not_an_svg_is_refused() {
    let mut app = ready(&["ls"]);
    app.output = "demo.png".into();
    app.step = Step::Review;
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.action, Action::None);
    assert!(app.status.unwrap().contains(".svg"));
}

#[test]
fn a_finished_wizard_asks_to_record() {
    let mut app = ready(&["cargo test"]);
    assert_eq!(app.step, Step::Theme);

    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.step, Step::Review);

    press(&mut app, KeyCode::Enter);
    assert_eq!(app.action, Action::Record);
}

#[test]
fn commands_become_type_enter_wait() {
    let app = ready(&["cargo test", "cargo clippy"]);
    let pause = app.pause;

    assert_eq!(
        app.ops(),
        vec![
            Op::Sleep(WARMUP),
            Op::Type("cargo test".into()),
            Op::Key(TapeKey::Enter),
            Op::Sleep(pause),
            Op::Type("cargo clippy".into()),
            Op::Key(TapeKey::Enter),
            Op::Sleep(pause),
        ]
    );
}

#[test]
fn blank_lines_never_reach_the_recording() {
    let mut app = ready(&["ls"]);
    app.commands.push("   ".into());
    app.commands.push(String::new());

    assert_eq!(app.lines(), vec!["ls"]);
    assert_eq!(app.ops().len(), 4);
}

#[test]
fn the_tape_it_leaves_behind_rebuilds_the_same_recording() {
    let mut app = ready(&["git status --short", r".\scripts\demo.ps1"]);
    app.step = Step::Details;
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char(' '));

    let cfg = app.config();
    let ops = app.ops();
    let tape = parse(&write::source(&cfg, &ops)).unwrap();

    assert_eq!(tape.ops, ops);
    assert_eq!(tape.config.cols, cfg.cols);
    assert_eq!(tape.config.rows, cfg.rows);
    assert_eq!(tape.config.theme, cfg.theme);
    assert_eq!(tape.config.window, cfg.window);
    assert_eq!(tape.config.sanitize, cfg.sanitize);
    assert_eq!(tape.config.output, cfg.output);
    assert_eq!(tape.config.trim_idle, cfg.trim_idle);
    assert_eq!(tape.config.tail, cfg.tail);
}

#[test]
fn the_tape_lands_next_to_the_svg() {
    let mut app = App::new();
    app.output = "docs/examples/demo.svg".into();
    assert_eq!(app.tape_path(), PathBuf::from("docs/examples/demo.tape"));
}

#[test]
fn restyling_after_a_recording_never_asks_to_record_again() {
    let mut app = ready(&["ls"]);
    app.step = Step::Done;

    press(&mut app, KeyCode::Right);
    assert_eq!(app.action, Action::Rerender);

    app.action = Action::None;
    press(&mut app, KeyCode::Char('w'));
    assert_eq!(app.action, Action::Rerender);
}

#[test]
fn redoing_from_the_last_screen_records_again() {
    let mut app = ready(&["ls"]);
    app.step = Step::Done;

    press(&mut app, KeyCode::Char('r'));
    assert_eq!(app.action, Action::Record);
    assert_eq!(app.step, Step::Review);
}

#[test]
fn every_preset_describes_itself_and_fits_on_screen() {
    for p in preset::ALL {
        assert!(!p.name.is_empty());
        assert!(p.about.len() > 40, "{} has no explanation", p.name);
        assert!((60..=120).contains(&p.cols), "{} is an odd width", p.name);
        assert!((10..=40).contains(&p.rows), "{} is an odd height", p.name);
        assert!(p.font_size >= 12.0);
    }
}
