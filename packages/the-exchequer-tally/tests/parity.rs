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
fn appended_entries_outside_the_year_are_refused() {
    let input = serde_json::json!({"financial_year":2027,"opening_balance":"5000","append_entries":[{"entry_date":"2027-07-01","entry_type":"FRANKED_DIST_PAID","amount":"100","description":"Synthetic"}]});
    assert_eq!(
        api::evaluate("franking-account", &input).unwrap_err(),
        "entry_date 2027-07-01 is outside FY2027 (2026-07-01 to 2027-06-30)"
    );
}
#[test]
fn direct_entries_preserve_ledger_invariants() {
    let entry = |entry_type: &str, amount: &str| {
        let input = serde_json::json!({"financial_year":2027,"entries":[{"entry_date":"2026-09-01","entry_type":entry_type,"amount":amount,"description":"Synthetic entry"}]});
        api::evaluate("franking-account", &input).unwrap_err()
    };
    for entry_type in ["TAX_REFUND", "PAYG_INSTALMENT"] {
        for amount in ["NaN", "Infinity", "-1"] {
            assert_eq!(
                entry(entry_type, amount),
                "amount must be a non-negative finite Decimal",
                "{entry_type} {amount}"
            );
        }
    }
    assert_eq!(
        entry("unrecognised", "1"),
        "entry_type must be a FrankingEntryType"
    );
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
fn negative_number_prefix_values_reach_the_type_check() {
    // Final stderr lines recorded from Python 0.2.0 on CPython 3.15: argparse's
    // `-\.?\d` matcher is a prefix match, so these are values, not options.
    for (args, expected) in [
        (
            ["dist-statement", "--amount", "-1\n"],
            "the-exchequer-tally dist-statement: error: argument --amount: money must be between 0 and 1000000000000.00 AUD: '-1\\n'",
        ),
        (
            ["dist-statement", "--amount", "-1x"],
            "the-exchequer-tally dist-statement: error: argument --amount: not a decimal amount: '-1x'",
        ),
        (
            ["dist-statement", "--amount", "-.5x"],
            "the-exchequer-tally dist-statement: error: argument --amount: not a decimal amount: '-.5x'",
        ),
        (
            ["bre-test", "--turnover", "-1x"],
            "the-exchequer-tally bre-test: error: argument --turnover: not a decimal amount: '-1x'",
        ),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_the-exchequer-tally"))
            .args(args)
            .env("PYTHON_COLORS", "0")
            .env_remove("FORCE_COLOR")
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        let last = stderr.trim_end().rsplit('\n').next().unwrap();
        assert_eq!(last.trim_end_matches('\r'), expected, "{args:?}");
    }
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
    let refusal = |input: Value| api::evaluate("corporate-tax", &input).unwrap_err();
    assert_eq!(refusal(Value::Null), "input must be a JSON object");
    assert_eq!(
        refusal(serde_json::json!({"financial_year":"2027"})),
        "financial_year must be an integer"
    );
    assert_eq!(
        refusal(serde_json::json!({"financial_year":2027,"aggregated_turnover":"1e10001"})),
        "aggregated_turnover: decimal exponent or adjusted exponent exceeds 10000"
    );
    assert_eq!(
        refusal(serde_json::json!({"financial_year":2027,"aggregated_turnover":"1".repeat(8193)})),
        "aggregated_turnover: decimal input exceeds 8192 bytes"
    );
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
fn api_process(command: &str, input: &Value) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_the-exchequer-tally"))
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let bytes = serde_json::to_vec(input).unwrap();
    child.stdin.take().unwrap().write_all(&bytes).unwrap();
    child.wait_with_output().unwrap()
}
fn native_line(text: &str) -> Vec<u8> {
    format!("{text}{}", if cfg!(windows) { "\r\n" } else { "\n" }).into_bytes()
}
fn statement_input() -> Value {
    serde_json::json!({"entity_name":"A","abn_or_acn":"1","recipient_name":"B","payment_date":"2026-10-18","total_distribution":"100.00","franking_percentage":"100.00","corporate_tax_rate":"0.25"})
}
#[test]
fn statement_refuses_unexpected_fields_as_python_keywords_did() {
    let refused = |key: &str, extra: Value| {
        let mut input = statement_input();
        for (k, v) in extra.as_object().unwrap() {
            input[k] = v.clone();
        }
        let out = api_process("statement", &input);
        assert_eq!(out.status.code(), Some(2), "{key}");
        assert!(out.stdout.is_empty(), "{key}");
        assert_eq!(
            out.stderr,
            native_line(&format!(
                "error: generate_distribution_statement() got an unexpected keyword argument '{key}'"
            ))
        );
    };
    refused(
        "withholding_tax_deducted",
        serde_json::json!({"withholding_tax_deducted":"12.00"}),
    );
    // Python refuses the keyword before the function body validates any amount.
    refused(
        "withholding_tax_deducted",
        serde_json::json!({"withholding_tax_deducted":"12.00","total_distribution":"0"}),
    );
    // The key is reported as given, plane-16 private-use pairs included.
    refused(
        "\u{10F7FF}\u{10F7FF}",
        serde_json::json!({"\u{10F7FF}\u{10F7FF}":"1"}),
    );
    // Python's other API functions took objects whose extra keys the adapter never passed on.
    let mut tax = serde_json::json!({"financial_year":2027,"aggregated_turnover":"1","assessable_income":"1","passive_income":"0","passive_incme":"1"});
    assert_eq!(api_process("corporate-tax", &tax).status.code(), Some(0));
    tax["financial_year"] = Value::Null;
    assert_eq!(api_process("corporate-tax", &tax).status.code(), Some(2));
}
#[test]
fn api_output_preserves_plane_16_private_use_text() {
    let mut input = statement_input();
    input["entity_name"] = "\u{10F7FF}\u{10F7FF}".into();
    input["recipient_name"] = "\u{10F7FF}\u{10F800}".into();
    let out = api_process("statement", &input);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let text = String::from_utf8(out.stdout).unwrap();
    let body = text.strip_suffix(if cfg!(windows) { "\r\n" } else { "\n" });
    let result: Value = serde_json::from_str(body.unwrap()).unwrap();
    assert_eq!(result["entity_name"], "\u{10F7FF}\u{10F7FF}");
    assert_eq!(result["recipient_name"], "\u{10F7FF}\u{10F800}");

    let ledger = serde_json::json!({"financial_year":2027,"opening_balance":"0","entries":[{"entry_date":"2026-07-01","entry_type":"PAYG_INSTALMENT","amount":"1","description":"\u{10F7FF}\u{10F7FF}"}]});
    let out = api_process("franking-account", &ledger);
    assert_eq!(out.status.code(), Some(0));
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(result["entries"][0]["description"], "\u{10F7FF}\u{10F7FF}");
}
