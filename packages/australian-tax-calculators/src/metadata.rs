// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

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

/// Schedule 1 (NAT 1004) weekly coefficients for payments from 1 July 2024 to
/// 30 June 2026.
const SCHEDULE_1_FROM_JULY_2024: &[(i64, &[CoefficientRow])] = &[
    (
        1,
        &[
            (Some(150), "0.1600", "0.1600"),
            (Some(371), "0.2117", "7.7550"),
            (Some(515), "0.1890", "-0.6702"),
            (Some(932), "0.3227", "68.2367"),
            (Some(2246), "0.3200", "65.7202"),
            (Some(3303), "0.3900", "222.9510"),
            (None, "0.4700", "487.2587"),
        ],
    ),
    (
        2,
        &[
            (Some(361), "0", "0"),
            (Some(500), "0.1600", "57.8462"),
            (Some(625), "0.2600", "107.8462"),
            (Some(721), "0.1800", "57.8462"),
            (Some(865), "0.1890", "64.3365"),
            (Some(1282), "0.3227", "180.0385"),
            (Some(2596), "0.3200", "176.5769"),
            (Some(3653), "0.3900", "358.3077"),
            (None, "0.4700", "650.6154"),
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
            (Some(361), "0", "0"),
            (Some(721), "0.1600", "57.8462"),
            (Some(865), "0.1690", "64.3365"),
            (Some(1282), "0.3027", "180.0385"),
            (Some(2596), "0.3000", "176.5769"),
            (Some(3653), "0.3700", "358.3077"),
            (None, "0.4500", "650.6154"),
        ],
    ),
    (
        6,
        &[
            (Some(361), "0", "0"),
            (Some(721), "0.1600", "57.8462"),
            (Some(843), "0.1690", "64.3365"),
            (Some(865), "0.2190", "106.4962"),
            (Some(1053), "0.3527", "222.1981"),
            (Some(1282), "0.3127", "180.0385"),
            (Some(2596), "0.3100", "176.5769"),
            (Some(3653), "0.3800", "358.3077"),
            (None, "0.4600", "650.6154"),
        ],
    ),
];

/// Schedule 1 (NAT 1004) weekly coefficients for payments from 1 July 2026.
const SCHEDULE_1_FROM_JULY_2026: &[(i64, &[CoefficientRow])] = &[
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
];

/// Schedule 1 weekly coefficients for payments made in each income year.
pub const PAYG_WITHHOLDING_COEFFICIENTS: &[(&str, &[(i64, &[CoefficientRow])])] = &[
    ("2024-25", SCHEDULE_1_FROM_JULY_2024),
    ("2025-26", SCHEDULE_1_FROM_JULY_2024),
    ("2026-27", SCHEDULE_1_FROM_JULY_2026),
];

/// One scale's Medicare levy adjustment: from `threshold` whole dollars of
/// weekly earnings, formula 1 is (x - `formula_1_less`) x `shade_in_rate` below
/// `shade_in`, formula 2 is x x `levy_rate` below the weekly family threshold
/// (WFT), and formula 3 is WFT x `levy_rate` - (x - WFT) x `shade_out_rate`
/// below the shading out point.
pub struct MedicareScale {
    pub threshold: i64,
    pub formula_1_less: &'static str,
    pub shade_in: i64,
    pub shade_in_rate: &'static str,
    pub levy_rate: &'static str,
    pub shade_out_rate: &'static str,
}

/// The rules of one Schedule 1 beside its coefficients.
pub struct Schedule1Rules {
    /// Scale 4 tax rates: (resident, foreign resident).
    pub no_tfn_rates: (&'static str, &'static str),
    /// Share of the yearly tax offsets claimed that each pay is reduced by.
    pub tax_offset_rates: [(&'static str, &'static str); 4],
    /// WFT = (family threshold + per child x children) / 52, to the cent; the
    /// shading out point is WFT x 0.1 / 0.08 in whole dollars.
    pub family_threshold: i64,
    pub family_threshold_per_child: i64,
    pub medicare_scales: [(i64, MedicareScale); 2],
    /// Additional withholding when a payee asks for it in a year of 53 weekly
    /// or 27 fortnightly pays: (lowest whole-dollar earnings, amount), lowest first.
    pub extra_weekly: [(i64, i64); 3],
    pub extra_fortnightly: [(i64, i64); 3],
}

const TAX_OFFSET_RATES: [(&str, &str); 4] = [
    ("weekly", "0.019"),
    ("fortnightly", "0.038"),
    ("monthly", "0.083"),
    ("quarterly", "0.25"),
];

/// Schedule 1 for payments from 1 July 2024 to 30 June 2026: 'Medicare levy
/// adjustment', 'Using a formula' and 'Working out the weekly earnings'.
const SCHEDULE_1_RULES_FROM_JULY_2024: Schedule1Rules = Schedule1Rules {
    no_tfn_rates: ("0.47", "0.45"),
    tax_offset_rates: TAX_OFFSET_RATES,
    family_threshold: 43846,
    family_threshold_per_child: 4027,
    medicare_scales: [
        (
            2,
            MedicareScale {
                threshold: 500,
                formula_1_less: "500",
                shade_in: 625,
                shade_in_rate: "0.1",
                levy_rate: "0.0200",
                shade_out_rate: "0.0800",
            },
        ),
        (
            6,
            MedicareScale {
                threshold: 843,
                formula_1_less: "843.19",
                shade_in: 1053,
                shade_in_rate: "0.05",
                levy_rate: "0.0100",
                shade_out_rate: "0.0400",
            },
        ),
    ],
    extra_weekly: [(875, 3), (2575, 6), (3650, 12)],
    extra_fortnightly: [(1750, 12), (5150, 26), (7250, 47)],
};

/// Schedule 1 for payments from 1 July 2026 (LI 2026/18), the same pages.
const SCHEDULE_1_RULES_FROM_JULY_2026: Schedule1Rules = Schedule1Rules {
    no_tfn_rates: ("0.47", "0.45"),
    tax_offset_rates: TAX_OFFSET_RATES,
    family_threshold: 47238,
    family_threshold_per_child: 4338,
    medicare_scales: [
        (
            2,
            MedicareScale {
                threshold: 538,
                formula_1_less: "538.67",
                shade_in: 673,
                shade_in_rate: "0.1",
                levy_rate: "0.0200",
                shade_out_rate: "0.0800",
            },
        ),
        (
            6,
            MedicareScale {
                threshold: 908,
                formula_1_less: "908.42",
                shade_in: 1135,
                shade_in_rate: "0.05",
                levy_rate: "0.0100",
                shade_out_rate: "0.0400",
            },
        ),
    ],
    extra_weekly: [(875, 3), (2575, 7), (3650, 12)],
    extra_fortnightly: [(1700, 12), (5200, 27), (7250, 48)],
};

/// Schedule 1 rules for payments made in each income year, keyed as the coefficients.
pub const PAYG_WITHHOLDING_RULES: &[(&str, &Schedule1Rules)] = &[
    ("2024-25", &SCHEDULE_1_RULES_FROM_JULY_2024),
    ("2025-26", &SCHEDULE_1_RULES_FROM_JULY_2024),
    ("2026-27", &SCHEDULE_1_RULES_FROM_JULY_2026),
];

/// Schedule 8 (NAT 3539) study and training support loan component coefficients:
/// for 2025-26, those of F2025L01016, which commenced on 24 September 2025.
pub const STUDY_LOAN_WITHHOLDING_COEFFICIENTS: &[(&str, &[(&str, &[CoefficientRow])])] = &[
    (
        "2025-26",
        &[
            (
                "threshold_or_foreign",
                &[
                    (Some(1288), "0", "0"),
                    (Some(2403), "0.15", "193.2692"),
                    (Some(3447), "0.17", "241.3462"),
                    (None, "0.10", "0"),
                ],
            ),
            (
                "no_threshold",
                &[
                    (Some(938), "0", "0"),
                    (Some(2053), "0.15", "140.7692"),
                    (Some(2597), "0.17", "181.8462"),
                    (None, "0.10", "0"),
                ],
            ),
        ],
    ),
    (
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
    ),
];

