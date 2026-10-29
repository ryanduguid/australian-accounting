//! Period, input and example metadata owned by the worksheet engine, ported
//! from `austaxcalc.metadata` without changing a rate, threshold or wording.
// Nested literal tables mirror the original's nested dicts and tuples.
#![allow(clippy::type_complexity)]

/// One row of a coefficient table: weekly earnings upper limit (None is no
/// limit), a and b for y = a * x - b.
pub type CoefficientRow = (Option<i64>, &'static str, &'static str);

/// Resident basic tax scale per income year: (lower bound, upper bound, rate).
/// Income Tax Rates Act 1986, Schedule 7 Part I clause 1.
pub const RESIDENT_TAX_SCALES: &[(&str, &[(i64, Option<i64>, &str)])] = &[
    (
        "2024-25",
        &[
            (18200, Some(45000), "0.16"),
            (45000, Some(135000), "0.30"),
            (135000, Some(190000), "0.37"),
            (190000, None, "0.45"),
        ],
    ),
    (
        "2025-26",
        &[
            (18200, Some(45000), "0.16"),
            (45000, Some(135000), "0.30"),
            (135000, Some(190000), "0.37"),
            (190000, None, "0.45"),
        ],
    ),
    (
        "2026-27",
        &[
            (18200, Some(45000), "0.15"),
            (45000, Some(135000), "0.30"),
            (135000, Some(190000), "0.37"),
            (190000, None, "0.45"),
        ],
    ),
];

/// Schedule 1 (NAT 1004) weekly coefficients for payments from 1 July 2026.
pub const PAYG_WITHHOLDING_COEFFICIENTS: &[(&str, &[(i64, &[CoefficientRow])])] = &[(
    "2026-27",
    &[
        (
            1,
            &[
                (Some(188), "0.1500", "0.1500"),
                (Some(371), "0.2084", "11.0185"),
                (Some(515), "0.1790", "0.1066"),
                (Some(932), "0.3227", "74.1674"),
                (Some(2246), "0.3200", "71.6508"),
                (Some(3303), "0.3900", "228.8816"),
                (None, "0.4700", "493.1893"),
            ],
        ),
        (
            2,
            &[
                (Some(362), "0", "0"),
                (Some(538), "0.1500", "54.3462"),
                (Some(673), "0.2500", "108.2135"),
                (Some(721), "0.1700", "54.3473"),
                (Some(865), "0.1790", "60.8377"),
                (Some(1282), "0.3227", "185.1935"),
                (Some(2596), "0.3200", "181.7319"),
                (Some(3653), "0.3900", "363.4627"),
                (None, "0.4700", "655.7704"),
            ],
        ),
        (
            3,
            &[
                (Some(2596), "0.3000", "0.3000"),
                (Some(3653), "0.3700", "181.7308"),
                (None, "0.4500", "474.0385"),
            ],
        ),
        (
            5,
            &[
                (Some(362), "0", "0"),
                (Some(721), "0.1500", "54.3462"),
                (Some(865), "0.1590", "60.8365"),
                (Some(1282), "0.3027", "185.1923"),
                (Some(2596), "0.3000", "181.7308"),
                (Some(3653), "0.3700", "363.4615"),
                (None, "0.4500", "655.7692"),
            ],
        ),
        (
            6,
            &[
                (Some(362), "0", "0"),
                (Some(721), "0.1500", "54.3462"),
                (Some(865), "0.1590", "60.8365"),
                (Some(908), "0.3027", "185.1923"),
                (Some(1135), "0.3527", "230.6135"),
                (Some(1282), "0.3127", "185.1923"),
                (Some(2596), "0.3100", "181.7308"),
                (Some(3653), "0.3800", "363.4615"),
                (None, "0.4600", "655.7692"),
            ],
        ),
    ],
)];

