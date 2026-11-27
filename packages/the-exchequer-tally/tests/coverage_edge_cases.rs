// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Output, Stdio};
use the_exchequer_tally::{
    api, cli,
    date::Date,
    engine::{self, Account, EntryType},
};

const BIN: &str = env!("CARGO_BIN_EXE_the-exchequer-tally");

fn run_cli(args: &[&str], envs: &[(&str, Option<&str>)]) -> Output {
    let mut cmd = Command::new(BIN);
    cmd.args(args)
        .env_remove("PYTHON_COLORS")
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("TERM")
        .env("COLUMNS", "80")
        .env_remove("LINES");
    for (k, v) in envs {
        match v {
            Some(val) => {
                cmd.env(k, val);
            }
            None => {
                cmd.env_remove(k);
            }
        }
    }
    cmd.output().unwrap()
}

fn run_api_bytes(command: &str, stdin_bytes: &[u8]) -> Output {
    let mut child = Command::new(BIN)
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin_bytes).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn company_tax_and_carry_back_input_validation_errors() {
    let mut v = json!({
        "financial_year": 2021, "aggregated_turnover": "60000000",
        "assessable_income": "100000", "passive_income": "0", "deductions": "1000000"
    });
    v["consolidated_group"] = json!("yes");
    assert_eq!(
        api::evaluate("company-tax-statement", &v).unwrap_err(),
        "consolidated_group must be a boolean"
    );
    v["consolidated_group"] = json!(false);
    v["loss_carry_back"] = json!("not-an-object");
    assert_eq!(
        api::evaluate("company-tax-statement", &v).unwrap_err(),
        "loss_carry_back must be an object"
    );
    let cond = json!({
        "corporate_tax_entity_throughout": true, "returns_lodged": true,
        "choice_in_approved_form": true, "carried_on_business_each_loss_year": true,
        "no_transferred_losses": true, "no_excess_franking_offset_losses": true,
        "no_integrity_scheme": true
    });
    v["loss_carry_back"] = json!({
        "franking_account_balance": "100000", "conditions": cond,
        "carry_backs": [{"loss_year": 2020, "earlier_year": 2019, "amount": "1000"}],
        "earlier_years": [{"year": 2019, "income_tax_liability": "10000", "liability_previously_used": "0"}]
    });
    assert!(
        api::evaluate("company-tax-statement", &v)
            .unwrap_err()
            .contains("loss_years has no entry")
    );
}

#[test]
fn carry_back_duplicate_and_missing_year_errors() {
    let cond = json!({
        "corporate_tax_entity_throughout": true, "returns_lodged": true,
        "choice_in_approved_form": true, "carried_on_business_each_loss_year": true,
        "no_transferred_losses": true, "no_excess_franking_offset_losses": true,
        "no_integrity_scheme": true
    });
    let base = |lcb: Value| {
        json!({
            "financial_year": 2021, "aggregated_turnover": "60000000",
            "assessable_income": "100000", "passive_income": "0", "deductions": "1000000",
            "loss_carry_back": lcb
        })
    };
    let dup_loss = base(json!({
        "franking_account_balance": "100000", "conditions": cond,
        "loss_years": [{"year": 2021, "tax_loss": "100", "aggregated_turnover": "1", "assessable_income": "1", "passive_income": "0"}],
        "carry_backs": [], "earlier_years": []
    }));
    assert!(
        api::evaluate("company-tax-statement", &dup_loss)
            .unwrap_err()
            .contains("lists the claim year")
    );
    let missing_earlier = base(json!({
        "franking_account_balance": "100000", "conditions": cond,
        "carry_backs": [{"loss_year": 2021, "earlier_year": 2019, "amount": "100"}],
        "earlier_years": []
    }));
    assert!(
        api::evaluate("company-tax-statement", &missing_earlier)
            .unwrap_err()
            .contains("earlier_years has no entry")
    );
    let dup_earlier = base(json!({
        "franking_account_balance": "100000", "conditions": cond,
        "carry_backs": [{"loss_year": 2021, "earlier_year": 2019, "amount": "100"}],
        "earlier_years": [
            {"year": 2019, "income_tax_liability": "1000", "liability_previously_used": "0"},
            {"year": 2019, "income_tax_liability": "1000", "liability_previously_used": "0"}
        ]
    }));
    assert_eq!(
        api::evaluate("company-tax-statement", &dup_earlier).unwrap_err(),
        "earlier_years lists a year twice"
    );
}

#[test]
fn api_validate_value_dates_and_kinds() {
    assert_eq!(
        api::evaluate("unknown-cmd", &json!({})).unwrap_err(),
        "unknown API command: unknown-cmd"
    );
    assert_eq!(
        api::evaluate("validate-value", &json!({"kind": "other", "value": "1"})).unwrap_err(),
        "kind must be money, percentage, rate or date"
    );
    for (val, ok) in [
        ("2024-02-29", true),
        ("2023-02-29", false),
        ("1900-02-29", false),
        ("2000-02-29", true),
        ("2026-13-01", false),
        ("0000-01-01", false),
        ("2026-04-31", false),
    ] {
        let res = api::evaluate("validate-value", &json!({"kind": "date", "value": val}));
        if ok {
            assert_eq!(res.unwrap(), json!(val));
        } else {
            assert!(res.is_err());
        }
    }
}