/// The first pay date each year's Schedule 8 coefficients apply to. Before 24
/// September 2025, F2025L00647's Schedule 8 used a percentage of weekly
/// earnings, which this worksheet does not hold.
pub const STUDY_LOAN_WITHHOLDING_FROM: &[(&str, &str)] =
    &[("2025-26", "2025-09-24"), ("2026-27", "2026-07-01")];

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

/// Medicare levy (threshold amount, phase-in limit) for a single person. Act
/// No. 58, 2026, Sch 5 item 14 applies the s 3(1) figures to 2025-26 and later
/// years, so 2026-27 has the 2025-26 figures until an amendment changes them.
pub const MEDICARE_LOW_INCOME_THRESHOLDS: &[(&str, (&str, &str))] = &[
    ("2024-25", ("27222", "34027")),
    ("2025-26", ("28011", "35013")),
    ("2026-27", ("28011", "35013")),
];
/// The same for a person entitled to the seniors and pensioners tax offset:
/// s 3(1) as Act No. 28, 2025, Sch 2 (2024-25 and later years) and Act No. 58,
/// 2026, Sch 5 (2025-26 and later years) set it.
pub const MEDICARE_SAPTO_THRESHOLDS: &[(&str, (&str, &str))] = &[
    ("2024-25", ("43020", "53775")),
    ("2025-26", ("44268", "55335")),
    ("2026-27", ("44268", "55335")),
];
/// s 8(5) and (7), from the same Acts: the family income threshold, the
/// threshold for a person entitled to the seniors and pensioners tax offset,
/// and the increase for each dependent child or student.
pub const MEDICARE_FAMILY_THRESHOLDS: &[(&str, (&str, &str, &str))] = &[
    ("2024-25", ("45907", "59886", "4216")),
    ("2025-26", ("47238", "61623", "4338")),
    ("2026-27", ("47238", "61623", "4338")),
];
/// s 6(1), s 7(2) and the 0.08 in the s 8(2) formula.
pub const MEDICARE_LEVY_RATES: [(&str, &str); 3] = [
    ("levy", "0.02"),
    ("shade_in", "0.10"),
    ("family_shade_out", "0.08"),
];

/// The Medicare levy surcharge singles tier 1, 2 and 3 thresholds (Private
/// Health Insurance Act 2007 s 22-35, indexed under s 22-45), from the ATO's
/// 'Medicare levy surcharge income, thresholds and rates' (last updated 22 June
/// 2026), read on 14 November 2026. Its family thresholds are double these.
pub const SURCHARGE_THRESHOLDS: &[(&str, [&str; 3])] = &[
    ("2024-25", ["97000", "113000", "151000"]),
    ("2025-26", ["101000", "118000", "158000"]),
    ("2026-27", ["105000", "123000", "164000"]),
];
/// Tiers 1 to 3: 1% (Medicare Levy Act s 8B(2)), raised by 0.25 and 0.5 of a
/// percentage point for tier 2 and tier 3 earners (s 8B(3) and (4)).
pub const SURCHARGE_RATES: [&str; 3] = ["0.01", "0.0125", "0.015"];
/// The increase in each family threshold for each dependent child after the
/// first, on the same ATO page for each year.
pub const SURCHARGE_CHILD_INCREASE: &str = "1500";

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

/// ITAA 1997 ss 296-30 and 296-35 (2026-27 thresholds; later years are indexed)
/// and the Superannuation (Building a Stronger and Fairer Super System) Imposition
/// Act 2026 s 5.
pub const DIVISION_296: [(&str, &str); 4] = [
    ("large_threshold", "3000000"),
    ("very_large_threshold", "10000000"),
    ("rate", "0.15"),
    ("very_large_rate", "0.10"),
];
pub const DIVISION_296_YEARS: [&str; 1] = ["2026-27"];

/// 2026-27 life benefit termination payment figures: the ETP cap amount (ITAA
/// 1997 s 82-160, indexed), the whole-of-income cap (s 82-10(4)(c)), the
/// genuine redundancy and early retirement scheme base and service amounts
/// (s 83-170(3), indexed), all as the ATO publishes them, and Schedule 11
/// Table A's withholding rates, which include the 2% Medicare levy.
pub const TERMINATION_PAYMENT: [(&str, &str); 7] = [
    ("etp_cap", "270000"),
    ("whole_of_income_cap", "180000"),
    ("base_amount", "13598"),
    ("service_amount", "6801"),
    ("preservation_age_rate", "0.17"),
    ("under_preservation_age_rate", "0.32"),
    ("above_cap_rate", "0.47"),
];
pub const TERMINATION_PAYMENT_YEARS: [&str; 1] = ["2026-27"];

/// Schedule 5 (NAT 3348): the schedules for payments from 1 July 2024, 1 July
/// 2025 and 1 July 2026 state the same methods and limit.
pub const SCHEDULE_5_YEARS: [&str; 3] = ["2024-25", "2025-26", "2026-27"];
/// Methods A and B(ii) limit withholding from an additional payment to 47% of it.
pub const SCHEDULE_5_LIMIT_RATE: &str = "0.47";

/// Schedule 15 (working holiday makers), columns 2 to 5, for a registered
/// employer's payment to a payee who gave a TFN: the payments made to them in
/// the income year before it that each column covers, lowest and highest
/// inclusive in cents ('less than $45,001', 'from $45,001 to $135,000', 'from
/// $135,001 to $190,000' and 'more than $190,000'), with Table A's rate. No
/// column covers $135,000.01 to $135,000.99. The schedules for payments from
/// 1 July 2024 and from 1 July 2026 give the same columns and rates.
pub const WORKING_HOLIDAY_MAKER_COLUMNS: [(Option<&str>, Option<&str>, &str); 4] = [
    (None, Some("45000.99"), "0.15"),
    (Some("45001"), Some("135000"), "0.30"),
    (Some("135001"), Some("190000"), "0.37"),
    (Some("190000.01"), None, "0.45"),
];
/// Schedule 15: 45% of payments to a working holiday maker who gave no TFN.
pub const WORKING_HOLIDAY_MAKER_NO_TFN_RATE: &str = "0.45";
pub const WORKING_HOLIDAY_MAKER_YEARS: [&str; 3] = ["2024-25", "2025-26", "2026-27"];

