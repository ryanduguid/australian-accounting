// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Family tax benefit Part B, rent assistance and jobseeker payment, read on
//! 26 November 2026, and disability support pension and carer payment, read
//! on 27 November 2026 (SS Guide 5.1.8.10's common pension rates and Services
//! Australia's carer payment rates and cut-off points from 20 September 2026). The vectors are the sources' published limits: the FTB
//! Part B secondary earner income limits in the DSS rates list for 1 July
//! 2026 (and its previous amounts for 2025-26); the rent ceilings and maximum
//! rates in SS Guide 5.1.7.10 and 'A guide to Australian Government payments'
//! for 1 July to 19 September 2026; the jobseeker payment income limits and
//! partner income free area in that guide for 20 September to 31 December
//! 2026. The rest are made-up figures worked by hand from the provisions
//! (docs/calculation-evidence.md#government-schemes).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(cmd: &str, input: &Value) -> Value {
    api::evaluate(cmd, input).unwrap_or_else(|e| panic!("{cmd}: {e:?}"))
}

fn code(cmd: &str, input: &Value) -> &'static str {
    api::evaluate_coded(cmd, input).unwrap_err().code
}

fn with(mut base: Value, changes: Value) -> Value {
    for (k, v) in changes.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

// ---- FTB Part B ------------------------------------------------------------

fn ftb(relationship: &str, age: i64, own: &str, partner: &str) -> Value {
    json!({"relationship": relationship, "youngest_child_age": age,
           "adjusted_taxable_income": own, "partner_adjusted_taxable_income": partner,
           "income_support": false, "grandparent": false, "energy_supplement": false,
           "facts": {"whole_year": true, "no_parental_leave_pay": true, "general_method": true},
           "year": "2026-27", "scope_confirmed": true})
}

fn part_b(input: &Value) -> Value {
    run("ftb-part-b", input)["amounts"]["part_b_rate"].clone()
}

#[test]
fn ftb_part_b_stops_at_the_published_secondary_earner_limits() {
    // Rates list, 1 July 2026: $35,661 (youngest under 5) and $27,777 (5 to
    // 18), with the end of year supplement; 2025-26: $34,438 and $26,828.
    for (year, age, below, limit, left) in [
        ("2026-27", 3, "35660.00", "35661.00", "0.10"),
        ("2026-27", 7, "27776.00", "27777.00", "0.10"),
        ("2025-26", 2, "34437.00", "34438.00", "0.15"),
        ("2025-26", 12, "26827.00", "26828.00", "0.10"),
    ] {
        let at = |lower: &str| with(ftb("couple", age, "90000.00", lower), json!({"year": year}));
        assert_eq!(part_b(&at(below)), left, "{year} {age}");
        assert_eq!(part_b(&at(limit)), "0.00", "{year} {age}");
    }
}

#[test]
fn ftb_part_b_single_gets_the_maximum_up_to_the_income_limit() {
    let single = |income: &str| ftb("single", 4, income, "0.00");
    let out = run("ftb-part-b", &single("124327.00"));
    // $5,223.15 standard rate and $478.15 supplement (rates list, 1 July 2026).
    assert_eq!(out["amounts"]["maximum_rate"], "5701.30");
    assert_eq!(out["amounts"]["part_b_rate"], "5701.30");
    assert_eq!(out["nil_because"], Value::Null);
    let over = run("ftb-part-b", &single("124328.00"));
    assert_eq!(over["amounts"]["part_b_rate"], "0.00");
    assert!(over["nil_because"].as_str().unwrap().contains("clause 28B"));
    // Clause 28B(2): income support lifts the limit.
    assert_eq!(
        part_b(&with(single("200000.00"), json!({"income_support": true}))),
        "5701.30"
    );
    // A couple's higher income is tested whichever member earns it.
    assert_eq!(part_b(&ftb("couple", 4, "10000.00", "130000.00")), "0.00");
    // Clause 31B: $51.10 energy supplement for a youngest child of 5 or over.
    assert_eq!(
        part_b(&with(
            ftb("single", 5, "50000.00", "0.00"),
            json!({"energy_supplement": true})
        )),
        "4175.60"
    );
}

#[test]
fn ftb_part_b_is_nil_for_a_couple_whose_youngest_is_13() {
    let couple = ftb("couple", 13, "90000.00", "0.00");
    let out = run("ftb-part-b", &couple);
    assert_eq!(out["amounts"]["part_b_rate"], "0.00");
    assert!(out["nil_because"].as_str().unwrap().contains("clause 28D"));
    // A grandparent keeps the rate (clause 28D(2)): $3,646.35 + $478.15.
    assert_eq!(
        part_b(&with(couple, json!({"grandparent": true}))),
        "4124.50"
    );
    // A single parent of a 13 year old is not caught by clause 28D.
    assert_eq!(part_b(&ftb("single", 13, "90000.00", "0.00")), "4124.50");
}

#[test]
fn ftb_part_b_refuses_what_it_cannot_work() {
    let base = ftb("single", 16, "50000.00", "0.00");
    let refused = |changes: Value| code("ftb-part-b", &with(base.clone(), changes));
    assert_eq!(refused(json!({})), "ftb_part_b.facts_not_stated");
    let senior = json!({"whole_year": true, "no_parental_leave_pay": true,
                        "general_method": true, "senior_secondary": true});
    assert_eq!(
        part_b(&with(base.clone(), json!({"facts": senior}))),
        "4124.50"
    );
    assert_eq!(
        refused(json!({"youngest_child_age": 19})),
        "ftb_part_b.age_invalid"
    );
    assert_eq!(
        refused(json!({"youngest_child_age": "3"})),
        "ftb_part_b.age_invalid"
    );
    assert_eq!(
        refused(json!({"relationship": "partnered"})),
        "ftb_part_b.relationship_invalid"
    );
    assert_eq!(
        refused(json!({"partner_adjusted_taxable_income": "1.00"})),
        "ftb_part_b.partner_income_single"
    );
    assert_eq!(refused(json!({"facts": 1})), "ftb_part_b.facts_invalid");
    assert_eq!(
        refused(json!({"facts": {"whole_year": true, "no_parental_leave_pay": true}})),
        "ftb_part_b.facts_not_stated"
    );
    assert_eq!(refused(json!({"year": "2024-25"})), "period.unsupported");
}

// ---- Rent assistance -------------------------------------------------------

fn rent(date: &str, situation: &str, amount: &str) -> Value {
    json!({"date": date, "situation": situation, "fortnightly_rent": amount,
           "facts": {"rate_calculator": true, "pays_private_rent": true,
                     "no_dependent_children": true, "partner_not_veteran": true},
           "year": "2026-27", "scope_confirmed": true})
}

fn ra(date: &str, situation: &str, amount: &str) -> Value {
    run("rent-assistance", &rent(date, situation, amount))["amounts"]["rent_assistance"].clone()
}

#[test]
fn rent_assistance_reaches_the_maximum_at_the_published_ceilings() {
    // SS Guide 5.1.7.10 (20 September 2026): threshold, rent for the maximum,
    // maximum. The ceiling is the threshold plus the maximum / 0.75, which the
    // guide rounds up to the cent.
    for (situation, threshold, ceiling, maximum) in [
        ("single", "157.80", "456.20", "223.80"),
        ("single_sharer", "157.80", "356.74", "149.20"),
        ("couple", "255.80", "537.14", "211.00"),
        ("illness_separated", "157.80", "456.20", "223.80"),
        ("respite_care", "157.80", "456.20", "223.80"),
        ("partner_in_gaol", "157.80", "456.20", "223.80"),
        ("temporarily_separated", "157.80", "439.14", "211.00"),
    ] {
        assert_eq!(
            ra("2026-10-01", situation, threshold),
            "0.00",
            "{situation}"
        );
        assert_eq!(ra("2026-10-01", situation, ceiling), maximum, "{situation}");
        assert_eq!(
            ra("2026-10-01", situation, "2000.00"),
            maximum,
            "{situation}"
        );
    }
    // 'A guide to Australian Government payments', 1 July to 19 September
    // 2026: single $219.40 from $447.34; sharer $146.27 (two-thirds, to the
    // cent); couple $206.80 from $526.54.
    assert_eq!(ra("2026-09-19", "single", "447.34"), "219.40");
    assert_eq!(ra("2026-07-01", "single_sharer", "349.83"), "146.27");
    assert_eq!(ra("2026-08-15", "couple", "526.54"), "206.80");
    assert_eq!(
        ra("2026-08-15", "temporarily_separated", "430.54"),
        "206.80"
    );
}

#[test]
fn rent_assistance_pays_75_cents_in_the_dollar_over_the_threshold() {
    let out = run("rent-assistance", &rent("2026-10-01", "single", "400.00"));
    // (400.00 - 157.80) x 0.75.
    assert_eq!(out["amounts"]["rate_a"], "181.65");
    assert_eq!(out["amounts"]["rent_assistance"], "181.65");
    assert_eq!(out["rates"]["rent_threshold"], "157.80");
    assert_eq!(ra("2026-10-01", "single", "100.00"), "0.00");
    assert_eq!(ra("2026-10-01", "couple", "400.00"), "108.15");
    // Each member of a couple both receiving it: half of (400.00 - 255.80) x
    // 0.75, up to half of $211.00.
    assert_eq!(ra("2026-10-01", "couple_both_receive", "400.00"), "54.08");
    assert_eq!(ra("2026-10-01", "couple_both_receive", "900.00"), "105.50");
    assert_eq!(ra("2026-10-01", "single_sharer", "300.00"), "106.65");
}

#[test]
fn rent_assistance_refuses_what_it_cannot_work() {
    let base = rent("2026-10-01", "single", "400.00");
    let refused = |changes: Value| code("rent-assistance", &with(base.clone(), changes));
    for date in ["2026-06-30", "2027-03-20", "2026-02-30", "soon"] {
        assert_eq!(
            refused(json!({"date": date})),
            "rent_assistance.date_uncovered",
            "{date}"
        );
    }
    assert_eq!(
        refused(json!({"situation": "single_with_children"})),
        "rent_assistance.situation_invalid"
    );
    assert_eq!(
        refused(json!({"facts": []})),
        "rent_assistance.facts_invalid"
    );
    assert_eq!(
        refused(json!({"facts": {"rate_calculator": true}})),
        "rent_assistance.facts_not_stated"
    );
    assert_eq!(
        refused(json!({"fortnightly_rent": "-1.00"})),
        "input.amount_invalid"
    );
}

// ---- Jobseeker payment -----------------------------------------------------

fn jsp(date: &str, situation: &str, income: &str) -> Value {
    json!({"date": date, "situation": situation, "pharmaceutical_allowance": false,
           "ordinary_income": income, "partner": "none", "partner_income": "0.00",
           "facts": {"qualified": true, "no_working_credit": true, "no_rent_assistance": true,
                     "assets_within_limit": true, "no_adjustments": true,
                     "energy_fortnightly": true},
           "year": "2026-27", "scope_confirmed": true})
}

fn paid(input: &Value) -> Value {
    run("jobseeker-payment", input)["amounts"]["rate"].clone()
}

#[test]
fn jobseeker_payment_stops_at_the_published_income_limits() {
    // 'A guide to Australian Government payments', 20 September to 31
    // December 2026, Module D: the typical total rate (basic rate, energy
    // supplement and, where shown, pharmaceutical allowance) and the income
    // at which it reaches nil.
    let partnered = json!({"partner": "not_receiving", "partner_income": "0.00"});
    for (situation, pha, extra, total, below, left, limit) in [
        (
            "single",
            false,
            json!({}),
            "833.70",
            "1557.00",
            "0.10",
            "1557.17",
        ),
        (
            "single_with_child",
            false,
            json!({}),
            "892.80",
            "1655.50",
            "0.10",
            "1655.67",
        ),
        (
            "single_principal_carer",
            true,
            json!({}),
            "899.80",
            "2399.00",
            "0.20",
            "2399.50",
        ),
        (
            "single_55_after_9_months",
            true,
            json!({}),
            "899.80",
            "1667.00",
            "0.20",
            "1667.34",
        ),
        (
            "single_partial_capacity",
            true,
            json!({}),
            "899.80",
            "1667.00",
            "0.20",
            "1667.34",
        ),
        (
            "partnered",
            false,
            partnered,
            "763.00",
            "1439.00",
            "0.20",
            "1439.34",
        ),
    ] {
        let at = |income: &str| {
            with(
                with(
                    jsp("2026-10-01", situation, income),
                    json!({"pharmaceutical_allowance": pha}),
                ),
                extra.clone(),
            )
        };
        assert_eq!(paid(&at("150.00")), total, "{situation}");
        assert_eq!(paid(&at(below)), left, "{situation}");
        assert_eq!(paid(&at(limit)), "0.00", "{situation}");
    }
    // 1 July to 19 September 2026: single $808.70 + $8.80.
    assert_eq!(paid(&jsp("2026-09-19", "single", "0.00")), "817.50");
}

#[test]
fn jobseeker_payment_tapers_at_50_then_60_cents() {
    let out = run("jobseeker-payment", &jsp("2026-10-01", "single", "400.00"));
    // 50% of $106 and 60% of $144.
    assert_eq!(out["amounts"]["ordinary_income_reduction"], "139.40");
    assert_eq!(out["amounts"]["rate"], "694.30");
    assert_eq!(paid(&jsp("2026-10-01", "single", "200.00")), "808.70");
    // Point 1068-G17: 40% of $250 for a single principal carer.
    assert_eq!(
        paid(&jsp("2026-10-01", "single_principal_carer", "400.00")),
        "792.80"
    );
}

#[test]
fn jobseeker_payment_applies_the_partner_income_test() {
    let couple = |partner: &str, own: &str, theirs: &str| {
        with(
            jsp("2026-10-01", "partnered", own),
            json!({"partner": partner, "partner_income": theirs}),
        )
    };
    // Module I: the partner income free area for a partner aged 22 or over is
    // $1,440 a fortnight.
    let out = run(
        "jobseeker-payment",
        &couple("not_receiving", "0.00", "1440.00"),
    );
    assert_eq!(out["amounts"]["partner_income_free_area"], "1440.00");
    assert_eq!(out["amounts"]["rate"], "763.00");
    assert_eq!(paid(&couple("not_receiving", "0.00", "1540.00")), "703.00");
    // Before 20 September 2026: $740.30 + $7.90 reaches nil at $1,414.67, so
    // $1,415.
    let july = with(
        couple("not_receiving", "0.00", "0.00"),
        json!({"date": "2026-08-01"}),
    );
    let out = run("jobseeker-payment", &july);
    assert_eq!(out["amounts"]["partner_income_free_area"], "1415.00");
    assert_eq!(out["amounts"]["rate"], "748.20");
    // Point 1068-G2: a pensioner partner's income is halved with the person's.
    let pension = run("jobseeker-payment", &couple("pension", "300.00", "500.00"));
    assert_eq!(pension["amounts"]["ordinary_income"], "400.00");
    assert_eq!(pension["amounts"]["rate"], "623.60");
    // Pharmaceutical allowance at the partnered $3.50.
    assert_eq!(
        paid(&with(
            couple("pension", "0.00", "0.00"),
            json!({"pharmaceutical_allowance": true})
        )),
        "766.50"
    );
}

#[test]
fn jobseeker_payment_refuses_what_it_cannot_work() {
    let base = jsp("2026-10-01", "single", "0.00");
    let refused = |changes: Value| code("jobseeker-payment", &with(base.clone(), changes));
    for date in ["2026-06-30", "2027-03-20", "2026-13-01"] {
        assert_eq!(
            refused(json!({"date": date})),
            "jobseeker_payment.date_uncovered",
            "{date}"
        );
    }
    assert_eq!(
        refused(json!({"situation": "single_exempt_principal_carer"})),
        "jobseeker_payment.situation_invalid"
    );
    assert_eq!(
        refused(json!({"partner": "benefit"})),
        "jobseeker_payment.partner_invalid"
    );
    assert_eq!(
        refused(json!({"partner": "not_receiving"})),
        "jobseeker_payment.partner_inconsistent"
    );
    assert_eq!(
        refused(json!({"partner_income": "5.00"})),
        "jobseeker_payment.partner_inconsistent"
    );
    assert_eq!(
        refused(json!({"situation": "partnered"})),
        "jobseeker_payment.partner_inconsistent"
    );
    assert_eq!(
        refused(json!({"facts": "yes"})),
        "jobseeker_payment.facts_invalid"
    );
    let mut facts = base["facts"].clone();
    facts["no_working_credit"] = json!(false);
    assert_eq!(
        refused(json!({"facts": facts})),
        "jobseeker_payment.facts_not_stated"
    );
    // Point 1068-C1: Module C does not apply where quarterly energy
    // supplement is payable, so that case is refused, not paid $8.80 more.
    let mut quarterly = base["facts"].clone();
    quarterly["energy_fortnightly"] = json!(false);
    assert_eq!(
        refused(json!({"facts": quarterly})),
        "jobseeker_payment.facts_not_stated"
    );
    assert_eq!(
        refused(json!({"pharmaceutical_allowance": "yes"})),
        "input.flag_invalid"
    );
    // Pharmaceutical allowance is indexed on 1 January; the rest is not.
    let january = json!({"date": "2027-01-01", "pharmaceutical_allowance": true});
    assert_eq!(
        refused(january),
        "jobseeker_payment.pharmaceutical_uncovered"
    );
    assert_eq!(
        paid(&with(
            base.clone(),
            json!({"date": "2026-12-31", "pharmaceutical_allowance": true})
        )),
        "840.70"
    );
    assert_eq!(
        paid(&with(base.clone(), json!({"date": "2027-03-19"}))),
        "833.70"
    );
}

// ---- Disability support pension and carer payment --------------------------

const DSP_FACTS: &str = r#"{"aged_21_or_with_child": true, "not_blind": true, "no_module_f_payment": true,
    "in_australia": true, "not_transitional": true, "no_rent_assistance": true,
    "no_quarterly_election": true}"#;
