// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The Schedule 5 (NAT 3348) worksheets against the ATO's withholding amounts
//! examples 1 to 4 in the schedules for payments from 1 July 2026, 1 July 2025
//! and 1 July 2024 (docs/calculation-evidence.md#schedule-5), each rule's
//! boundary and the refusals.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const A: &str = "schedule-5-method-a";
const B_I: &str = "schedule-5-method-b-i";
const B_II: &str = "schedule-5-method-b-ii";

fn method_a(year: &str, earnings: &str, additional: &str, periods: i64, pay: &str) -> Value {
    json!({
        "earnings": earnings, "additional_payment": additional, "pay_periods": periods,
        "pay_period": pay, "scale": 2, "study_loan": false, "year": year, "scope_confirmed": true,
    })
}

/// `count` earlier pay periods, each with the same earnings, back payment and amount withheld.
fn method_b_i(year: &str, earnings: &str, period: [&str; 3], count: usize, pay: &str) -> Value {
    let [paid, back, withheld] = period;
    let periods =
        vec![json!({"earnings": paid, "back_payment": back, "withheld": withheld}); count];
    json!({
        "earnings": earnings, "back_payment_periods": periods, "pay_period": pay, "scale": 2,
        "study_loan": false, "year": year, "scope_confirmed": true,
    })
}

/// Earnings, the additional payment, earnings to date and pay periods to date,
/// with no earlier Method B(ii) payments.
fn method_b_ii(
    year: &str,
    earnings: &str,
    additional: &str,
    to_date: (&str, i64),
    pay: &str,
) -> Value {
    json!({
        "earnings": earnings, "additional_payment": additional,
        "year_to_date_earnings": to_date.0, "pay_periods_to_date": to_date.1,
        "earlier_additional_payments": "0.00", "earlier_additional_withholding": "0.00",
        "pay_period": pay, "scale": 2, "study_loan": false, "year": year, "scope_confirmed": true,
    })
}

fn with(mut facts: Value, overrides: Value) -> Value {
    for (k, v) in overrides.as_object().unwrap() {
        facts[k] = v.clone();
    }
    facts
}

fn run(command: &str, facts: &Value) -> Value {
    api::evaluate(command, facts).unwrap_or_else(|e| panic!("{command} {facts}: {e}"))
}

fn refusal(command: &str, facts: &Value) -> &'static str {
    api::evaluate_coded(command, facts).unwrap_err().code
}

fn amounts<'a, const N: usize>(result: &'a Value, keys: [&str; N]) -> [&'a str; N] {
    keys.map(|key| {
        result["amounts"][key]
            .as_str()
            .unwrap_or_else(|| panic!("no {key}"))
    })
}

const A_KEYS: [&str; 8] = [
    "earnings_used",
    "withholding_on_earnings",
    "additional_payment_per_period",
    "withholding_on_earnings_and_share",
    "withholding_before_limit",
    "withholding_limit",
    "withholding_on_additional_payment",
    "withholding",
];
const B_I_KEYS: [&str; 4] = [
    "back_payment",
    "withholding_on_back_payment",
    "withholding_on_earnings",
    "withholding",
];
const B_II_KEYS: [&str; 9] = [
    "average_total_earnings",
    "withholding_on_average_earnings",
    "additional_payments_per_period",
    "withholding_on_average_and_share",
    "withholding_for_the_year",
    "withholding_limit",
    "withholding_on_additional_payment",
    "withholding_on_earnings",
    "withholding",
];

fn study_loan() -> Value {
    json!({"study_loan": true})
}

/// Example 1 (Jarrod, payments from 1 July 2026): a $900 annual bonus on $1,400
/// a week, scale 2, with a HELP debt. PAYG: $267, $17, $272, $5 x 52 = $260.
/// HELP: $10, $12, $2 x 52 = $104. Limit $423; $364 on the bonus; $641 in all.
#[test]
fn jarrod_method_a_gives_the_ato_example_1() {
    let jarrod = method_a("2026-27", "1400.00", "900.00", 52, "weekly");
    assert_eq!(
        amounts(&run(A, &jarrod), A_KEYS),
        [
            "1400.00", "267.00", "17.00", "272.00", "260.00", "423.00", "260.00", "527.00"
        ]
    );
    let with_help = run(A, &with(jarrod, study_loan()));
    assert_eq!(
        amounts(&with_help, A_KEYS),
        [
            "1400.00", "277.00", "17.00", "284.00", "364.00", "423.00", "364.00", "641.00"
        ]
    );
    assert_eq!(with_help["period"], "2026-27 weekly");
    assert_eq!(with_help["rates"]["limit_rate"], "0.47");
    assert_eq!(with_help["rates"]["study_loan"], "true");
}