/// Schedule 9 (NAT 4466) weekly coefficients for payments from 1 July 2024 to
/// 30 June 2026, scales 1 to 9 in order.
const SCHEDULE_9_FROM_JULY_2024: [&[CoefficientRow]; 9] = [
    // Scale 1: payee is single.
    &[
        (Some(629), "0", "0"),
        (Some(671), "0.1600", "100.7308"),
        (Some(721), "0.2850", "184.6707"),
        (Some(790), "0.2940", "191.1611"),
        (Some(865), "0.3940", "270.1784"),
        (Some(987), "0.5277", "385.8803"),
        (Some(1014), "0.4477", "306.8630"),
        (Some(1282), "0.3227", "180.0385"),
        (Some(2596), "0.3200", "176.5769"),
        (Some(3653), "0.3900", "358.3077"),
        (None, "0.4700", "650.6154"),
    ],
    // Scale 2: payee is a member of an illness-separated couple.
    &[
        (Some(606), "0", "0"),
        (Some(648), "0.1600", "97.0769"),
        (Some(721), "0.2850", "178.1635"),
        (Some(790), "0.2940", "184.6538"),
        (Some(865), "0.3940", "263.6712"),
        (Some(962), "0.5277", "379.3731"),
        (Some(987), "0.4027", "259.0558"),
        (Some(1282), "0.3227", "180.0385"),
        (Some(2596), "0.3200", "176.5769"),
        (Some(3653), "0.3900", "358.3077"),
        (None, "0.4700", "650.6154"),
    ],
    // Scale 3: payee is a member of a couple.
    &[
        (Some(554), "0", "0"),
        (Some(596), "0.1600", "88.6538"),
        (Some(721), "0.2850", "163.1587"),
        (Some(790), "0.2940", "169.6490"),
        (Some(842), "0.3940", "248.6663"),
        (Some(865), "0.2690", "143.3538"),
        (Some(987), "0.4027", "259.0558"),
        (Some(1282), "0.3227", "180.0385"),
        (Some(2596), "0.3200", "176.5769"),
        (Some(3653), "0.3900", "358.3077"),
        (None, "0.4700", "650.6154"),
    ],
    // Scale 4: payee is single and claiming a FULL Medicare levy exemption.
    &[
        (Some(629), "0", "0"),
        (Some(671), "0.1600", "100.7308"),
        (Some(721), "0.2850", "184.6707"),
        (Some(865), "0.2940", "191.1611"),
        (Some(1014), "0.4277", "306.8630"),
        (Some(1282), "0.3027", "180.0385"),
        (Some(2596), "0.3000", "176.5769"),
        (Some(3653), "0.3700", "358.3077"),
        (None, "0.4500", "650.6154"),
    ],
    // Scale 5: payee is a member of an illness-separated couple and claiming a FULL Medicare levy exemption.
    &[
        (Some(606), "0", "0"),
        (Some(648), "0.1600", "97.0769"),
        (Some(721), "0.2850", "178.1635"),
        (Some(865), "0.2940", "184.6538"),
        (Some(962), "0.4277", "300.3558"),
        (Some(1282), "0.3027", "180.0385"),
        (Some(2596), "0.3000", "176.5769"),
        (Some(3653), "0.3700", "358.3077"),
        (None, "0.4500", "650.6154"),
    ],
    // Scale 6: payee is a member of a couple and claiming a FULL Medicare levy exemption.
    &[
        (Some(554), "0", "0"),
        (Some(596), "0.1600", "88.6538"),
        (Some(721), "0.2850", "163.1587"),
        (Some(842), "0.2940", "169.6490"),
        (Some(865), "0.1690", "64.3365"),
        (Some(1282), "0.3027", "180.0385"),
        (Some(2596), "0.3000", "176.5769"),
        (Some(3653), "0.3700", "358.3077"),
        (None, "0.4500", "650.6154"),
    ],
    // Scale 7: payee is single and claiming a HALF Medicare levy exemption.
    &[
        (Some(629), "0", "0"),
        (Some(671), "0.1600", "100.7308"),
        (Some(721), "0.2850", "184.6707"),
        (Some(865), "0.2940", "191.1611"),
        (Some(1014), "0.4277", "306.8630"),
        (Some(1099), "0.3027", "180.0385"),
        (Some(1282), "0.3527", "235.0365"),
        (Some(1374), "0.3500", "231.5750"),
        (Some(2596), "0.3100", "176.5769"),
        (Some(3653), "0.3800", "358.3077"),
        (None, "0.4600", "650.6154"),
    ],
    // Scale 8: payee is a member of an illness-separated couple and claiming a HALF Medicare levy exemption.
    &[
        (Some(606), "0", "0"),
        (Some(648), "0.1600", "97.0769"),
        (Some(721), "0.2850", "178.1635"),
        (Some(865), "0.2940", "184.6538"),
        (Some(962), "0.4277", "300.3558"),
        (Some(1099), "0.3027", "180.0385"),
        (Some(1282), "0.3527", "235.0365"),
        (Some(1374), "0.3500", "231.5750"),
        (Some(2596), "0.3100", "176.5769"),
        (Some(3653), "0.3800", "358.3077"),
        (None, "0.4600", "650.6154"),
    ],
    // Scale 9: payee is a member of a couple and claiming a HALF Medicare levy exemption.
    &[
        (Some(554), "0", "0"),
        (Some(596), "0.1600", "88.6538"),
        (Some(721), "0.2850", "163.1587"),
        (Some(842), "0.2940", "169.6490"),
        (Some(865), "0.1690", "64.3365"),
        (Some(1099), "0.3027", "180.0385"),
        (Some(1282), "0.3527", "235.0365"),
        (Some(1374), "0.3500", "231.5750"),
        (Some(2596), "0.3100", "176.5769"),
        (Some(3653), "0.3800", "358.3077"),
        (None, "0.4600", "650.6154"),
    ],
];

/// Schedule 9 (NAT 4466) weekly coefficients for payments from 1 July 2026.
const SCHEDULE_9_FROM_JULY_2026: [&[CoefficientRow]; 9] = [
    // Scale 1: payee is single.
    &[
        (Some(648), "0", "0"),
        (Some(692), "0.1500", "97.2308"),
        (Some(721), "0.2750", "183.8510"),
        (Some(851), "0.2840", "190.3413"),
        (Some(865), "0.3840", "275.4721"),
        (Some(1036), "0.5277", "399.8279"),
        (Some(1064), "0.4027", "270.3231"),
        (Some(1282), "0.3227", "185.1923"),
        (Some(2596), "0.3200", "181.7319"),
        (Some(3653), "0.3900", "363.4627"),
        (None, "0.4700", "655.7704"),
    ],
    // Scale 2: payee is a member of an illness-separated couple.
    &[
        (Some(623), "0", "0"),
        (Some(668), "0.1500", "93.5769"),
        (Some(721), "0.2750", "177.1514"),
        (Some(851), "0.2840", "183.6418"),
        (Some(865), "0.3840", "268.7726"),
        (Some(982), "0.5277", "393.1284"),
        (Some(1064), "0.4027", "270.3231"),
        (Some(1282), "0.3227", "185.1923"),
        (Some(2596), "0.3200", "181.7319"),
        (Some(3653), "0.3900", "363.4627"),
        (None, "0.4700", "655.7704"),
    ],
    // Scale 3: payee is a member of a couple.
    &[
        (Some(567), "0", "0"),
        (Some(612), "0.1500", "85.1538"),
        (Some(721), "0.2750", "161.7091"),
        (Some(851), "0.2840", "168.1995"),
        (Some(858), "0.3840", "253.3303"),
        (Some(865), "0.2590", "145.9673"),
        (Some(1064), "0.4027", "270.3231"),
        (Some(1282), "0.3227", "185.1923"),
        (Some(2596), "0.3200", "181.7319"),
        (Some(3653), "0.3900", "363.4627"),
        (None, "0.4700", "655.7704"),
    ],
    // Scale 4: payee is single and claiming a FULL Medicare levy exemption.
    &[
        (Some(648), "0", "0"),
        (Some(692), "0.1500", "97.2308"),
        (Some(721), "0.2750", "183.8510"),
        (Some(865), "0.2840", "190.3413"),
        (Some(1036), "0.4277", "314.6971"),
        (Some(1282), "0.3027", "185.1923"),
        (Some(2596), "0.3000", "181.7308"),
        (Some(3653), "0.3700", "363.4615"),
        (None, "0.4500", "655.7692"),
    ],
    // Scale 5: payee is a member of an illness-separated couple and claiming a FULL Medicare levy exemption.
    &[
        (Some(623), "0", "0"),
        (Some(668), "0.1500", "93.5769"),
        (Some(721), "0.2750", "177.1514"),
        (Some(865), "0.2840", "183.6418"),
        (Some(982), "0.4277", "307.9976"),
        (Some(1282), "0.3027", "185.1923"),
        (Some(2596), "0.3000", "181.7308"),
        (Some(3653), "0.3700", "363.4615"),
        (None, "0.4500", "655.7692"),
    ],
    // Scale 6: payee is a member of a couple and claiming a FULL Medicare levy exemption.
    &[
        (Some(567), "0", "0"),
        (Some(612), "0.1500", "85.1538"),
        (Some(721), "0.2750", "161.7091"),
        (Some(858), "0.2840", "168.1995"),
        (Some(865), "0.1590", "60.8365"),
        (Some(1282), "0.3027", "185.1923"),
        (Some(2596), "0.3000", "181.7308"),
        (Some(3653), "0.3700", "363.4615"),
        (None, "0.4500", "655.7692"),
    ],
    // Scale 7: payee is single and claiming a HALF Medicare levy exemption.
    &[
        (Some(648), "0", "0"),
        (Some(692), "0.1500", "97.2308"),
        (Some(721), "0.2750", "183.8510"),
        (Some(865), "0.2840", "190.3413"),
        (Some(1036), "0.4277", "314.6971"),
        (Some(1185), "0.3027", "185.1923"),
        (Some(1282), "0.3527", "244.4452"),
        (Some(1481), "0.3500", "240.9837"),
        (Some(2596), "0.3100", "181.7308"),
        (Some(3653), "0.3800", "363.4615"),
        (None, "0.4600", "655.7692"),
    ],
    // Scale 8: payee is a member of an illness-separated couple and claiming a HALF Medicare levy exemption.
    &[
        (Some(623), "0", "0"),
        (Some(668), "0.1500", "93.5769"),
        (Some(721), "0.2750", "177.1514"),
        (Some(865), "0.2840", "183.6418"),
        (Some(982), "0.4277", "307.9976"),
        (Some(1185), "0.3027", "185.1923"),
        (Some(1282), "0.3527", "244.4452"),
        (Some(1481), "0.3500", "240.9837"),
        (Some(2596), "0.3100", "181.7308"),
        (Some(3653), "0.3800", "363.4615"),
        (None, "0.4600", "655.7692"),
    ],
    // Scale 9: payee is a member of a couple and claiming a HALF Medicare levy exemption.
    &[
        (Some(567), "0", "0"),
        (Some(612), "0.1500", "85.1538"),
        (Some(721), "0.2750", "161.7091"),
        (Some(858), "0.2840", "168.1995"),
        (Some(865), "0.1590", "60.8365"),
        (Some(1185), "0.3027", "185.1923"),
        (Some(1282), "0.3527", "244.4452"),
        (Some(1481), "0.3500", "240.9837"),
        (Some(2596), "0.3100", "181.7308"),
        (Some(3653), "0.3800", "363.4615"),
        (None, "0.4600", "655.7692"),
    ],
];

