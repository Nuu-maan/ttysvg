use std::time::Duration;

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use ttysvg::term::Frame;
use ttysvg::ui::app::{App, Done, Step};
use ttysvg::ui::draw::draw;
use ttysvg::ui::preview;

const STEPS: [Step; 6] = [
    Step::Preset,
    Step::Command,
    Step::Theme,
    Step::Details,
    Step::Review,
    Step::Done,
];

fn app_at(step: Step) -> App {
    let mut app = App::new();
    app.step = step;
    app.commands = vec!["cargo test".into(), "cargo clippy".into()];
    app.term_cols = 80;
    app.done = Some(Done {
        raw: vec![(Duration::ZERO, Frame::blank(4))],
        frames: 61,
        secs: 7.2,
        bytes: 74_100,
        svg: "demo.svg".into(),
        tape: "demo.tape".into(),
        last: preview::sample(),
    });
    app
}

fn render(app: &App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| draw(f, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect()
}

fn cursor(app: &App, width: u16, height: u16) -> (u16, u16) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| draw(f, app)).unwrap();
    let position = terminal.get_cursor_position().unwrap();
    (position.x, position.y)
}

#[test]
fn every_step_draws_at_a_normal_size() {
    for step in STEPS {
        let out = render(&app_at(step), 100, 30);
        assert!(out.contains("ttysvg"), "{step:?} lost its frame");
    }
}

#[test]
fn every_step_survives_a_cramped_terminal() {
    for step in STEPS {
        for (w, h) in [(20u16, 6u16), (40, 10), (60, 14)] {
            render(&app_at(step), w, h);
        }
    }
}

#[test]
fn every_step_survives_a_terminal_too_small_to_use() {
    for step in STEPS {
        for (w, h) in [(1u16, 1u16), (3, 2), (8, 4)] {
            render(&app_at(step), w, h);
        }
    }
}

#[test]
fn the_last_screen_shows_where_the_files_went() {
    let out = render(&app_at(Step::Done), 120, 30);
    assert!(out.contains("demo.svg"));
    assert!(out.contains("demo.tape"));
    assert!(out.contains("ttysvg build demo.tape"));
}

#[test]
fn the_review_screen_lists_the_commands_it_will_run() {
    let out = render(&app_at(Step::Review), 120, 30);
    assert!(out.contains("cargo test"));
    assert!(out.contains("cargo clippy"));
    assert!(out.contains("tokyonight"));
}

#[test]
fn a_recording_wider_than_the_terminal_is_called_out() {
    let mut app = app_at(Step::Review);
    app.term_cols = 40;
    assert!(app.too_wide());

    let out = render(&app, 120, 30);
    assert!(out.contains("40 columns"));
}

#[test]
fn a_status_message_reaches_the_screen() {
    let mut app = app_at(Step::Review);
    app.status = Some("something went sideways".into());
    let out = render(&app, 120, 30);
    assert!(out.contains("something went sideways"));
}

#[test]
fn the_cursor_sits_where_the_typing_goes() {
    let mut app = app_at(Step::Command);
    app.commands = vec!["cargo --version".into()];
    app.line = 0;
    assert_eq!(cursor(&app, 104, 20), (8 + 15, 4));

    app.commands.push("cargo clippy".into());
    app.line = 1;
    assert_eq!(cursor(&app, 104, 20), (8 + 12, 5));
}

#[test]
fn the_cursor_follows_the_chosen_row_on_every_list() {
    let mut app = app_at(Step::Preset);
    app.preset = 2;
    assert_eq!(cursor(&app, 104, 20).1, 6);

    let mut app = app_at(Step::Theme);
    app.theme = 3;
    assert_eq!(cursor(&app, 104, 20).1, 7);
}

#[test]
fn the_cursor_stays_inside_a_tiny_terminal() {
    for step in STEPS {
        for (w, h) in [(1u16, 1u16), (4, 3), (12, 5)] {
            let (x, y) = cursor(&app_at(step), w, h);
            assert!(
                x < w && y < h,
                "{step:?} put the cursor off screen at {w}x{h}"
            );
        }
    }
}

#[test]
fn the_last_screen_without_a_recording_draws_nothing_and_does_not_panic() {
    let mut app = App::new();
    app.step = Step::Done;
    app.done = None;
    render(&app, 100, 30);
}
