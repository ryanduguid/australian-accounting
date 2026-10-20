use serde_json::{Value, json};
use solomons_sword::api;
use solomons_sword::decimal::Decimal;
use std::io::Write;
use std::process::{Command, Stdio};

/// FNV-1a 64 of LF-normalised bytes, the fingerprint the corpus stores for
/// outputs over 64 KiB (million-digit amounts).
fn fnv(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h = (h ^ u64::from(*b)).wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

fn matches(actual: &[u8], expected: &Value) -> bool {
    match expected {
        Value::Object(o) => {
            o["len"].as_u64() == Some(actual.len() as u64) && o["fnv"] == json!(fnv(actual))
        }
        _ => expected == &json!(actual),
    }
}

fn replay(commands: &[&str]) {
    let rows: Value = serde_json::from_str(include_str!("data/python-api.json")).unwrap();
    let mut n = 0;
    for row in rows.as_array().unwrap() {
        let command = row["command"].as_str().unwrap();
        if !commands.contains(&command) {
            continue;
        }
        let actual = match api::evaluate(command, &row["input"]) {
            Ok(v) => json!({"ok": v}),
            Err(e) => json!({"error": e}),
        };
        let expected = &row["expected"];
        if expected.get("fnv").is_some() {
            let text = serde_json::to_string(&actual).unwrap();
            assert!(
                matches(text.as_bytes(), expected),
                "{}: {command}",
                row["test"]
            );
        } else {
            assert_eq!(
                &actual, expected,
                "{}: {command} {}",
                row["test"], row["input"]
            );
        }
        n += 1;
    }
    assert!(n > 0);
}

#[test]
fn division_6_allocation_refusals_rounding_and_footing() {
    replay(&["proportionate-share"]);
}

#[test]
fn section_100a_zoning_and_unestablished_facts() {
    replay(&["s100a-risk"]);
}

#[test]
fn section_99b_exemptions_caveats_and_range_checks() {
    replay(&["s99b-liability"]);
}

#[test]
fn trust_resolution_deadlines_and_unestablished_facts() {
    replay(&["trust-resolution"]);
}

#[test]
fn decimal_type_refuses_what_is_not_a_finite_decimal() {
    replay(&["validate-amount"]);
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn lf(bytes: &[u8]) -> Vec<u8> {
    String::from_utf8(bytes.to_vec())
        .unwrap()
        .replace("\r\n", "\n")
        .into_bytes()
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
        let mut command = Command::new(env!("CARGO_BIN_EXE_solomons-sword"));
        command
            .args(&args)
            .env("PYTHON_COLORS", "0")
            .env("COLUMNS", "80")
            .env_remove("FORCE_COLOR")
            .env_remove("NO_COLOR");
        for (key, value) in row["env"].as_object().unwrap() {
            if key == "FORCE_COLOR" {
                command.env_remove("PYTHON_COLORS");
            }
            command.env(key, value.as_str().unwrap());
        }
        let run = command.output().unwrap();
        let expected = &row["expected"];
        assert_eq!(
            run.status.code().unwrap(),
            expected["code"].as_i64().unwrap() as i32,
            "{args:?}"
        );
        // Windows rows carry CRLF; compare text-mode bytes LF-normalised.
        for (stream, actual) in [("stdout", &run.stdout), ("stderr", &run.stderr)] {
            match &expected[stream] {
                Value::String(hex) => {
                    assert_eq!(lf(actual), lf(&unhex(hex)), "{stream}: {args:?}")
                }
                digest => assert!(matches(&lf(actual), digest), "{stream}: {args:?}"),
            }
        }
        if cfg!(windows) {
            assert!(
                !run.stdout
                    .windows(2)
                    .any(|w| w[1] == b'\n' && w[0] != b'\r')
            );
        }
    }
}

/// `percentages_from` in the retired property test: 100.00 split across the
/// weights at two places, largest remainder first.
fn percentages_from(weights: &[u64]) -> Vec<String> {
    let total: u64 = weights.iter().sum();
    let mut floored: Vec<u64> = weights.iter().map(|w| w * 10_000 / total).collect();
    let remainder = |i: usize| weights[i] * 10_000 % total;
    let shortfall = 10_000 - floored.iter().sum::<u64>();
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(remainder(i)));
    for &i in order.iter().take(shortfall as usize) {
        floored[i] += 1;
    }
    floored
        .iter()
        .map(|p| format!("{}.{:02}", p / 100, p % 100))
        .collect()
}

fn sum(values: &[Value], key: &str) -> Decimal {
    values
        .iter()
        .fold(Decimal::parse("0.00").unwrap(), |acc, v| {
            acc.add(&Decimal::parse(v[key].as_str().unwrap()).unwrap())
                .unwrap()
        })
}

