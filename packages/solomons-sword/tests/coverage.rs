// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Behavioural coverage tests for API boundaries, calendar dates, decimal
//! edge cases, CLI parsing, terminal sizing, colour selection and stdio handling.

use serde_json::json;
use solomons_sword::api;
use solomons_sword::date::Date;
use solomons_sword::decimal::{Decimal, Rounding, Signal};
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn d(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

fn run_cli(args: &[&str], envs: &[(&str, Option<&str>)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_solomons-sword"));
    cmd.args(args)
        .env_remove("PYTHON_COLORS")
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("COLUMNS")
        .env_remove("LINES")
        .env("TERM", "xterm");
    for (k, v) in envs {
        match v {
            Some(val) => cmd.env(k, val),
            None => cmd.env_remove(k),
        };
    }
    cmd.output().unwrap()
}

fn pipe_api(command: &str, payload: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_solomons-sword"))
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(payload).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn api_rejects_unknown_commands_and_malformed_types() {
    assert_eq!(
        api::evaluate("no-such-command", &json!({})),
        Err("unknown command no-such-command".into())
    );
    let bad_array = json!({
        "trust_accounting_income": "100",
        "section95_net_taxable_income": "100",
        "beneficiaries": "none"
    });
    assert_eq!(
        api::evaluate("proportionate-share", &bad_array),
        Err("beneficiaries must be an array".into())
    );
    let bad_year = json!({"financial_year": "2026"});
    assert_eq!(
        api::evaluate("trust-resolution", &bad_year),
        Err("financial_year must be an integer".into())
    );
    let res = api::evaluate(
        "trust-resolution",
        &json!({
            "financial_year": 2026,
            "resolution_date": "2026-06-30",
            "is_signed_by_trustee": true,
            "streaming_powers_in_deed": true,
            "default_beneficiary_clause_exists": true,
            "allocated_percentages_total": "100",
            "uses_specific_streaming": false,
            "deed_resolution_deadline": "2026-07-15"
        }),
    )
    .unwrap();
    assert_eq!(res["result"], json!(true));
    assert_eq!(res["resolution_deadline"], json!("2026-06-30"));
}

#[test]
fn date_handles_leap_years_and_year_bounds() {
    assert_eq!(
        Date::new(0, 1, 1),
        Err("year must be in 1..9999, not 0".into())
    );
    assert_eq!(
        Date::new(10000, 1, 1),
        Err("year must be in 1..9999, not 10000".into())
    );
    assert_eq!(Date::new(2024, 2, 29).unwrap().to_string(), "2024-02-29");
    assert_eq!(Date::new(2000, 2, 29).unwrap().to_string(), "2000-02-29");
    assert_eq!(
        Date::new(1900, 2, 29),
        Err("day is out of range for month".into())
    );
    assert_eq!(
        Date::new(2025, 13, 1),
        Err("day is out of range for month".into())
    );
    assert_eq!(Date::parse("2024-02-29").unwrap().to_string(), "2024-02-29");
    assert_eq!(Date::parse("2025-02-29"), None);
    assert_eq!(Date::parse("2025-04-31"), None);
    assert_eq!(Date::parse("2025-00-10"), None);
}

#[test]
fn decimal_parse_rejects_internal_unicode_spaces_and_extreme_exponents() {
    assert_eq!(
        Decimal::parse("1\u{a0}0").err(),
        Some(Signal::InvalidOperation)
    );
    assert_eq!(
        Decimal::parse("1\u{00b2}").err(),
        Some(Signal::InvalidOperation)
    );
    assert_eq!(
        Decimal::parse("0e-9999999999999999999").err(),
        Some(Signal::InvalidOperation)
    );
    assert_eq!(
        Decimal::parse("1e-9999999999999999999").err(),
        Some(Signal::InvalidOperation)
    );
    assert_eq!(d("1.23").exponent(), -2);
    assert_eq!(d("NaN").adjusted(), 0);
}

#[test]
fn decimal_special_and_extreme_arithmetic() {
    assert_eq!(
        d("Infinity").add(&d("-Infinity")).err(),
        Some(Signal::InvalidOperation)
    );
    assert_eq!(
        d("Infinity").add(&d("1")).unwrap().to_sci_string(),
        "Infinity"
    );
    assert_eq!(
        d("1").add(&d("Infinity")).unwrap().to_sci_string(),
        "Infinity"
    );
    assert_eq!(d("NaN").sub(&d("1")).unwrap().to_sci_string(), "NaN");
    assert_eq!(d("1").sub(&d("NaN")).unwrap().to_sci_string(), "NaN");
    assert_eq!(d("sNaN").sub(&d("1")).err(), Some(Signal::InvalidOperation));
    assert_eq!(d("NaN").mul(&d("2")).unwrap().to_sci_string(), "NaN");
    assert_eq!(
        d("Infinity").mul(&d("-2")).unwrap().to_sci_string(),
        "-Infinity"
    );
    assert_eq!(d("0").mul(&d("5")).unwrap().to_sci_string(), "0");
    assert_eq!(d("NaN").div(&d("2")).unwrap().to_sci_string(), "NaN");
    assert_eq!(
        d("Infinity").div(&d("Infinity")).err(),
        Some(Signal::InvalidOperation)
    );
    assert_eq!(
        d("Infinity").div(&d("-2")).unwrap().to_sci_string(),
        "-Infinity"
    );
}

#[test]
fn decimal_division_quantize_rounding_and_ordering() {
    let huge = d("1000000000000000000000000000000");
    assert_eq!(
        huge.div(&d("2")).unwrap().to_sci_string(),
        "5.000000000000000000000000000E+29"
    );
    assert_eq!(
        d("1").div(&d("7")).unwrap().to_sci_string(),
        "0.1428571428571428571428571429"
    );
    assert_eq!(
        d("1").quantize(1_000_000, Rounding::HalfUp).err(),
        Some(Signal::InvalidOperation)
    );
    assert_eq!(
        d("0")
            .quantize(-2, Rounding::HalfUp)
            .unwrap()
            .to_sci_string(),
        "0.00"
    );
    assert_eq!(
        d("9999999999999999999999999999.9")
            .quantize(0, Rounding::HalfUp)
            .err(),
        Some(Signal::InvalidOperation)
    );
    assert_eq!(d("1E-10").fixed(2, false), "0.00");
    assert_eq!(d("1E+1000000").add(&d("0")).err(), Some(Signal::Overflow));
    assert_eq!(
        d("9.9999999999999999999999999995E+999999")
            .add(&d("0"))
            .err(),
        Some(Signal::Overflow)
    );
    assert_eq!(
        d("1E-1000050").add(&d("0")).unwrap().to_sci_string(),
        "0E-1000026"
    );
    assert_eq!(
        d("5E-1000027").add(&d("0")).unwrap().to_sci_string(),
        "0E-1000026"
    );
    assert_eq!(d("0").add(&d("5")).unwrap().to_sci_string(), "5");
    assert_eq!(d("1").sub(&d("5")).unwrap().to_sci_string(), "-4");
}

#[test]
fn decimal_comparisons_and_signs() {
    assert_eq!(
        d("1").equals(&d("sNaN")).err(),
        Some(Signal::InvalidOperation)
    );
    assert_eq!(d("1").equals(&d("NaN")), Ok(false));
    assert_eq!(d("Infinity").equals(&d("Infinity")), Ok(true));
    assert_eq!(d("-Infinity").lt(&d("Infinity")), Ok(true));
    assert_eq!(d("1").lt(&d("Infinity")), Ok(true));
    assert_eq!(d("0").lt(&d("5")), Ok(true));
    assert_eq!(d("0").gt(&d("-5")), Ok(true));
    assert_eq!(d("-5").lt(&d("0")), Ok(true));
    assert_eq!(d("-5").lt(&d("5")), Ok(true));
    assert_eq!(d("-5").lt(&d("-3")), Ok(true));
    assert_eq!(d("1").le(&d("1")), Ok(true));
    assert_eq!(d("1").gt(&d("2")), Ok(false));
}

#[test]
fn cli_api_help_and_stdin_boundaries() {
    let help = run_cli(&["--api-help"], &[]);
    assert_eq!(help.status.code(), Some(0));
    let stdout = String::from_utf8(help.stdout).unwrap();
    for cmd in api::COMMANDS {
        assert!(stdout.contains(cmd));
    }

    let bad_json = pipe_api("validate-amount", b"{");
    assert_eq!(bad_json.status.code(), Some(2));
    assert!(
        String::from_utf8(bad_json.stderr)
            .unwrap()
            .contains("invalid JSON:")
    );

    let oversized = vec![b' '; (8 << 20) + 1];
    let too_big = pipe_api("validate-amount", &oversized);
    assert_eq!(too_big.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(too_big.stderr)
            .unwrap()
            .replace("\r\n", "\n"),
        "error: JSON input exceeds 8 MiB\n"
    );
}

#[test]
fn cli_broken_pipes_exit_nonzero() {
    let mut out_closed = Command::new(env!("CARGO_BIN_EXE_solomons-sword"))
        .arg("validate-amount")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(out_closed.stdout.take());
    out_closed
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"value":"10"}"#)
        .unwrap();
    let res = out_closed.wait_with_output().unwrap();
    let expected = if cfg!(unix) { 120 } else { 1 };
    assert_eq!(res.status.code(), Some(expected));

    let mut err_closed = Command::new(env!("CARGO_BIN_EXE_solomons-sword"))
        .arg("validate-amount")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(err_closed.stderr.take());
    err_closed.stdin.take().unwrap().write_all(b"{").unwrap();
    let res = err_closed.wait_with_output().unwrap();
    assert_eq!(res.status.code(), Some(expected));
}

#[test]
fn cli_colour_and_terminal_width_env_branches() {
    let no_color = run_cli(&["--help"], &[("NO_COLOR", Some("1"))]);
    assert_eq!(no_color.status.code(), Some(0));
    assert!(!String::from_utf8(no_color.stdout).unwrap().contains('\x1b'));

    let dumb = run_cli(&["--help"], &[("TERM", Some("dumb"))]);
    assert_eq!(dumb.status.code(), Some(0));
    assert!(!String::from_utf8(dumb.stdout).unwrap().contains('\x1b'));

    let py_color = run_cli(&["wrong"], &[("PYTHON_COLORS", Some("1"))]);
    assert_eq!(py_color.status.code(), Some(2));
    assert!(
        String::from_utf8(py_color.stderr)
            .unwrap()
            .contains("\x1b[")
    );

    let piped_default = run_cli(&["wrong"], &[]);
    assert_eq!(piped_default.status.code(), Some(2));
    assert!(
        !String::from_utf8(piped_default.stderr)
            .unwrap()
            .contains('\x1b')
    );

    let narrow = run_cli(
        &["s100a-check", "--help"],
        &[("COLUMNS", Some("20")), ("LINES", Some("24"))],
    );
    assert_eq!(narrow.status.code(), Some(0));
    assert!(
        String::from_utf8(narrow.stdout)
            .unwrap()
            .replace("\r\n", "\n")
            .contains("usage: solomons-sword s100a-check\n")
    );

    let zero_cols = run_cli(
        &["--help"],
        &[("COLUMNS", Some("0")), ("LINES", Some("24"))],
    );
    assert_eq!(zero_cols.status.code(), Some(0));
    assert!(
        String::from_utf8(zero_cols.stdout)
            .unwrap()
            .contains("Available subcommands")
    );
}

#[test]
fn cli_parser_edges_prefixes_negative_numbers_and_long_choices() {
    let double_dash_ok = run_cli(
        &["--", "s100a-check", "--beneficiary", "A", "--amount", "10"],
        &[],
    );
    assert_eq!(double_dash_ok.status.code(), Some(0));

    let double_dash_extra = run_cli(
        &[
            "s100a-check",
            "--beneficiary",
            "A",
            "--amount",
            "10",
            "--",
            "x",
        ],
        &[],
    );
    assert_eq!(double_dash_extra.status.code(), Some(2));
    assert!(
        String::from_utf8(double_dash_extra.stderr)
            .unwrap()
            .contains("unrecognized arguments: -- x")
    );

    let ambiguous = run_cli(&["s100a-check", "--c"], &[]);
    assert_eq!(ambiguous.status.code(), Some(2));
    assert!(
        String::from_utf8(ambiguous.stderr)
            .unwrap()
            .contains("ambiguous option: --c could match")
    );

    let prefix_eq = run_cli(&["s100a-check", "--bene=Alice", "--amount=-5"], &[]);
    assert_eq!(prefix_eq.status.code(), Some(2));
    assert!(
        String::from_utf8(prefix_eq.stderr)
            .unwrap()
            .contains("distribution amount must be positive")
    );
}

#[test]
fn cli_help_ignored_arg_and_long_unknown_subcommand() {
    let help_eq = run_cli(&["--help=foo"], &[]);
    assert_eq!(help_eq.status.code(), Some(2));
    assert!(
        String::from_utf8(help_eq.stderr)
            .unwrap()
            .contains("ignored explicit argument 'foo'")
    );

    let short_hx = run_cli(&["-hx"], &[]);
    assert_eq!(short_hx.status.code(), Some(0));
    assert!(
        String::from_utf8(short_hx.stdout)
            .unwrap()
            .contains("Available subcommands")
    );

    let pair_eq = run_cli(
        &[
            "s100a-check",
            "--beneficiary=A",
            "--amount=10",
            "--circular=1",
        ],
        &[],
    );
    assert_eq!(pair_eq.status.code(), Some(2));
    assert!(
        String::from_utf8(pair_eq.stderr)
            .unwrap()
            .contains("argument --circular: ignored explicit argument '1'")
    );

    let long_unknown = format!("s100a-check{}", "z".repeat(200));
    let out = run_cli(&[&long_unknown], &[]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("invalid choice:")
    );
}

#[cfg(unix)]
#[test]
fn cli_unix_non_utf8_arg_is_backslash_escaped() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let bad = OsStr::from_bytes(b"bad\xff");
    let out = Command::new(env!("CARGO_BIN_EXE_solomons-sword"))
        .arg(bad)
        .env("PYTHON_COLORS", "0")
        .env("COLUMNS", "80")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("\\udcff"));
}

#[cfg(windows)]
#[test]
fn cli_windows_unpaired_surrogate_arg_is_backslash_escaped() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    let bad = OsString::from_wide(&[b'x' as u16, 0xD800]);
    let out = Command::new(env!("CARGO_BIN_EXE_solomons-sword"))
        .arg(bad)
        .env("PYTHON_COLORS", "0")
        .env("COLUMNS", "80")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("\\ud800"));
}