/// Schedule 9 coefficients for payments made in each income year, scales 1 to
/// 9: scales 1 to 3 for a payee who is single, a member of an
/// illness-separated couple or a member of a couple, 4 to 6 for the same
/// payees claiming a full Medicare levy exemption and 7 to 9 for those claiming
/// a half exemption. A step with no coefficients ('-') withholds nothing.
pub const SENIORS_PENSIONERS_COEFFICIENTS: &[(&str, &[&[CoefficientRow]; 9])] = &[
    ("2024-25", &SCHEDULE_9_FROM_JULY_2024),
    ("2025-26", &SCHEDULE_9_FROM_JULY_2024),
    ("2026-27", &SCHEDULE_9_FROM_JULY_2026),
];

/// The seniors and pensioners tax offset's base rebate amounts (Income Tax
/// Assessment (1936 Act) Regulations 2025 s 9(2), and the 2015 Regulation's
/// s 9(2) for years starting before 1 October 2025), in the order of the
/// `status` values: not a spouse, a member of an illness-separated couple, a
/// spouse.
pub const SAPTO_BASE_REBATE_AMOUNTS: [&str; 3] = ["2230", "2040", "1602"];
/// The same instruments' s 4 rebate maximum amount, rebate reduction rate and
/// rebate reduction threshold, the 12.5 cents a dollar of s 11(2), and the
/// $6,000 and 0.15 of the s 12(3)(b)(ii) transfer formula.
pub const SAPTO_RULES: [(&str, &str); 6] = [
    ("rebate_maximum_amount", "445"),
    ("rebate_reduction_rate", "0.015"),
    ("rebate_reduction_threshold", "37000"),
    ("shade_out_rate", "0.125"),
    ("transfer_income_floor", "6000"),
    ("transfer_rate", "0.15"),
];

pub const CARRY_FORWARD_BALANCE_LIMIT: &str = "500000";