/// Schedule 8 (NAT 3539) study and training support loan component coefficients.
pub const STUDY_LOAN_WITHHOLDING_COEFFICIENTS: &[(&str, &[(&str, &[CoefficientRow])])] = &[(
    "2026-27",
    &[
        (
            "threshold_or_foreign",
            &[
                (Some(1337), "0", "0"),
                (Some(2494), "0.15", "200.5615"),
                (Some(3577), "0.17", "250.4527"),
                (None, "0.10", "0"),
            ],
        ),
        (
            "no_threshold",
            &[
                (Some(987), "0", "0"),
                (Some(2144), "0.15", "148.0615"),
                (Some(2727), "0.17", "190.9527"),
                (None, "0.10", "0"),
            ],
        ),
    ],
)];

/// (general concessional cap, general transfer balance cap) per income year.
pub const CONTRIBUTION_CAPS: &[(&str, (&str, &str))] = &[
    ("2024-25", ("30000", "1900000")),
    ("2025-26", ("30000", "2000000")),
    ("2026-27", ("32500", "2100000")),
];

/// Study loan repayment thresholds: minimum, step, printed base, flat-from.
pub const STUDY_LOAN_REPAYMENT_THRESHOLDS: &[(&str, [(&str, &str); 4])] = &[
    (
        "2025-26",
        [
            ("minimum", "67000"),
            ("step", "125000"),
            ("printed_base", "8700"),
            ("flat_from", "179286"),
        ],
    ),
    (
        "2026-27",
        [
            ("minimum", "69528"),
            ("step", "129717"),
            ("printed_base", "9028"),
            ("flat_from", "186051"),
        ],
    ),
];
pub const STUDY_LOAN_REPAYMENT_RATES: [(&str, &str); 3] =
    [("lower", "0.15"), ("upper", "0.17"), ("flat", "0.10")];

/// Medicare levy (threshold amount, phase-in limit) for a single person.
pub const MEDICARE_LOW_INCOME_THRESHOLDS: &[(&str, (&str, &str))] = &[
    ("2024-25", ("27222", "34027")),
    ("2025-26", ("28011", "35013")),
];
pub const MEDICARE_LEVY_RATES: [(&str, &str); 2] = [("levy", "0.02"), ("shade_in", "0.10")];

/// ITAA 1997 s 61-115(1) table, in the Python dict's order.
pub const LOW_INCOME_TAX_OFFSET: [(&str, &str); 7] = [
    ("maximum", "700"),
    ("first_step", "37500"),
    ("first_rate", "0.05"),
    ("second_step", "45000"),
    ("second_amount", "325"),
    ("second_rate", "0.015"),
    ("cut_out", "66667"),
];
pub const LOW_INCOME_TAX_OFFSET_YEARS: [&str; 3] = ["2024-25", "2025-26", "2026-27"];

pub const DIVISION_293: [(&str, &str); 2] = [("threshold", "250000"), ("rate", "0.15")];
pub const DIVISION_293_YEARS: [&str; 3] = ["2024-25", "2025-26", "2026-27"];

pub const CARRY_FORWARD_BALANCE_LIMIT: &str = "500000";

/// Statutory or benchmark interest rate by FBT year ended 31 March, for the
/// car operating cost method's deemed interest. Its keys are both car
/// worksheets' supported years; the 20% statutory rate and the 25% deemed
/// depreciation rate apply in each.
pub const FBT_BENCHMARK_INTEREST_RATES: &[(&str, &str)] = &[("2026", "0.0862"), ("2027", "0.0827")];

/// SISR Schedule 7 factors as (lowest age, factor), oldest band first.
pub const PENSION_MINIMUM_FACTORS: [(i64, &str); 7] = [
    (95, "0.14"),
    (90, "0.11"),
    (85, "0.09"),
    (80, "0.07"),
    (75, "0.06"),
    (65, "0.05"),
    (0, "0.04"),
];

