// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `cli.main` with snapshot files of odd shapes. Each expected value is
//! what the retired Python `lodgeitadapter.cli.main` printed, returned or
//! raised under CPython 3.15.0 on Linux, with the transport replaced by a
//! 200 reply of `[]`.

use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use lodgeit_calculator_adapter::cli::{self, Io};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::transport::{Connection, Dial};

const HOST: &str = "https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app";
const SNAPSHOT: &str = r#""provider": "p", "snapshot_id": "s1", "read_at": "t", "read_by": "r",
    "source_urls": []"#;

/// Every dial answers 200 with an empty JSON list.
struct EmptyList;

struct Reply(io::Cursor<Vec<u8>>);

impl Read for Reply {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for Reply {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Dial for EmptyList {
    fn dial(&mut self, _: &str, _: u16, _: bool, _: f64) -> io::Result<Box<dyn Connection>> {
        Ok(Box::new(Reply(io::Cursor::new(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n[]"
                .to_vec(),
        ))))
    }
}

/// A fresh, empty folder for one test.
fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lodgeit-cov-d-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test folder");
    dir
}

/// `dir/name.json` holding `text`, and the `--contract` value naming it.
fn snapshot(dir: &Path, name: &str, text: &str) -> String {
    std::fs::write(dir.join(format!("{name}.json")), text).expect("write the snapshot");
    dir.join(name)
        .to_str()
        .expect("a UTF-8 temp path")
        .to_owned()
}

/// `cli.main(argv)` with captured streams, no environment and every dial
/// answered by [`EmptyList`]. Output reads `\n` for Windows' `\r\n`.
fn run(args: &[&str]) -> (i32, String, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let env = |_: &str| -> Option<OsString> { None };
    let mut sleep = |_: f64| -> Result<(), Exc> { panic!("no request should be retried") };
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = cli::run_io(
        &args,
        &mut stdout,
        &mut stderr,
        &mut Io {
            env: &env,
            dial: &mut EmptyList,
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

/// The last line of a traceback on stderr.
fn raised(stderr: &str) -> &str {
    assert!(
        stderr.starts_with("Traceback (most recent call last):\n"),
        "{stderr}"
    );
    stderr.lines().last().expect("a traceback")
}

#[test]
fn a_snapshot_integer_past_the_digit_limit_escapes_main() {
    // `contract.load` catches only `UnicodeDecodeError` and `JSONDecodeError`.
    let dir = tmp_dir("digits");
    let contract = snapshot(&dir, "digits", &format!(r#"{{"x": {}}}"#, "1".repeat(4301)));
    let (code, stdout, stderr) = run(&["--contract", &contract, "contract"]);
    assert_eq!((code, stdout.as_str()), (1, ""));
    assert_eq!(
        raised(&stderr),
        "ValueError: Exceeds the limit (4300 digits) for integer string conversion: value has \
         4301 digits; use sys.set_int_max_str_digits() to increase the limit"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn drift_without_openapi_schemas_reports_live_discovery_agreeing() {
    let dir = tmp_dir("agree");
    let text = format!(r#"{{{SNAPSHOT}, "calculators": {{}}, "response_contract": {{}}}}"#);
    let contract = snapshot(&dir, "empty", &text);
    let args = [
        "--contract",
        &contract,
        "drift",
        "--enable-network",
        "--base-url",
        HOST,
    ];
    assert_eq!(
        run(&args),
        (
            0,
            "drift: live discovery agrees with snapshot s1\ndrift: a difference is for a person \
             to review. Nothing here updates a snapshot.\n"
                .to_owned(),
            String::new()
        )
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_route_that_is_not_a_string_escapes_invoke() {
    let dir = tmp_dir("route");
    let text = format!(
        r#"{{{SNAPSHOT}, "response_contract": {{}},
        "calculators": {{"c": {{"supported_periods": ["p1"], "route": 5}}}}}}"#
    );
    let contract = snapshot(&dir, "route", &text);
    let body = dir.join("body.json");
    std::fs::write(&body, "{}").expect("write the body");
    let body = body.to_str().expect("a UTF-8 temp path");
    let args = [
        "--contract",
        &contract,
        "invoke",
        "--calculator",
        "c",
        "--period",
        "p1",
        "--body",
        body,
    ];
    let (code, stdout, stderr) = run(&args);
    assert_eq!((code, stdout.as_str()), (1, ""));
    assert_eq!(
        raised(&stderr),
        "AttributeError: 'int' object has no attribute 'replace'"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn response_rules_that_are_not_a_mapping_escape_the_evidence_record() {
    let dir = tmp_dir("rules");
    let text = format!(
        r#"{{{SNAPSHOT}, "response_contract": [1],
        "calculators": {{"c": {{"supported_periods": ["p1"], "route": "/v1/calculators/c"}}}}}}"#
    );
    let contract = snapshot(&dir, "rules", &text);
    let body = dir.join("body.json");
    std::fs::write(&body, "{}").expect("write the body");
    let record = dir.join("rec.json");
    let args = [
        "--contract",
        &contract,
        "--evidence-out",
        record.to_str().expect("a UTF-8 temp path"),
        "invoke",
        "--enable-network",
        "--base-url",
        HOST,
        "--calculator",
        "c",
        "--period",
        "p1",
        "--body",
        body.to_str().expect("a UTF-8 temp path"),
    ];
    let (code, stdout, stderr) = run(&args);
    assert_eq!(code, 1);
    assert_eq!(
        stdout,
        "{\n  \"status\": \"CONTRACT_FAILURE\",\n  \"http_status\": 200,\n  \"findings\": [\n    \
         \"a 200 body that is not a JSON object\"\n  ],\n  \"result\": null\n}\n"
    );
    assert_eq!(
        raised(&stderr),
        "AttributeError: 'list' object has no attribute 'get'"
    );
    assert!(!record.exists());
    let _ = std::fs::remove_dir_all(&dir);
}