#[test]
fn api_array_and_refund_classification_errors() {
    assert_eq!(
        api::evaluate(
            "benchmark",
            &json!({"corporate_tax_rate": "0.25", "distributions": 1})
        )
        .unwrap_err(),
        "distributions must be an array"
    );
    assert_eq!(
        api::evaluate(
            "franking-account",
            &json!({"financial_year": 2027, "records": [{"entry_date": "2026-09-01", "entry_type": "TAX_REFUND", "amount": "1", "includes_r_and_d_offset": "no"}]})
        )
        .unwrap_err(),
        "includes_r_and_d_offset must be a boolean"
    );
}

#[test]
fn engine_account_opening_and_record_validation() {
    let err = api::evaluate(
        "franking-account",
        &json!({"financial_year": 2027, "opening_balance": "NaN"}),
    )
    .unwrap_err();
    assert!(err.contains("opening_balance must be a finite amount"));

    let mut a = Account {
        fy: 2027,
        opening: Some(engine::d("0")),
        entries: vec![],
    };
    let err = a
        .record(
            Date::parse("2026-09-01").unwrap(),
            EntryType::Payg,
            engine::d("0"),
            None,
            None,
        )
        .unwrap_err();
    assert!(err.contains("must be a positive finite amount"));
}

#[test]
fn main_api_help_invalid_json_and_oversized_input() {
    let help = run_cli(&["--api-help"], &[]);
    assert_eq!(help.status.code(), Some(0));
    let stdout = String::from_utf8(help.stdout).unwrap();
    assert!(stdout.contains("JSON stdin commands:"));

    let bad_json = run_api_bytes("corporate-tax", b"{");
    assert_eq!(bad_json.status.code(), Some(2));
    assert!(
        String::from_utf8(bad_json.stderr)
            .unwrap()
            .contains("invalid JSON")
    );

    let big = vec![b' '; (8 << 20) + 1];
    let too_big = run_api_bytes("corporate-tax", &big);
    assert_eq!(too_big.status.code(), Some(2));
    assert!(
        String::from_utf8(too_big.stderr)
            .unwrap()
            .contains("JSON input exceeds 8 MiB")
    );
}

#[test]
fn argtheme_env_controls_colour_and_plain_output() {
    let coloured = run_cli(&["nope"], &[("PYTHON_COLORS", Some("1"))]);
    assert_eq!(coloured.status.code(), Some(2));
    assert!(
        String::from_utf8(coloured.stderr)
            .unwrap()
            .contains("\x1b[1;35m")
    );

    let no_color = run_cli(
        &["nope"],
        &[("NO_COLOR", Some("1")), ("FORCE_COLOR", Some("1"))],
    );
    assert_eq!(no_color.status.code(), Some(2));
    assert!(
        !String::from_utf8(no_color.stderr)
            .unwrap()
            .contains("\x1b[")
    );

    let dumb = run_cli(&["nope"], &[("TERM", Some("dumb"))]);
    assert_eq!(dumb.status.code(), Some(2));
    assert!(!String::from_utf8(dumb.stderr).unwrap().contains("\x1b["));

    let piped = run_cli(&["nope"], &[]);
    assert_eq!(piped.status.code(), Some(2));
    assert!(!String::from_utf8(piped.stderr).unwrap().contains("\x1b["));
}

#[test]
fn cli_narrow_terminal_wrapping_and_lines_env() {
    for cols in ["1", "20", "30", "40"] {
        for args in [
            &["--help"][..],
            &["bre-test", "--help"][..],
            &["dist-statement", "--help"][..],
        ] {
            let out = run_cli(args, &[("COLUMNS", Some(cols)), ("LINES", Some("24"))]);
            assert_eq!(out.status.code(), Some(0));
            assert!(String::from_utf8(out.stdout).unwrap().contains("usage:"));
        }
    }
}

#[test]
fn cli_close_match_and_option_parsing_edges() {
    let empty_cmd = run_cli(&[""], &[("PYTHON_COLORS", Some("0"))]);
    assert_eq!(empty_cmd.status.code(), Some(2));

    let long_cmd = format!("{}bre-test", "a".repeat(205));
    let long_out = run_cli(&[&long_cmd], &[("PYTHON_COLORS", Some("0"))]);
    assert_eq!(long_out.status.code(), Some(2));
    assert!(
        String::from_utf8(long_out.stderr)
            .unwrap()
            .contains("invalid choice")
    );

    let hx = run_cli(&["bre-test", "-hx"], &[("PYTHON_COLORS", Some("0"))]);
    assert_eq!(hx.status.code(), Some(0));
    assert!(String::from_utf8(hx.stdout).unwrap().contains("usage:"));

    for args in [
        &["--help=foo"][..],
        &["-h-"][..],
        &["bre-test", "--help=foo"][..],
    ] {
        let out = run_cli(args, &[("PYTHON_COLORS", Some("0"))]);
        assert_eq!(out.status.code(), Some(2));
        assert!(
            String::from_utf8(out.stderr)
                .unwrap()
                .contains("ignored explicit argument")
        );
    }
}