fn keys<T>(table: &[(&'static str, T)]) -> Vec<&'static str> {
    table.iter().map(|(k, _)| *k).collect()
}

/// `SUPPORTED_PERIODS[kind]`.
pub fn supported_periods(kind: &str) -> Vec<&'static str> {
    match kind {
        "gst" | "capital_gains" | "depreciation" | "quarterly_sg" => vec!["2025-26"],
        "resident_tax" => keys(RESIDENT_TAX_SCALES),
        "fbt" => vec!["2026"],
        "fbt_car_statutory" | "fbt_car_operating_cost" => keys(FBT_BENCHMARK_INTEREST_RATES),
        "payg_withholding" => keys(PAYG_WITHHOLDING_COEFFICIENTS),
        "contribution_caps" => keys(CONTRIBUTION_CAPS),
        "pension_minimum" => vec!["2024-25", "2025-26", "2026-27"],
        "study_loan_repayment" => keys(STUDY_LOAN_REPAYMENT_THRESHOLDS),
        "study_loan_withholding" => keys(STUDY_LOAN_WITHHOLDING_COEFFICIENTS),
        "medicare_levy" => keys(MEDICARE_LOW_INCOME_THRESHOLDS),
        "low_income_tax_offset" => LOW_INCOME_TAX_OFFSET_YEARS.to_vec(),
        "division_293" => DIVISION_293_YEARS.to_vec(),
        _ => Vec::new(),
    }
}

/// `SOURCE_REVIEWS[kind]`: (checked, passage).
pub fn source_review(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "gst" => ("2026-09-10", "GST fast facts: Charging GST"),
        "resident_tax" => (
            "2026-09-10",
            "Income Tax Rates Act 1986, Schedule 7 Part I clause 1",
        ),
        "capital_gains" => (
            "2026-09-10",
            "Capital gains tax guide 2014, step 6: Applying current year capital losses",
        ),
        "fbt" => (
            "2026-09-27",
            "Calculating your FBT, steps 3 to 7 and the MantCo example; FBT return 2026 instructions, items 14A to 16",
        ),
        "fbt_car_statutory" => (
            "2026-10-29",
            "Fringe benefits tax - a guide for employers, 7.8 statutory formula method and examples 23 and 26; FBT return 2026 instructions, item 23A and example 6; Electric cars exemption, reportable fringe benefits",
        ),
        "fbt_car_operating_cost" => (
            "2026-10-29",
            "Fringe benefits tax - a guide for employers, 7.9 operating cost method and example 31; FBT return 2026 instructions, item 23B and example 7; FBT rates and thresholds, benchmark interest rate; Electric cars exemption, reportable fringe benefits",
        ),
        "depreciation" => (
            "2026-09-24",
            "Decline in value: diminishing value and prime cost methods; taxable use",
        ),
        "quarterly_sg" => (
            "2026-09-10",
            "Super guarantee, tables 21 and 24, 2025-26 rows",
        ),
        "payg_withholding" => (
            "2026-09-24",
            "Schedule 1 (NAT 1004) for payments from 1 July 2026: weekly coefficients, using a formula, working out the weekly earnings",
        ),
        "contribution_caps" => (
            "2026-09-24",
            "Contributions caps, tables 1.1 and 4 and unused concessional cap carry forward; Non-concessional contributions cap, bring-forward arrangement",
        ),
        "pension_minimum" => (
            "2026-09-24",
            "Income stream (pension) rules and payments: how to calculate the minimum annual payment; SISR Schedule 7 clauses 1 to 5",
        ),
        "study_loan_repayment" => (
            "2026-10-20",
            "Study and training loan minimum repayment income and rates: tables 1 and 2, repayment income, examples 1 to 3; Gazette C2026G00249 for the 2026-27 minimum repayment income and replacement indexable amount",
        ),
        "study_loan_withholding" => (
            "2026-10-15",
            "Schedule 8 (NAT 3539) for payments from 1 July 2026: study and training support loans component rates, calculating fortnightly and monthly amounts, examples 1 to 3, rounding",
        ),
        "medicare_levy" => (
            "2026-10-15",
            "Medicare Levy Act 1986 ss 3(1), 6(1) and 7(1) and (2), compilation 53; ATO Medicare levy reduction for low-income earners (2025-26) and myTax 2025 table 2 (2024-25)",
        ),
        "low_income_tax_offset" => (
            "2026-10-15",
            "ITAA 1997 ss 61-110 and 61-115, compilation 266 (1 July 2026); ATO Low income tax offset page",
        ),
        "division_293" => (
            "2026-10-15",
            "ITAA 1997 ss 293-15 to 293-25, compilation 266 (1 July 2026); Imposition Act 2013 s 5; ATO Key superannuation rates, table 7",
        ),
        _ => ("", ""),
    }
}

/// `LIBRARY_EVIDENCE[kind]`: document, paragraphs, reviewed, checked.
pub fn library_evidence(kind: &str) -> [(&'static str, &'static str); 4] {
    let (document, paragraphs, reviewed, checked) = match kind {
        "gst" => (
            "tax-examples-gst-and-other-indirect-taxes",
            "12-020",
            "2026-06-30",
            "2026-09-15",
        ),
        "resident_tax" => (
            "tax-examples-individuals",
            "7-010",
            "2025-06-30",
            "2026-09-15",
        ),
        "capital_gains" => (
            "tax-examples-capital-gains-tax-cgt",
            "2-040, 2-240",
            "2026-06-30",
            "2026-09-15",
        ),
        "fbt" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-000, 3-020",
            "2026-06-30",
            "2026-09-15",
        ),
        "fbt_car_statutory" | "fbt_car_operating_cost" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-020",
            "2026-06-30",
            "2026-10-29",
        ),
        "depreciation" => (
            "tax-examples-depreciation",
            "6-000, 6-020",
            "2025-06-30",
            "2026-09-15",
        ),
        "quarterly_sg" => (
            "superannuation-instant-reference-rates-thresholds-and-checklists",
            "18-600, 18-620",
            "2026-06-30",
            "2026-09-15",
        ),
        "payg_withholding" => (
            "ATO Schedule 1 withholding amounts sample data",
            "weekly, fortnightly and monthly tables",
            "2026-06-17",
            "2026-09-24",
        ),
        "contribution_caps" => (
            "tax-examples-individuals",
            "7-278",
            "2025-06-30",
            "2026-09-24",
        ),
        "pension_minimum" => (
            "superannuation-instant-reference-rates-thresholds-and-checklists",
            "18-500",
            "2026-06-30",
            "2026-09-24",
        ),
        "study_loan_repayment" => (
            "ATO study and training loan minimum repayment income and rates",
            "tables 1 and 2, examples 1 to 3",
            "2026-10-13",
            "2026-10-20",
        ),
        "study_loan_withholding" => (
            "ATO Schedule 8 statement of formulas for STSL components",
            "component rates, examples 1 to 3",
            "2026-06-17",
            "2026-10-15",
        ),
        "medicare_levy" => (
            "ATO Medicare levy reduction for low-income earners",
            "table of single thresholds, example (Angie)",
            "2026-06-30",
            "2026-10-15",
        ),
        "low_income_tax_offset" => (
            "ATO low and middle income earner tax offsets",
            "example: taxable income over $37,000 but under $48,000",
            "2025-06-16",
            "2026-10-15",
        ),
        "division_293" => (
            "ATO Division 293 tax on concessional contributions by high-income earners",
            "example: Division 293 tax calculation",
            "2026-08-24",
            "2026-10-15",
        ),
        _ => ("", "", "", ""),
    };
    [
        ("document", document),
        ("paragraphs", paragraphs),
        ("reviewed", reviewed),
        ("checked", checked),
    ]
}