const CARER_FACTS: &str = r#"{"carer_payment_only": true, "in_australia": true,
    "not_transitional": true, "no_rent_assistance": true, "no_quarterly_election": true}"#;
const AGE_FACTS: &str = r#"{"in_australia": true, "not_blind": true, "not_transitional": true,
    "no_rent_assistance": true}"#;

/// One Calculator A case for `command` on 1 October 2026.
fn calc_a(command: &str, relationship: &str, homeowner: bool, income: &str, assets: &str) -> Value {
    let mut facts: Value = serde_json::from_str(match command {
        "disability-support-pension" => DSP_FACTS,
        "carer-payment" => CARER_FACTS,
        _ => AGE_FACTS,
    })
    .unwrap();
    if relationship == "couple" {
        facts["partner_pensioner"] = json!(true);
    }
    json!({"date": "2026-10-01", "relationship": relationship, "homeowner": homeowner,
           "ordinary_income": income, "financial_assets": "0", "assets": assets,
           "facts": facts, "year": "2026-27", "scope_confirmed": true})
}

#[test]
fn dsp_and_carer_payment_meet_the_common_pension_rates_and_cut_off_points() {
    for command in ["disability-support-pension", "carer-payment"] {
        let rate = |relationship, homeowner, income, assets| {
            let out = run(
                command,
                &calc_a(command, relationship, homeowner, income, assets),
            );
            out["amounts"]["rate"].as_str().unwrap().to_owned()
        };
        // SS Guide 5.1.8.10 and Services Australia's carer payment rates from
        // 20 September 2026: $1,237.70 single, $933.00 each of a couple.
        assert_eq!(rate("single", true, "0", "333000"), "1237.70", "{command}");
        assert_eq!(rate("couple", false, "0", "766000"), "933.00", "{command}");
        // Services Australia's carer payment part pension cut-off points.
        assert_eq!(rate("single", true, "0", "745750"), "0.00", "{command}");
        assert_eq!(rate("single", true, "0", "745500"), "0.20", "{command}");
        assert_eq!(rate("single", false, "0", "1012750"), "0.00", "{command}");
        assert_eq!(rate("couple", true, "0", "1121000"), "0.00", "{command}");
        assert_eq!(rate("couple", false, "0", "1388000"), "0.00", "{command}");
        // ($2,701.40 - $226) x 0.5 = $1,237.70: nil at the single income cut-off.
        assert_eq!(rate("single", true, "2701.40", "0"), "0.00", "{command}");
        let out = run(command, &calc_a(command, "single", true, "0", "0"));
        assert_eq!(out["calculation"], command.replace('-', "_"));
        let sources = out["sources"].as_array().unwrap();
        let dss = json!("https://guides.dss.gov.au/social-security-guide/5/1/8/10");
        assert!(sources.contains(&dss), "{command}");
    }
}

