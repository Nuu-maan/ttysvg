use std::time::Duration;

use ttysvg::capture;
use ttysvg::redact::Rules;

const MARKER: &str = "ttysvg-path-probe";

#[test]
fn the_recorded_shell_sees_the_parent_path() {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let entry = if cfg!(windows) {
        format!("C:\\{MARKER}")
    } else {
        format!("/tmp/{MARKER}")
    };

    let original = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{entry}{separator}{original}"));

    let argv: Vec<String> = if cfg!(windows) {
        vec!["cmd.exe".into(), "/c".into(), "echo %PATH%".into()]
    } else {
        vec![
            "/bin/sh".into(),
            "-c".into(),
            "printf '%s' \"$PATH\"".into(),
        ]
    };

    let session = capture::spawn(&argv, 200, 10, false, &Rules::default()).expect("spawn failed");
    let seen = session.wait_for(MARKER, Duration::from_secs(15));
    let _ = session.finish(Duration::ZERO);

    std::env::set_var("PATH", original);

    seen.expect("the recorded shell did not inherit the parent PATH");
}
