// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.
//! `cli.main` paths the ported suites do not reach, all offline. Each
//! expected value is what the retired Python `lodgeitadapter.cli` printed,
//! returned or raised under CPython 3.15 on Linux; a stack figure in a
//! `RecursionError` is the platform's, so only its wording is checked.
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use lodgeit_calculator_adapter::cli::{self, Io};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::transport::{Connection, Dial};

const CALC: &str = "urn:sbrm:calculator:div7a:at";
const PERIOD: &str = "urn:sbrm:period:div7a:fy2026";

struct NoSockets;

impl Dial for NoSockets {
    fn dial(&mut self, host: &str, port: u16, _: bool, _: f64) -> io::Result<Box<dyn Connection>> {
        panic!("a socket was opened when none should have been ({host}:{port})");
    }
}

/// A fresh, empty folder for one test.
fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lodgeit-cov-b-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test folder");
    dir
}

fn path_text(path: &std::path::Path) -> String {
    path.to_str().expect("a UTF-8 temp path").to_owned()
}

/// `cli.main(argv)` with captured streams, the given `LODGEIT_ADAPTER_*`
/// variables and no sockets. Output reads `\n` for Windows' `\r\n`.
fn run_env(args: &[&str], vars: &[(&str, &str)]) -> (i32, String, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let vars: Vec<(String, String)> = vars
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    let env = move |name: &str| -> Option<OsString> {
        vars.iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| OsString::from(v))
    };
    let mut sleep = |_: f64| -> Result<(), Exc> { panic!("no request should be retried") };
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = cli::run_io(
        &args,
        &mut stdout,
        &mut stderr,
        &mut Io {
            env: &env,
            dial: &mut NoSockets,
            sleep: &mut sleep,
        },
    );
    let decode = |bytes: Vec<u8>| {
        String::from_utf8(bytes)
            .expect("UTF-8 output")
            .replace("\r\n", "\n")
    };
    (code, decode(stdout), decode(stderr))
}

fn run(args: &[&str]) -> (i32, String, String) {
    run_env(args, &[])
}

fn on_a_big_stack(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(test)
        .expect("spawn the test thread")
        .join()
        .expect("the test thread ran");
}

fn invoke_body(name: &str, bytes: &[u8]) -> (i32, String, String, String) {
    let dir = tmp_dir(name);
    let body = dir.join("body.json");
    std::fs::write(&body, bytes).expect("write the body");
    let shown = path_text(&body);
    let (code, stdout, stderr) = run(&[
        "invoke",
        "--calculator",
        CALC,
        "--period",
        PERIOD,
        "--body",
        &shown,
    ]);
    let _ = std::fs::remove_dir_all(&dir);
    (code, stdout, stderr, shown)
}

#[test]
fn a_body_integer_past_the_digit_limit_is_reported_as_unreadable() {
    let body = format!(r#"{{"loan_amount": {}}}"#, "1".repeat(4301));
    let (code, stdout, stderr, shown) = invoke_body("digits", body.as_bytes());
    assert_eq!((code, stdout.as_str()), (2, ""));
    assert_eq!(
        stderr,
        format!(
            "lodgeit-adapter: {shown} could not be read: Exceeds the limit (4300 digits) for \
             integer string conversion: value has 4301 digits; use \
             sys.set_int_max_str_digits() to increase the limit\n"
        )
    );
}

#[test]
fn a_body_that_is_not_utf8_is_reported_as_unreadable() {
    let (code, _, stderr, shown) = invoke_body("utf8", b"\xff{}");
    assert_eq!(code, 2);
    assert_eq!(
        stderr,
        format!(
            "lodgeit-adapter: {shown} could not be read: 'utf-8' codec can't decode byte 0xff \
             in position 0: invalid start byte\n"
        )
    );
}

#[test]
fn a_body_nested_past_the_stack_raises_a_recursion_error() {
    on_a_big_stack(|| {
        let (code, stdout, stderr, _) = invoke_body("deep", "[".repeat(200_000).as_bytes());
        assert_eq!((code, stdout.as_str()), (1, ""));
        let last = stderr.lines().last().expect("a traceback");
        assert!(
            stderr.starts_with("Traceback (most recent call last):\n"),
            "{stderr}"
        );
        assert!(last.starts_with("RecursionError: "), "{last}");
        assert!(
            last.ends_with(" while decoding a JSON array from a unicode string"),
            "{last}"
        );
    });
}

#[test]
fn verify_lets_the_digit_limit_value_error_escape() {
    // `except (OSError, json.JSONDecodeError)` does not catch it.
    let dir = tmp_dir("verify-digits");
    let record = dir.join("record.json");
    std::fs::write(&record, format!(r#"{{"x": {}}}"#, "1".repeat(4301))).expect("write");
    let (code, stdout, stderr) = run(&["verify", "--evidence", &path_text(&record)]);
    assert_eq!((code, stdout.as_str()), (1, ""));
    assert!(
        stderr.ends_with(
            "\nValueError: Exceeds the limit (4300 digits) for integer string conversion: \
             value has 4301 digits; use sys.set_int_max_str_digits() to increase the limit\n"
        ),
        "{stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_discover_record_takes_the_command_as_its_label() {
    // `getattr(args, "label", args.command)`: discover has no --label.
    // The digest is Python's with this crate's `contracts/lodgeit-calculators.json`.
    let dir = tmp_dir("discover-label");
    let target = dir.join("out").join("rec.json");
    let shown = path_text(&target);
    let (code, stdout, stderr) = run(&["discover", "--evidence-out", &shown]);
    assert_eq!(code, 1);
    assert!(
        stdout.starts_with("{\n  \"status\": \"REFUSED_TO_SEND\",\n"),
        "{stdout}"
    );
    assert!(
        stdout.ends_with(&format!(
            "evidence written to {shown} (calculation_sha256 \
             871e18a0574d1e4587e63121c3ebd27b6f862ba09076d2ac0946f39e44b7db60)\n"
        )),
        "{stdout}"
    );
    assert_eq!(
        stderr,
        "lodgeit-adapter: REFUSED_TO_SEND. This is not a result and must not be read as one.\n"
    );
    let written = std::fs::read_to_string(&target).expect("the record was written");
    assert!(written.contains("\"label\": \"discover\""), "{written}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_environment_base_url_off_the_allowlist_raises() {
    // No --base-url, so `base.base_url` from LODGEIT_ADAPTER_BASE_URL is used,
    // and the transport's DisallowedTargetError escapes main.
    let (code, stdout, stderr) = run_env(
        &["discover"],
        &[("LODGEIT_ADAPTER_BASE_URL", "https://calc.example.invalid")],
    );
    assert_eq!((code, stdout.as_str()), (1, ""));
    assert!(
        stderr.ends_with(
            "DisallowedTargetError: https://calc.example.invalid: host calc.example.invalid is \
             not on the configured allowlist (fbt-calculator-api-qkp3j5bjnq-ts.a.run.app, \
             fano-engine-79859053141.australia-southeast1.run.app, \
             fano-engine-afmurhqkaq-ts.a.run.app)\n"
        ),
        "{stderr}"
    );
}

#[test]
fn a_body_that_is_not_json_is_reported_with_the_decoder_message() {
    let (code, _, stderr, shown) = invoke_body("not-json", b"{not json");
    assert_eq!(code, 2);
    assert_eq!(
        stderr,
        format!(
            "lodgeit-adapter: {shown} could not be read: Expecting property name enclosed in \
             double quotes: line 1 column 2 (char 1)\n"
        )
    );
}