/// Neither command holds an amount of its own: on the first and last day of
/// every dated period age-pension reads, each gives age-pension's amounts and
/// rates for the same facts.
#[test]
fn dsp_and_carer_payment_take_every_amount_from_age_pensions_dated_rates() {
    let days = [
        ("2025-09-20", "2025-26"),
        ("2026-03-19", "2025-26"),
        ("2026-03-20", "2025-26"),
        ("2026-06-30", "2025-26"),
        ("2026-07-01", "2026-27"),
        ("2026-09-19", "2026-27"),
        ("2026-09-20", "2026-27"),
        ("2027-03-19", "2026-27"),
    ];
    for (date, year) in days {
        for relationship in ["single", "couple"] {
            let on = |command| {
                with(
                    calc_a(command, relationship, false, "250.00", "650000.00"),
                    json!({"date": date, "year": year, "financial_assets": "120000.00"}),
                )
            };
            let age = run("age-pension", &on("age-pension"));
            for command in ["disability-support-pension", "carer-payment"] {
                let out = run(command, &on(command));
                assert_eq!(out["amounts"], age["amounts"], "{command} {date}");
                assert_eq!(out["rates"], age["rates"], "{command} {date}");
            }
        }
    }
}

/// The commands' own source holds no amount: every rate is age-pension's,
/// whose tables carry each amount's dates and source.
#[test]
fn dsp_and_carer_payment_hard_code_no_rate() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/pension_calculator_a.rs"
    );
    let source = std::fs::read_to_string(path).unwrap();
    let code: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect();
    let quoted_amount = code
        .split('"')
        .skip(1)
        .step_by(2)
        .any(|text| text.starts_with(|c: char| c.is_ascii_digit() || c == '$'));
    assert!(!quoted_amount, "a quoted amount in {path}");
    assert!(!code.contains("d(") && !code.contains("int("), "{path}");
}