/// `INPUT_UNITS[kind]`, in the Python dict's order.
pub fn input_units(kind: &str) -> Vec<(&'static str, &'static str)> {
    let payg = vec![
        (
            "earnings",
            "AUD, gross earnings and allowances subject to withholding for the pay period",
        ),
        ("pay_period", "weekly, fortnightly or monthly"),
        ("scale", "integer 1, 2, 3, 5 or 6"),
    ];
    match kind {
        "gst" => vec![("amount", "AUD"), ("gst_inclusive", "boolean")],
        "resident_tax" => vec![("taxable_income", "AUD, whole dollars")],
        "capital_gains" => vec![
            ("other_gains", "AUD"),
            ("discount_gains", "AUD"),
            ("current_losses", "AUD"),
            ("prior_losses", "AUD"),
        ],
        "fbt" => vec![("type_one_value", "AUD"), ("type_two_value", "AUD")],
        "fbt_car_statutory" => vec![
            (
                "cost_price",
                "AUD, the part of the base value the one-third reduction applies to: for a car the provider or an associate owned at the earliest holding time, its cost price with dealer delivery and accessories fitted at acquisition; otherwise its leased car value then, the lessor's cost price only if the lease began then and the lessor bought the car at or about that time, else its arm's-length purchase price",
            ),
            (
                "later_accessories",
                "AUD, non-business accessories fitted after the earliest holding time and still fitted in the year",
            ),
            ("one_third_reduction", "boolean"),
            (
                "days_available",
                "integer from 0 to the days in the FBT year",
            ),
            ("employee_contribution", "AUD"),
            ("electric_car_exempt", "boolean"),
        ],
        "fbt_car_operating_cost" => vec![
            (
                "running_costs",
                "AUD, GST-inclusive car expenses for the holding period under FBTAA s 10(3), including those the employee paid: repairs, maintenance and fuel, and registration and insurance attributable to the period; excluding insured repair expenses, an owned car's registration and insurance incurred before the provider owned it, and expenses the lessor bears under the lease",
            ),
            (
                "lease_charges",
                "AUD, for the holding period; 0 for an owned car",
            ),
            (
                "depreciated_value",
                "AUD, an owned car's FBTAA s 12 depreciated value at the start of the FBT year (cost price less earlier FBT deemed depreciation, not the income tax value), or its cost price if acquired in the year; a hire purchase car is owned; 0 for a leased car",
            ),
            (
                "days_held",
                "integer, the days in the holding period the car was owned or leased, from 0 to the days in the FBT year",
            ),
            ("business_use", "decimal fraction from 0 to 1"),
            ("employee_contribution", "AUD"),
            (
                "statutory_formula_taxable_value",
                "AUD, the same car's s 9 value for the same holding period: fbt_car_statutory's taxable_value_before_exemption",
            ),
            ("electric_car_exempt", "boolean"),
        ],
        "depreciation" => vec![
            ("cost", "AUD"),
            ("effective_life", "years, decimal string from 0.01 to 1000"),
            ("days", "integer from 0 to 365"),
            ("taxable_use", "fraction, decimal string from 0 to 1"),
            ("method", "method name"),
        ],
        "quarterly_sg" => vec![
            ("ordinary_time_earnings", "AUD"),
            ("qualifying_contributions", "AUD"),
            ("quarter", "integer from 1 to 4"),
        ],
        "payg_withholding" | "study_loan_withholding" => payg,
        "contribution_caps" => vec![
            (
                "total_super_balance",
                "AUD, total super balance at 30 June before the income year",
            ),
            ("concessional_contributions", "AUD"),
            (
                "unused_concessional_cap",
                "AUD, unexpired unused amounts from the previous 5 years",
            ),
            ("non_concessional_contributions", "AUD"),
            (
                "under_75_in_year",
                "boolean, under 75 at any time in the income year",
            ),
        ],
        "pension_minimum" => vec![
            (
                "account_balance",
                "AUD, at 1 July or on the commencement day in the first year",
            ),
            (
                "age",
                "integer years on the same day as the account balance",
            ),
            (
                "days",
                "integer days in the income year from and including the commencement day, or all days in the year for a pension running on 1 July",
            ),
        ],
        "study_loan_repayment" => vec![(
            "repayment_income",
            "AUD, whole dollars, established repayment income for the year",
        )],
        "medicare_levy" => vec![
            ("taxable_income", "AUD, whole dollars"),
            (
                "section_9a_adjustment_applies",
                "boolean, whether Medicare Levy Act 1986 s 9A removes lump sums in arrears; true is refused",
            ),
        ],
        "low_income_tax_offset" => vec![("taxable_income", "AUD, whole dollars")],
        "division_293" => vec![
            (
                "division_293_income",
                "AUD, income for surcharge purposes disregarding reportable super contributions",
            ),
            (
                "low_tax_contributions",
                "AUD, low tax contributed amounts less excess concessional contributions for the financial year",
            ),
        ],
        _ => Vec::new(),
    }
}