/// The 2024 and 2025 schedules work Jarrod on the 1 July 2024 Schedule 1:
/// $272, $17, $277, $5 x 52 = $260. This crate has no Schedule 8 for those
/// years, so the HELP part is refused rather than worked.
#[test]
fn jarrod_method_a_gives_the_earlier_schedules_payg_figures() {
    for year in ["2024-25", "2025-26"] {
        let jarrod = method_a(year, "1400.00", "900.00", 52, "weekly");
        assert_eq!(
            amounts(&run(A, &jarrod), A_KEYS),
            [
                "1400.00", "272.00", "17.00", "277.00", "260.00", "423.00", "260.00", "532.00"
            ],
            "{year}"
        );
        let refused = api::evaluate_coded(A, &with(jarrod, study_loan())).unwrap_err();
        assert_eq!(refused.code, "period.unsupported");
        // The year is one the worksheet supports, so the message names the study loan.
        assert_eq!(
            refused.message,
            format!(
                "study_loan is supported for 2026-27 only: there is no Schedule 8 study and training support loan component for {year}."
            )
        );
    }
}

/// Example 2 (Robyn, 2026): $1,600 a week for pay periods 1 to 8, then $1,700;
/// an $800 back payment, $100 for each. PAYG: $363 less $331 = $32 x 8 = $256,
/// $363 on the current pay, $619. HELP: $55 less $40 = $15 x 8 = $120; $794 in all.
#[test]
fn robyn_method_b_i_gives_the_ato_example_2() {
    let payg = method_b_i(
        "2026-27",
        "1700.00",
        ["1600.00", "100.00", "331.00"],
        8,
        "weekly",
    );
    assert_eq!(
        amounts(&run(B_I, &payg), B_I_KEYS),
        ["800.00", "256.00", "363.00", "619.00"]
    );
    // With HELP, each period had $331 + $40 withheld.
    let help = method_b_i(
        "2026-27",
        "1700.00",
        ["1600.00", "100.00", "371.00"],
        8,
        "weekly",
    );
    let help = run(B_I, &with(help, study_loan()));
    assert_eq!(
        amounts(&help, B_I_KEYS),
        ["800.00", "376.00", "418.00", "794.00"]
    );
    assert_eq!(help["rates"]["back_payment_periods"], "8");
}

/// The 2024 and 2025 schedules' Robyn: $1,100 a week, then $1,200: $208 less
/// $175 = $33 x 8 = $264, $208 on the current pay, $472.
#[test]
fn robyn_method_b_i_gives_the_earlier_schedules_payg_figures() {
    for year in ["2024-25", "2025-26"] {
        let robyn = method_b_i(
            year,
            "1200.00",
            ["1100.00", "100.00", "175.00"],
            8,
            "weekly",
        );
        assert_eq!(
            amounts(&run(B_I, &robyn), B_I_KEYS),
            ["800.00", "264.00", "208.00", "472.00"],
            "{year}"
        );
    }
}

/// Example 3 (Laura, 2026): $1,800 a fortnight, $18,000 to date over 10 pays,
/// $2,480 of arrears: $1,800 gives $212, $95 a period, $1,895 gives $242, $30 x 26
/// = $780, limit $1,165, $780 on the arrears and $992 in all. The 2024 and 2025
/// schedules give $222, $252, $780 and $1,002.
#[test]
fn laura_method_b_ii_gives_the_ato_example_3() {
    let laura = |year| method_b_ii(year, "1800.00", "2480.00", ("18000.00", 10), "fortnightly");
    assert_eq!(
        amounts(&run(B_II, &laura("2026-27")), B_II_KEYS),
        [
            "1800.00", "212.00", "95.00", "242.00", "780.00", "1165.60", "780.00", "212.00",
            "992.00"
        ]
    );
    for year in ["2024-25", "2025-26"] {
        assert_eq!(
            amounts(&run(B_II, &laura(year)), B_II_KEYS),
            [
                "1800.00", "222.00", "95.00", "252.00", "780.00", "1165.60", "780.00", "222.00",
                "1002.00"
            ],
            "{year}"
        );
    }
}