#[test]
fn dsp_and_carer_payment_refuse_what_they_do_not_model() {
    let not_stated = |command: &str, fact: &str| {
        let mut input = calc_a(command, "single", true, "0", "0");
        input["facts"][fact] = json!(false);
        let e = api::evaluate_coded(command, &input).unwrap_err();
        (e.code, e.message)
    };
    for (fact, cites) in [
        ("aged_21_or_with_child", "s 1066A"),
        ("not_blind", "s 1065"),
        ("no_module_f_payment", "Module F"),
        ("no_quarterly_election", "1064-C2"),
    ] {
        let (refusal, message) = not_stated("disability-support-pension", fact);
        assert_eq!(refusal, "disability_support_pension.facts_not_stated");
        assert!(message.starts_with("Not computed"), "{message}");
        assert!(message.contains(cites), "{message}");
    }
    let (refusal, message) = not_stated("carer-payment", "carer_payment_only");
    assert_eq!(refusal, "carer_payment.facts_not_stated");
    assert!(message.contains("carer allowance (Part 2.19)"), "{message}");
    // Carer payment of a blind person is still Calculator A's (s 1064(2)), so
    // the command takes no not_blind fact.
    let mut blind = calc_a("carer-payment", "single", true, "0", "0");
    blind["facts"]["not_blind"] = json!(true);
    assert_eq!(code("carer-payment", &blind), "input.field_unknown");
}

#[test]
fn dsp_and_carer_payment_refuse_dates_outside_the_rates_and_bad_input() {
    for (command, kind) in [
        ("disability-support-pension", "disability_support_pension"),
        ("carer-payment", "carer_payment"),
    ] {
        let base = calc_a(command, "single", true, "0", "0");
        let refused = |changes: Value| code(command, &with(base.clone(), changes));
        let cases = [
            (
                json!({"date": "2025-09-19", "year": "2025-26"}),
                "date_uncovered",
            ),
            (json!({"date": "2027-03-20"}), "date_uncovered"),
            (json!({"date": "2026-05-01"}), "date_invalid"),
            (json!({"relationship": "widowed"}), "relationship_invalid"),
            (json!({"financial_assets": "1.00"}), "assets_inconsistent"),
            (json!({"relationship": "couple"}), "facts_not_stated"),
            (json!({"facts": 1}), "facts_invalid"),
        ];
        for (changes, suffix) in cases {
            let want = format!("{kind}.{suffix}");
            assert_eq!(refused(changes.clone()), want, "{changes}");
        }
        assert_eq!(refused(json!({"homeowner": "yes"})), "input.flag_invalid");
        assert_eq!(refused(json!({"year": "2024-25"})), "period.unsupported");
    }
}
