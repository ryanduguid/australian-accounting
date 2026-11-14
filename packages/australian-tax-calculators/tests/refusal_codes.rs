// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Every refusal carries a catalogued code, and the code changes nothing a
//! caller already reads: the frozen messages, `error:` line and exit 2 stay.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::Decimal;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// The first quoted token after each `opener(` in `text`, where one follows.
fn codes_after(text: &str, opener: &str) -> Vec<String> {
    text.match_indices(opener)
        .filter_map(|(at, _)| {
            let rest = text[at + opener.len()..].trim_start().strip_prefix('"')?;
            rest.split('"').next().map(str::to_string)
        })
        .collect()
}

fn catalogue() -> BTreeSet<&'static str> {
    api::refusal_codes().iter().map(|(code, _)| *code).collect()
}

#[test]
fn every_code_at_a_refusal_site_is_catalogued_and_every_entry_is_used() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let read = |name: &str| std::fs::read_to_string(src.join(name)).unwrap();
    let mut used = BTreeSet::new();
    for (file, openers) in [
        ("engine.rs", &["refuse(", "bad("][..]),
        ("api.rs", &["refusal("][..]),
        ("main.rs", &["Refusal::new("][..]),
        ("decimal.rs", &["=> "][..]),
    ] {
        let text = read(file);
        for opener in openers {
            used.extend(
                codes_after(&text, opener)
                    .into_iter()
                    .filter(|code| file != "decimal.rs" || code.starts_with("decimal.")),
            );
        }
    }
    let catalogue: BTreeSet<String> = catalogue().iter().map(|c| c.to_string()).collect();
    assert_eq!(used, catalogue);
}

#[test]
fn the_catalogue_is_sorted_unique_and_semantic() {
    let codes: Vec<&str> = api::refusal_codes().iter().map(|(code, _)| *code).collect();
    assert!(codes.windows(2).all(|pair| pair[0] < pair[1]));
    for (code, meaning) in api::refusal_codes() {
        let dotted = code.split('.').count() == 2;
        let plain = code
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.');
        assert!(dotted && plain, "{code}");
        assert!(meaning.ends_with('.'), "{code}");
    }
}

/// Every frozen call gives the same answer through either entry point, and
/// each frozen refusal now has a catalogued code.
#[test]
fn every_frozen_refusal_keeps_its_message_and_has_a_code() {
    let corpus: Vec<Value> =
        serde_json::from_str(include_str!("data/python-api.json")).expect("corpus parses");
    let catalogue = catalogue();
    let mut seen = BTreeSet::new();
    let mut refusals = 0;
    for row in &corpus {
        let command = row["command"].as_str().unwrap();
        let input = &row["input"];
        match (
            api::evaluate(command, input),
            api::evaluate_coded(command, input),
        ) {
            (Ok(plain), Ok(coded)) => assert_eq!(plain, coded),
            (Err(message), Err(refusal)) => {
                assert_eq!(message, refusal.message);
                assert!(catalogue.contains(refusal.code), "{}", refusal.code);
                seen.insert(refusal.code);
                refusals += 1;
            }
            (plain, coded) => panic!("{command}: {plain:?} against {coded:?}"),
        }
    }
    assert!(refusals > 500, "too few frozen refusals: {refusals}");
    assert!(seen.len() >= 15, "too few codes exercised: {seen:?}");
}

#[test]
fn an_interpolated_message_keeps_one_code_for_one_failure() {
    let input = |year: &str| json!({"amount": "1100", "gst_inclusive": true, "scope_confirmed": true, "year": year});
    for year in ["1999-00", "2099-00"] {
        let refusal = api::evaluate_coded("gst", &input(year)).unwrap_err();
        assert_eq!(refusal.code, "period.unsupported");
        assert!(refusal.message.contains(year), "{}", refusal.message);
    }
}

#[test]
fn unreadable_decimal_text_and_an_undefined_step_have_their_own_codes() {
    let refused = |amount: &str| {
        let input = json!({"amount": amount, "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"});
        api::evaluate_coded("gst", &input).unwrap_err()
    };
    let text = refused("nope");
    assert_eq!(
        (text.code, text.message.as_str()),
        ("input.decimal_unreadable", "decimal.InvalidOperation")
    );
    // Worksheet inputs refuse NaN first, so this arithmetic origin shows at the signal.
    let nan = Decimal::parse("NaN").unwrap();
    let step = nan.lt(&Decimal::parse("1").unwrap()).unwrap_err();
    assert_eq!(step.code(), "decimal.invalid_operation");
}