/// Example 4 (Bradley, 2026): monthly salary $6,200, then $7,000, backdated
/// $10,400: $1,600 for 2 periods this year by B(i), $8,800 for last year by B(ii).
/// PAYG: B(i) $1,452 less $1,196 = $256 x 2 = $512; B(ii) average $7,000 gives
/// $1,452, $733 a period, $7,733 gives $1,686, $234 x 12 = $2,808. FS: B(i)
/// $182 less $61 = $121 x 2 = $242; B(ii) $290 less $182 = $108 x 12 = $1,296.
/// Limit $4,136; earnings $1,452 + $182; $6,492 in all.
#[test]
fn bradley_methods_b_i_and_b_ii_give_the_ato_example_4() {
    let back = method_b_i(
        "2026-27",
        "7000.00",
        ["6200.00", "800.00", "1196.00"],
        2,
        "monthly",
    );
    let prior = method_b_ii("2026-27", "7000.00", "8800.00", ("21000.00", 3), "monthly");
    assert_eq!(
        amounts(&run(B_I, &back), B_I_KEYS),
        ["1600.00", "512.00", "1452.00", "1964.00"]
    );
    let payg = run(B_II, &prior);
    assert_eq!(payg["amounts"]["withholding_for_the_year"], "2808.00");
    assert_eq!(
        payg["amounts"]["withholding_on_additional_payment"],
        "2808.00"
    );

    // With FS, each earlier period had $1,196 + $61 withheld.
    let back = with(
        method_b_i(
            "2026-27",
            "7000.00",
            ["6200.00", "800.00", "1257.00"],
            2,
            "monthly",
        ),
        study_loan(),
    );
    let back = run(B_I, &back);
    let prior = run(B_II, &with(prior, study_loan()));
    assert_eq!(
        amounts(&back, B_I_KEYS),
        ["1600.00", "754.00", "1634.00", "2388.00"]
    );
    assert_eq!(
        amounts(&prior, B_II_KEYS),
        [
            "7000.00", "1634.00", "733.00", "1976.00", "4104.00", "4136.00", "4104.00", "1634.00",
            "5738.00"
        ]
    );
    // Both methods in one pay count the earnings' amount once.
    let [earnings, on_back] =
        ["withholding_on_earnings", "withholding_on_back_payment"].map(|key| {
            back["amounts"][key]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap()
        });
    let on_prior = prior["amounts"]["withholding_on_additional_payment"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert_eq!(earnings + on_back + on_prior, 6492.0);
}

/// The 2024 and 2025 schedules' Bradley: $4,600 a month, then $5,400. PAYG:
/// B(i) $962 less $706 = $256 x 2 = $512; B(ii) average $5,400 gives $962, $733,
/// $6,133 gives $1,200, $238 x 12 = $2,856.
#[test]
fn bradley_gives_the_earlier_schedules_payg_figures() {
    for year in ["2024-25", "2025-26"] {
        let back = method_b_i(
            year,
            "5400.00",
            ["4600.00", "800.00", "706.00"],
            2,
            "monthly",
        );
        assert_eq!(
            amounts(&run(B_I, &back), B_I_KEYS),
            ["1600.00", "512.00", "962.00", "1474.00"],
            "{year}"
        );
        let prior = method_b_ii(year, "5400.00", "8800.00", ("16200.00", 3), "monthly");
        let prior = run(B_II, &prior);
        assert_eq!(
            amounts(
                &prior,
                [
                    "withholding_on_average_and_share",
                    "withholding_for_the_year"
                ]
            ),
            ["1200.00", "2856.00"],
            "{year}"
        );
    }
}

/// Step 1 ignores the cents of the earnings, so a monthly amount ending in 33
/// cents does not gain the cent Schedule 1 adds: $4,346.33 is worked as $4,346
/// ($598), not as Schedule 1 works $4,346.33 ($602).
#[test]
fn method_a_ignores_the_cents_of_the_earnings_before_the_table() {
    let pay = run(A, &method_a("2026-27", "4346.33", "1200.00", 12, "monthly"));
    assert_eq!(
        amounts(&pay, ["earnings_used", "withholding_on_earnings"]),
        ["4346.00", "598.00"]
    );
    let schedule_1 = |earnings: &str| {
        let pay = json!({"earnings": earnings, "pay_period": "monthly", "scale": 2,
                         "year": "2026-27", "scope_confirmed": true});
        run("payg-withholding", &pay)["amounts"]["withholding"].clone()
    };
    assert_eq!(schedule_1("4346.00"), "598.00");
    assert_eq!(schedule_1("4346.33"), "602.00");
}

/// Step 3 ignores the cents of the share: $884 over 52 is $17, $883.99 is $16.
#[test]
fn method_a_ignores_the_cents_of_the_share() {
    let share = |bonus| {
        let pay = run(A, &method_a("2026-27", "1400.00", bonus, 52, "weekly"));
        pay["amounts"]["additional_payment_per_period"].clone()
    };
    assert_eq!(share("884.00"), "17.00");
    assert_eq!(share("883.99"), "16.00");
}

/// A commission for a defined period of less than 12 months may be divided by
/// the pay periods it relates to and the difference multiplied by the same:
/// $400 for 4 weeks on $1,400 a week is $100 a period, $1,500 gives $299, $32 x 4.
/// One pay period is refused, as is more than a year's 52, 26 or 12.
#[test]
fn method_a_spreads_over_the_pay_periods_given_from_2_to_a_year() {
    let pay = run(A, &method_a("2026-27", "1400.00", "400.00", 4, "weekly"));
    assert_eq!(
        amounts(&pay, A_KEYS),
        [
            "1400.00", "267.00", "100.00", "299.00", "128.00", "188.00", "128.00", "395.00"
        ]
    );
    assert_eq!(pay["rates"]["pay_periods"], "4");
    for (periods, pay, accepted) in [
        (json!(1), "weekly", false),
        (json!(2), "weekly", true),
        (json!(52), "weekly", true),
        (json!(53), "weekly", false),
        (json!(26), "fortnightly", true),
        (json!(27), "fortnightly", false),
        (json!(12), "monthly", true),
        (json!(13), "monthly", false),
        (json!("52"), "weekly", false),
        (json!(52.0), "weekly", false),
    ] {
        let facts = with(
            method_a("2026-27", "1400.00", "900.00", 2, pay),
            json!({"pay_periods": periods}),
        );
        if accepted {
            run(A, &facts);
        } else {
            assert_eq!(
                refusal(A, &facts),
                "input.pay_periods_invalid",
                "{periods} {pay}"
            );
        }
    }
}

/// The 47% limit binds when the marginal rate with the study loan component
/// exceeds it: $5,200.99 on $4,000 a week gives $57 x 52 = $2,964 before the
/// limit of $2,444.47, which becomes $2,444 with cents ignored.
#[test]
fn the_47_percent_limit_binds_with_cents_ignored() {
    let pay = with(
        method_a("2026-27", "4000.00", "5200.99", 52, "weekly"),
        study_loan(),
    );
    assert_eq!(
        amounts(&run(A, &pay), A_KEYS),
        [
            "4000.00", "1625.00", "100.00", "1682.00", "2964.00", "2444.47", "2444.00", "4069.00"
        ]
    );
    let average = with(
        method_b_ii("2026-27", "4000.00", "5200.99", ("40000.00", 10), "weekly"),
        study_loan(),
    );
    assert_eq!(
        amounts(
            &run(B_II, &average),
            [
                "withholding_for_the_year",
                "withholding_limit",
                "withholding_on_additional_payment"
            ]
        ),
        ["2964.00", "2444.47", "2444.00"]
    );
}

/// Method B(i) has no 47% limit; a negative total, when more was withheld
/// than the recalculation gives, is nil.
#[test]
fn method_b_i_takes_a_negative_total_as_nil() {
    // $1,600 a week gives $331; $400 was withheld, and the back payment is nil.
    let over = method_b_i(
        "2026-27",
        "1700.00",
        ["1600.00", "0.00", "400.00"],
        1,
        "weekly",
    );
    assert_eq!(
        amounts(&run(B_I, &over), B_I_KEYS),
        ["0.00", "0.00", "363.00", "363.00"]
    );
    // Periods net: $32 owed on one, $69 over on the other, so nil.
    let mut netted = method_b_i(
        "2026-27",
        "1700.00",
        ["1600.00", "100.00", "331.00"],
        1,
        "weekly",
    );
    netted["back_payment_periods"]
        .as_array_mut()
        .unwrap()
        .push(json!({"earnings": "1600.00", "back_payment": "0.00", "withheld": "400.00"}));
    assert_eq!(
        run(B_I, &netted)["amounts"]["withholding_on_back_payment"],
        "0.00"
    );
    // In either order: no period's difference is taken as nil on its own.
    netted["back_payment_periods"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert_eq!(
        run(B_I, &netted)["amounts"]["withholding_on_back_payment"],
        "0.00"
    );
}

/// At most 52 weekly, 26 fortnightly or 11 monthly earlier periods, and each
/// must be an object of exactly 3 decimal strings.
#[test]
fn method_b_i_refuses_a_malformed_list_of_periods() {
    let period = ["1600.00", "100.00", "331.00"];
    for (count, pay, accepted) in [
        (0, "weekly", false),
        (52, "weekly", true),
        (53, "weekly", false),
        (26, "fortnightly", true),
        (27, "fortnightly", false),
        (11, "monthly", true),
        (12, "monthly", false),
    ] {
        let facts = method_b_i("2026-27", "1700.00", period, count, pay);
        if accepted {
            run(B_I, &facts);
        } else {
            assert_eq!(
                refusal(B_I, &facts),
                "input.back_payment_periods_invalid",
                "{count} {pay}"
            );
        }
    }
    for periods in [
        json!({"earnings": "1600.00"}),
        json!([{"earnings": "1600.00", "back_payment": "100.00"}]),
        json!([{"earnings": "1600.00", "back_payment": "100.00", "withheld": "331.00", "date": "2026-07-07"}]),
        json!([{"earnings": 1600, "back_payment": "100.00", "withheld": "331.00"}]),
        json!([{"earnings": "1,600", "back_payment": "100.00", "withheld": "331.00"}]),
        json!(["1600.00"]),
    ] {
        let facts = with(
            method_b_i("2026-27", "1700.00", period, 1, "weekly"),
            json!({"back_payment_periods": periods}),
        );
        assert_eq!(
            refusal(B_I, &facts),
            "input.back_payment_periods_invalid",
            "{periods}"
        );
    }
    let negative = method_b_i(
        "2026-27",
        "1700.00",
        ["1600.00", "-100.00", "331.00"],
        1,
        "weekly",
    );
    assert_eq!(refusal(B_I, &negative), "input.amount_invalid");
}

/// Step 1 of Method B(ii) ignores the cents of the average.
#[test]
fn method_b_ii_ignores_the_cents_of_the_average() {
    let average = |to_date| {
        let pay = method_b_ii(
            "2026-27",
            "1800.00",
            "2480.00",
            (to_date, 10),
            "fortnightly",
        );
        run(B_II, &pay)["amounts"]["average_total_earnings"].clone()
    };
    assert_eq!(average("18009.99"), "1800.00");
    assert_eq!(average("18010.00"), "1801.00");
}

/// Earlier Method B(ii) payments in the year join this one at step 3, and
/// what was withheld from them comes off at step 8: Laura's $2,480 paid twice
/// gives $190 a period, $1,990 gives $272, $60 x 26 = $1,560, less the $780
/// withheld before. More withheld before than that gives nil.
#[test]
fn method_b_ii_counts_earlier_payments_and_their_withholding() {
    let laura = method_b_ii(
        "2026-27",
        "1800.00",
        "2480.00",
        ("18000.00", 10),
        "fortnightly",
    );
    let again = with(
        laura.clone(),
        json!({"earlier_additional_payments": "2480.00", "earlier_additional_withholding": "780.00"}),
    );
    assert_eq!(
        amounts(&run(B_II, &again), B_II_KEYS),
        [
            "1800.00", "212.00", "190.00", "272.00", "1560.00", "1165.60", "780.00", "212.00",
            "992.00"
        ]
    );
    let over = with(laura, json!({"earlier_additional_withholding": "1000.00"}));
    assert_eq!(
        run(B_II, &over)["amounts"]["withholding_on_additional_payment"],
        "0.00"
    );
}

/// Pay periods to date run from 1 to the most a year holds: 53 weekly, 27
/// fortnightly or 12 monthly. A year of 53 or 27 pays still divides the
/// additional payments by 52 or 26: $2,480 is $47 or $95 a period.
#[test]
fn method_b_ii_takes_pay_periods_to_date_up_to_a_years_pays() {
    for (pays, pay, share) in [
        (0, "weekly", None),
        (1, "weekly", Some("47.00")),
        (53, "weekly", Some("47.00")),
        (54, "weekly", None),
        (27, "fortnightly", Some("95.00")),
        (28, "fortnightly", None),
        (12, "monthly", Some("206.00")),
        (13, "monthly", None),
    ] {
        let facts = method_b_ii("2026-27", "1800.00", "2480.00", ("18000.00", pays), pay);
        if let Some(share) = share {
            assert_eq!(
                run(B_II, &facts)["amounts"]["additional_payments_per_period"],
                share,
                "{pays} {pay}"
            );
        } else {
            assert_eq!(
                refusal(B_II, &facts),
                "input.pay_periods_invalid",
                "{pays} {pay}"
            );
        }
    }
}

#[test]
fn every_method_refuses_what_it_does_not_cover() {
    let cases = [
        (A, method_a("2026-27", "1400.00", "900.00", 52, "weekly")),
        (
            B_I,
            method_b_i(
                "2026-27",
                "1700.00",
                ["1600.00", "100.00", "331.00"],
                8,
                "weekly",
            ),
        ),
        (
            B_II,
            method_b_ii(
                "2026-27",
                "1800.00",
                "2480.00",
                ("18000.00", 10),
                "fortnightly",
            ),
        ),
    ];
    for (command, facts) in cases {
        for (overrides, code) in [
            (json!({"scope_confirmed": false}), "scope.not_confirmed"),
            (json!({"year": "2027-28"}), "period.unsupported"),
            (json!({"year": "2023-24"}), "period.unsupported"),
            (json!({"scale": 4}), "input.tax_scale_unsupported"),
            (
                json!({"pay_period": "quarterly"}),
                "input.pay_period_invalid",
            ),
            (json!({"study_loan": "yes"}), "input.flag_invalid"),
            (json!({"earnings": "1400.001"}), "input.amount_invalid"),
            (
                json!({"study_loan": true, "year": "2025-26"}),
                "period.unsupported",
            ),
            // A field the method does not take is refused, never ignored.
            (json!({"back_payment": "100.00"}), "input.field_unknown"),
            (json!({"defined_period": true}), "input.field_unknown"),
        ] {
            assert_eq!(
                refusal(command, &with(facts.clone(), overrides.clone())),
                code,
                "{command} {overrides}"
            );
        }
    }
}

/// Each catalogue example runs as given and is the ATO's example 1, 2 or 3.
#[test]
fn the_catalogue_examples_are_the_ato_examples() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    for (command, kind, total) in [
        (A, "schedule_5_method_a", "641.00"),
        (B_I, "schedule_5_method_b_i", "794.00"),
        (B_II, "schedule_5_method_b_ii", "992.00"),
    ] {
        let entry = &catalogue[kind];
        let mut facts = entry["example"]["facts"].clone();
        facts.as_object_mut().unwrap().remove("kind");
        assert_eq!(
            run(command, &facts)["amounts"]["withholding"],
            total,
            "{kind}"
        );
        let periods = entry["supported_periods"].as_array().unwrap();
        let ids: Vec<&str> = periods.iter().map(|p| p["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["2024-25", "2025-26", "2026-27"], "{kind}");
        assert_eq!(entry["source_checked"], "2026-11-14", "{kind}");
    }
}