/// An example fact value: a string, a boolean or an integer.
#[derive(Clone, Copy)]
pub enum Example {
    Text(&'static str),
    Flag(bool),
    Whole(i64),
}

/// `EXAMPLES[kind]`, in the Python dict's order.
pub fn examples(kind: &str) -> Vec<(&'static str, Example)> {
    use Example::{Flag, Text, Whole};
    match kind {
        "gst" => vec![("amount", Text("1100.00")), ("gst_inclusive", Flag(true))],
        "resident_tax" => vec![("taxable_income", Text("45000"))],
        "capital_gains" => vec![
            ("other_gains", Text("100.00")),
            ("discount_gains", Text("1000.00")),
            ("current_losses", Text("200.00")),
            ("prior_losses", Text("300.00")),
        ],
        "fbt" => vec![
            ("type_one_value", Text("1000.00")),
            ("type_two_value", Text("1000.00")),
        ],
        "fbt_car_statutory" => vec![
            ("cost_price", Text("32000.00")),
            ("later_accessories", Text("0.00")),
            ("one_third_reduction", Flag(false)),
            ("days_available", Whole(274)),
            ("employee_contribution", Text("0.00")),
            ("electric_car_exempt", Flag(false)),
        ],
        "fbt_car_operating_cost" => vec![
            ("running_costs", Text("5200.00")),
            ("lease_charges", Text("0.00")),
            ("depreciated_value", Text("50000.00")),
            ("days_held", Whole(365)),
            ("business_use", Text("0.3636")),
            ("employee_contribution", Text("1500.00")),
            ("statutory_formula_taxable_value", Text("8500.00")),
            ("electric_car_exempt", Flag(false)),
        ],
        "depreciation" => vec![
            ("cost", Text("3000.00")),
            ("effective_life", Text("4")),
            ("days", Whole(146)),
            ("taxable_use", Text("0.4")),
            ("method", Text("prime_cost")),
        ],
        "quarterly_sg" => vec![
            ("ordinary_time_earnings", Text("80000.00")),
            ("qualifying_contributions", Text("2000.00")),
            ("quarter", Whole(1)),
        ],
        "payg_withholding" => vec![
            ("earnings", Text("1000.00")),
            ("pay_period", Text("weekly")),
            ("scale", Whole(2)),
        ],
        "contribution_caps" => vec![
            ("total_super_balance", Text("450000.00")),
            ("concessional_contributions", Text("40000.00")),
            ("unused_concessional_cap", Text("15000.00")),
            ("non_concessional_contributions", Text("150000.00")),
            ("under_75_in_year", Flag(true)),
        ],
        "pension_minimum" => vec![
            ("account_balance", Text("600000.00")),
            ("age", Whole(67)),
            ("days", Whole(365)),
        ],
        "study_loan_repayment" => vec![("repayment_income", Text("86380"))],
        "study_loan_withholding" => vec![
            ("earnings", Text("2000.00")),
            ("pay_period", Text("weekly")),
            ("scale", Whole(2)),
        ],
        "medicare_levy" => vec![
            ("taxable_income", Text("30000")),
            ("section_9a_adjustment_applies", Flag(false)),
        ],
        "low_income_tax_offset" => vec![("taxable_income", Text("40000"))],
        "division_293" => vec![
            ("division_293_income", Text("240000.00")),
            ("low_tax_contributions", Text("20000.00")),
        ],
        _ => Vec::new(),
    }
}