fn run(args: &[&str], stdin: &[u8]) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_australian-tax-calculators"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // The binary may refuse and exit before it reads everything.
    let _ = child.stdin.take().unwrap().write_all(stdin);
    let out = child.wait_with_output().unwrap();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap().replace("\r\n", "\n");
    (
        out.status.code().unwrap(),
        text(out.stdout),
        text(out.stderr),
    )
}

const REFUSED: &[u8] =
    br#"{"amount": "1100", "gst_inclusive": true, "scope_confirmed": false, "year": "2025-26"}"#;
const ACCEPTED: &[u8] =
    br#"{"amount": "1100", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"}"#;
const SCOPE: &str = "Establish all stated scope conditions before calculating.";

#[test]
fn by_default_a_refusal_writes_nothing_to_stdout() {
    assert_eq!(
        run(&["gst"], REFUSED),
        (2, String::new(), format!("error: {SCOPE}\n"))
    );
}

#[test]
fn the_flag_adds_the_code_on_stdout_and_keeps_the_error_line_and_exit() {
    let (code, out, err) = run(&["gst", "--refusal-json"], REFUSED);
    assert_eq!((code, err), (2, format!("error: {SCOPE}\n")));
    let refusal: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        refusal,
        json!({"ok": false, "code": "scope.not_confirmed", "message": SCOPE})
    );
}

#[test]
fn the_flag_leaves_an_accepted_worksheet_unchanged() {
    assert_eq!(
        run(&["gst", "--refusal-json"], ACCEPTED),
        run(&["gst"], ACCEPTED)
    );
}

#[test]
fn input_the_engine_never_reaches_is_coded_too() {
    let code = |stdin: &[u8]| {
        let (exit, out, err) = run(&["gst", "--refusal-json"], stdin);
        assert_eq!(exit, 2, "{err}");
        let refusal: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            err,
            format!("error: {}\n", refusal["message"].as_str().unwrap())
        );
        refusal["code"].as_str().unwrap().to_string()
    };
    assert_eq!(code(b"{"), "input.invalid_json");
    assert_eq!(code(b"[]"), "input.not_object");
    assert_eq!(code(b"{}"), "input.missing_field");
    assert_eq!(code(&vec![b' '; (8 << 20) + 2]), "input.too_large");
}

#[test]
fn a_repeated_key_is_refused_by_name_not_resolved_to_its_last_value() {
    // gst ignores fields it does not take, so `extra` reaches only the decoder.
    let gst = |rest: &str| {
        format!(
            r#"{{"amount": "1100", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26", {rest}}}"#
        )
    };
    for (rest, key) in [
        (r#""amount": "2200""#, "amount"),
        // "amount" with its m as a JSON Unicode escape; \x5c is the backslash.
        ("\"a\x5cu006dount\": \"2200\"", "amount"),
        (r#""extra": {"k": 1, "k": 2}"#, "k"),
        (r#""extra": [{"k": 1, "k": 2}]"#, "k"),
    ] {
        let (exit, out, err) = run(&["gst", "--refusal-json"], gst(rest).as_bytes());
        assert_eq!(exit, 2, "{rest}: {out}");
        let refusal: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(refusal["code"], "input.invalid_json", "{rest}");
        let message = refusal["message"].as_str().unwrap();
        assert_eq!(message, format!("invalid JSON: duplicate key {key:?}"));
        assert_eq!(err, format!("error: {message}\n"));
    }
    for rest in [
        r#""extra": [{"k": 1}, {"k": 2}], "more": {"k": 3}"#,
        r#""extra": "amount", "note": "\"amount\": \"2200\", {\"k\": 1}""#,
    ] {
        assert_eq!(
            run(&["gst"], gst(rest).as_bytes()),
            run(&["gst"], ACCEPTED),
            "{rest}"
        );
    }
}

#[test]
fn the_flag_is_not_a_command() {
    let (code, out, err) = run(&["--refusal-json", "gst"], REFUSED);
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(
        err.starts_with("usage: australian-tax-calculators COMMAND"),
        "{err}"
    );
}
