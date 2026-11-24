// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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
    // The engine's worksheets in their own modules refuse as engine.rs does.
    let modules = [
        "engine",
        "engine/balancing",
        "engine/cgt_parcels",
        "engine/fbt_benefits",
        "engine/fbt_benefits/entertainment",
        "engine/fhss",
        "engine/franking_account",
        "engine/fuel_tax_credits",
        "engine/alcohol_excise",
        "engine/hire_purchase",
        "engine/luxury_car_tax",
        "engine/wine_equalisation_tax",
        "engine/sb_cgt_concessions",
        "engine/individual_offsets",
        "engine/work_expenses",
        "engine/ato_interest_penalties",
        "engine/transfer_balance",
        "engine/rental_property",
        "engine/super_benefit_tax",
        "engine/gst_adjustments",
        "engine/gst_food_retailer_sam",
        "engine/gst_food_retailer_sam/methods",
        "engine/deferred_tax",
        "engine/leave_provisions",
        "engine/long_service_leave",
        "engine/foreign_income_conversion",
        "engine/foreign_income_tax_offset",
        "engine/ess_discount",
        "engine/workers_compensation_premium",
    ]
    .into_iter()
    .flat_map(|dir| {
        std::fs::read_dir(src.join(dir))
            .unwrap()
            .filter_map(move |entry| {
                let name = entry.unwrap().file_name().into_string().unwrap();
                name.ends_with(".rs")
                    .then(|| (format!("{dir}/{name}"), &["refuse(", "bad("][..]))
            })
    });
    for (file, openers) in [
        ("engine.rs", &["refuse(", "bad("][..]),
        ("api.rs", &["refusal("][..]),
        ("estimate.rs", &["Refusal::new("][..]),
        ("dgr.rs", &["Refusal::new("][..]),
        ("dgr/input.rs", &["Refusal::new("][..]),
        ("fbt_return.rs", &["Refusal::new("][..]),
        ("fbt_return/input.rs", &["Refusal::new("][..]),
        ("fbt_return/kind.rs", &["Refusal::new("][..]),
        ("contract_register.rs", &["Refusal::new("][..]),
        ("contract_register/csv.rs", &["Refusal::new("][..]),
        ("contract_register/figures.rs", &["Refusal::new("][..]),
        ("contract_register/input.rs", &["Refusal::new("][..]),
        ("main.rs", &["Refusal::new("][..]),
        ("decimal.rs", &["=> "][..]),
    ]
    .map(|(file, openers)| (file.to_string(), openers))
    .into_iter()
    .chain(modules)
    {
        let text = read(&file);
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

/// `input.field_unknown`'s meaning names every command that refuses a field
/// it does not take, the Schedule 5 methods together.
#[test]
fn field_unknown_names_every_command_that_refuses_an_unknown_field() {
    let meaning = api::refusal_codes()
        .iter()
        .find(|(code, _)| *code == "input.field_unknown")
        .unwrap()
        .1;
    for command in api::commands() {
        let refused = api::evaluate_coded(command, &json!({"not_an_input": true}));
        if refused.is_err_and(|r| r.code == "input.field_unknown") {
            let named = if command.starts_with("schedule-5-") {
                "the Schedule 5 methods"
            } else {
                command
            };
            assert!(meaning.contains(named), "{command}");
        }
    }
}

/// A command that refuses an unknown field accepts each of its catalogued
/// required inputs, so every one of them must be an input it reads: its
/// catalogue example runs, and runs no longer without any one of them. A
/// required input the command ignored would be taken and silently dropped.
#[test]
fn every_catalogued_required_input_of_a_strict_command_is_read() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut checked = 0;
    for command in api::commands() {
        let refused = api::evaluate_coded(command, &json!({"not_an_input": true}));
        let entry = &catalogue[command.replace('-', "_")];
        if !refused.is_err_and(|r| r.code == "input.field_unknown") || entry.is_null() {
            continue;
        }
        let mut facts = entry["example"]["facts"].clone();
        facts.as_object_mut().unwrap().remove("kind");
        assert!(api::evaluate(command, &facts).is_ok(), "{command}");
        for name in entry["required_inputs"].as_object().unwrap().keys() {
            if name == "kind" {
                continue;
            }
            let mut without = facts.clone();
            without.as_object_mut().unwrap().remove(name);
            assert!(
                api::evaluate(command, &without).is_err(),
                "{command} {name}"
            );
        }
        checked += 1;
    }
    // The 14 commands that refused an unknown field before the FBT benefit
    // worksheets (the 3 Schedule 5 methods counted apart), the 11 FBT benefit
    // worksheets, payg-gross-up, hire-purchase-schedule,
    // chattel-mortgage-schedule, loan-transactions,
    // fbt-entertainment-classifier, unused-leave-payment,
    // lump-sum-arrears-tax-offset, income-tests,
    // business-capital-expenditure, balancing-adjustment, trade-in,
    // etp-tax-free-component, fbt-meal-entertainment-register,
    // fbt-minor-benefit-check, gst-margin-scheme, wine-equalisation-tax,
    // luxury-car-tax, borrowing-expenses-schedule, payroll-tax,
    // fuel-tax-credits, alcohol-excise, dasp-tax, ppsr-timing, cgt-parcels,
    // fbt-ev-exemption-check, fhss, franking-account, sb-cgt-concessions,
    // depreciation-low-value-pool, depreciation-small-business-pool,
    // capital-works, ato-interest-penalties, work-expenses,
    // small-business-income-offset, government-super-amounts,
    // beneficiary-tax-offset, invalid-carer-tax-offset, zone-tax-offset,
    // net-medical-expenses-tax-offset, non-commercial-losses, psi-test,
    // first-home-owner-grant,
    // fbt-instalment, private-health-insurance-rebate, rental-property, super-benefit-tax and
    // foreign-resident-withholding,
    // horticulture-shearing-withholding, upward-variation-withholding,
    // daily-casual-withholding, no-abn-withholding, entertainer-withholding,
    // return-to-work-withholding, annuity-withholding,
    // voluntary-agreement-withholding, residency-tests, lodgment-required,
    // rd-tax-offset, gst-creditable-purpose-change, gst-annual-apportionment,
    // gst-financial-acquisitions-threshold, gst-going-concern,
    // gst-second-hand-goods, gst-bad-debt, deferred-tax, leave-provisions,
    // cgt-cost-base-adjustments, super-withholding, payg-instalment,
    // foreign-income-conversion,
    // foreign-income-tax-offset, trading-stock, prepayments, minors-income,
    // smsf-tax, ess-discount, help-indexation,
    // instant-asset-write-off, defined-benefit-income-cap,
    // foreign-pension-upp, scholarship-exemption, super-crystallisation and
    // endorsement-review.
    // With transfer-balance, excess-contributions, asic-fees, vic-property-taxes, land-tax, pcg-2021-4,
    // workers-compensation-premium, transfer-duty, foreign-purchaser-surcharge, esic and
    // gst-food-retailer-sam.
    assert_eq!(checked, 123);
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

#[test]
fn gst_accepts_plain_decimal_strings_at_the_cli_boundary() {
    let input = |amount| {
        json!({"amount": amount, "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"})
            .to_string()
    };
    let (status, expected, stderr) = run(&["gst"], input("1000").as_bytes());
    assert_eq!(status, 0, "{stderr}");
    for plain in ["1000.00", " 1000.00 ", "\u{a0}1000.00\u{3000}"] {
        assert_eq!(
            run(&["gst"], input(plain).as_bytes()),
            (0, expected.clone(), String::new()),
            "{plain:?}"
        );
    }
    for alternate in [
        "1e3", "1000e0", "1e-1", "1_000", "1__000", "+1000", ".10", "1000.", "١٢",
    ] {
        let (status, stdout, stderr) = run(&["gst", "--refusal-json"], input(alternate).as_bytes());
        assert_eq!(status, 2, "{alternate:?} {stderr}");
        let refusal: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(refusal["code"], "input.decimal_unreadable", "{alternate:?}");
    }
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
