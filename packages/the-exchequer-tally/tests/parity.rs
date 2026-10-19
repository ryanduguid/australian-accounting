use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};
use the_exchequer_tally::{
    api,
    date::Date,
    engine::{self, Account, Entry, EntryType},
};

fn replay(commands: &[&str]) {
    let rows: Value = serde_json::from_str(include_str!("data/python-api.json")).unwrap();
    let mut n = 0;
    for row in rows.as_array().unwrap() {
        let command = row["command"].as_str().unwrap();
        if !commands.contains(&command) {
            continue;
        }
        let actual = match api::evaluate(command, &row["input"]) {
            Ok(v) => serde_json::json!({"ok":v}),
            Err(e) => serde_json::json!({"error":e}),
        };
        assert_eq!(
            actual, row["expected"],
            "{}: {} {}",
            row["test"], command, row["input"]
        );
        n += 1;
    }
    assert!(n > 0);
}
#[test]
fn corporate_tax_and_unknowns() {
    replay(&["corporate-tax", "bre-properties", "turnover-threshold"]);
}
#[test]
fn maximum_franking_rate_uses_prior_facts_current_scale() {
    replay(&["max-franking-rate"]);
}
#[test]
fn statements_validate_and_round_down_credit() {
    replay(&["statement"]);
}
#[test]
fn event_measurements_require_rate_even_for_nil_distribution() {
    replay(&["distribution-event"]);
}
#[test]
fn benchmark_uses_exact_ratio_stable_date_order_and_cent_tolerance() {
    replay(&["benchmark"]);
}
#[test]
fn ledger_and_fdt_unknowns_classification_and_periods() {
    replay(&["franking-account"]);
}
#[test]
fn decimal_and_date_validators() {
    replay(&["validate-value"]);
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn native_bytes(text: &str) -> Vec<u8> {
    let bytes = unhex(text);
    if cfg!(windows) {
        bytes
    } else {
        String::from_utf8(bytes)
            .unwrap()
            .replace("\r\n", "\n")
            .into_bytes()
    }
}
#[test]
fn original_cli_exit_stdout_stderr_bytes() {
    let rows: Value = serde_json::from_str(include_str!("data/python-cli.json")).unwrap();
    for row in rows.as_array().unwrap() {
        let args: Vec<_> = row["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a.as_str().unwrap())
            .collect();
        let run = Command::new(env!("CARGO_BIN_EXE_the-exchequer-tally"))
            .args(&args)
            .env("PYTHON_COLORS", "0")
            .env("COLUMNS", "80")
            .env_remove("FORCE_COLOR")
            .output()
            .unwrap();
        assert_eq!(
            run.status.code().unwrap(),
            row["expected"]["code"].as_i64().unwrap() as i32,
            "{args:?}"
        );
        assert_eq!(
            run.stdout,
            native_bytes(row["expected"]["stdout"].as_str().unwrap()),
            "stdout: {args:?}"
        );
        assert_eq!(
            run.stderr,
            native_bytes(row["expected"]["stderr"].as_str().unwrap()),
            "stderr: {args:?}"
        );
    }
}
#[test]
fn refused_records_do_not_post_and_mutated_entries_are_revalidated() {
    let mut a = Account {
        fy: 2027,
        opening: Some(engine::d("5000")),
        entries: vec![],
    };
    for classification in [None, Some(true)] {
        assert!(
            a.record(
                Date::parse("2027-03-01").unwrap(),
                EntryType::Refund,
                engine::d("2000"),
                None,
                classification
            )
            .unwrap_err()
            .contains("R&D")
        );
        assert!(a.entries.is_empty());
        assert_eq!(a.closing().unwrap().unwrap().to_string(), "5000.00");
    }
    a.record(
        Date::parse("2027-03-01").unwrap(),
        EntryType::Refund,
        engine::d("2000"),
        Some("Fabricated ordinary refund".into()),
        Some(false),
    )
    .unwrap();
    assert_eq!(a.entries.len(), 1);
    assert_eq!(a.entries[0].description, "Fabricated ordinary refund");
    assert!(
        a.record(
            Date::parse("2027-03-01").unwrap(),
            EntryType::Refund,
            engine::d("-1"),
            None,
            Some(false)
        )
        .is_err()
    );
    assert_eq!(a.entries.len(), 1);
    assert_eq!(a.closing().unwrap().unwrap().to_string(), "3000.00");
    a.entries.push(
        Entry::new(
            Date::parse("2027-07-01").unwrap(),
            EntryType::DistributionPaid,
            engine::d("100"),
            "Synthetic".into(),
            Some(String::new()),
        )
        .unwrap(),
    );
    assert!(a.totals().unwrap_err().contains("outside FY2027"));
    assert!(a.closing().unwrap_err().contains("outside FY2027"));
    assert!(a.deficit().unwrap_err().contains("outside FY2027"));
    a.opening = None;
    assert!(a.closing().unwrap_err().contains("outside FY2027"));
}

#[test]
fn direct_entries_preserve_ledger_invariants() {
    for kind in [EntryType::Refund, EntryType::Payg] {
        for amount in ["NaN", "Infinity", "-1"] {
            assert!(
                Entry::new(
                    Date::parse("2026-09-01").unwrap(),
                    kind,
                    engine::d(amount),
                    "Synthetic entry".into(),
                    Some(String::new())
                )
                .is_err()
            );
        }
    }
    assert!(EntryType::parse("unrecognised").is_err());
}
#[test]
fn forced_colour_error_line_matches_python_3_15() {
    // Final stderr line recorded from Python 0.1.8 on CPython 3.15 with FORCE_COLOR=1.
    let out = Command::new(env!("CARGO_BIN_EXE_the-exchequer-tally"))
        .arg("nope")
        .env_remove("NO_COLOR")
        .env_remove("PYTHON_COLORS")
        .env("FORCE_COLOR", "1")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8(out.stderr).unwrap();
    let last = stderr.trim_end().rsplit('\n').next().unwrap();
    assert_eq!(
        last,
        "the-exchequer-tally: \x1b[1;35merror:\x1b[0m \x1b[35margument command: invalid choice: 'nope' (choose from 'bre-test', 'dist-statement')\x1b[0m"
    );
}
#[test]
fn ledger_years_dates_and_null_references_follow_python() {
    // Expected text recorded from Python 0.1.8 FrankingAccount(fy) on CPython 3.15.
    for (fy, expected) in [
        (0, "year must be in 1..9999, not -1"),
        (-5, "year must be in 1..9999, not -6"),
        (10000, "year must be in 1..9999, not 10000"),
        (2147483648, "year must be in 1..9999, not 2147483647"),
        (-2147483647, "year must be in 1..9999, not -2147483648"),
        (-2147483648, "Python int too large to convert to C int"),
        (i64::MAX, "Python int too large to convert to C int"),
    ] {
        let input = serde_json::json!({ "financial_year": fy });
        assert_eq!(
            api::evaluate("franking-account", &input).unwrap_err(),
            expected,
            "FY{fy}"
        );
    }
    let entry = |reference: Value| {
        let mut e = serde_json::json!({"entry_date":"2026-07-01","entry_type":"PAYG_INSTALMENT","amount":"1","description":"x"});
        if reference != "omit" {
            e["statutory_reference"] = reference;
        }
        let out = api::evaluate(
            "franking-account",
            &serde_json::json!({"financial_year":2027,"opening_balance":"0","entries":[e],"append_entries":[e]}),
        )
        .unwrap();
        let refs: Vec<Value> = out["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["statutory_reference"].clone())
            .collect();
        refs
    };
    assert_eq!(entry(Value::Null), vec![Value::Null, Value::Null]);
    assert_eq!(entry("omit".into()), vec![Value::from(""), Value::from("")]);
    assert_eq!(
        entry("s 1".into()),
        vec![Value::from("s 1"), Value::from("s 1")]
    );
    let event = serde_json::json!({"event_date":"20261018","recipient_name":"B","distribution_amount":"1","franking_credit":"0"});
    assert_eq!(
        api::evaluate("distribution-event", &event).unwrap_err(),
        "event_date must be YYYY-MM-DD"
    );
}
#[test]
fn json_boundary_rejects_wrong_types_and_excessive_numbers() {
    assert!(api::evaluate("corporate-tax", &Value::Null).is_err());
    assert!(
        api::evaluate(
            "corporate-tax",
            &serde_json::json!({"financial_year":"2027"})
        )
        .is_err()
    );
    assert!(api::decimal(&serde_json::json!({"x":"1e10001"}), "x").is_err());
    assert!(api::decimal(&serde_json::json!({"x":"1".repeat(8193)}), "x").is_err());
    let extreme = serde_json::json!({"current_fy":i64::MIN,"prior_year_test":{"financial_year":2026,"aggregated_turnover":"0","assessable_income":"1","passive_income":"0"}});
    assert_eq!(
        api::evaluate("max-franking-rate", &extreme).unwrap_err(),
        "prior_year_test must be for FY-9223372036854775809, got FY2026"
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_the-exchequer-tally"))
        .arg("statement")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"{}").unwrap();
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("entity_name must be a string")
    );
}
