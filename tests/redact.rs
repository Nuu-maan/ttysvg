use std::time::Duration;

use ttysvg::redact::Rules;
use ttysvg::session::Capture;
use ttysvg::tape::{self, Config};
use ttysvg::term::{Color, Frame, Row, Run, Style};

fn row(text: &str) -> Row {
    Row {
        runs: vec![Run {
            col: 0,
            width: text.chars().count() as u16,
            text: text.to_string(),
            style: Style {
                fg: Color::Idx(2),
                ..Style::default()
            },
        }],
    }
}

fn frame(text: &str) -> Frame {
    Frame {
        rows: vec![row(text)],
        cursor: None,
    }
}

fn with_pattern(pattern: &str) -> Rules {
    let mut rules = Rules::default();
    rules.pattern(pattern).unwrap();
    rules
}

#[test]
fn a_pattern_is_masked_without_changing_width() {
    let rules = with_pattern("sk-[A-Za-z0-9]+");
    assert_eq!(
        rules.text("export KEY=sk-Ab3xQ9 done"),
        "export KEY=********* done"
    );
}

#[test]
fn masking_leaves_the_run_width_alone() {
    let rules = with_pattern("sk-[A-Za-z0-9]+");
    let mut f = frame("KEY=sk-Ab3xQ9");
    let before = f.rows[0].runs[0].width;
    rules.frame(&mut f);

    let run = &f.rows[0].runs[0];
    assert_eq!(run.text, "KEY=*********");
    assert_eq!(run.width, before);
    assert_eq!(run.style.fg, Color::Idx(2));
}

#[test]
fn a_literal_replacement_shrinks_the_run() {
    let mut rules = Rules::default();
    rules.replace("C:\\Users\\numan", "~");
    let mut f = frame("C:\\Users\\numan\\projects");
    rules.frame(&mut f);

    let run = &f.rows[0].runs[0];
    assert_eq!(run.text, "~\\projects");
    assert_eq!(run.width, run.text.chars().count() as u16);
}

#[test]
fn a_replacement_is_never_longer_than_what_it_replaces() {
    let mut rules = Rules::default();
    rules.replace("ab", "replacement");
    assert_eq!(rules.text("xx ab yy"), "xx re yy");
}

#[test]
fn word_replacements_do_not_match_inside_other_words() {
    let mut rules = Rules::default();
    rules.replace_word("root", "user");
    assert_eq!(rules.text("root chroot rooted"), "user chroot rooted");
}

#[test]
fn rules_apply_in_order_so_paths_win_over_usernames() {
    let mut rules = Rules::default();
    rules.replace("/home/numan", "~");
    rules.replace_word("numan", "user");
    assert_eq!(rules.text("/home/numan and numan"), "~ and user");
}

#[test]
fn sanitize_ignores_case() {
    let mut rules = Rules::default();
    rules.replace("C:\\Users\\numan", "~");
    rules.replace_word("NUMAN", "host");

    assert_eq!(rules.text("user@Numan"), "user@host");
    assert_eq!(rules.text("user@numan"), "user@host");
    assert_eq!(rules.text("C:\\USERS\\NUMAN\\src"), "~\\src");
}

#[test]
fn a_supplied_pattern_stays_case_sensitive() {
    let rules = with_pattern("sk-[a-z0-9]+");
    assert_eq!(rules.text("sk-abc"), "******");
    assert_eq!(rules.text("SK-ABC"), "SK-ABC");
}

#[test]
fn no_rules_means_no_change() {
    let rules = Rules::default();
    assert!(rules.is_empty());
    let mut f = frame("nothing to hide");
    let before = f.clone();
    rules.frame(&mut f);
    assert_eq!(f, before);
}

#[test]
fn a_broken_pattern_is_reported() {
    let mut rules = Rules::default();
    let err = rules.pattern("sk-[").unwrap_err().to_string();
    assert!(err.contains("bad redact pattern"), "{err}");
}

#[test]
fn sanitize_rewrites_the_home_directory() {
    let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) else {
        return;
    };
    let mut rules = Rules::default();
    rules.sanitize();
    assert_eq!(rules.text(&format!("{home}/projects")), "~/projects");
}

#[test]
fn the_tape_carries_redaction_settings() {
    let tape = tape::parse(
        r#"
        redact "sk-[A-Za-z0-9]+"
        redact "ghp_[A-Za-z0-9]+"
        sanitize on
        type "ls"
        "#,
    )
    .unwrap();

    assert_eq!(tape.config.redact.len(), 2);
    assert!(tape.config.sanitize);

    let rules = Rules::from_config(&tape.config).unwrap();
    assert_eq!(rules.text("ghp_AAbb11"), "**********");
}

#[test]
fn a_saved_capture_does_not_keep_the_command_in_the_clear() {
    let cfg = Config {
        cols: 20,
        rows: 1,
        shell: vec![
            "sh".into(),
            "-c".into(),
            "curl -H 'Bearer sk-Ab3xQ9'".into(),
        ],
        ..Config::default()
    };
    let rules = with_pattern("sk-[A-Za-z0-9]+");
    let raw = vec![(Duration::ZERO, frame("KEY=sk-Ab3xQ9"))];

    let capture = Capture::new(&["curl".into(), "sk-Ab3xQ9".into()], &cfg, &raw, &rules);
    let json = capture.to_json().unwrap();

    assert!(
        !json.contains("sk-Ab3xQ9"),
        "secret survived into the capture"
    );
    assert_eq!(capture.command[1], "*********");
    assert_eq!(capture.config.shell[2], "curl -H 'Bearer *********'");
}
