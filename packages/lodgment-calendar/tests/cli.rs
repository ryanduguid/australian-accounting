use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lodgment-calendar"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let bytes = input.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&bytes);
    });
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    output
}

#[test]
fn success_writes_one_json_result_and_errors_only_use_stderr() {
    let input=br#"{"obligation":"income_tax_return","entity_type":"partnership","financial_year":"2025-26","channel":"self"}"#;
    let output = run(&["due-dates"], input);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["engine_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value["obligations"][0]["payment"], "none");
    for (args, input) in [
        (vec!["due-dates"], b"{".as_slice()),
        (vec!["due-dates"], b"[]".as_slice()),
        (vec!["unknown"], b"".as_slice()),
    ] {
        let output = run(&args, input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(output.stderr.starts_with(b"error: "));
    }
}

#[test]
fn bounded_input_and_command_help() {
    let output = run(&["due-dates"], &vec![b' '; (8 << 20) + 1]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: JSON input exceeds 8 MiB\n"
    );
    for flag in ["--version", "--help", "--api-help"] {
        let output = run(&[flag], b"");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn duplicate_fields_at_every_depth_are_refused() {
    for input in [
        br#"{"obligation":"bas","obligation":"ias"}"#.as_slice(),
        br#"{"facts":{"lodges_electronically":true,"lodges_electronically":false}}"#.as_slice(),
        br#"{"extra_holidays":{"2027-07-28":"One","2027-07-28":"Two"}}"#.as_slice(),
        br#"{"facts":[{"a":1,"a":2}]}"#.as_slice(),
    ] {
        let output = run(&["due-dates"], input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("duplicate JSON field")
        );
    }
}

#[test]
fn hostile_refusal_values_do_not_panic_or_emit_controls() {
    for input in [
        br#"{"obligation":1e99999}"#.as_slice(),
        br#"{"obligation":"a\u0000\u001b"}"#.as_slice(),
        br#"{"a\u0000\u001b":true}"#.as_slice(),
        br#"{"a\u0000\u001b":true,"a\u0000\u001b":false}"#.as_slice(),
    ] {
        let output = run(&["due-dates"], input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.contains(&0));
        assert!(!output.stderr.contains(&27));
    }
}