#[test]
fn cli_equals_and_abbreviated_options_succeed() {
    let ok = run_cli(
        &[
            "--",
            "bre-test",
            "--fy=+2025",
            "--turn=60000000",
            "--assessable",
            "0",
            "--passive",
            "0",
        ],
        &[("PYTHON_COLORS", Some("0"))],
    );
    assert_eq!(ok.status.code(), Some(0));
    let stdout = String::from_utf8(ok.stdout).unwrap();
    assert!(stdout.contains("n/a (no assessable income)"));
    assert!(stdout.contains("<= 80%: None"));
}

#[test]
fn cli_double_dash_single_dash_and_spaced_extras_fail() {
    let extra_after = run_cli(
        &[
            "bre-test",
            "--fy",
            "2025",
            "--turnover",
            "1",
            "--assessable",
            "1",
            "--passive",
            "0",
            "--",
            "extra",
        ],
        &[("PYTHON_COLORS", Some("0"))],
    );
    assert_eq!(extra_after.status.code(), Some(2));
    assert!(
        String::from_utf8(extra_after.stderr)
            .unwrap()
            .contains("unrecognized arguments")
    );

    let dash = run_cli(
        &[
            "bre-test",
            "-",
            "-foo bar",
            "--fy",
            "2025",
            "--turnover",
            "1",
            "--assessable",
            "1",
            "--passive",
            "0",
        ],
        &[("PYTHON_COLORS", Some("0"))],
    );
    assert_eq!(dash.status.code(), Some(2));
    assert!(
        String::from_utf8(dash.stderr)
            .unwrap()
            .contains("unrecognized arguments: - -foo bar")
    );
}

#[test]
fn cli_option_validation_errors() {
    let missing = run_cli(&["bre-test", "--fy"], &[("PYTHON_COLORS", Some("0"))]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        String::from_utf8(missing.stderr)
            .unwrap()
            .contains("expected one argument")
    );

    let big_int = "9".repeat(4301);
    for args in [
        vec!["bre-test", "--fy", &big_int],
        vec!["bre-test", "--turnover", "NaN"],
        vec!["bre-test", "--turnover", "1000000000001"],
        vec!["bre-test", "--turnover", "1.001"],
        vec!["dist-statement", "--franking-pct", "101"],
        vec!["dist-statement", "--tax-rate", "1.5"],
    ] {
        let out = run_cli(&args, &[("PYTHON_COLORS", Some("0"))]);
        assert_eq!(out.status.code(), Some(2));
    }
}

#[test]
fn cli_dispatch_errors_and_decimal_overflow_exit_1() {
    let bad_fy = run_cli(
        &[
            "bre-test",
            "--fy",
            "2030",
            "--turnover",
            "1",
            "--assessable",
            "1",
            "--passive",
            "0",
        ],
        &[("PYTHON_COLORS", Some("0"))],
    );
    assert_eq!(bad_fy.status.code(), Some(2));
    assert!(String::from_utf8(bad_fy.stderr).unwrap().contains("FY2030"));

    let bad_stmt = run_cli(
        &[
            "dist-statement",
            "--entity",
            "A",
            "--acn",
            "1",
            "--recipient",
            "B",
            "--payment-date",
            "2026-09-01",
            "--amount",
            "0",
            "--tax-rate",
            "0.25",
        ],
        &[("PYTHON_COLORS", Some("0"))],
    );
    assert_eq!(bad_stmt.status.code(), Some(2));

    let dec_err = run_cli(
        &[
            "dist-statement",
            "--entity",
            "A",
            "--acn",
            "1",
            "--recipient",
            "B",
            "--payment-date",
            "2026-09-01",
            "--amount",
            "1000000000000",
            "--tax-rate",
            "0.99999999999999999999999999",
        ],
        &[("PYTHON_COLORS", Some("0"))],
    );
    assert_eq!(dec_err.status.code(), Some(1));
    assert!(
        String::from_utf8(dec_err.stderr)
            .unwrap()
            .contains("decimal.InvalidOperation")
    );
}

#[test]
fn cli_decode_os_and_output_bytes_handle_surrogates() {
    let raw_pair = "\u{10F7FF}\u{10F880}";
    assert_eq!(cli::output_bytes(raw_pair, false), vec![0x80]);
    assert_eq!(cli::output_bytes(raw_pair, true), b"\\ud880");

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let os = std::ffi::OsStr::from_bytes(b"a\xffb");
        let decoded = cli::decode_os(os);
        assert_eq!(cli::output_bytes(&decoded, false), b"a\xffb");
        assert_eq!(cli::output_bytes(&decoded, true), b"a\\udcffb");
    }
}

#[cfg(unix)]
#[test]
fn unix_pty_and_broken_pipe_paths() {
    let mut child = Command::new(BIN)
        .arg("--api-help")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(1));
}
