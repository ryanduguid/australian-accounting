use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gst-review"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn json_success_errors_and_usage_have_separate_streams() {
    let output = run(&["review-coding"], br#"{"lines":[],"registrations":{}}"#);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["review"], "gst_coding");
    for (args, input) in [
        (vec!["review-coding"], b"{bad".as_slice()),
        (vec!["reconcile-bas"], b"null".as_slice()),
        (vec!["review-coding", "extra"], b"".as_slice()),
        (vec!["--unknown"], b"".as_slice()),
    ] {
        let output = run(&args, input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(output.stderr.starts_with(b"error: "));
    }
    for args in [
        vec![],
        vec!["--help"],
        vec!["--version"],
        vec!["--api-help"],
    ] {
        let output = run(&args, b"");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert!(!output.stdout.is_empty());
    }
}
#[test]
fn bounded_input_and_invalid_utf8_are_refused() {
    let output = run(&["review-coding"], &vec![b' '; (8 << 20) + 1]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stderr, b"error: JSON input exceeds 8 MiB\n");
    assert!(output.stdout.is_empty());
    let output = run(
        &["review-coding"],
        b"{\"lines\":[],\"registrations\":{},\"x\":\"\xff\"}",
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}
// macOS has no /dev/full.
#[cfg(target_os = "linux")]
#[test]
fn failed_output_stream_is_a_failure() {
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_gst-review"))
        .arg("--help")
        .stdout(full)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(1));
}