/// Statutory or benchmark interest rate by FBT year ended 31 March, for the
/// car operating cost method's deemed interest. Its keys are both car
/// worksheets' supported years; the 20% statutory rate applies in each.
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
        "gst" | "capital_gains" | "depreciation" => vec!["2025-26", "2026-27"],
        "quarterly_sg" => vec!["2025-26"],
        "resident_tax" => keys(RESIDENT_TAX_SCALES),
        "main_residence" => vec!["2024-25", "2025-26", "2026-27"],
        // s 26-155 quarantines residential rental deductions from 2027-28.
        "borrowing_expenses" => vec!["2024-25", "2025-26", "2026-27"],
        "fbt" => vec!["2026", "2027"],
        "fbt_car_statutory" | "fbt_car_operating_cost" => keys(FBT_BENCHMARK_INTEREST_RATES),
        "payg_withholding" => keys(PAYG_WITHHOLDING_COEFFICIENTS),
        "contribution_caps" => keys(CONTRIBUTION_CAPS),
        "pension_minimum" => vec!["2024-25", "2025-26", "2026-27"],
        "study_loan_repayment" => keys(STUDY_LOAN_REPAYMENT_THRESHOLDS),
        "study_loan_withholding" => keys(STUDY_LOAN_WITHHOLDING_COEFFICIENTS),
        "medicare_levy" => keys(MEDICARE_LOW_INCOME_THRESHOLDS),
        "medicare_levy_surcharge" => keys(SURCHARGE_THRESHOLDS),
        "low_income_tax_offset" => LOW_INCOME_TAX_OFFSET_YEARS.to_vec(),
        "division_293" => DIVISION_293_YEARS.to_vec(),
        "division_296" => DIVISION_296_YEARS.to_vec(),
        "termination_payment" => TERMINATION_PAYMENT_YEARS.to_vec(),
        "schedule_5_method_a" | "schedule_5_method_b_i" | "schedule_5_method_b_ii" => {
            SCHEDULE_5_YEARS.to_vec()
        }
        "working_holiday_maker_withholding" => WORKING_HOLIDAY_MAKER_YEARS.to_vec(),
        "seniors_pensioners_withholding" => keys(SENIORS_PENSIONERS_COEFFICIENTS),
        // The years checked against the regulations; the thresholds follow each
        // year's Schedule 7 rates, but a new year needs its own review.
        "seniors_pensioners_tax_offset" => vec!["2024-25", "2025-26", "2026-27"],
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
            "2026-11-04",
            "Calculating your FBT, steps 3 to 7 and the MantCo example; FBT return 2026 instructions, items 14A to 16; FBT rates and thresholds, the FBT rate and gross-up rates for years ending 31 March 2023 to 2027",
        ),
        "main_residence" => (
            "2026-11-01",
            "Treating former home as main residence: the 6-year rule, Roya and Helen examples; Using your home for rental or business: working out the capital gain, the home first used to produce income rule and its examples",
        ),
        "borrowing_expenses" => (
            "2026-11-04",
            "ITAA 1997 s 25-25 and s 26-50, compilation 268 (1 October 2026); Borrowing expenses: the Peter and the Fiona and Max examples",
        ),
        "fbt_car_statutory" => (
            "2026-10-29",
            "Fringe benefits tax - a guide for employers, 7.8 statutory formula method and examples 23 and 26; FBT return 2026 instructions, item 23A and example 6; Electric cars exemption, reportable fringe benefits",
        ),
        "fbt_car_operating_cost" => (
            "2026-11-06",
            "Fringe benefits tax - a guide for employers, 7.9 operating cost method and example 31; FBT return 2026 instructions, item 23B and example 7; FBT rates and thresholds, benchmark interest rate; Electric cars exemption, reportable fringe benefits; Income Tax Assessment (Effective Life of Depreciating Assets) Determination 2025 (LI 2025/20, F2025L01097) and TR 2022/1, Table B, effective lives of cars",
        ),
        "depreciation" => (
            "2026-11-13",
            "Decline in value: diminishing value and prime cost methods, with the later-year diminishing value example; taxable use; ITAA 1997 ss 40-70(1), 40-72 and 40-75(1) and (7), base value and the limit on decline in a later year",
        ),
        "quarterly_sg" => (
            "2026-09-10",
            "Super guarantee, tables 21 and 24, 2025-26 rows",
        ),
        "payg_withholding" => (
            "2026-11-14",
            "Schedule 1 (NAT 1004) for payments from 1 July 2026, and for payments from 1 July 2024 to 30 June 2026: weekly coefficients and scale 4, using a formula (rounding and the 53 and 27 pay tables), working out the weekly earnings (quarterly pays and tax offsets), Medicare levy adjustment, examples of calculating the withholding amounts, sample data",
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
            "2026-11-14",
            "Schedule 8 (NAT 3539) for payments from 1 July 2026, and Schedule 8 of the Taxation Administration (Withholding Schedules) Instrument (No. 2) 2025 (F2025L01016) for payments from 24 September 2025: study and training support loans component rates, calculating fortnightly, monthly and quarterly amounts, examples 1 to 3, rounding",
        ),
        "medicare_levy" => (
            "2026-11-14",
            "Medicare Levy Act 1986 ss 3(1) and (3), 6(1), 7(1) and (2) and 8, compilations 52 (2024-25) and 53; Treasury Laws Amendment (More Cost of Living Relief) Act 2025 Sch 2 item 14 (2024-25 and later years); Treasury Laws Amendment (Delivering an Efficient and Trusted Tax System) Act 2026 Sch 5 item 14 (2025-26 and later years); ATO Medicare levy reduction for low-income earners and Medicare levy reduction - family income, with the Ashton example (2025-26); myTax 2025 tables 2 and 3 and worksheet 2 (2024-25); exemptions: Medicare Levy Act 1986 s 9(1), ITAA 1936 ss 251T and 251U(1) to (3), ATO Medical exemption from Medicare levy (the Paul example), Not entitled to Medicare benefits (the Priya example) and M1 Medicare levy reduction or exemption 2026, part B",
        ),
        "medicare_levy_surcharge" => (
            "2026-11-14",
            "Medicare Levy Act 1986 ss 3(1), (3) and (3A), 3A, 8B and 8D, compilations 52 (2024-25) and 53; A New Tax System (Medicare Levy Surcharge - Fringe Benefits) Act 1999 ss 3, 6, 7, 12, 15 and 16, compilations 41 (2024-25) and 42; ATO Medicare levy surcharge income, thresholds and rates, with the Tom example (2026-27); M2 Medicare levy surcharge 2026, with the Josh, Jacinta and Jill and Kevin examples (2025-26)",
        ),
        "low_income_tax_offset" => (
            "2026-10-15",
            "ITAA 1997 ss 61-110 and 61-115, compilation 266 (1 July 2026); ATO Low income tax offset page",
        ),
        "division_293" => (
            "2026-10-15",
            "ITAA 1997 ss 293-15 to 293-25, compilation 266 (1 July 2026); Imposition Act 2013 s 5; ATO Key superannuation rates, table 7",
        ),
        "division_296" => (
            "2026-11-13",
            "ITAA 1997 ss 296-15 to 296-55, compilation 268 (1 October 2026); Imposition Act 2026 s 5; Income Tax (Transitional Provisions) Act 1997 s 296-1; Explanatory Memorandum examples 1.1 to 1.3; ATO Division 296 tax on large super balances",
        ),
        "termination_payment" => (
            "2026-11-14",
            "ATO Schedule 11, tax table for employment termination payments (from 1 July 2026), Table A and examples 1 to 4; ATO Key superannuation rates, tables 17 and 20; ITAA 1997 ss 82-10, 82-130 to 82-160 and 83-170 to 83-180, compilation 268 (1 October 2026); SISR reg 6.01(2), compilation 159 (1 July 2026)",
        ),
        "schedule_5_method_a" | "schedule_5_method_b_i" | "schedule_5_method_b_ii" => (
            "2026-11-14",
            "Schedule 5 (NAT 3348) for payments from 1 July 2026, from 1 July 2025 to 30 June 2026 and from 1 July 2024 to 30 June 2025: about this schedule, working out the withholding amount (Methods A, B(i) and B(ii), the withholding limit, study and training support loans) and withholding amounts examples 1 to 4; Schedule 8's coefficients for the pay date",
        ),
        "working_holiday_maker_withholding" => (
            "2026-11-14",
            "Schedule 15 (working holiday makers) for payments from 1 July 2026 and for payments from 1 July 2024 to 30 June 2026: using this schedule, working out the withholding amount, registered and unregistered employers, using a formula, Table A, examples 1 and 2, rounding and TFN declarations",
        ),
        "seniors_pensioners_withholding" => (
            "2026-11-14",
            "Schedule 9 (NAT 4466, seniors and pensioners) for payments from 1 July 2026 and for payments from 1 July 2024 to 30 June 2026: using this schedule, working out the withholding amount and its example, calculating fortnightly or monthly withholding amounts, using a formula and the coefficients for scales 1 to 9, rounding, claiming tax offsets, Medicare levy exemptions and adjustments, and the weekly tax table",
        ),
        "seniors_pensioners_tax_offset" => (
            "2026-11-14",
            "ITAA 1936 ss 6(1) (rebate income) and 160AAAA, compilation 193 (1 October 2026); Income Tax Assessment (1936 Act) Regulations 2025 ss 4 and 9 to 12 and 23 (F2025L01060), and the Income Tax Assessment (1936 Act) Regulation 2015 ss 4 and 9 to 12, compilation 8; ATO Seniors and pensioners tax offset: income limits, rates and thresholds for 2025-26, tax offset reduction and rounding, examples and transferring unused SAPTO with example 9; Do you need to lodge a tax return? 2025, reason 2",
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
        "main_residence" => (
            "tax-examples-capital-gains-tax-cgt",
            "2-060, 2-075",
            "2026-06-30",
            "2026-11-01",
        ),
        "borrowing_expenses" => (
            "tax-examples-deductions",
            "4-160",
            "2026-06-30",
            "2026-11-04",
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
            "ATO Schedule 1 sample data",
            "withholding amounts, Medicare levy adjustment scale 2 and Medicare half-levy adjustment scale 6: weekly, fortnightly and monthly tables",
            "2026-06-17",
            "2026-11-14",
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
            "ATO Schedule 8 statement of formulas for STSL components, and F2025L01016 Schedule 8 for 2025-26",
            "component rates, examples 1 to 3 in each",
            "2026-06-17",
            "2026-11-14",
        ),
        "medicare_levy" => (
            "ATO Medicare levy reduction for low-income earners; Medicare levy reduction - family income; Not entitled to Medicare benefits",
            "table of single thresholds, example (Angie); family taxable income thresholds, example (Ashton); claiming an exemption, example (Priya)",
            "2026-06-30",
            "2026-11-14",
        ),
        "medicare_levy_surcharge" => (
            "ATO Medicare levy surcharge income, thresholds and rates; M2 Medicare levy surcharge 2026",
            "thresholds and rates for 2024-25 to 2026-27, example (Tom); examples (Josh, Jacinta, Jill and Kevin)",
            "2026-06-22",
            "2026-11-14",
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
        "division_296" => (
            "superannuation-contributions-to-superannuation-funds-and-rsas",
            "6-495",
            "2026-06-30",
            "2026-11-13",
        ),
        "termination_payment" => (
            "ATO Schedule 11 tax table for employment termination payments",
            "examples 1 to 4",
            "2026-06-17",
            "2026-11-14",
        ),
        "schedule_5_method_a" | "schedule_5_method_b_i" | "schedule_5_method_b_ii" => (
            "ATO Schedule 5 (NAT 3348) withholding amounts examples",
            "examples 1 to 4",
            "2026-06-17",
            "2026-11-14",
        ),
        "working_holiday_maker_withholding" => (
            "ATO Schedule 15 tax table for working holiday makers",
            "withholding lookup tool example, examples 1 and 2",
            "2026-06-17",
            "2026-11-14",
        ),
        "seniors_pensioners_withholding" => (
            "ATO Schedule 9 tax table for seniors and pensioners (NAT 4466)",
            "the worked example ($662.95 a week) and every row of the weekly tax table",
            "2026-06-17",
            "2026-11-14",
        ),
        "seniors_pensioners_tax_offset" => (
            "ATO Seniors and pensioners tax offset",
            "rates and thresholds for 2025-26, the examples (Clare and Roy, Deb and Ivan, Ying and Li Jun, Marko, Simon, José, Keith and Jean, Vanh and Mai) and example 9",
            "2026-10-02",
            "2026-11-14",
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

/// Inputs a worksheet takes but does not require, with their units.
pub fn optional_inputs(kind: &str) -> Vec<(&'static str, &'static str)> {
    match kind {
        "depreciation" => vec![(
            "opening_adjustable_value",
            "AUD, for a year after the start time only: cost less the decline in value of earlier years, not the deductions; omit in the start year",
        )],
        "medicare_levy" => vec![
            (
                "sapto_entitled",
                "boolean, whether the person is entitled to the seniors and pensioners tax offset for the year at more than nil; true applies the s 3(1) and s 8(7) thresholds; omit for false",
            ),
            (
                "spouse_taxable_income",
                "AUD, whole dollars, the taxable income of the person's spouse on the last day of the year as s 3(3) reads it (not one living separately and apart, but one living apart only because of illness counts, as does one who died in the year); omit for no spouse",
            ),
            (
                "spouse_sapto_entitled",
                "boolean, required with spouse_taxable_income and taken only with it: whether the spouse is entitled to the seniors and pensioners tax offset at more than nil",
            ),
            (
                "dependant_offset_entitled",
                "boolean, for a person without a spouse only: whether they are entitled to the invalid and invalid carer tax offset for their child (ITAA 1997 Subdivision 61-A) or the sole parent notional tax offset (Subdivision 961-B), without which s 8 does not apply to them; omit for false",
            ),
            (
                "dependent_children",
                "integer, the dependent children and students (ITAA 1997 s 961-5(1)(c)) for whom the person or their spouse is entitled to the Subdivision 961-A notional tax offset; without a spouse, only those for whom family tax benefit was payable to the person (s 8(5) and (6)), and only with dependant_offset_entitled true; omit for none",
            ),
            (
                "medicare_levy_exemption_full_days",
                "integer from 0 to the days in the year, the days the person qualified for a full exemption (myTax 'Full 2% levy exemption - number of days', return label M1 V): a prescribed person under ITAA 1936 s 251U(1) in the medical category or as one not entitled to Medicare benefits, not as a foreign resident; omit for none",
            ),
            (
                "medicare_levy_exemption_half_days",
                "integer from 0 to the days in the year, the days the person qualified for a half exemption (label M1 W): in the medical category, but with a dependant outside the exemption categories (s 251U(3)); a day in both counts once, as a full day; omit for none",
            ),
        ],
        "medicare_levy_surcharge" => vec![
            (
                "spouse_income_for_surcharge_purposes",
                "AUD, the income for surcharge purposes of the person's spouse, for a person married for the whole year (s 3(3): not one living separately and apart; one who died in the year counts, s 3(3A)), including the spouse's share of trust net income on which the trustee is assessed under ITAA 1936 s 98 (s 8D(5)); omit for a person without a spouse or dependants all year",
            ),
            (
                "dependent_children",
                "integer, the MLS dependent children (Australian residents the person maintained: under 21, or 21 to 24 and studying full time), only with spouse_income_for_surcharge_purposes; each after the first raises the family thresholds; omit for none",
            ),
        ],
        "payg_withholding" => vec![
            (
                "foreign_resident",
                "boolean, scale 4 only and required there: true for a payee who is a foreign resident, false for a resident",
            ),
            (
                "tax_offsets",
                "AUD, scales 2, 5 and 6 only: the yearly total the payee claimed at the tax offsets questions on their Withholding declaration",
            ),
            (
                "medicare_adjustment_children",
                "integer from 0, scales 2 and 6 only, when the payee's Medicare levy variation declaration claims the adjustment (yes at question 10, and yes at question 9 or 12): the dependent children shown at question 12, or 0 for a spouse and no children",
            ),
            (
                "extra_pay_withholding",
                "boolean, weekly and fortnightly pays only: true when the financial year has 53 weekly or 27 fortnightly pays and the payee asked for the additional amount",
            ),
        ],
        "study_loan_withholding" => vec![(
            "pay_date",
            "YYYY-MM-DD date the payment is made, within the income year; required for 2025-26, whose Schedule 8 coefficients apply from 24 September 2025",
        )],
        "seniors_pensioners_tax_offset" => vec![
            (
                "spouse_rebate_income",
                "AUD, whole dollars, required unless status is single and taken only then: the spouse's rebate income, less any amount in their assessable income under ITAA 1936 s 100, plus any amount a trustee is assessed on under s 98 for the spouse (s 160AAAA(4))",
            ),
            (
                "spouse_taxable_income",
                "AUD, whole dollars, not for a single person: to transfer the spouse's unused base rebate amount (regulations s 12), the spouse's taxable income for the year with any exempt disability support, carer, invalidity service or partner service pension added (s 12(5)); give it only for a spouse entitled to the offset whose whole taxable income is ordinary taxable income taxed at the resident scale (Income Tax Rates Act 1986 Sch 7 Part I item table, with no superannuation or employment termination remainder taxed at 45%); omit for no transfer",
            ),
        ],
        "schedule_5_method_a" | "schedule_5_method_b_i" | "schedule_5_method_b_ii" => vec![(
            "pay_date",
            "YYYY-MM-DD date this pay is made, within the income year, with study_loan true only; required with it for 2025-26, whose Schedule 8 coefficients apply from 24 September 2025; Method B(i) refuses a 2025-26 study loan",
        )],
        _ => Vec::new(),
    }
}

/// `INPUT_UNITS[kind]`, in the Python dict's order.
pub fn input_units(kind: &str) -> Vec<(&'static str, &'static str)> {
    let payg = [
        (
            "earnings",
            "AUD, gross earnings and allowances subject to withholding for the pay period",
        ),
        ("pay_period", "weekly, fortnightly or monthly"),
        ("scale", "integer 1, 2, 3, 5 or 6"),
    ];
    let schedule_5_pay = vec![
        ("pay_period", "weekly, fortnightly or monthly"),
        ("scale", "integer 1, 2, 3, 5 or 6"),
        (
            "study_loan",
            "boolean, whether the payee declared a study and training support loan debt; true adds the Schedule 8 component for the pay date to each table amount, for 2026-27 and, except in Method B(i), 2025-26 pays made from 24 September 2025",
        ),
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
        "main_residence" => vec![
            (
                "capital_gain",
                "AUD, the gain on the caller's whole ownership interest before any exemption or discount; 0.00 when there is a loss",
            ),
            (
                "capital_loss",
                "AUD, from the reduced cost base; 0.00 when there is a gain",
            ),
            (
                "market_value_basis",
                "boolean, true when the gain or loss was worked out from market value when income use first began (s 118-192)",
            ),
            (
                "ownership_start",
                "YYYY-MM-DD, first day of the ownership period",
            ),
            (
                "ownership_end",
                "YYYY-MM-DD, last day of the ownership period",
            ),
            (
                "lived_periods",
                "list of {start, end} YYYY-MM-DD, days the dwelling was the main residence, both ends included",
            ),
            (
                "income_use_periods",
                "list of {start, end, floor_area}, days the dwelling or part of it produced assessable income with interest deductible; floor_area a decimal string above 0 and at most 1",
            ),
            (
                "absence_choice_periods",
                "list of {start, end}, days covered by the s 118-145 choice, each starting the day after a lived period",
            ),
            (
                "new_home_acquired",
                "YYYY-MM-DD or null, when a new home that became the main residence was acquired (s 118-140)",
            ),
        ],
        "borrowing_expenses" => vec![
            (
                "borrowing_expenses",
                "AUD, this loan's expenditure for borrowing the money, such as establishment, valuation and mortgage document fees and stamp duty on the mortgage, but not interest; incurred in the income year the loan began",
            ),
            (
                "other_borrowing_expenses_in_year",
                "AUD, the income-producing share of the taxpayer's other borrowing expenses incurred in that income year, for the s 25-25(6) $100 test",
            ),
            (
                "income_use",
                "decimal fraction from 0 to 1, the extent the borrowed money is used to produce assessable income, the same in every income year of the loan",
            ),
            (
                "loan_start",
                "YYYY-MM-DD, the first day the money was borrowed",
            ),
            (
                "loan_term_end",
                "YYYY-MM-DD or null, the last day of the period the original loan contract specifies",
            ),
            (
                "repaid_on",
                "YYYY-MM-DD or null, the day the loan was repaid",
            ),
            (
                "section_26_50_applies",
                "boolean, false only when the property is not a leisure facility, or s 26-50(3) applies to it at all times in the income year (such as a holiday home held mainly to produce rent)",
            ),
        ],
        "fbt" => vec![("type_one_value", "AUD"), ("type_two_value", "AUD")],
        "fbt_car_statutory" => vec![
            (
                "cost_price",
                "AUD, the part of the base value the one-third reduction applies to: for a car the provider or an associate owned at the earliest holding time, its cost price with dealer delivery and accessories fitted at acquisition, excluding registration, tax on registration, insurance and extended warranty; otherwise its leased car value then, the lessor's cost price only if the lease began then and the lessor bought the car at or about that time, else its arm's-length purchase price",
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
            (
                "employee_contribution",
                "AUD, the recipient's payment under FBTAA s 9(2)(e), net of any reimbursement",
            ),
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
                "effective_life_years",
                "years from 2, the car's effective life in the Commissioner's determination in effect when the provider last became its owner (FBTAA s 11(1AA)): 8 for cars generally, 5 for rental cars and 4 for taxis in Table B of the 2025 determination (LI 2025/20, F2025L01097) and TR 2022/1; 0 for a leased car or an owned car with no depreciated value left",
            ),
            (
                "days_held",
                "integer, the days in the holding period the car was owned or leased, from 0 to the days in the FBT year",
            ),
            ("business_use", "decimal fraction from 0 to 1"),
            (
                "employee_contribution",
                "AUD, the recipient's payment under FBTAA s 10(3)(c), net of any reimbursement",
            ),
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
        "payg_withholding" => vec![
            payg[0],
            ("pay_period", "weekly, fortnightly, monthly or quarterly"),
            (
                "scale",
                "integer 1 to 6; scale 4 also needs foreign_resident",
            ),
        ],
        "study_loan_withholding" => vec![
            payg[0],
            ("pay_period", "weekly, fortnightly, monthly or quarterly"),
            payg[2],
        ],
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
        "medicare_levy_surcharge" => vec![
            (
                "taxable_income",
                "AUD, whole dollars, including any amount on which family trust distribution tax was paid (s 3(2A))",
            ),
            (
                "reportable_fringe_benefits",
                "AUD, the reportable fringe benefits total",
            ),
            (
                "income_for_surcharge_purposes",
                "AUD, the person's income for surcharge purposes, as the ATO's thresholds page lists it: taxable income without any assessable first home super saver released amount, plus reportable fringe benefits, total net investment loss, reportable super contributions, any amount on which family trust distribution tax was paid and, if taxable income is $1 or more, any exempt foreign employment income",
            ),
            (
                "days_without_cover",
                "integer from 0 to the days in the year, the days on which the person or, for a person with a spouse, the person, the spouse or a dependent child had no appropriate private patient hospital cover (s 3(5)), not counting a spouse or child who was then a prescribed person, or would have been but for ITAA 1936 s 251U(2) (s 8D(1)(b)); the days in the year for none",
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
        "division_296" => vec![
            (
                "total_super_balance_end_of_year",
                "AUD, the total superannuation balance at the end of 30 June 2027 for Division 296, including reversionary retirement phase interests and Division 296 excluded interests, disregarding any LRBA amount (s 296-260) and foreign superannuation fund interests (s 307-230(1)(a))",
            ),
            (
                "total_super_earnings",
                "AUD, may be negative: total superannuation earnings for the income year (s 296-55), the relevant superannuation earnings for each interest, including one supporting a retirement phase income stream received because of another person's death (s 296-55(1)(b)), with foreign fund, excluded and prescribed amounts as nil",
            ),
            (
                "child_recipient",
                "boolean, whether the individual was a child recipient of a superannuation income stream at any time in the year (s 296-20)",
            ),
            (
                "structured_settlement_contribution",
                "boolean, whether a structured settlement contribution was made in respect of the individual in the year or an earlier income year (s 296-25)",
            ),
            (
                "died_in_year",
                "boolean, whether the individual died on or before 30 June 2027 (Income Tax (Transitional Provisions) Act 1997 s 296-1(3))",
            ),
        ],
        "termination_payment" => vec![
            (
                "redundancy_payment",
                "AUD, the genuine redundancy payment (s 83-175) or early retirement scheme payment (s 83-180): the part of the termination payment above what voluntary termination would have paid, excluding salary, wages, unused annual and long service leave and payments in lieu of superannuation; 0.00 for none",
            ),
            (
                "completed_years_of_service",
                "integer from 0 to 150, the whole years in the period of employment the redundancy payment relates to (s 83-170(3))",
            ),
            (
                "excluded_payment",
                "AUD, the taxable component of any other excluded payment in the termination payment: one that would be a genuine redundancy payment but for the pension age condition, or compensation in a genuine dispute for personal injury, unfair dismissal, harassment or discrimination (s 82-10(6)); not an invalidity payment",
            ),
            (
                "non_excluded_payment",
                "AUD, the taxable component of the rest of the termination payment, such as a golden handshake, gratuity, payment in lieu of notice, unused sick leave or rostered days off, or a non-genuine redundancy payment",
            ),
            (
                "other_taxable_payments",
                "AUD, the taxable payments the payer made to the employee in the income year, excluding this termination payment, such as salary, wages and unused leave paid on termination (Schedule 11 step 1)",
            ),
            (
                "date_of_birth",
                "YYYY-MM-DD, the employee's, for their preservation age (SISR reg 6.01(2))",
            ),
        ],
        "schedule_5_method_a" => [
            vec![
                (
                    "earnings",
                    "AUD, the pay's gross earnings excluding additional payments; cents are ignored",
                ),
                (
                    "additional_payment",
                    "AUD, the back payments, commissions, bonuses and similar payments made in this pay",
                ),
                (
                    "pay_periods",
                    "integer, 52 weekly, 26 fortnightly or 12 monthly pay periods; fewer, from 2, only for a commission, bonus or similar payment for a defined period of less than 12 months: the pay periods it relates to",
                ),
            ],
            schedule_5_pay,
        ]
        .concat(),
        "schedule_5_method_b_i" => [
            vec![
                (
                    "earnings",
                    "AUD, the current pay's gross earnings excluding additional payments",
                ),
                (
                    "back_payment_periods",
                    "list of {earnings, back_payment, withheld} decimal strings, one for each earlier pay period in the current financial year the back payment applies to: the normal earnings paid for it, including back payments already applied to it by Method B(i), the part of the back payment for it, and the amount withheld from it, including any study and training support loan component; at most 52 weekly, 26 fortnightly or 11 monthly periods",
                ),
            ],
            schedule_5_pay,
        ]
        .concat(),
        "schedule_5_method_b_ii" => [
            vec![
                (
                    "earnings",
                    "AUD, the current pay's gross earnings excluding additional payments",
                ),
                (
                    "additional_payment",
                    "AUD, the additional payment in this pay worked out by Method B(ii)",
                ),
                (
                    "year_to_date_earnings",
                    "AUD, normal earnings paid in the financial year to date, including the current pay, plus current-year back payments for which Method B(i) was used",
                ),
                (
                    "pay_periods_to_date",
                    "integer, pay periods in the financial year to date, including the current one: at most 53 weekly, 27 fortnightly or 12 monthly",
                ),
                (
                    "earlier_additional_payments",
                    "AUD, additional payments made earlier in the financial year for which Method B(ii) was used",
                ),
                (
                    "earlier_additional_withholding",
                    "AUD, the amounts withheld from those earlier additional payments, including any study and training support loan component",
                ),
            ],
            schedule_5_pay,
        ]
        .concat(),
        "working_holiday_maker_withholding" => vec![
            (
                "payment",
                "AUD, the total payments to the working holiday maker for the pay period: salary, wages, allowances, back payments, bonuses and other payments Schedule 15 covers",
            ),
            (
                "payments_before",
                "AUD, the total payments made to them in the income year before this payment",
            ),
            (
                "employer_registered",
                "boolean, whether the employer is registered with the ATO to withhold at working holiday maker rates",
            ),
            (
                "tfn_quoted",
                "boolean, true when the payee quoted a TFN, or advised on their TFN declaration that they applied for one or made an enquiry with the ATO and 28 days have not passed; false when they did neither, or 28 days passed without a TFN",
            ),
            (
                "pay_period",
                "weekly, fortnightly or monthly; for an unregistered employer it selects the Schedule 1 foreign resident amounts",
            ),
        ],
        "seniors_pensioners_withholding" => vec![
            payg[0],
            payg[1],
            (
                "status",
                "single, illness_separated or couple: the payee's status on their Withholding declaration, which selects column 2, 3 or 4 of the tax table",
            ),
            (
                "medicare_levy_exemption",
                "none, full or half: the Medicare levy exemption the payee claimed on a Medicare levy variation declaration",
            ),
        ],
        "seniors_pensioners_tax_offset" => vec![
            (
                "rebate_income",
                "AUD, whole dollars: taxable income (disregarding an assessable FHSS released amount), reportable superannuation contributions, total net investment loss and adjusted fringe benefits total (ITAA 1936 s 6(1))",
            ),
            (
                "status",
                "single (not a spouse at any time in the year), illness_separated (a member of an illness-separated couple at any time in the year) or couple (a spouse for the whole year, never illness-separated)",
            ),
        ],
        _ => Vec::new(),
    }
}

/// An example fact value: a string, a boolean, an integer or a JSON literal.
#[derive(Clone, Copy)]
pub enum Example {
    Text(&'static str),
    Flag(bool),
    Whole(i64),
    Json(&'static str),
}

/// `EXAMPLES[kind]`, in the Python dict's order.
pub fn examples(kind: &str) -> Vec<(&'static str, Example)> {
    use Example::{Flag, Json, Text, Whole};
    match kind {
        "gst" => vec![("amount", Text("1100.00")), ("gst_inclusive", Flag(true))],
        "resident_tax" => vec![("taxable_income", Text("45000"))],
        "capital_gains" => vec![
            ("other_gains", Text("100.00")),
            ("discount_gains", Text("1000.00")),
            ("current_losses", Text("200.00")),
            ("prior_losses", Text("300.00")),
        ],
        "main_residence" => vec![
            ("capital_gain", Text("230000.00")),
            ("capital_loss", Text("0.00")),
            ("market_value_basis", Flag(false)),
            ("ownership_start", Text("2013-10-01")),
            ("ownership_end", Text("2026-03-30")),
            (
                "lived_periods",
                Json(r#"[{"start": "2016-07-01", "end": "2026-03-30"}]"#),
            ),
            (
                "income_use_periods",
                Json(r#"[{"start": "2013-10-01", "end": "2016-06-30", "floor_area": "1"}]"#),
            ),
            ("absence_choice_periods", Json("[]")),
            ("new_home_acquired", Json("null")),
        ],
        "borrowing_expenses" => vec![
            ("borrowing_expenses", Text("1600.00")),
            ("other_borrowing_expenses_in_year", Text("0.00")),
            ("income_use", Text("1")),
            ("loan_start", Text("2024-07-03")),
            ("loan_term_end", Text("2049-07-02")),
            ("repaid_on", Json("null")),
            ("section_26_50_applies", Flag(false)),
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
            ("effective_life_years", Text("8")),
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
        // The current schedule, which the 2026 sample data in the evidence checks.
        "payg_withholding" => vec![
            ("year", Text("2026-27")),
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
            ("year", Text("2026-27")),
            ("earnings", Text("2000.00")),
            ("pay_period", Text("weekly")),
            ("scale", Whole(2)),
        ],
        "medicare_levy" => vec![
            ("taxable_income", Text("30000")),
            ("section_9a_adjustment_applies", Flag(false)),
        ],
        // The ATO's Tom example, 2026-27.
        "medicare_levy_surcharge" => vec![
            ("year", Text("2026-27")),
            ("taxable_income", Text("90000")),
            ("reportable_fringe_benefits", Text("27000.00")),
            ("income_for_surcharge_purposes", Text("117000.00")),
            ("days_without_cover", Whole(365)),
        ],
        "low_income_tax_offset" => vec![("taxable_income", Text("40000"))],
        "division_293" => vec![
            ("division_293_income", Text("240000.00")),
            ("low_tax_contributions", Text("20000.00")),
        ],
        "division_296" => vec![
            ("total_super_balance_end_of_year", Text("4000000.00")),
            ("total_super_earnings", Text("100000.00")),
            ("child_recipient", Flag(false)),
            ("structured_settlement_contribution", Flag(false)),
            ("died_in_year", Flag(false)),
        ],
        "termination_payment" => vec![
            ("redundancy_payment", Text("48000.00")),
            ("completed_years_of_service", Whole(5)),
            ("excluded_payment", Text("0.00")),
            ("non_excluded_payment", Text("17678.00")),
            ("other_taxable_payments", Text("140000.00")),
            ("date_of_birth", Text("1996-09-01")),
        ],
        // Schedule 5's examples 1 to 3 (Jarrod, Robyn and Laura), for payments from 1 July 2026.
        "schedule_5_method_a" => vec![
            ("year", Text("2026-27")),
            ("earnings", Text("1400.00")),
            ("additional_payment", Text("900.00")),
            ("pay_periods", Whole(52)),
            ("pay_period", Text("weekly")),
            ("scale", Whole(2)),
            ("study_loan", Flag(true)),
        ],
        "schedule_5_method_b_i" => vec![
            ("year", Text("2026-27")),
            ("earnings", Text("1700.00")),
            (
                "back_payment_periods",
                Json(
                    r#"[{"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}, {"earnings": "1600.00", "back_payment": "100.00", "withheld": "371.00"}]"#,
                ),
            ),
            ("pay_period", Text("weekly")),
            ("scale", Whole(2)),
            ("study_loan", Flag(true)),
        ],
        "schedule_5_method_b_ii" => vec![
            ("year", Text("2026-27")),
            ("earnings", Text("1800.00")),
            ("additional_payment", Text("2480.00")),
            ("year_to_date_earnings", Text("18000.00")),
            ("pay_periods_to_date", Whole(10)),
            ("earlier_additional_payments", Text("0.00")),
            ("earlier_additional_withholding", Text("0.00")),
            ("pay_period", Text("fortnightly")),
            ("scale", Whole(2)),
            ("study_loan", Flag(false)),
        ],
        // Schedule 15's example 2, for payments from 1 July 2026.
        "working_holiday_maker_withholding" => vec![
            ("year", Text("2026-27")),
            ("payment", Text("8173.20")),
            ("payments_before", Text("47000.00")),
            ("employer_registered", Flag(true)),
            ("tfn_quoted", Flag(true)),
            ("pay_period", Text("monthly")),
        ],
        // Schedule 9's worked example, for payments from 1 July 2026.
        "seniors_pensioners_withholding" => vec![
            ("year", Text("2026-27")),
            ("earnings", Text("662.95")),
            ("pay_period", Text("weekly")),
            ("status", Text("single")),
            ("medicare_levy_exemption", Text("none")),
        ],
        // The ATO's José example, for 2025-26.
        "seniors_pensioners_tax_offset" => vec![
            ("year", Text("2025-26")),
            ("rebate_income", Text("39000")),
            ("status", Text("single")),
        ],
        _ => Vec::new(),
    }
}