#[test]
fn every_allocated_column_foots_to_its_pool() {
    // The fast-check property's domain, from a fixed xorshift seed.
    let mut state: u64 = 0x5010;
    let mut next = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    let cents = |c: u64| format!("{}.{:02}", c / 100, c % 100);
    let mut runs = 0;
    while runs < 500 {
        let n = 2 + next(7) as usize;
        let weights: Vec<u64> = (0..n).map(|_| 1 + next(10_000)).collect();
        let percentages = percentages_from(&weights);
        if percentages.iter().any(|p| p == "0.00") {
            continue;
        }
        let (income, s95, credits) = (
            cents(1 + next(10_000_000_000)),
            cents(1 + next(10_000_000_000)),
            cents(next(10_000_000_001)),
        );
        let beneficiaries: Vec<Value> = percentages
            .iter()
            .enumerate()
            .map(|(i, p)| json!({"beneficiary_name": format!("B{i}"), "is_resident": true, "is_under_legal_disability": false, "percentage_entitlement": p}))
            .collect();
        let input = json!({"trust_accounting_income": income, "section95_net_taxable_income": s95, "franking_credits": credits, "beneficiaries": beneficiaries});
        let shares = api::evaluate("proportionate-share", &input).unwrap();
        let shares = shares.as_array().unwrap();
        for (key, pool) in [
            ("trust_income_entitlement", &income),
            ("section95_net_income_share", &s95),
            ("franking_credit_grossup", &credits),
        ] {
            let pool = Decimal::parse(pool).unwrap();
            assert!(sum(shares, key).equals(&pool).unwrap(), "{key}: {input}");
            for s in shares {
                assert!(
                    !Decimal::parse(s[key].as_str().unwrap())
                        .unwrap()
                        .lt(&Decimal::parse("0").unwrap())
                        .unwrap()
                );
            }
        }
        for s in shares {
            assert_eq!(
                s["total_taxable_component"],
                s["section95_net_income_share"]
            );
        }
        runs += 1;
    }
}

#[test]
fn percentages_helper_splits_exactly_one_hundred() {
    for weights in [&[1, 1, 1][..], &[2, 3, 5], &[1; 6]] {
        let parts: Vec<Value> = percentages_from(weights)
            .into_iter()
            .map(|p| json!({ "p": p }))
            .collect();
        assert!(
            sum(&parts, "p")
                .equals(&Decimal::parse("100.00").unwrap())
                .unwrap()
        );
    }
}

#[test]
fn json_boundary_rejects_wrong_types_and_oversized_input() {
    assert!(api::evaluate("s100a-risk", &Value::Null).is_err());
    assert_eq!(
        api::evaluate(
            "s100a-risk",
            &json!({"beneficiary_name": "X", "distribution_amount": 5})
        ),
        Err("distribution_amount must be a string".into())
    );
    assert!(
        api::evaluate(
            "s99b-liability",
            &json!({"beneficiary_name": "X", "gross_amount_received_aud": "1".repeat(8193)})
        )
        .unwrap_err()
        .contains("exceeds 8192 bytes")
    );
    assert!(
        api::evaluate(
            "trust-resolution",
            &json!({"financial_year": 2026, "resolution_date": "2026-02-30", "allocated_percentages_total": "100"})
        )
        .unwrap_err()
        .contains("YYYY-MM-DD")
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_solomons-sword"))
        .arg("proportionate-share")
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
            .contains("trust_accounting_income must be a string")
    );
}

#[test]
fn residency_is_a_required_input_not_a_default() {
    // An omitted residency is refused, never routed past the s 98(2A)/(3) refusal.
    let input = json!({"trust_accounting_income": "100", "section95_net_taxable_income": "100",
        "beneficiaries": [{"beneficiary_name": "A", "is_under_legal_disability": false, "percentage_entitlement": "100.00"}]});
    assert!(
        api::evaluate("proportionate-share", &input)
            .unwrap_err()
            .contains("A is a non-resident")
    );
}

#[test]
fn a_narrow_console_codepage_does_not_end_the_run() {
    // Output is always UTF-8, so a Python encoding override cannot end a finished run.
    let run = Command::new(env!("CARGO_BIN_EXE_solomons-sword"))
        .args([
            "s100a-check",
            "--beneficiary",
            "Zoë Ŧrust",
            "--amount",
            "100",
        ])
        .env("PYTHONIOENCODING", "cp437")
        .output()
        .unwrap();
    assert_eq!(run.status.code(), Some(0));
    let out = String::from_utf8(run.stdout).unwrap();
    assert!(out.contains("Section 100A Risk Evaluation — Zoë Ŧrust"));
}
