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
    (
        "2027-28",
        &[
            (18200, Some(45000), "0.14"),
            (45000, Some(135000), "0.30"),
            (135000, Some(190000), "0.37"),
            (190000, None, "0.45"),
        ],
    ),
];

/// Foreign resident basic tax scale per income year, in the same form. Income
/// Tax Rates Act 1986, Schedule 7 Part II clause 1: the second and third
/// resident personal tax rates (Part I items 2 and 3 for the year) to $135,000
/// and $190,000, then 45%, from $0, for the 2024-25 and later years.
pub const FOREIGN_RESIDENT_TAX_SCALES: &[(&str, &[(i64, Option<i64>, &str)])] = &[
    (
        "2024-25",
        &[
            (0, Some(135000), "0.30"),
            (135000, Some(190000), "0.37"),
            (190000, None, "0.45"),
        ],
    ),
    (
        "2025-26",
        &[
            (0, Some(135000), "0.30"),
            (135000, Some(190000), "0.37"),
            (190000, None, "0.45"),
        ],
    ),
    (
        "2026-27",
        &[
            (0, Some(135000), "0.30"),
            (135000, Some(190000), "0.37"),
            (190000, None, "0.45"),
        ],
    ),
    (
        "2027-28",
        &[
            (0, Some(135000), "0.30"),
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
/// below the shading out point, WFT x the first of `shading_out_point` / the
/// second in whole dollars (the 'Medicare levy parameters' table's multiplier
/// and divisor).
pub struct MedicareScale {
    pub threshold: i64,
    pub formula_1_less: &'static str,
    pub shade_in: i64,
    pub shade_in_rate: &'static str,
    pub levy_rate: &'static str,
    pub shade_out_rate: &'static str,
    pub shading_out_point: (&'static str, &'static str),
}

/// The rules of one Schedule 1 beside its coefficients.
pub struct Schedule1Rules {
    /// Scale 4 tax rates: (resident, foreign resident).
    pub no_tfn_rates: (&'static str, &'static str),
    /// Share of the yearly tax offsets claimed that each pay is reduced by.
    pub tax_offset_rates: [(&'static str, &'static str); 4],
    /// WFT = (family threshold + per child x children) / 52, to the cent.
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
                shading_out_point: ("0.1000", "0.0800"),
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
                shading_out_point: ("0.0500", "0.0400"),
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
                shading_out_point: ("0.1000", "0.0800"),
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
                shading_out_point: ("0.0500", "0.0400"),
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

/// The general concessional contributions cap (ITAA 1997 s 291-20(2)) for the
/// earlier income years whose unused cap `contribution_caps` carries forward,
/// from 2018-19 (s 291-20(7)): Table 1.1 of the ATO's 'Contributions caps'
/// page (last updated 11 September 2026), read on 16 November 2026 at 17:23
/// AEDT. Later years' caps are in [`CONTRIBUTION_CAPS`].
pub const EARLIER_CONCESSIONAL_CAPS: &[(&str, &str)] = &[
    ("2018-19", "25000"),
    ("2019-20", "25000"),
    ("2020-21", "25000"),
    ("2021-22", "27500"),
    ("2022-23", "27500"),
    ("2023-24", "27500"),
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
/// years, so 2026-27 and 2027-28 have the 2025-26 figures until an amendment
/// changes them.
pub const MEDICARE_LOW_INCOME_THRESHOLDS: &[(&str, (&str, &str))] = &[
    ("2024-25", ("27222", "34027")),
    ("2025-26", ("28011", "35013")),
    ("2026-27", ("28011", "35013")),
    ("2027-28", ("28011", "35013")),
];
/// The same for a person entitled to the seniors and pensioners tax offset:
/// s 3(1) as Act No. 28, 2025, Sch 2 (2024-25 and later years) and Act No. 58,
/// 2026, Sch 5 (2025-26 and later years) set it.
pub const MEDICARE_SAPTO_THRESHOLDS: &[(&str, (&str, &str))] = &[
    ("2024-25", ("43020", "53775")),
    ("2025-26", ("44268", "55335")),
    ("2026-27", ("44268", "55335")),
    ("2027-28", ("44268", "55335")),
];
/// s 8(5) and (7), from the same Acts: the family income threshold, the
/// threshold for a person entitled to the seniors and pensioners tax offset,
/// and the increase for each dependent child or student.
pub const MEDICARE_FAMILY_THRESHOLDS: &[(&str, (&str, &str, &str))] = &[
    ("2024-25", ("45907", "59886", "4216")),
    ("2025-26", ("47238", "61623", "4338")),
    ("2026-27", ("47238", "61623", "4338")),
    ("2027-28", ("47238", "61623", "4338")),
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
pub const LOW_INCOME_TAX_OFFSET_YEARS: [&str; 4] = ["2024-25", "2025-26", "2026-27", "2027-28"];

pub const DIVISION_293: [(&str, &str); 2] = [("threshold", "250000"), ("rate", "0.15")];
pub const DIVISION_293_YEARS: [&str; 4] = ["2024-25", "2025-26", "2026-27", "2027-28"];

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

/// Unused leave paid on termination: Schedule 7's 32% rate for the parts a tax
/// offset caps at 30% (30% plus the 2% Medicare levy) and its $300 limit for a
/// small post-17 August 1993 payment, its 47% (45% for a foreign resident) of
/// the payment when no TFN is provided (payments from 1 July 2026), and the 5%
/// of pre-16 August 1978 long service leave that ITAA 1997 s 83-80(1) makes
/// assessable.
pub const UNUSED_LEAVE_PAYMENT: [(&str, &str); 5] = [
    ("flat_rate", "0.32"),
    ("pre_1978_assessable_rate", "0.05"),
    ("small_payment", "300"),
    ("no_tfn_rate", "0.47"),
    ("no_tfn_foreign_resident_rate", "0.45"),
];
pub const UNUSED_LEAVE_PAYMENT_YEARS: [&str; 1] = ["2026-27"];

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

/// Resident tax rates for the income years before [`RESIDENT_TAX_SCALES`], in
/// its form, for the recent accrual years of `lump_sum_arrears_tax_offset`
/// only: the ATO's "Tax rates – Australian resident" (last updated 13 August
/// 2026), read on 16 November 2026, for a full-year resident entitled to the
/// whole tax-free threshold. In 2014-15 to 2016-17, the temporary budget repair
/// levy years (Income Tax (Transitional Provisions) Act 1997 s 4-11(5)), the
/// last row is that levy: extra income tax of 2% on taxable income above
/// $180,000 (s 4-11(1) and Income Tax Rates Act 1986 s 34), which the ATO's
/// tables leave out and which overlaps the 45% row. 2011-12, with its flood
/// levy, and earlier years are not here.
pub const PRIOR_RESIDENT_TAX_SCALES: &[(&str, &[(i64, Option<i64>, &str)])] = &[
    ("2012-13", &SCALE_2012_13),
    ("2013-14", &SCALE_2012_13),
    ("2014-15", &SCALE_2014_15),
    ("2015-16", &SCALE_2014_15),
    (
        "2016-17",
        &[
            (18200, Some(37000), "0.19"),
            (37000, Some(87000), "0.325"),
            (87000, Some(180000), "0.37"),
            (180000, None, "0.45"),
            (180000, None, "0.02"),
        ],
    ),
    (
        "2017-18",
        &[
            (18200, Some(37000), "0.19"),
            (37000, Some(87000), "0.325"),
            (87000, Some(180000), "0.37"),
            (180000, None, "0.45"),
        ],
    ),
    ("2018-19", &SCALE_2018_19),
    ("2019-20", &SCALE_2018_19),
    ("2020-21", &SCALE_2020_21),
    ("2021-22", &SCALE_2020_21),
    ("2022-23", &SCALE_2020_21),
    ("2023-24", &SCALE_2020_21),
];
const SCALE_2012_13: [(i64, Option<i64>, &str); 4] = [
    (18200, Some(37000), "0.19"),
    (37000, Some(80000), "0.325"),
    (80000, Some(180000), "0.37"),
    (180000, None, "0.45"),
];
const SCALE_2014_15: [(i64, Option<i64>, &str); 5] = [
    (18200, Some(37000), "0.19"),
    (37000, Some(80000), "0.325"),
    (80000, Some(180000), "0.37"),
    (180000, None, "0.45"),
    (180000, None, "0.02"),
];
const SCALE_2018_19: [(i64, Option<i64>, &str); 4] = [
    (18200, Some(37000), "0.19"),
    (37000, Some(90000), "0.325"),
    (90000, Some(180000), "0.37"),
    (180000, None, "0.45"),
];
const SCALE_2020_21: [(i64, Option<i64>, &str); 4] = [
    (18200, Some(45000), "0.19"),
    (45000, Some(120000), "0.325"),
    (120000, Some(180000), "0.37"),
    (180000, None, "0.45"),
];

pub const CARRY_FORWARD_BALANCE_LIMIT: &str = "500000";

/// Statutory or benchmark interest rate by FBT year ended 31 March, for the
/// car operating cost method's deemed interest. Its keys are both car
/// worksheets' supported years; the 20% statutory rate applies in each.
pub const FBT_BENCHMARK_INTEREST_RATES: &[(&str, &str)] = &[("2026", "0.0862"), ("2027", "0.0827")];

/// Car parking threshold by FBT year ended 31 March, from the ATO's FBT rates
/// and thresholds page, read on 15 November 2026: the ATO publishes it there
/// and not in a taxation determination. Its keys are the car parking
/// worksheet's supported years.
pub const FBT_CAR_PARKING_THRESHOLDS: &[(&str, &str)] = &[("2026", "11.03"), ("2027", "11.48")];

/// The in-house property inputs the expense payment and property worksheets
/// share: they take them only under the in-house methods.
const FBT_LOWEST_PRICE: &str = "AUD, in_house_trade_price and in_house_public_price only and required there: the lowest price at which the provider sold identical property it manufactured, produced, processed or treated, at or about the provision time under an arm's length transaction, to manufacturers, wholesalers or retailers (FBTAA s 42(1)(a)(i)), or failing any such sale the lowest it could reasonably be expected to have sold it for, or to members of the public in similar circumstances (s 42(1)(a)(ii))";
const FBT_ARMS_LENGTH_PRICE: &str = "AUD, in_house_acquired only and required there: the provider's cost price for property it acquired in the ordinary course of business under an arm's length transaction, otherwise what it could reasonably be expected to have paid under one (FBTAA s 42(2))";
const FBT_IN_HOUSE_REDUCTION: &str = "AUD from 0 to 1000.00, every in-house method but in_house_salary_packaging and required there: the part of the employee's $1,000 in-house reduction for the FBT year (FBTAA s 62) their other in-house fringe benefits have not used; 1000.00 for their only in-house fringe benefit";

/// Housing indexation factors by FBT year ended 31 March and State or
/// Territory (FBTAA s 28), from Table 11 of the ATO's FBT rates and thresholds
/// page (last updated 20 May 2026), read on 15 November 2026: the ATO
/// publishes them there and not in a taxation determination. Its keys are the
/// housing worksheet's supported years.
pub const FBT_HOUSING_INDEXATION: &[(&str, [(&str, &str); 8])] = &[
    (
        "2026",
        [
            ("NSW", "1.076"),
            ("VIC", "1.066"),
            ("QLD", "1.076"),
            ("SA", "1.060"),
            ("WA", "1.100"),
            ("TAS", "0.988"),
            ("NT", "1.031"),
            ("ACT", "1.010"),
        ],
    ),
    (
        "2027",
        [
            ("NSW", "1.040"),
            ("VIC", "1.044"),
            ("QLD", "1.051"),
            ("SA", "1.051"),
            ("WA", "1.073"),
            ("TAS", "0.999"),
            ("NT", "1.029"),
            ("ACT", "1.007"),
        ],
    ),
];

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
        // The GST rate (GST Act s 9-70) has no year; the years are those read.
        "gst" => vec!["2025-26", "2026-27", "2027-28"],
        // The income years holding the ATO rate periods bundled in
        // data/fuel-tax-credit-rates.json, 1 July 2022 to 31 January 2027.
        "fuel_tax_credits" => vec!["2022-23", "2023-24", "2024-25", "2025-26", "2026-27"],
        "capital_gains" | "depreciation" => vec!["2025-26", "2026-27"],
        "quarterly_sg" => vec!["2025-26"],
        "resident_tax" => keys(RESIDENT_TAX_SCALES),
        "foreign_resident_tax" => keys(FOREIGN_RESIDENT_TAX_SCALES),
        "main_residence" => vec!["2024-25", "2025-26", "2026-27"],
        // ITAA 1997 s 25-25 sets no figure by year; its step 1 was last
        // substituted by Act No 15 of 2017 from 1 April 2017. From 2027-28 the
        // result is flagged for s 26-155, which it does not apply.
        "borrowing_expenses" | "borrowing_expenses_schedule" => vec![
            "2017-18", "2018-19", "2019-20", "2020-21", "2021-22", "2022-23", "2023-24", "2024-25",
            "2025-26", "2026-27", "2027-28",
        ],
        // Hire purchase: the years read.
        "hire_purchase_schedule" | "chattel_mortgage_schedule" | "loan_transactions" => {
            vec!["2024-25", "2025-26", "2026-27"]
        }
        // s 40-880 as amended by Act No 92 of 2020 applies to expenditure
        // incurred from 1 July 2020, unamended to compilation 268. It and
        // Subdivision 328-C set no figure by year, and the Register's one later
        // version (1 December 2026) changes neither, so 2027-28 is taken.
        "business_capital_expenditure" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27", "2027-28",
        ],
        // Subdivision 40-D as compiled on 1 October 2026: ss 40-285, 40-325
        // and 40-370 unamended since 2015, s 40-290 given a note by Act No 49 of
        // 2026 pointing to the s 40-291A choice from 2026-27. 2027-28 is refused:
        // Schedule 1 to that Act changes capital gains from 1 July 2027.
        "balancing_adjustment" => vec!["2024-25", "2025-26", "2026-27"],
        // Dealings from 1 July 2023, after the transitional rebate rules for
        // 2017 and earlier vintages ended (WETR 2009/2; the ATO's QC22756
        // and QC22757); the rate and cap are set by the Act, not indexed.
        "wine_equalisation_tax" => vec!["2023-24", "2024-25", "2025-26", "2026-27", "2027-28"],
        // Division 75 sets no figure by year; the years are gst's.
        "gst_margin_scheme" => vec!["2025-26", "2026-27", "2027-28"],
        // The 2026-27 rates and thresholds each revenue office published, read
        // on 19 November 2026.
        "payroll_tax" => vec!["2026-27"],
        // Tax periods from 1 July 2019, when the $10,000 refunds began
        // (Act No. 94 of 2019 Sch 2), to the last year whose thresholds the
        // ATO publishes (QC38161, 1 June 2026).
        "luxury_car_tax" => vec![
            "2019-20", "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        // The years whose car limit was read on 18 November 2026; 2027-28 is
        // refused as for balancing_adjustment, and its car limit is not yet
        // published.
        "trade_in" => vec!["2025-26", "2026-27"],
        "fbt" => vec!["2026", "2027"],
        "fbt_car_statutory" | "fbt_car_operating_cost" | "fbt_loan" => {
            keys(FBT_BENCHMARK_INTEREST_RATES)
        }
        // The car worksheets' years. The year ended 31 March 2028 is refused:
        // the change announced for 1 April 2027 was not law on 19 November
        // 2026.
        "fbt_ev_exemption_check" => vec!["2026", "2027"],
        // The methods use no published rate; the years are those the ATO's
        // return instructions and rates page were read for.
        "fbt_meal_entertainment" => vec!["2026", "2027"],
        "fbt_meal_entertainment_register" => vec!["2026", "2027"],
        "fbt_car_parking" => keys(FBT_CAR_PARKING_THRESHOLDS),
        // No published rate; the years are those the ATO's return
        // instructions and guide were read for.
        "fbt_debt_waiver" | "fbt_expense_payment" | "fbt_property" => vec!["2026", "2027"],
        "payg_withholding" | "payg_gross_up" => keys(PAYG_WITHHOLDING_COEFFICIENTS),
        "contribution_caps" => keys(CONTRIBUTION_CAPS),
        "pension_minimum" => vec!["2024-25", "2025-26", "2026-27", "2027-28"],
        "study_loan_repayment" => keys(STUDY_LOAN_REPAYMENT_THRESHOLDS),
        "study_loan_withholding" => keys(STUDY_LOAN_WITHHOLDING_COEFFICIENTS),
        "medicare_levy" => keys(MEDICARE_LOW_INCOME_THRESHOLDS),
        // 2027-28 only with no day without cover, whose surcharge is nil
        // whatever the thresholds, which are not yet published.
        "medicare_levy_surcharge" => [keys(SURCHARGE_THRESHOLDS), vec!["2027-28"]].concat(),
        "low_income_tax_offset" => LOW_INCOME_TAX_OFFSET_YEARS.to_vec(),
        "division_293" => DIVISION_293_YEARS.to_vec(),
        "division_296" => DIVISION_296_YEARS.to_vec(),
        "termination_payment" => TERMINATION_PAYMENT_YEARS.to_vec(),
        // ITAA 1997 ss 82-140 to 82-155 set no figure by year; the years whose
        // law was read as compiled on 19 November 2026.
        "etp_tax_free_component" => vec!["2026-27", "2027-28"],
        "unused_leave_payment" => UNUSED_LEAVE_PAYMENT_YEARS.to_vec(),
        "schedule_5_method_a" | "schedule_5_method_b_i" | "schedule_5_method_b_ii" => {
            SCHEDULE_5_YEARS.to_vec()
        }
        "working_holiday_maker_withholding" => WORKING_HOLIDAY_MAKER_YEARS.to_vec(),
        "seniors_pensioners_withholding" => keys(SENIORS_PENSIONERS_COEFFICIENTS),
        // The years checked against the regulations; the thresholds follow each
        // year's Schedule 7 rates, but a new year needs its own review.
        "seniors_pensioners_tax_offset" => vec!["2024-25", "2025-26", "2026-27", "2027-28"],
        // The years of receipt with rates for an earlier, recent accrual year,
        // to 2026-27: later years have not been checked against their law.
        "lump_sum_arrears_tax_offset" => [&PRIOR_RESIDENT_TAX_SCALES[1..], RESIDENT_TAX_SCALES]
            .into_iter()
            .flat_map(keys)
            .filter(|year| *year <= "2026-27")
            .collect(),
        // The years the ATO's return instructions and employer's guide were
        // read for; housing's indexed method also takes the year's factors.
        "fbt_residual" | "fbt_lafha" | "fbt_board" | "fbt_tax_exempt_body_entertainment" => {
            vec!["2026", "2027"]
        }
        // No published rate: s 58P(1)(e)'s $300 is the Act's own. The years
        // are those the other FBT benefit worksheets cover.
        "fbt_minor_benefit_check" => vec!["2026", "2027"],
        "fbt_entertainment_classifier" => vec!["2026", "2027"],
        "fbt_housing" => keys(FBT_HOUSING_INDEXATION),
        // The years whose definitions were read: the 1 July 2024 compilations
        // define them as the current ones do, and the ATO's 2025 and 2026
        // return instructions and its current pages apply them.
        "income_tests" => vec!["2024-25", "2025-26", "2026-27"],
        _ => Vec::new(),
    }
}

/// `SOURCE_REVIEWS[kind]`: (checked, passage).
pub fn source_review(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "gst" => ("2026-09-10", "GST fast facts: Charging GST"),
        "fuel_tax_credits" => (
            "2026-11-19",
            "Fuel Tax Act 2006 ss 41-5, 41-20, 43-5, 43-10, 47-5 and 47-10 at the ATO's legal database; the ATO's 'Rates - business' (QC44494, last updated 29 July 2026) and its pages for 2022-23 to 2026-27 (QC69964 and QC72768, last updated 27 June 2024; QC102637, 29 January 2025; QC105117, 28 April 2026; QC107638, 29 July 2026), their 192 rates equal to data.gov.au's 'Historical FTC rates' (August 2026); 'Working out your fuel tax credits' (QC44511, last updated 6 July 2021, step 3); 'Simplified fuel tax credits' (QC48520, 6 July 2021); 'Basic method for heavy vehicles' (QC64655, 2 June 2021: Table 1 and Jim's example); PCG 2016/2 paras 4 and 9 to 13; PCG 2021/2 paras 4 to 12 and its We Deliver example (paras 19 to 24)",
        ),
        "resident_tax" => (
            "2026-09-10",
            "Income Tax Rates Act 1986, Schedule 7 Part I clause 1",
        ),
        "foreign_resident_tax" => (
            "2026-11-14",
            "Income Tax Rates Act 1986 s 3(1) (prescribed non-resident, second and third resident personal tax rates) and Schedule 7 Parts I and II clause 1, compilations 63, 65 and 66; ATO Tax rates - foreign resident, 2024-25 and 2025-26; ITAA 1936 s 251S(1); ITAA 1997 s 61-110(1)",
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
        "borrowing_expenses" | "borrowing_expenses_schedule" => (
            "2026-11-19",
            "ITAA 1997 s 25-25 with its history notes, s 26-50 and s 26-155 on the ATO's legal database; Acts Interpretation Act 1901 ss 2G and 36, compilation 39 (28 March 2026); Borrowing expenses (last updated 22 May 2026): the Peter and the Fiona and Max examples",
        ),
        "business_capital_expenditure" => (
            "2026-11-16",
            "ITAA 1997 s 40-880, compilation 268 (1 October 2026), last amended by Act No 92 of 2020 (Schedule 3 items 25 to 27, applied by item 40(7) from 1 July 2020); TR 2011/6, the Commissioner's view, paras 10, 128 to 134, 160 to 212, 318A to 321 and examples 14, 16 to 18, 21 to 25, 40A and 41; Deductions for depreciating assets and capital expenses (last updated 28 August 2026), other capital expenses",
        ),
        "balancing_adjustment" => (
            "2026-11-18",
            "ITAA 1997 ss 40-25, 40-85, 40-230, 40-285, 40-290, 40-291A, 40-295, 40-300, 40-305, 40-325, 40-370, 104-235, 104-240, 118-5 and 118-10 (its example) and Subdivision 27-B, compilation 268 (1 October 2026), with the endnotes' amendment history; Treasury Laws Amendment (Tax Reform No. 1) Act 2026 (No 49 of 2026), Schedule 4 item 17 (s 40-291A from 2026-27); Guide to depreciating assets 2026, 'What happens if you no longer hold or use a depreciating asset?', its Bridget, Andrew and Louise examples and the car limit formula",
        ),
        "payroll_tax" => (
            "2026-11-19",
            "Payroll Tax Act 2007 (NSW) Sch 1 cl 1 (version current from 10 July 2026); Payroll Tax Act 2007 (Vic) s 3A and Sch 1 cl 1, 1A, 1B and 5 (authorised version 041, 1 May 2026); Payroll Tax Act 1971 (Qld) ss 10, 29 and 43I (reprint current as at 22 October 2026); Pay-roll Tax Assessment Act 2002 (WA) ss 8 and 13 (04-d0-03) and Pay-roll Tax Act 2002 (WA) s 5(4) (00-d0-02); Payroll Tax Act 2009 (SA) Sch 1 cl 1, 4 and 5 (version 31.3.2026); Payroll Tax Act 2008 (Tas) Sch 1 cl 1 and 4; Payroll Tax Act 2011 (ACT) Sch 1 (R20) with DI2026-151 s 5; Payroll Tax Act 2009 (NT) s 7A and Sch 1 cl 1, 3 and 5 (as in force at 1 July 2026); and each revenue office's rates and thresholds page",
        ),
        "gst_margin_scheme" => (
            "2026-11-19",
            "GST Act ss 9-90, 11-5, 75-5, 75-10, 75-11, 75-12, 75-13, 75-14, 75-15, 75-20, 75-35 and 195-1 ('approved valuation', 'long-term lease', 'margin scheme'), from the ATO's legal database; GSTR 2006/8 (paras 15 to 39F: the agreement, a conditional agreement, ineligible supplies and examples 1 and 2); the ATO's 'GST and the margin scheme' (last updated 19 August 2021)",
        ),
        "wine_equalisation_tax" => (
            "2026-11-19",
            "A New Tax System (Wine Equalisation Tax) Act 1999 ss 5-5, 5-50, 7-10, 9-5, 9-25, 9-30, 9-35, 9-40, 9-45, 13-5, 19-5, 19-15, 19-20, 19-25, 31-1 and 33-1 at the ATO's legal database, and compilation 32 (1 April 2026) at the Federal Register of Legislation for ss 19-15 and 19-25; Treasury Laws Amendment (Supporting Choice in Superannuation and Other Measures) Act 2026 (No. 12, 2026) Schedule 6 (the $400,000 cap for assessable dealings from 1 July 2026); WETR 2009/1 paras 143 to 153 and 159; WETR 2009/2 para 67 and note 37; the ATO's 'Wholesale sales' (QC22750) and 'Retail sales and own use' (QC22751), last updated 19 March 2025; 'Producer rebate' (QC22756, 30 June 2026); 'Calculate and claim your rebate' (QC22758, 2 July 2026); 'Reporting and paying WET' (QC22762, 19 March 2025); 'Wine equalisation tax (WET)' (QC33703, 1 May 2025); 'Instructions for completing your BAS' (QC17458, 14 September 2026)",
        ),
        "luxury_car_tax" => (
            "2026-11-19",
            "A New Tax System (Luxury Car Tax) Act 1999 ss 5-15, 5-20, 7-15, 9-5, 9-15, 15-20, 15-25, 15-30, 18-5, 18-10, 25-1 and 27-1 at the ATO's legal database, with their history notes (ss 18-5(3) and 18-10(3) substituted by Act No. 94 of 2019 for cars supplied or imported from 1 July 2019), and compilation 27 (1 April 2025) at the Federal Register of Legislation; Treasury Laws Amendment (Tax Incentives and Integrity) Act 2025 (No. 29, 2025) Schedule 1 (the 3.5 litre fuel-efficient rating from 1 July 2025 and item 4's continued 7 litre rating); Luxury Car Tax Imposition (General) Act 1999 and (Customs) Act 1999 s 4 (33%); the ATO's 'Luxury car tax rate and thresholds' (QC38161, 1 June 2026), 'Working out the LCT on a sale' (QC22107, 1 June 2026), 'Working out the LCT on an import' (QC22108, 1 June 2026), 'Luxury car tax credits and refunds' (QC39467, 16 July 2025) and the BAS page 'Luxury car tax (LCT)' (QC33694, 1 May 2025); 'Instructions for completing your BAS' (QC17458, 14 September 2026)",
        ),
        "trade_in" => (
            "2026-11-18",
            "As for balancing_adjustment, with ITAA 1997 ss 27-80, 27-95, 40-180, 40-185, 40-225, 40-230 and 40-365, compilation 268 (1 October 2026); GST Act ss 9-5, 9-70, 9-75, 11-5, 11-25, 11-30, 69-10 and 132-5, compilation 96 (1 January 2026); the ATO's 'Purchasing a motor vehicle' (last updated 3 June 2026: the 2026-27 car limit and Darren's example), 'Disposing of a motor vehicle' (last updated 29 May 2024: trade-ins) and 'GST and the disposal of capital assets' (last updated 24 May 2017: Ian's decreasing adjustment)",
        ),
        "hire_purchase_schedule" => (
            "2026-11-18",
            "ITAA 1997 Division 240 (ss 240-17, 240-20, 240-25, 240-55, 240-60, 240-65, 240-70, 240-75, 240-85, 240-105 and 240-110), s 40-230 and s 995-1 'termination amount', compilation 268 (1 October 2026); GST Act ss 9-70, 9-75, 9-90, 11-5, 11-25, 11-30, 29-10, 69-10 and 158-5, compilation 96 (1 January 2026); the ATO's 'GST – Hire purchase and leasing' (last updated 6 April 2017), 'Purchasing a motor vehicle' (last updated 3 June 2026: the 2026-27 car limit), 'Assets and exclusions' (last updated 27 May 2026: the 2024-25 and 2025-26 car limits) and the GST issues register 'Financial services - questions and answers' (issues 1.1, 1.3, 1.9, 1.15 and 1.17, updated to 7 September 2012 and 2 May 2013)",
        ),
        "chattel_mortgage_schedule" => (
            "2026-11-19",
            "ITAA 1997 ss 8-1, 240-10 and 995-1 'hire purchase agreement', compilation 268 (1 October 2026); GST Act ss 9-75, 9-90, 11-25, 11-30, 29-10, 40-5 and 69-10, compilation 96 (1 January 2026); the ATO's GST issues register 'Financial services - questions and answers' (issues 1.1, 1.14 and 1.19 to 1.21, updated to 2 May 2013), TR 93/16 (paras 2, 18 and 28 to 31) and 'Purchasing a motor vehicle' (last updated 3 June 2026: the 2026-27 car limit and Darren's example)",
        ),
        "loan_transactions" => (
            "2026-11-19",
            "ITAA 1997 s 8-1, compilation 268 (1 October 2026)",
        ),
        "fbt_car_statutory" => (
            "2026-10-29",
            "Fringe benefits tax - a guide for employers, 7.8 statutory formula method and examples 23 and 26; FBT return 2026 instructions, item 23A and example 6; Electric cars exemption, reportable fringe benefits",
        ),
        "fbt_ev_exemption_check" => (
            "2026-11-19",
            "FBTAA ss 7(1), 8A, 53(1), 135P and 136(1), compilation 97 (1 July 2026); Treasury Laws Amendment (Electric Car Discount) Act 2022 (No. 86 of 2022), as made, s 8A(2)(c) and (5) as inserted, Schedule 1 item 7 and Schedule 2 item 5; A New Tax System (Luxury Car Tax) Act 1999 ss 5-20 and 25-1, compilation of 1 April 2025; the ATO's 'Electric cars exemption' (last updated 1 April 2026: its Shelly, John and GHI Co examples), 'FBT on plug-in hybrid electric vehicles' (last updated 14 March 2025: its Sonia, Simon, Tony, Courtney and pool car examples), 'Luxury car tax rate and thresholds' (last updated 1 June 2026) and 'Electric car discount – more sustainable fringe benefits tax treatment of electric cars' (last updated 3 November 2026: not yet law)",
        ),
        "fbt_car_operating_cost" => (
            "2026-11-06",
            "Fringe benefits tax - a guide for employers, 7.9 operating cost method and example 31; FBT return 2026 instructions, item 23B and example 7; FBT rates and thresholds, benchmark interest rate; Electric cars exemption, reportable fringe benefits; Income Tax Assessment (Effective Life of Depreciating Assets) Determination 2025 (LI 2025/20, F2025L01097) and TR 2022/1, Table B, effective lives of cars",
        ),
        "fbt_loan" => (
            "2026-11-15",
            "FBTAA ss 16 to 19 and s 136(1), notional amount of interest and statutory interest rate, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, 8.5 and 8.8 and their examples; FBT return 2026 instructions, item 23C and example 8; FBT rates and thresholds, statutory or benchmark interest rate; Loan and debt waiver fringe benefits",
        ),
        "fbt_meal_entertainment" => (
            "2026-11-16",
            "FBTAA ss 37AB, 37BA, 37CB and 37CD, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, 14.8 methods of valuing meal entertainment; FBT return 2026 instructions, item 23P and example 18",
        ),
        "fbt_meal_entertainment_register" => (
            "2026-11-19",
            "FBTAA ss 37CA to 37CF, on the ATO's legal database; Fringe benefits tax - a guide for employers, 14.8 (chapter 14 last updated January 2020), 'Valid register' and 'Details to be included in register'",
        ),
        "fbt_car_parking" => (
            "2026-11-16",
            "FBTAA ss 39A, 39C, 39D, 39DA, 39E, 39FA to 39FE and 39GB to 39GF, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, chapter 16, 16.2.5, 16.2.8, examples 12, 13, 15, 17 and 18 and the 12 week register steps; FBT return 2026 instructions, item 23N and example 17; FBT rates and thresholds, car parking threshold; Car parking and FBT",
        ),
        "fbt_debt_waiver" => (
            "2026-11-15",
            "FBTAA ss 14 and 15, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, 8.1 and 8.2; FBT return 2026 instructions, item 23D and example 9; Loan and debt waiver fringe benefits",
        ),
        "fbt_expense_payment" => (
            "2026-11-16",
            "FBTAA ss 20, 22A, 23, 24, 48, 49 and 62 and s 136(1), providers portion and recipients portion, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, 9.2 to 9.4 and their examples, 17.3, 18.4 and its example and 19.5; FBT return 2026 instructions, item 23E and example 10",
        ),
        "fbt_property" => (
            "2026-11-15",
            "FBTAA ss 40, 42, 43, 44 and 62 and s 136(1), notional value, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, 17.2 to 17.5 and their examples and 19.5; FBT return 2026 instructions, item 23K and example 14",
        ),
        "depreciation" => (
            "2026-11-13",
            "Decline in value: diminishing value and prime cost methods, with the later-year diminishing value example; taxable use; ITAA 1997 ss 40-70(1), 40-72 and 40-75(1) and (7), base value and the limit on decline in a later year",
        ),
        "quarterly_sg" => (
            "2026-09-10",
            "Super guarantee, tables 21 and 24, 2025-26 rows",
        ),
        "payg_gross_up" => (
            "2026-11-16",
            "Schedule 1 (NAT 1004) and Schedule 8 (NAT 3539) for payments from 1 July 2026: Schedule 1's weekly coefficients, Schedule 8's component rates and its coefficients with the component, examples 1 to 3, calculating fortnightly, monthly and quarterly amounts, rounding and software; earlier payments as payg_withholding and study_loan_withholding record",
        ),
        "payg_withholding" => (
            "2026-11-14",
            "Schedule 1 (NAT 1004) for payments from 1 July 2026, and for payments from 1 July 2024 to 30 June 2026: weekly coefficients and scale 4, using a formula (rounding and the 53 and 27 pay tables), working out the weekly earnings (quarterly pays and tax offsets), Medicare levy adjustment, examples of calculating the withholding amounts, sample data",
        ),
        "contribution_caps" => (
            "2026-11-16",
            "Contributions caps, tables 1.1, 2 and 4 and unused concessional cap carry forward; Concessional contributions cap, carry forward unused contribution cap amounts and the Sam example; ITAA 1997 s 291-20(1) to (7), compilation 268; Non-concessional contributions cap, bring-forward arrangement",
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
        "etp_tax_free_component" => (
            "2026-11-19",
            "ITAA 1997 ss 82-10(6), 82-140, 82-145, 82-150 and 82-155 and s 995-1(1) 'last retirement day', on the ATO's legal database; ATO 'Tax-free component of ETPs' (last updated 24 July 2025), Jackson's example; ATO Employment termination payment with an invalidity segment worksheet (NAT 71949, June 2014), steps 1 to 9",
        ),
        "unused_leave_payment" => (
            "2026-11-16",
            "ATO Schedule 7, tax table for unused leave payments on termination of employment (from 1 July 2026): using this schedule, working out the withholding amount, the rates and labels table, rounding, the marginal rate calculation, normal gross earnings, the long service leave example and when a TFN has not been provided; ITAA 1997 ss 83-10, 83-15 and 83-70 to 83-115, compilation 268 (1 October 2026); ATO STP Phase 2 employer reporting guidelines, 'When an employee transfers or leaves'; Superannuation Guarantee (Administration) Act 1992 ss 6(1) 'ordinary time earnings' and 10A(1), compilation 78 (1 July 2026); ATO 'What payments are qualifying earnings', table 13",
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
            "2026-11-15",
            "Schedule 9 (NAT 4466, seniors and pensioners) for payments from 1 July 2026 and for payments from 1 July 2024 to 30 June 2026: using this schedule, working out the withholding amount and its example, calculating fortnightly or monthly withholding amounts, using a formula and the coefficients for scales 1 to 9, rounding, claiming tax offsets, Medicare levy exemptions and adjustments, study and training support loan debts, and the weekly tax table; the study and training support loans weekly tax table it refers to (NAT 2173, from 1 July 2026 and from 24 September 2025) with Schedule 8's component rates and conversions",
        ),
        "seniors_pensioners_tax_offset" => (
            "2026-11-14",
            "ITAA 1936 ss 6(1) (rebate income) and 160AAAA, compilation 193 (1 October 2026); Income Tax Assessment (1936 Act) Regulations 2025 ss 4 and 9 to 12 and 23 (F2025L01060), and the Income Tax Assessment (1936 Act) Regulation 2015 ss 4 and 9 to 12, compilation 8; ATO Seniors and pensioners tax offset: income limits, rates and thresholds for 2025-26, tax offset reduction and rounding, examples and transferring unused SAPTO with example 9; Do you need to lodge a tax return? 2025, reason 2",
        ),
        "lump_sum_arrears_tax_offset" => (
            "2026-11-16",
            "ITAA 1936 Part III Division 17 Subdivision AB, ss 159ZR to 159ZRD, compilation 193 (1 October 2026), with the endnotes' amendment history (ss 159ZRA to 159ZRD as added by Act No. 153, 1988; s 159ZR last amended by Act No. 112, 2015); ATO Lump sum payment in arrears: qualifying payments, multiple payments in one income year and the Yan example, the 10% threshold test, the LSPIA tax offset and the Sudeep, Anthony and Heather examples, the Medicare levy exemption and the MLS tax offset; ATO Tax rates – Australian resident, 2012-13 to 2026-27; Income Tax (Transitional Provisions) Act 1997 s 4-11, compilation 104 (1 October 2026), and the Income Tax Rates Amendment (Temporary Budget Repair Levy) Act 2014, as made",
        ),
        "fbt_residual" => (
            "2026-11-15",
            "FBTAA ss 45, 48 to 52 and 62 and s 136(1), notional value and recipients contribution, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, chapter 18, 18.4 and 18.7 and their examples; FBT return 2026 instructions, item 23M and example 16",
        ),
        "fbt_housing" => (
            "2026-11-15",
            "FBTAA ss 25 to 28 and 58ZC and s 136(1), recipients rent and tenancy period, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, chapter 10, 10.4 and 10.5 and their examples; FBT return 2026 instructions, item 23F and example 11; FBT rates and thresholds, Table 11, housing indexation",
        ),
        "fbt_lafha" => (
            "2026-11-15",
            "FBTAA ss 30 to 31H and s 136(1), accommodation component, food component and statutory food amount, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, chapter 11, 11.4 to 11.6 and their examples; FBT return 2026 instructions, item 23G and example 12",
        ),
        "fbt_board" => (
            "2026-11-15",
            "FBTAA ss 35 to 37 and s 136(1), board meal and recipients contribution, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, chapter 13 and its example; FBT return 2026 instructions, item 23J and example 13",
        ),
        "fbt_tax_exempt_body_entertainment" => (
            "2026-11-15",
            "FBTAA ss 38 and 39 and s 136(1), non-deductible exempt entertainment expenditure, compilation 97 (1 July 2026); Fringe benefits tax - a guide for employers, chapter 15, 15.9 and its per head example; FBT return 2026 instructions, item 23L and example 15",
        ),
        "fbt_minor_benefit_check" => (
            "2026-11-19",
            "FBTAA ss 58P, 136(1) ('fringe benefit' para (g) and 'eligible premises') and 152B, from the ATO's legal database (s 58P(1)(b) repealed by Act No. 88 of 2013); TR 2007/12 (paras 8 to 23 and examples 1 to 13)",
        ),
        "fbt_entertainment_classifier" => (
            "2026-11-16",
            "ITAA 1997 Division 32, s 27-5 and s 995-1, compilation 268 (1 October 2026); FBTAA ss 5E, 20, 37A to 37CF, 40, 41, 45, 58P and 135P(1) and s 136(1), compilation 97 (1 July 2026); Fringe Benefits Tax Assessment Regulations 2018, regulations 6 to 11, as made; ITAA 1936 ss 51AEA and 51AEB, compilation 193 (1 October 2026); GST Act s 69-5, compilation 96 (1 January 2026); TR 97/17, paras 7 to 143 and the Part A table, and TR 2007/12, paras 8 to 22 and examples 1 to 3, as the Commissioner's view",
        ),
        "income_tests" => (
            "2026-11-16",
            "ITAA 1997 s 995-1 (income for surcharge purposes, reportable superannuation contributions, total net investment loss, adjusted taxable income for offsets, exempt foreign employment income, assessable FHSS released amount) and ss 4-15, 61-580, 291-15, 293-20, 301-20 and 313-20, compilation 268 (1 October 2026); Medicare Levy Act 1986 s 3(1), compilation 53 (1 July 2026); A New Tax System (Medicare Levy Surcharge—Fringe Benefits) Act 1999 s 3(1), compilation 42 (1 July 2026); ITAA 1936 s 6(1) (rebate income, adjusted taxable income for rebates, adjusted fringe benefits total) and s 251S, compilation 193 (1 October 2026); HESA ss 154-5, 154-16 and 154-17, compilation 105 (17 October 2026); A New Tax System (Family Assistance) Act 1999 Sch 3 cls 2, 4, 5, 7 and 8, compilation 124 (1 July 2026); FBTAA ss 135N, 135P and 135Q (1 July 2026); SISR reg 6.01(2) preservation age, compilation 159 (1 July 2026); Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 8, compilation 28 (1 July 2018); and the 1 July 2024 compilations of the ITAA 1997 (250), ITAA 1936 (184) and Family Assistance Act (119), compared; ATO Income tests (last updated 8 June 2026), Rebate income and Adjusted taxable income for you and your dependants for 2025 and 2026, IT1 Total reportable fringe benefits amounts 2025 and 2026, A3 Government super contributions 2025 and 2026, Medicare levy surcharge income, thresholds and rates (22 June 2026), When you must repay your loan (13 October 2026), Division 293 tax on concessional contributions by high-income earners (24 August 2026), Exempt income and other tax obligations (3 June 2026) and Fringe benefits tax - rates and thresholds (20 May 2026)",
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
        "fuel_tax_credits" => (
            "tax-examples-gst-and-other-indirect-taxes",
            "12-300",
            "2026-06-30",
            "2026-11-19",
        ),
        "resident_tax" => (
            "tax-examples-individuals",
            "7-010",
            "2025-06-30",
            "2026-09-15",
        ),
        "foreign_resident_tax" => (
            "ATO Tax rates - foreign resident",
            "foreign resident tax rates 2024-25 and 2025-26",
            "2026-06-01",
            "2026-11-14",
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
        "borrowing_expenses" | "borrowing_expenses_schedule" => (
            "tax-examples-deductions",
            "4-160",
            "2026-06-30",
            "2026-11-19",
        ),
        "business_capital_expenditure" => (
            "TR 2011/6",
            "examples 14, 16 to 18, 21 to 25, 40A and 41",
            "2011-11-30",
            "2026-11-16",
        ),
        "fbt_minor_benefit_check" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-060",
            "2026-06-30",
            "2026-11-19",
        ),
        "balancing_adjustment" => (
            "Guide to depreciating assets 2026; ITAA 1997 s 118-10(1)",
            "Bridget, Andrew and Louise examples; the Gayle example",
            "2026-06-01",
            "2026-11-18",
        ),
        "payroll_tax" => (
            "Revenue office payroll tax pages: Revenue NSW thresholds and rates; SRO Victoria payroll tax surcharges; QRO payroll tax deductions; RevenueWA employer guide; SRO Tasmania annual adjustment return guideline",
            "Revenue NSW's 2 examples; SRO Victoria example 2; QRO's annual deduction example; RevenueWA's $1,104,000; SRO Tasmania examples 1 and 2",
            "2026-07-01",
            "2026-11-19",
        ),
        "gst_margin_scheme" => (
            "tax-examples-gst-and-other-indirect-taxes",
            "12-200",
            "2026-06-30",
            "2026-11-19",
        ),
        "wine_equalisation_tax" => (
            "gst-wine-equalisation-tax",
            "22-280, 22-290",
            "2024-12-31",
            "2026-11-19",
        ),
        "luxury_car_tax" => (
            "tax-examples-gst-and-other-indirect-taxes",
            "12-340",
            "2026-06-30",
            "2026-11-19",
        ),
        "trade_in" => (
            "ATO GST pages: Purchasing a motor vehicle; GST and the disposal of capital assets",
            "Darren's example; Ian's example",
            "2026-06-03",
            "2026-11-18",
        ),
        "hire_purchase_schedule" => (
            "ATO Legal database, ITAA 1997 Division 240",
            "s 240-60(1) steps 1 to 4, with s 240-60 as amended by No 79 of 2010",
            "2026-11-15",
            "2026-11-15",
        ),
        "chattel_mortgage_schedule" => (
            "GST: Transport, Travel and Vehicles",
            "12-110 (the car limit on input tax credits)",
            "2025-06-03",
            "2026-11-19",
        ),
        "loan_transactions" => (
            "ITAA 1997, compilation 268",
            "s 8-1",
            "2026-10-01",
            "2026-11-19",
        ),
        "fbt_car_statutory" | "fbt_car_operating_cost" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-020",
            "2026-06-30",
            "2026-10-29",
        ),
        "fbt_ev_exemption_check" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-045",
            "2026-06-30",
            "2026-11-19",
        ),
        "fbt_loan" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-100",
            "2026-06-30",
            "2026-11-15",
        ),
        "fbt_meal_entertainment" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-220",
            "2026-06-30",
            "2026-11-15",
        ),
        "fbt_meal_entertainment_register" => (
            "ATO Fringe benefits tax - a guide for employers, chapter 14",
            "14.8, Valid register and Details to be included in register",
            "2020-01-01",
            "2026-11-19",
        ),
        "fbt_car_parking" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-055",
            "2026-06-30",
            "2026-11-15",
        ),
        "fbt_debt_waiver" => (
            "ATO FBT return 2026 instructions, item 23",
            "item 23D, example 9",
            "2026-04-01",
            "2026-11-15",
        ),
        "fbt_expense_payment" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-240, 3-340",
            "2026-06-30",
            "2026-11-15",
        ),
        "fbt_property" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-260",
            "2026-06-30",
            "2026-11-15",
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
        "payg_gross_up" => (
            "ATO Schedule 1 sample data; ATO Schedule 8 statement of formulas for STSL components, and F2025L01016 Schedule 8 for 2025-26",
            "withholding amounts: weekly, fortnightly and monthly tables, each row's net pay grossed up; Schedule 8 examples 1 to 3, each pay's net after the Schedule 1 amount and the component grossed up",
            "2026-06-17",
            "2026-11-16",
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
        "etp_tax_free_component" => (
            "superannuation-superannuation-benefits-termination-payments",
            "8-850",
            "2026-06-30",
            "2026-11-19",
        ),
        "unused_leave_payment" => (
            "ATO Schedule 7 tax table for unused leave payments on termination of employment",
            "example: long service leave (Norman)",
            "2026-06-17",
            "2026-11-16",
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
            "the worked example ($662.95 a week) and every row of the weekly tax table; with a study loan, the tax-free threshold column of every row of the study and training support loans weekly tax table (NAT 2173) from 1 July 2026",
            "2026-06-17",
            "2026-11-15",
        ),
        "seniors_pensioners_tax_offset" => (
            "ATO Seniors and pensioners tax offset",
            "rates and thresholds for 2025-26, the examples (Clare and Roy, Deb and Ivan, Ying and Li Jun, Marko, Simon, José, Keith and Jean, Vanh and Mai) and example 9",
            "2026-10-02",
            "2026-11-14",
        ),
        "lump_sum_arrears_tax_offset" => (
            "ATO Lump sum payment in arrears",
            "the 10% threshold test and the Sudeep, Anthony, Heather and Yan examples",
            "2026-06-08",
            "2026-11-16",
        ),
        "fbt_housing" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-120",
            "2026-06-30",
            "2026-11-15",
        ),
        "fbt_lafha" => (
            "tax-examples-fringe-benefits-tax-fbt",
            "3-160, 3-180",
            "2026-06-30",
            "2026-11-15",
        ),
        "income_tests" => (
            "ATO income test pages",
            "Christina's repayment income (When you must repay your loan) and Tom's income for MLS purposes (Medicare levy surcharge income, thresholds and rates)",
            "2026-10-13",
            "2026-11-16",
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
        "fuel_tax_credits" => vec![
            (
                "small_claimant",
                "boolean, required with a basic_heavy_vehicle or litres_from_cost line or rate_at_period_end and refused otherwise: true states that the fuel tax credit entitlement for the previous 12 months was $10,000 or less (for the basic method alone, PCG 2021/2 footnote 3 also accepts no claim in those 12 months) and is reasonably expected to be $10,000 or less for the current 12 months (PCG 2016/2 para 4; PCG 2021/2 para 6); false is refused",
            ),
            (
                "rate_at_period_end",
                "object with start and end (YYYY-MM-DD), a tax period inside the income year that holds every line's acquisition: PCG 2016/2 Method 2 then takes every line at the rate in force on end (para 11); refused across a rate change other than indexation alone, when end is after the last day a rate is held for, and for lpg or lng_cng lines",
            ),
            (
                "return_date",
                "YYYY-MM-DD, the day the return taking these credits into account is given, not before any line's acquisition: lines acquired more than 4 years before it are listed in time_limit_lines, with the s 47-5(2) warning",
            ),
        ],
        "balancing_adjustment" => vec![(
            "car_limit",
            "for a car only, and then required: null when s 40-230 did not reduce its cost to the car limit, or an object with limit (AUD, the car limit for the financial year in which the car was first used for any purpose), second_element_cost (AUD) and cost_ignoring_car_limit (AUD, the first and second elements ignoring the car limit, after Subdivision 27-B), cost then being limit plus second_element_cost: ITAA 1997 s 40-325",
        )],
        "fbt_ev_exemption_check" => vec![
            (
                "commitment_before_1_april_2025",
                "boolean, for a plug_in_hybrid_electric vehicle only and then required: true when, before 1 April 2025, the employer, the employee or an associate of either committed (a financially binding commitment, not an option to extend) to the car's application or availability, in respect of the employee's employment by the employer, for a period that began before 1 April 2025 and includes the benefit (Electric Car Discount Act 2022 Sch 2 item 5(2)(b))",
            ),
            (
                "new_commitment_from_1_april_2025",
                "boolean, for a plug_in_hybrid_electric vehicle only and then required: true when, on or after 1 April 2025 and before or at the benefit, any of them committed again to the car's application or availability for a period that includes the benefit, as an exercised extension, a break in a novation, a change to the lease payments or residual value, or a change of employer does (item 5(2)(c); the ATO's 'FBT on plug-in hybrid electric vehicles')",
            ),
            (
                "exempt_benefit_before_1_april_2025",
                "boolean, for a plug_in_hybrid_electric vehicle only and then required: true when a car benefit relating to the car was provided before 1 April 2025 and was exempt under s 8A as it then read (item 5(2)(d) and (e))",
            ),
        ],
        "payroll_tax" => vec![(
            "regional_employer",
            "boolean, for Victoria only and then required: true when the employer is a regional employer, one that pays at least 85% of its taxable wages to regional employees (Payroll Tax Act 2007 (Vic) s 3A)",
        )],
        "wine_equalisation_tax" => vec![
            (
                "rebate_claimed_earlier_in_year",
                "AUD, required with a dealing's rebate_conditions and refused otherwise: the producer rebates already claimed for earlier tax periods of the financial year, the group's when associated_producer is true; the cap left for this period is the year's cap less it (WET Act s 19-15(2) and (3); WETR 2009/2 para 67)",
            ),
            (
                "associated_producer",
                "boolean, required with a dealing's rebate_conditions and refused otherwise: true states that the producer is an associated producer of one or more other producers at any time in the financial year (WET Act s 19-20), so the cap is the group's",
            ),
        ],
        "gst_margin_scheme" => vec![
            (
                "acquisition_consideration",
                "AUD, with method consideration and then required: the consideration you gave for acquiring the entire interest, unit or lease, without improvements, development work, legal fees, stamp duty or other acquisitions in relation to the property (GST Act ss 75-10(2) and 75-14)",
            ),
            (
                "unpaid_acquisition_consideration",
                "AUD, with method consideration and then required: the part of acquisition_consideration not yet paid when you make the supply, 0.00 when paid in full (s 75-12)",
            ),
            (
                "approved_valuation",
                "AUD, with method approved_valuation and then required: a valuation of the entire interest, unit or lease made in accordance with the Commissioner's requirements (s 75-35), as at valuation_day",
            ),
            (
                "valuation_day",
                "YYYY-MM-DD date, with method approved_valuation and then required: the day the valuation is as at, which s 75-10(3) fixes: 1 July 2000 (items 1 and 3), the date of effect of your registration or the earlier day you applied (item 2), or the day of the supply (item 4)",
            ),
            (
                "registered_on_1_july_2000",
                "boolean, with method approved_valuation and then required: true when you were registered or required to be registered for GST on 1 July 2000 (s 75-10(3) item 2)",
            ),
            (
                "government_unimproved_on_1_july_2000",
                "boolean, with method approved_valuation and then required: true when you are the Commonwealth, a State or a Territory and there were no improvements on the land or premises on 1 July 2000 (s 75-10(3) item 4)",
            ),
            (
                "subdivision_share",
                "decimal, more than 0 and at most subdivision_whole, with subdivision_whole: for a supply of part of the land you acquired, its measure on a reasonable basis such as area or expected sale price (s 75-15)",
            ),
            (
                "subdivision_whole",
                "decimal, with subdivision_share: the same measure of all the land you acquired (s 75-15)",
            ),
        ],
        "trade_in" => vec![
            (
                "car_limit",
                "the traded-in asset's, as for balancing_adjustment: for a car only, and then required",
            ),
            (
                "adjusted_input_tax_credit",
                "AUD, the traded-in asset's input tax credit as adjusted (GST Act s 132-5(2)); required when trade_in_taxable is true and refused otherwise",
            ),
            (
                "full_input_tax_credit",
                "AUD, the input tax credit for the traded-in asset had it been acquired solely for a creditable purpose (GST Act s 132-5(2)); required when trade_in_taxable is true and refused otherwise",
            ),
            (
                "replacement_luxury_car_tax",
                "AUD, the luxury car tax included in replacement_price, 0.00 for none; required for a passenger_car and refused otherwise",
            ),
            (
                "creditable_purpose",
                "decimal fraction above 0 and at most 1, the extent of creditable purpose of the replacement's acquisition (GST Act s 11-30); required when replacement_creditable_acquisition is true and refused otherwise",
            ),
        ],
        "chattel_mortgage_schedule" => vec![
            (
                "instalment",
                "AUD, the level instalment, from which the rate is found; give it or annual_rate, not both",
            ),
            (
                "annual_rate",
                "decimal fraction from 0 to 1, the stated annual rate, charged annual_rate / 12 a month or / 52 a week, from which the instalment is worked out; for monthly or weekly instalments in arrears only; give it or instalment, not both",
            ),
            (
                "frequency",
                "weekly, fortnightly, monthly, quarterly or annual, the instalments' frequency; monthly when omitted",
            ),
            (
                "deposit",
                "AUD, paid at the purchase towards the price, beside the amount financed; 0.00 when omitted",
            ),
            (
                "trade_in",
                "AUD, the trade-in allowance credited at the purchase towards the price; 0.00 when omitted",
            ),
            (
                "gst",
                "object of GST facts, for the purchaser's input tax credit on the price: as hire_purchase_schedule takes them. A passenger_car worth more than the car limit and not quoted for has its credit capped at 1/11 of the limit (GST Act s 69-10(1))",
            ),
        ],
        "borrowing_expenses_schedule" => vec![(
            "other_borrowing_expenses",
            "object of income years (such as \"2025-26\") and AUD amounts: the income-producing share of borrowing expenses incurred that year on loans not in loans, for the s 25-25(6) $100 test",
        )],
        "loan_transactions" => vec![(
            "rate_changes",
            "list of objects in date order, each with date (YYYY-MM-DD, after start_date and by end_date, one change a day) and annual_rate (decimal fraction from 0 to 1), the rate from that day",
        )],
        "hire_purchase_schedule" => vec![
            (
                "instalment",
                "AUD, the level instalment; required without segments and refused with them",
            ),
            (
                "instalments",
                "integer, the number of instalments: 1 to 360 monthly, 1,565 weekly, 782 fortnightly, 120 quarterly or 30 annual; required without segments and refused with them",
            ),
            (
                "frequency",
                "weekly, fortnightly, monthly, quarterly or annual, the level instalments' frequency; monthly when omitted; omit with segments",
            ),
            (
                "segments",
                "list of objects in place of instalment, instalments and frequency, each with start (YYYY-MM-DD: the first segment's is the start_date, each later one's the end of the segment before it), count (integer of 1 or more), amount (AUD, 0.00 for a payment holiday) and frequency (all weekly or fortnightly, or all monthly, quarterly or annual), ending within 30 years",
            ),
            (
                "deposit",
                "AUD, paid at or before the start for the cost of the goods, beside the amount financed (ITAA 1997 s 240-25(3)); 0.00 when omitted",
            ),
            (
                "trade_in",
                "AUD, credited by the financier as paid at or before the start for the cost of the goods (s 240-25(3)); 0.00 when omitted",
            ),
            (
                "payout",
                "object with date (YYYY-MM-DD, the end of an arrangement payment period before the end of the term), amount (AUD) and known_at_start (boolean: false states the payout was neither known nor expected at the start, so the reasonable estimate then was no payout; true is refused): an early payout replacing every payment due on or after its date; refused with gst",
            ),
            (
                "gst",
                "object of GST facts, for an input tax credit: registered (boolean); when true, accounting_basis (cash or accruals), creditable_purpose (decimal fraction from 0 to 1) and asset (passenger_car, a car GST Act s 69-10 applies to, or other); for a passenger_car, first_used_year (2024-25 to 2026-27), gst_inclusive_market_value (AUD), luxury_car_tax (AUD) and quotes_abn_for_luxury_car_tax (boolean). A missing fact gives no GST figure, and nor does a passenger_car worth more than the car limit and not quoted for (gst.review says why)",
            ),
        ],
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
        "contribution_caps" => vec![
            (
                "earlier_concessional_contributions",
                "list instead of unused_concessional_cap: one object for each earlier income year, each year from the 5th before the year to the one before it and any earlier years back to 2018-19, without a gap: year (YYYY-YY), concessional_contributions (AUD) and, where they exceed that year's cap, total_super_balance (AUD, at the 30 June before that year)",
            ),
            (
                "defined_benefit_interest",
                "boolean, required with earlier_concessional_contributions: whether the person had a defined benefit interest in any year given; true is refused",
            ),
            (
                "proposed_concessional_contribution",
                "AUD, with earlier_concessional_contributions only: a further concessional contribution for the year; the unused cap it applies and what is left are worked out after it; omit for none",
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
        // Scale 4's residency, which payg_withholding lists first, then a
        // study loan, with the pay date as seniors_pensioners_withholding takes it.
        "payg_gross_up" => vec![
            optional_inputs("payg_withholding")[0],
            (
                "study_loan",
                "boolean, whether the payee declared a study and training support loan debt and claimed no Medicare levy reduction for a spouse or dependants and low family income; true grosses up study-loan-withholding's amount, which adds Schedule 8's component, for 2026-27 and 2025-26 pays made from 24 September 2025; not with scale 4 or foreign_resident; omit for false",
            ),
            optional_inputs("seniors_pensioners_withholding")[1],
        ],
        "study_loan_withholding" => vec![(
            "pay_date",
            "YYYY-MM-DD date the payment is made, within the income year; required for 2025-26, whose Schedule 8 coefficients apply from 24 September 2025",
        )],
        "seniors_pensioners_withholding" => vec![
            (
                "study_loan",
                "boolean, whether the payee declared a study and training support loan debt and claimed no Medicare levy reduction for a spouse or dependants and low family income; true adds Schedule 8's tax-free threshold component for the pay date, for 2026-27 and 2025-26 pays made from 24 September 2025; omit for false",
            ),
            (
                "pay_date",
                "YYYY-MM-DD date the payment is made, within the income year, with study_loan true only; required with it for 2025-26, whose Schedule 8 coefficients apply from 24 September 2025",
            ),
        ],
        "fbt_meal_entertainment_register" => vec![(
            "register_years_expenditure",
            "list of decimal strings, required when year_ended is a later year of the register's validity and refused otherwise: the meal entertainment expenses (FBTAA s 37CB(3)) of each FBT year from the register's first valid year to the year before year_ended, oldest first (s 37CD(3))",
        )],
        "fbt_meal_entertainment" => vec![
            (
                "register_fringe_benefits_value",
                "AUD, 12_week_register only and required there: the total value of the meal entertainment fringe benefits in the 12 week register (FBTAA s 37CB(2))",
            ),
            (
                "register_meal_entertainment_value",
                "AUD, 12_week_register only and required there: the total value of all meal entertainment in the register, more than nil and at least register_fringe_benefits_value",
            ),
            (
                "register_first_year",
                "integer, 12_week_register only and required there: the FBT year ended 31 March the register was first valid for, from 4 years before year_ended to year_ended (FBTAA s 37CD); whether a later register replaced it is the operator's to establish",
            ),
            (
                "register_years_expenditure",
                "list of AUD decimal strings, 12_week_register only and required there: the employer's total meal entertainment expenses, excluding amounts employees paid as for meal_entertainment_expenditure (FBTAA ss 37AB and 37CB(3)), for each FBT year from register_first_year to the year before year_ended, oldest first, so empty when they are the same year; the register ceased to be valid at the end of a year whose expenses were more than 20% higher than its first valid year's (s 37CD(3))",
            ),
        ],
        "fbt_car_parking" => vec![
            (
                "benefits",
                "integer from 0, commercial_parking_station, market_value and average_cost only and required there: the car parking fringe benefits provided in the FBT year",
            ),
            (
                "lowest_fee",
                "AUD, commercial_parking_station only and required there: the lowest all-day fee any commercial parking station within 1 km charged on each day a benefit was provided (FBTAA s 39C), the same on every one of those days; a fee for another period is converted under s 39E",
            ),
            (
                "market_value",
                "AUD, market_value only and required there: the amount the recipient could reasonably be expected to have paid the provider for each benefit at arm's length (FBTAA s 39D(2)(a)), the same for every benefit, as a suitably qualified valuer at arm's length reported it before the declaration date in the approved form (s 39D(3))",
            ),
            (
                "first_day_fee",
                "AUD, average_cost only and required there: the lowest all-day fee any commercial parking station within 1 km charged on the day a car parking benefit was first provided in the FBT year (FBTAA s 39DA(3), A)",
            ),
            (
                "last_day_fee",
                "AUD, average_cost only and required there: the lowest all-day fee any commercial parking station within 1 km charged on the day a car parking benefit was last provided in the FBT year (FBTAA s 39DA(3), B)",
            ),
            (
                "employee_contribution",
                "AUD, commercial_parking_station, market_value, average_cost and spaces only and required there: the recipients' payments for these benefits in total, net of any reimbursement, each benefit's payment counted only up to that benefit's value (an excess on one benefit reduces no other); with spaces, every relevant recipients contribution for the employees the election covers, for the FBT year (FBTAA s 39FE)",
            ),
            (
                "space_groups",
                "list, spaces only and required there: one object for each group of spaces with at least one car parking benefit in the FBT year for an employee the election covers and the same daily rate amount and availability period, with first_business_day_fee (AUD, as above, for the stations within 1 km of the spaces' premises; more than the car parking threshold), daily_rate_amount (AUD, a benefit's value for the space by the commercial parking station, market value or average cost method, as if there were no recipients contribution; FBTAA s 39FC), availability_days (integer from 1, the days from the first to the last day in the FBT year with a car parking benefit for the space; s 39FD) and spaces (integer from 1)",
            ),
            (
                "employees_at_start",
                "integer from 0, spaces only and required there: the employees the election covers at the start of the parking period, the first day in the FBT year a covered space gave rise to a car parking fringe benefit for one of them (FBTAA s 39FB(3) and (5))",
            ),
            (
                "employees_at_end",
                "integer from 0, spaces only and required there: the employees the election covers at the end of the parking period, the last such day (FBTAA s 39FB(3) and (5))",
            ),
            (
                "eligible_spaces_at_start",
                "integer from 0, spaces only and required there: the spaces with an availability period at the start of the parking period (FBTAA s 39FB(1) and (4))",
            ),
            (
                "eligible_spaces_at_end",
                "integer from 0, spaces only and required there: the spaces with an availability period at the end of the parking period (FBTAA s 39FB(1) and (4))",
            ),
            (
                "register_value",
                "AUD, 12_week_register only and required there: the total value of car parking benefits (register) under FBTAA s 39GC, the register period's benefits valued by the commercial parking station, market value or average cost method, net of recipients' payments",
            ),
            (
                "availability_days",
                "integer from 0 to the days in the FBT year, 12_week_register only and required there: the days in the car parking availability period (FBTAA s 39GD), from the first to the last day in the FBT year with a car parking benefit for an employee the election covers",
            ),
            (
                "register_first_year",
                "integer, 12_week_register only and required there: the FBT year ended 31 March the register was first valid for, from 4 years before year_ended to year_ended (FBTAA s 39GF); whether it has since ceased to be valid is the operator's to establish",
            ),
        ],
        "fbt_expense_payment" => vec![
            (
                "income_use",
                "decimal fraction from 0 to 1, external only and required there: the share of recipients_expenditure that would have been a once-only income tax deduction to the employee had they incurred and paid it unreimbursed (FBTAA s 24(1)(b)(ii), the gross deduction); 0 for an associate or private expenditure",
            ),
            (
                "recipients_portion_deduction",
                "AUD, external only and required there: the once-only deduction allowable to the employee for the expenditure they bore themselves (FBTAA s 24(1)(ba), RD), at most the recipient's portion, recipients_expenditure less amount_paid net of employee_contribution; 0.00 when they bore none",
            ),
            (
                "lowest_price",
                "AUD, in_house_trade_price, in_house_public_price and in_house_residual_public_price only and required there: for the property methods, as for fbt-property (FBTAA s 42(1)(a)); for in_house_residual_public_price, the lowest price at which the provider provided an identical benefit, or for a period benefit an identical overall benefit, to a member of the public in the ordinary course of business under an arm's length transaction, at or about the comparison time, in similar circumstances and on identical terms other than price (s 48(a), s 49(a) and s 22A(3)(a))",
            ),
            (
                "notional_value",
                "AUD, in_house_salary_packaging, in_house_produced_other, in_house_acquired, in_house_other, in_house_residual_salary_packaging and in_house_residual_other only and required there: the amount the recipient could reasonably be expected to have paid to obtain the property, or the residual benefit (for a period benefit, the recipient's overall benefit; s 22A(3)(b)), from its provider under an arm's length transaction, at the provision or comparison time (FBTAA s 136(1))",
            ),
            ("arms_length_price", FBT_ARMS_LENGTH_PRICE),
            (
                "in_house_reduction_available",
                "AUD from 0 to 1000.00, every in-house method but in_house_salary_packaging and in_house_residual_salary_packaging and required there: the part of the employee's $1,000 in-house reduction for the FBT year (FBTAA s 62) their other in-house fringe benefits have not used; 1000.00 for their only in-house fringe benefit",
            ),
        ],
        "fbt_property" => vec![
            ("lowest_price", FBT_LOWEST_PRICE),
            (
                "notional_value",
                "AUD, in_house_salary_packaging, in_house_produced_other, in_house_acquired, in_house_other and external_notional_value only and required there: the amount the recipient could reasonably be expected to have paid to obtain the property from the provider under an arm's length transaction, at the provision time (FBTAA s 136(1))",
            ),
            ("arms_length_price", FBT_ARMS_LENGTH_PRICE),
            (
                "cost_price",
                "AUD, external_cost_price only and required there: the cost price to the employer or its associate, as the provider, of property it bought under an arm's length transaction at or about the provision time (FBTAA s 43(a))",
            ),
            (
                "employer_expenditure",
                "AUD, external_expenditure only and required there: the expenditure the employer or an associate incurred under an arm's length transaction to a provider that is neither, in respect of providing the property (FBTAA s 43(b))",
            ),
            ("in_house_reduction_available", FBT_IN_HOUSE_REDUCTION),
        ],
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
        "unused_leave_payment" => vec![
            (
                "foreign_resident",
                "boolean, scale 4 only and required there: true for a payee who is a foreign resident (45% withheld), false for a resident (47%)",
            ),
            (
                "annual_leave_before_18_august_1993",
                "AUD, the part of annual_leave in respect of employment before 18 August 1993 (s 83-15(b)); required, 0 if none, for a normal termination when leave_accrual_start is before that day and annual_leave is not nil; refused above nil when it is not before that day",
            ),
            (
                "leave_loading_before_18_august_1993",
                "AUD, the part of leave_loading in respect of employment before 18 August 1993; required and refused as annual_leave_before_18_august_1993 is",
            ),
            (
                "long_service_leave_before_16_august_1978",
                "AUD, the part of long_service_leave attributable to the pre-16/8/78 period (ss 83-90(1) and 83-95 to 83-115); required, 0 if none, when leave_accrual_start is before 16 August 1978 and long_service_leave is not nil, whatever the reason; refused above nil otherwise",
            ),
            (
                "long_service_leave_16_august_1978_to_17_august_1993",
                "AUD, the part of long_service_leave attributable to the pre-18/8/93 period, 16 August 1978 to 17 August 1993 (ss 83-90(2) and 83-95 to 83-115); required, 0 if none, for a normal termination when leave_accrual_start is before 18 August 1993 and long_service_leave is not nil; refused above nil when it is not before that day",
            ),
        ],
        "schedule_5_method_a" | "schedule_5_method_b_i" | "schedule_5_method_b_ii" => vec![(
            "pay_date",
            "YYYY-MM-DD date this pay is made, within the income year, with study_loan true only; required with it for 2025-26, whose Schedule 8 coefficients apply from 24 September 2025; Method B(i) refuses a 2025-26 study loan",
        )],
        "fbt_residual" => vec![
            (
                "notional_value",
                "AUD, in_house_salary_packaging, in_house_other and external_notional_value only and required there: the amount the recipient could reasonably be expected to have paid the provider for the benefit under an arm's length transaction (FBTAA s 136(1)), for the part of a period benefit provided in the FBT year",
            ),
            (
                "lowest_price",
                "AUD, in_house_public_price only and required there: the lowest price at which the provider sold an identical benefit to the public in the ordinary course of business, at or about the comparison time and on identical terms other than price (FBTAA s 48(a) and s 49(a)), for the part of a period benefit provided in the FBT year",
            ),
            (
                "cost_price",
                "AUD, external_cost_price only and required there: the amount the provider, being the employer or its associate, paid or is to pay for the benefit under an arm's length transaction (FBTAA s 50(a) and s 51(a))",
            ),
            (
                "employer_expenditure",
                "AUD, external_expenditure only and required there: the expenditure the employer or its associate incurred under an arm's length transaction to a provider that is neither (FBTAA s 50(b) and s 51(b))",
            ),
            (
                "in_house_reduction_available",
                "AUD from 0 to 1000.00, in_house_public_price and in_house_other only and required there: the part of the employee's $1,000 in-house reduction for the FBT year (FBTAA s 62) their other in-house fringe benefits have not used; 1000.00 when this is their only one",
            ),
        ],
        "fbt_housing" => vec![
            (
                "market_value",
                "AUD, outside_australia, accommodation_business and base_year only and required there: the market value of the recipient's current housing right, the right to occupy the unit for the tenancy period in the FBT year, disregarding any right to have the occupant's expenses paid and onerous conditions of employment (FBTAA s 27)",
            ),
            (
                "in_house",
                "boolean, accommodation_business only and required there: true when the benefit, were it not a housing fringe benefit, would be an in-house residual fringe benefit, so 75% of the market value applies (FBTAA s 26(1)(b)(iv))",
            ),
            (
                "tenancy_days",
                "integer, base_year and indexed only and required there: the whole days in the FBT year the housing right subsisted (FBTAA s 136(1), tenancy period), from 1 under base_year and from 0 under indexed, up to the days in the year",
            ),
            (
                "prior_statutory_annual_value",
                "AUD, indexed only and required there: the statutory annual value for the year before for this housing right or an equivalent one, or the weighted average of those values where that year was a base year for 2 or more (FBTAA s 26(2)(b))",
            ),
            (
                "state",
                "string, indexed only and required there: NSW, VIC, QLD, SA, WA, TAS, NT or ACT, where the unit is; the Jervis Bay Territory is part of NSW, and Christmas Island and the Cocos (Keeling) Islands part of NT (FBTAA s 28(5))",
            ),
            (
                "base_year_ended",
                "integer, indexed only and required there: the last FBT year ended 31 March that was a base year for this housing right or an equivalent one, from 9 years before year_ended to the year before (FBTAA s 26(3))",
            ),
        ],
        "fbt_lafha" => vec![
            (
                "accommodation_component",
                "AUD, maintains_home and fly_in_fly_out only and required there: the part of the allowance reasonably concluded to compensate for the eligible family members' accommodation expenses in the period (FBTAA s 136(1))",
            ),
            (
                "accommodation_expenses",
                "AUD, maintains_home and fly_in_fly_out only and required there: the accommodation expenses the employee incurred for eligible family members in the period, substantiated under FBTAA s 31G",
            ),
            (
                "food_component",
                "AUD, maintains_home and fly_in_fly_out only and required there: the part of the allowance reasonably concluded to compensate for the eligible family members' food or drink expenses in the period (FBTAA s 136(1))",
            ),
            (
                "food_expenses",
                "AUD, maintains_home and fly_in_fly_out only and required there: the food or drink expenses the employee incurred for eligible family members in the period, substantiated under FBTAA s 31G where they exceed the Commissioner's reasonable amount (where they do not, the ATO accepts the lesser of the food component and that amount: the employer's guide, 11.6); 0.00 for days the employee resumed living at the normal residence (s 31(3))",
            ),
            (
                "normal_food_amount",
                "AUD, maintains_home and fly_in_fly_out only and required there: the amount reasonably expected to be the eligible family members' normal food or drink expenses at home in the period that was taken into account in working out the food component (FBTAA s 31H(2)); 0.00 when the food component is for total food costs",
            ),
            (
                "adult_days",
                "integer from 0 to 1,000,000, maintains_home and fly_in_fly_out only and required there: the days in the period of each eligible family member who was 12 or older before the FBT year began, added together; each has a statutory food amount of $42 a week (FBTAA s 136(1))",
            ),
            (
                "child_days",
                "integer from 0 to 1,000,000, maintains_home and fly_in_fly_out only and required there: the days in the period of each other eligible family member, added together; each has a statutory food amount of $21 a week (FBTAA s 136(1))",
            ),
        ],
        "fbt_tax_exempt_body_entertainment" => vec![
            (
                "employee_heads",
                "integer, per_head only and required there: the employees and their associates entertained, from 0 to total_heads",
            ),
            (
                "total_heads",
                "integer from 1, per_head only and required there: everyone entertained at the function, employees, their associates and others",
            ),
        ],
        "fbt_minor_benefit_check" => vec![
            (
                "in_house",
                "boolean, for an expense_payment, property or residual benefit and then required: true when, were it a fringe benefit of that kind, it would be an in-house fringe benefit (FBTAA s 58P(1)(c))",
            ),
            (
                "tax_exempt_body_exception",
                "for tax_exempt_body_entertainment and then required: incidental_to_outsiders_no_meal (incidental to entertaining outsiders, and neither a meal nor provided in connection with a meal, other than light refreshments: s 58P(1)(d)(i)), special_achievement_on_premises (on eligible premises of the employer, s 136(1): its premises, a related company's for a company, or a location at or next to a site where the employee works, solely to recognise the employee's special achievements in the employment: (d)(ii)) or none",
            ),
            (
                "fifty_fifty_split",
                "boolean, for meal_entertainment or entertainment_facility_leasing and then required: true when the employer elected the 50/50 split method, of Division 9A for meal entertainment or s 152B for entertainment facility leasing (TR 2007/12 para 14); false is refused, as the benefit is then screened under its own category",
            ),
        ],
        "fbt_entertainment_classifier" => vec![
            (
                "method",
                "actual, 50_50_split or 12_week_register: how the employer values meal entertainment for the FBT year (FBTAA Division 9A); required when a line is meal entertainment",
            ),
            (
                "income_tax_register_percentage",
                "decimal string from 0 to 100, 12_week_register only and required there when a line is meal entertainment: the register percentage of ITAA 1936 s 51AEB(2), the deductions that would be allowable for the register period's meal entertainment over its expenses, x 100",
            ),
        ],
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
        "fuel_tax_credits" => vec![(
            "lines",
            "list of 1 to 1,000 objects, each acquired (YYYY-MM-DD, the day the fuel was acquired, inside the income year and from 1 July 2022 to 31 January 2027) and either a line by use, with fuel (diesel, petrol, b5, b20, e10, e85, lpg, lng_cng or b100), quantity (decimal string at most 4 decimal places: litres, or kilograms for lng_cng) and use (heavy_vehicle_public_road: a vehicle above 4.5 tonnes GVM travelling on a public road, including fuel to power a bus's or coach's passenger air-conditioning, QC107638 note 1; other_business: every other business use, including other auxiliary equipment, off public roads and public road travel incidental to the vehicle's main use; light_vehicle_public_road: 4.5 tonnes GVM or less on a public road, which gets nil), or a line by the basic method for heavy vehicles (PCG 2021/2), with method basic_heavy_vehicle, fuel diesel, litres (the vehicle's eligible diesel), exactly one of public_road_km (actual) and total_km (98.4% taken as on public roads), and exactly one of rigid_truck_gross_weight_tonnes (above 4.5, at most 3 decimal places) and litres_per_km (the engine diagnostic report's driving fuel economy, above 0, at most 6 decimal places), or a line by use whose litres PCG 2016/2 Method 1 works out, with method litres_from_cost, fuel (a liquid fuel: diesel, petrol, b5, b20, e10, e85 or b100), use, cost (AUD, the GST inclusive spend on the fuel) and average_price_cents_per_litre (above 0, at most 4 decimal places: the average GST inclusive retail price of that fuel in the area of operation over the tax period, read from the Australian Institute of Petroleum's weekly pump prices, https://aip.com.au/pricing/pump-prices-retail/, as PCG 2016/2 para 9 says; this command holds no prices)",
        )],
        "resident_tax" | "foreign_resident_tax" => vec![("taxable_income", "AUD, whole dollars")],
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
        "borrowing_expenses_schedule" => vec![(
            "loans",
            "list of objects, one for each loan, each stating: borrowing_expenses (AUD, as for borrowing_expenses, incurred in the income year the loan began); loan_start (YYYY-MM-DD, the first day the money was borrowed, from 1 July 2017 to the end of the year asked about); loan_term_end and repaid_on (YYYY-MM-DD or null, as for borrowing_expenses); income_use (a decimal fraction from 0 to 1 for every income year of the loan, or an object giving one for each income year from the year the loan began to the year its period ends, such as {\"2025-26\": \"1\", \"2026-27\": \"0.5\"}); section_26_50_applies (boolean, as for borrowing_expenses; true is refused); and optionally refinances (the number, from 1, of an earlier loan in the list that this loan repaid on its loan_start, which must be that loan's repaid_on)",
        )],
        "chattel_mortgage_schedule" => vec![
            (
                "amount_financed",
                "AUD, the amount lent and paid to the supplier for the goods: the price less any deposit or trade-in",
            ),
            (
                "instalments",
                "integer, the number of instalments: 1 to 360 monthly, 1,565 weekly, 782 fortnightly, 120 quarterly or 30 annual",
            ),
            (
                "balloon",
                "AUD, paid at the end of the term; 0.00 for none",
            ),
            (
                "timing",
                "in_arrears (each instalment at its period's end) or in_advance (at its start)",
            ),
            (
                "start_date",
                "YYYY-MM-DD, the day of the purchase and the loan, within the income year",
            ),
        ],
        "loan_transactions" => vec![
            (
                "opening_balance",
                "AUD, the balance owing at the start of start_date; 0.00 for a new loan",
            ),
            (
                "annual_rate",
                "decimal fraction from 0 to 1, the annual interest rate from start_date",
            ),
            (
                "start_date",
                "YYYY-MM-DD, the statement's first day, within the income year",
            ),
            (
                "end_date",
                "YYYY-MM-DD, the statement's last day, on or after start_date and less than 30 years after it",
            ),
            (
                "transactions",
                "list of at most 5,000 objects in date order, each with date (YYYY-MM-DD, from start_date to end_date), kind (drawing or repayment) and amount (AUD); [] for none",
            ),
        ],
        "hire_purchase_schedule" => vec![
            (
                "amount_financed",
                "AUD, the notional loan principal: the price of the goods less any deposit or trade-in",
            ),
            (
                "balloon",
                "AUD, paid at the end of the term (with the last instalment in arrears, at the end of its period in advance) and shown as the last period's balloon; 0.00 for none",
            ),
            (
                "timing",
                "in_arrears (each instalment at its period's end) or in_advance (at its start)",
            ),
            (
                "start_date",
                "YYYY-MM-DD, the day the agreement starts, within the income year",
            ),
        ],
        "business_capital_expenditure" => vec![(
            "items",
            "list of objects, one for each item of capital expenditure, each stating: amount (AUD); incurred_in (the income year incurred, from 2020-21 to the year asked about); capital (boolean, true: the operator establishes that the outlay is capital; false is refused); business (current, former, proposed or wind_up: s 40-880(2)(a) to (d)); taxable_purpose (decimal fraction from 0 to 1, the extent s 40-880(3) or (4)(a) allows, as the business is, was or is proposed to be carried on for a taxable purpose); exclusions (list of the paragraphs that exclude it, from \"5(a)\" to \"5(j)\", \"8\", \"9(a)\" and \"9(b)\", [] for none; 5(a) and 5(f) are refused). A former or proposed business also states carried_on_by (you or another_entity); another entity's, and wind_up, assessable_income_connection (boolean, s 40-880(4)(b)); a proposed business within_reasonable_time (boolean, s 40-880(7)) and start_up_expense (advice_or_services, government_fees or neither: s 40-880(2A)(b)), with entity_test for the income year incurred unless neither (small_business_entity, covered_by_2b, not_in_business_and_unconnected or none: s 40-880(2A)(c) and (2B)); and 5(d) or 5(f) goodwill_right (boolean, s 40-880(6))",
        )],
        "balancing_adjustment" => vec![
            (
                "cost",
                "AUD, the first and second elements of cost after Subdivision 27-B and, for a car, the car limit (s 40-230)",
            ),
            (
                "termination_value",
                "AUD, under ss 40-300 to 40-320 after Subdivision 27-B, before s 40-325",
            ),
            ("asset", "car (ITAA 1997 s 995-1), motor_cycle_or_similar or other"),
            (
                "years",
                "list of objects, one for each income year held from the start time to the event, each with income_year (from 2001-02, and for a car from 2015-16, to the event's year), decline_in_value (AUD; for a cents per kilometre year the notional decline of s 40-370(3)) and deduction (AUD, the decline less the s 40-25 reductions); a car's rows also state method (log_book or cents_per_kilometre, which takes no deduction) and days_held (integer from 1 to the days in that income year, 366 when it includes 29 February)",
            ),
        ],
        "fbt_ev_exemption_check" => vec![
            (
                "vehicle",
                "battery_electric (only an electric motor for propulsion, no fuel cell or internal combustion engine: FBTAA s 8A(3)), hydrogen_fuel_cell_electric (an electric motor and a fuel cell converting hydrogen, no internal combustion engine: s 8A(4)), plug_in_hybrid_electric (an electric motor, energy taken and stored from an external source and an internal combustion engine: s 8A(5) as made) or other",
            ),
            (
                "car",
                "boolean, true when the vehicle is a car: a motor vehicle, except a motor cycle or similar vehicle, designed to carry a load of less than 1 tonne and fewer than 9 passengers (FBTAA s 136(1), ITAA 1997 s 995-1)",
            ),
            (
                "current_employee",
                "boolean, true when the benefit is provided in respect of the employment of a current employee (FBTAA s 8A(1)(a))",
            ),
            (
                "first_held_and_used",
                "YYYY-MM-DD date, the earliest day on which any person both held the car (owned it, including under hire purchase, leased it or had it made available) and used it or had it available for use (Electric Car Discount Act 2022 Sch 1 item 7(2)); not after the FBT year ends",
            ),
            (
                "first_retail_sale",
                "YYYY-MM-DD date of the car's first retail sale, whose financial year sets the fuel-efficient luxury car tax threshold (2012-13 to 2026-27); a sale after the FBT year comes after every benefit in it, so no threshold applies to it (FBTAA s 8A(1)(c))",
            ),
            (
                "luxury_car_tax_value",
                "AUD, the car's luxury car tax value at its first retail sale: its price including GST and customs duty, excluding luxury car tax and other Australian taxes, fees and charges such as registration and stamp duty, and including accessories supplied with it (LCT Act s 5-20)",
            ),
            (
                "luxury_car_tax_on_other_supply",
                "boolean, true when luxury car tax became payable on any other supply or importation of the car before the benefit (FBTAA s 8A(1)(c))",
            ),
        ],
        "payroll_tax" => vec![
            ("jurisdiction", "nsw, vic, qld, wa, sa, tas, act or nt"),
            (
                "taxable_wages",
                "AUD, the employer's taxable wages in the jurisdiction for the financial year, after that jurisdiction's exemptions",
            ),
            (
                "australian_wages",
                "AUD, the employer's taxable wages and interstate wages for the financial year: all its wages taxable in any Australian jurisdiction, at least taxable_wages",
            ),
            (
                "group_member",
                "boolean, true when the employer is a member of a group for payroll tax; true is refused",
            ),
            (
                "employed_all_year",
                "boolean, true when the employer paid or was liable to pay wages on every day of the financial year; false is refused",
            ),
        ],
        "gst_margin_scheme" => vec![
            (
                "interest",
                "freehold_interest, stratum_unit, long_term_lease (at least 50 years, GST Act s 195-1) or other: what the supply sells or grants (s 75-5(1))",
            ),
            (
                "agreement",
                "written_on_or_before_supply, written_in_further_period_allowed (within a further period the Commissioner allowed) or none: the written agreement between you and the recipient that the margin scheme is to apply (s 75-5(1) and (1A))",
            ),
            (
                "acquisition",
                "how you acquired the entire interest, unit or lease: taxable_supply_under_margin_scheme, taxable_supply_without_margin_scheme (ineligible: s 75-5(3)(a)), other_supply_from_1_july_2000 (a supply on or after 1 July 2000 that was not a taxable supply, such as a purchase from a seller not registered for GST) or before_1_july_2000 (acquired, or contracted to acquire, before 1 July 2000: GSTR 2006/8 para 39A, and not through a taxable supply on which the GST was worked out without the margin scheme, such as a pre-1 July 2000 contract settled later as a fully taxable supply, which is taxable_supply_without_margin_scheme); refused with the provisions that apply: gst_group_member, joint_venture_operator, inheritance, gst_free_going_concern, gst_free_farm_land, associate, gst_free_government_land and part_through_ineligible_supply",
            ),
            (
                "consideration",
                "AUD, the consideration for the supply, the GST-inclusive price (s 75-10(2))",
            ),
            (
                "recipient_associate",
                "boolean, true when the recipient is your associate at the time of the supply; refused, as s 75-13 then works the margin from the GST inclusive market value",
            ),
            (
                "method",
                "consideration (the consideration for your acquisition: s 75-10(2)) or approved_valuation (for an interest acquired before 1 July 2000 only: s 75-10(3)). For an interest acquired before 1 July 2000, consideration states that no approved valuation as at the day s 75-10(3) fixes has been made: once one has, s 75-10(3) works the margin from it",
            ),
        ],
        "wine_equalisation_tax" => vec![(
            "dealings",
            "list of 1 to 1,000 objects, the tax period's dealings in wine, each with kind and its inputs: wholesale_sale (a sale to an entity that buys to resell: AD1a, AD1b or AD11b) with price_excluding_wet_and_gst (AUD, net of trade discounts, volume rebates and settlement discounts); retail_sale (a retail sale taxed at the notional wholesale selling price, such as a manufacturer's cellar door or mail order sale) with retail_price_including_wet_and_gst (AUD) and, to choose the average wholesale price method for grape wine, average_wholesale; own_use (an application to own use connected with retail sales: consuming or giving the wine away in connection with retail sales, such as tastings) with either retail_price_including_wet_and_gst (the price for which it would normally be sold by retail) or average_wholesale; quoted_sale (a sale for which the purchaser quoted its ABN, not taxable: s 7-10) with price_excluding_wet_and_gst and purchaser_intention (taxable_dealing where the quote states none of the intentions in s 5-50(1)(b), or manufacture, gst_free_supply or sale_under_quote); or own_use_purchased_under_quote (an application to own use of wine purchased under quote: AD3c(a) or AD13c(a)) with purchase_price_excluding_gst. average_wholesale is an object with grape_wine (true: grape wine, s 31-2), wholesale_share_at_least_10_percent (true: at least 10% by value of the tax period's sales of grape wine of that vintage and grape varieties are wholesale sales, s 9-25(3)), quantity (this dealing's quantity, a decimal string above 0 with at most 4 decimal places, in the wholesale sales' unit) and wholesale_sales (1 to 1,000 objects, the tax period's wholesale sales of that vintage and those grape varieties, exports included, each with quantity and price_excluding_wet_and_gst). Any kind but own_use_purchased_under_quote may add rebate_conditions, an object of the booleans producer (s 19-5(1)(a)), source_product_owned_85_percent (s 19-5(1)(d)) and container_and_trade_mark (s 19-5(1)(e) and (7)), to claim the producer rebate on that dealing",
        )],
        "luxury_car_tax" => vec![(
            "lines",
            "list of 1 to 1,000 objects, the tax period's luxury car tax lines, each with kind and its inputs: sale (a taxable supply of a luxury car, s 5-10) or importation (a taxable importation, s 7-5) with luxury_car_tax_value (AUD, the value including GST and customs duty but not luxury car tax: s 5-20 for a sale, s 7-15(2) for an importation), fuel_consumption_litres_per_100km (the combined rating under the national road vehicle standards, a decimal string from 0 to 100 with at most 2 decimal places, or null for none), used_before_1_july_2025 (boolean, required only for a car rated above 3.5 and at most 7 litres supplied or imported from 1 July 2025: true where, before that day, an entity supplied or imported the car and it was used in Australia other than for a quotable purpose) and, for a sale, lct_previously_payable (AUD, optional: luxury car tax payable on an earlier supply or importation of the car, s 5-15(2)); price_adjustment (an adjustment event on an earlier sale, ss 15-20 and 15-25) with supplied_in (the financial year of the sale, 2012-13 to the tax period's), corrected_luxury_car_tax_value, the car's rating facts, lct_previously_payable (optional) and previously_attributed_lct (AUD, the luxury car tax previously attributed to the supply); change_of_use_increasing (a car supplied under quote, or one on whose supply a s 15-30(1) decreasing adjustment arose, then used other than for a quotable purpose, s 15-30(3) and (4)) with supplied_in, luxury_car_tax_value, the rating facts and lct_previously_payable (optional); change_of_use_decreasing (s 15-30(1)) with lct_payable_on_supply (AUD); and refund (s 18-5 or 18-10) with claimant (primary_producer or tourism_operator), refund_eligible_car (boolean), lct_borne (AUD) and supplied_in (optional, from 2019-20)",
        )],
        "trade_in" => vec![
            (
                "cost",
                "AUD, the traded-in asset's cost, as for balancing_adjustment",
            ),
            (
                "asset",
                "the traded-in asset: car (ITAA 1997 s 995-1), motor_cycle_or_similar or other",
            ),
            (
                "years",
                "the traded-in asset's years held, as for balancing_adjustment",
            ),
            (
                "trade_in_allowance",
                "AUD, including any GST: the amount by which the replacement's price is reduced for the traded-in asset (ITAA 1997 s 40-305(1)(b) item 2)",
            ),
            (
                "trade_in_taxable",
                "boolean, true when trading in the asset is a taxable supply (GST Act s 9-5)",
            ),
            (
                "replacement_price",
                "AUD, the replacement's price before the trade-in, including GST and any luxury car tax",
            ),
            (
                "replacement_asset",
                "passenger_car (a car designed mainly for carrying passengers, to which ITAA 1997 s 40-230 and GST Act s 69-10 apply) or other",
            ),
            (
                "replacement_creditable_acquisition",
                "boolean, true when acquiring the replacement is a creditable acquisition (GST Act s 11-5)",
            ),
            (
                "arms_length",
                "boolean, true when the parties deal at arm's length; false is refused",
            ),
            (
                "involuntary_disposal",
                "boolean, true when the traded-in asset was lost, destroyed or compulsorily acquired (ITAA 1997 s 40-365(2)); true is refused",
            ),
        ],
        "fbt" => vec![("type_one_value", "AUD"), ("type_two_value", "AUD")],
        "fbt_loan" => vec![
            (
                "daily_balance_total",
                "AUD, the loan's balance at the end of each day in the FBT year on which the employee owed it, added together (a balance of $20,000 for 182 days is 3640000.00)",
            ),
            (
                "interest_accrued",
                "AUD, the interest that accrued on the loan in respect of the FBT year (FBTAA s 18(1)); 0.00 for an interest-free loan",
            ),
            (
                "income_use",
                "decimal fraction from 0 to 1, the share of the notional interest that would have been a once-only income tax deduction to the employee had they paid it (FBTAA s 19(1)(b)); 0 for a loan used only privately",
            ),
            (
                "accrued_interest_deduction",
                "AUD, the once-only deduction that would have been allowable to the employee for the interest accrued (FBTAA s 19(1)(ba), RD), at most interest_accrued; 0.00 when no interest accrued",
            ),
        ],
        "fbt_meal_entertainment" => vec![
            ("method", "50_50_split or 12_week_register"),
            (
                "meal_entertainment_expenditure",
                "AUD, the employer's total expenses for providing meal entertainment in the FBT year to all persons, excluding amounts employees paid (FBTAA ss 37AB and 37CB(3))",
            ),
        ],
        "fbt_meal_entertainment_register" => vec![
            (
                "register_start",
                "YYYY-MM-DD, the first day of the continuous period the 12 week register was kept for (FBTAA s 37CC(1))",
            ),
            (
                "register_end",
                "YYYY-MM-DD, its last day; at least 83 days after register_start",
            ),
            (
                "entries",
                "list of at most 10000 objects, one for each provision of meal entertainment the register records in its period, each with date (YYYY-MM-DD, within the period), recipients (an object of whole-number counts of employees, associates of employees and others), cost (AUD), kind (text, such as dinner or a corporate box), location (text), on_premises (boolean: on the employer's premises), in_house_dining_facility (boolean, for on_premises only: in an in-house dining facility under ITAA 1997 s 32-55) and fringe_benefits_value (AUD, required with cost: the part of the cost that is meal entertainment fringe benefits to employees and their associates, at most the cost and 0.00 when none received any). A detail the register does not record is left out or null, and the check reports it (FBTAA s 37CE(1))",
            ),
        ],
        "fbt_car_parking" => vec![
            (
                "method",
                "commercial_parking_station, market_value, average_cost, spaces or 12_week_register",
            ),
            (
                "first_business_day_fee",
                "AUD, every method but spaces, which takes it for each group of spaces: the highest of the lowest all-day fees the commercial parking stations within 1 km each charged on the first business day of the FBT year: car parking benefits arise only when one station's lowest fee is more than the car parking threshold (FBTAA s 39A(1)(a)(iii); the employer's guide, example 12), and benefits are valued at their own fee, which can be lower",
            ),
        ],
        "fbt_debt_waiver" => vec![(
            "amount_waived",
            "AUD, the amounts whose payment or repayment the employer or an associate waived in the FBT year for the employee or an associate, principal and interest together (FBTAA ss 14 and 15); not a genuine bad debt written off for reasons unrelated to the employment",
        )],
        "fbt_expense_payment" => vec![
            (
                "method",
                "external, in_house_salary_packaging, in_house_trade_price, in_house_public_price, in_house_produced_other, in_house_acquired, in_house_other, in_house_residual_salary_packaging, in_house_residual_public_price or in_house_residual_other",
            ),
            (
                "recipients_expenditure",
                "AUD, the expenditure the recipient incurred that the payment or reimbursement relates to (FBTAA s 136(1), recipients expenditure)",
            ),
            (
                "amount_paid",
                "AUD, the employer's or an associate's payment to a third party discharging the recipient's obligation (FBTAA s 20(a)), or its reimbursement of the recipient (s 20(b)), at most recipients_expenditure",
            ),
            (
                "employee_contribution",
                "AUD, the recipient's contribution: what the recipient or employee paid the provider or employer for the benefit (FBTAA s 136(1)), which arises only for a payment under s 20(a); 0.00 for a reimbursement",
            ),
        ],
        "fbt_property" => vec![
            (
                "method",
                "in_house_salary_packaging, in_house_trade_price, in_house_public_price, in_house_produced_other, in_house_acquired, in_house_other, external_cost_price, external_expenditure or external_notional_value",
            ),
            (
                "employee_contribution",
                "AUD, the recipient's contribution: the consideration the recipient or employee paid the provider or employer for the property, less any reimbursement of it (FBTAA s 136(1))",
            ),
            (
                "income_use",
                "decimal fraction from 0 to 1, the share of the value before the contribution that would have been a once-only income tax deduction to the employee had they bought the property for that amount (FBTAA s 44(1)(b), the gross deduction); 0 for an associate, private use or property whose cost would be deducted over more than one year",
            ),
            (
                "contribution_deduction",
                "AUD, the once-only deduction that would have been allowable to the employee for their contribution had they paid it at the provision time (FBTAA s 44(1)(ba), RD), at most employee_contribution; 0.00 when there is no contribution",
            ),
        ],
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
        "payg_gross_up" => {
            let mut units = input_units("payg_withholding");
            units[0] = (
                "net_pay",
                "AUD, the pay for the pay period after Schedule 1 withholding",
            );
            units
        }
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
        "etp_tax_free_component" => vec![
            (
                "payment",
                "AUD, the whole life benefit employment termination payment (ITAA 1997 s 82-130), every part of it made because of the termination: such as a golden handshake, compensation, unused sick leave and rostered days off, but not unused annual or long service leave (s 82-135)",
            ),
            (
                "employment_start",
                "YYYY-MM-DD, the first day of the employment to which the payment relates, counted",
            ),
            (
                "termination_date",
                "YYYY-MM-DD, the day the employment was terminated, counted in the employment days",
            ),
            (
                "last_retirement_day",
                "YYYY-MM-DD, the employee's last retirement day (s 995-1(1)): the day the employment would have ended on reaching a particular age or completing a particular period of service, or otherwise the day they would turn 65",
            ),
            (
                "ill_health",
                "boolean, true when the payment was made because the employee stopped being gainfully employed, and they stopped because they suffered from ill-health, physical or mental (s 82-150(1)(a) and (b))",
            ),
            (
                "medical_certification",
                "boolean, true when 2 legally qualified medical practitioners have certified that, because of the ill-health, the employee is unlikely ever to be gainfully employed in a capacity for which they are reasonably qualified because of education, experience or training (s 82-150(1)(d))",
            ),
        ],
        "unused_leave_payment" => vec![
            (
                "annual_leave",
                "AUD, the unused annual leave paid in consequence of the termination (s 83-10(3)(a) and (c)), excluding leave loading",
            ),
            (
                "leave_loading",
                "AUD, the leave loading, bonus or other additional payment for that unused annual leave (s 83-10(3)(b))",
            ),
            (
                "long_service_leave",
                "AUD, the unused long service leave paid in consequence of the termination (ss 83-70 and 83-75), all periods",
            ),
            (
                "leave_accrual_start",
                "YYYY-MM-DD, the first day of the earliest period over which the unused leave paid accrued: for long service leave, its long service leave employment period (s 83-90(4))",
            ),
            (
                "termination_reason",
                "normal (resignation, dismissal for inefficiency, retirement and every other reason), genuine_redundancy (s 83-175), early_retirement_scheme (s 83-180) or invalidity (an ETP or super benefit with an invalidity segment, s 82-150); death is refused",
            ),
            (
                "working_holiday_maker",
                "boolean, whether the employee is a working holiday maker; true is refused (Schedule 15 covers them)",
            ),
            (
                "earnings",
                "AUD, the employee's normal gross earnings: all payments except termination payments in the last full pay period, including taxable allowances, overtime and bonuses, or the year-to-date average pay where pay fluctuates significantly (Schedule 7, 'Normal gross earnings'); not used for scale 4",
            ),
            ("pay_period", "weekly, fortnightly or monthly"),
            (
                "scale",
                "integer 1, 2, 3, 5 or 6, the Schedule 1 scale used for the employee's normal earnings (3 for a foreign resident who gave a TFN), or 4 when no TFN was provided",
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
        "lump_sum_arrears_tax_offset" => vec![
            (
                "taxable_income",
                "AUD, whole dollars: the taxable income of the year of receipt, including the total arrears amount (ITAA 1936 s 159ZRB)",
            ),
            (
                "net_capital_gain",
                "AUD, whole dollars: the net capital gain included in that taxable income (ITAA 1997 s 102-5), which normal taxable income leaves out (ITAA 1936 s 159ZR(1)); 0 for none",
            ),
            (
                "accrual_years",
                "list of objects, one for each earlier income year in which part of the eligible lump sums accrued, each year once: year (such as \"2023-24\") and arrears (AUD, whole dollars, more than nil: the annual arrears amount, every payer's amount for that year added together); for a recent accrual year (the 2 latest when there are 3 or more, otherwise each one) also taxable_income (AUD, whole dollars, as assessed for that year) and, where they apply, rebate_year_arrears (the total arrears amount of that year, when it was itself a rebate year: ITAA 1936 s 159ZRC(b)), earlier_rebate_arrears (what accrued in that year of the total arrears amount of an earlier rebate year: s 159ZRC(c)) and, when there are distant accrual years, net_capital_gain (that year's, left out of its normal taxable income for s 159ZRD); a distant accrual year takes year and arrears only",
            ),
        ],
        "fbt_residual" => vec![
            (
                "method",
                "in_house_salary_packaging, in_house_public_price, in_house_other, external_cost_price, external_expenditure or external_notional_value",
            ),
            ("employee_contribution", "AUD, the recipients contribution (FBTAA s 136(1)): consideration the recipient or employee paid for the benefits, net of any reimbursement, each benefit's counted only up to its value: it cannot take the value below nil, and an excess on one benefit reduces no other"),
            (
                "income_use",
                "decimal fraction from 0 to 1, the share of the gross taxable value, before the contribution, that would have been a once-only income tax deduction to the employee had they paid it (FBTAA s 52(1)(b)); 0 for a benefit used only privately or provided to an associate",
            ),
            (
                "contribution_deduction",
                "AUD, at most employee_contribution: the deduction that would have been allowable to the employee for the contribution (FBTAA s 52(1)(ba), RD); 0.00 when there is none",
            ),
        ],
        "fbt_housing" => vec![
            (
                "method",
                "outside_australia, accommodation_business, base_year or indexed",
            ),
            (
                "employee_contribution",
                "AUD, the recipients rent (FBTAA s 136(1)): rent or other consideration the recipient or employee paid for the housing right in the FBT year, net of any reimbursement; it cannot take the value below nil",
            ),
        ],
        "fbt_lafha" => vec![
            ("method", "maintains_home, fly_in_fly_out or other"),
            (
                "allowance",
                "AUD, the living-away-from-home allowance fringe benefit for the period: the allowance, or the part of it, in the nature of compensation for the employee's additional non-deductible expenses, and other disadvantages, of living away from the normal residence (FBTAA s 30)",
            ),
        ],
        "fbt_board" => vec![
            (
                "adult_meals",
                "integer from 0 to 100,000,000: the board meals provided in the FBT year to recipients who were 12 or older before it began (FBTAA s 36(a))",
            ),
            (
                "child_meals",
                "integer from 0 to 100,000,000: the board meals provided in the FBT year to other recipients (FBTAA s 36(b))",
            ),
            ("employee_contribution", "AUD, the recipients contribution (FBTAA s 136(1)): consideration the recipient or employee paid for the benefits, net of any reimbursement, each benefit's counted only up to its value: it cannot take the value below nil, and an excess on one benefit reduces no other"),
            (
                "income_use",
                "decimal fraction from 0 to 1, the share of the gross taxable value, before the contribution, that would have been a once-only income tax deduction to the employee had they paid it (FBTAA s 37(b)); 0 for a benefit used only privately or provided to an associate",
            ),
            (
                "contribution_deduction",
                "AUD, at most employee_contribution: the deduction that would have been allowable to the employee for the contribution (FBTAA s 37(c), RD); 0.00 when there is none",
            ),
        ],
        "fbt_tax_exempt_body_entertainment" => vec![
            ("method", "actual or per_head"),
            (
                "entertainment_expenditure",
                "AUD, non-deductible exempt entertainment expenditure (FBTAA s 136(1)): under actual, the part incurred in entertaining employees and their associates; under per_head, the whole function's",
            ),
        ],
        "fbt_minor_benefit_check" => vec![
            (
                "category",
                "expense_payment, property, residual, tax_exempt_body_entertainment (where the provider incurs non-deductible exempt entertainment expenditure for it), meal_entertainment, entertainment_facility_leasing or other (such as a car, loan or board benefit)",
            ),
            (
                "notional_taxable_value",
                "AUD, the benefit's notional taxable value in the current FBT year, for one benefit to one employee or associate (FBTAA s 58P(1)(e); TR 2007/12 paras 17 and 18)",
            ),
            (
                "salary_sacrifice",
                "boolean, true when the benefit is provided under a salary sacrifice arrangement (TR 2007/12 para 16)",
            ),
            (
                "frequency",
                "infrequent_and_irregular, infrequent_and_regular (such as each Christmas) or frequent: how often identical or similar benefits, or benefits provided with it, have been or can reasonably be expected to be provided (s 58P(1)(f)(i))",
            ),
            (
                "similar_benefits_substantial",
                "boolean, true when the sum of the notional taxable values of the benefit and identical or similar associated benefits, in this and other years, is substantial (s 58P(1)(f)(ii))",
            ),
            (
                "other_associated_benefits_substantial",
                "boolean, true when the sum of the notional taxable values of other associated benefits, such as those provided with it, in this and other years, is substantial (s 58P(1)(f)(iii))",
            ),
            (
                "valuation_difficult",
                "boolean, true when working out the notional taxable values of the benefit and its associated benefits (other than car benefits) is practically difficult (s 58P(1)(f)(iv))",
            ),
            (
                "unexpected_event",
                "boolean, true when the benefit was provided to help the employee deal with an unexpected event (s 58P(1)(f)(v)(A))",
            ),
            (
                "reward_for_services",
                "boolean, true when the benefit was provided wholly or principally as a reward for services rendered or to be rendered (s 58P(1)(f)(v)(B))",
            ),
        ],
        "fbt_entertainment_classifier" => vec![
            (
                "lines",
                "list of 1 to 10,000 expense line objects, each with cost (AUD decimal string: the employer's expense, less any contribution from an employee or associate it does not reimburse, FBTAA s 37AB), gst_status (includes_gst or no_gst), provided (food_or_drink, recreation, accommodation_or_travel_for_food_or_drink or accommodation_or_travel_for_recreation), venue (business_premises_working_day, business_premises_other_day or elsewhere; business premises exclude a corporate box and premises used mainly for entertainment, FBTAA s 136(1)), paid_by (employer, including by corporate card, or reimbursed, where the employer reimbursed or paid for an attendee who incurred it) and attendees (an object of employees, associates and others, such as clients, each a head count from 0, or null where unknown; a line of more than one class needs every count); optionally occasion, food_or_drink and accommodation_or_travel_for_food_or_drink only, the food or drink's occasion (social_function, business_meal, with_other_entertainment, light_refreshments, or travel_meal where every attendee is travelling for work; without it the line is undetermined), and infrequent_and_irregular (boolean, whether identical or similar benefits are provided infrequently and irregularly, FBTAA s 58P(1)(f)(i))",
            ),
            (
                "income_tax_exempt",
                "boolean, whether the employer is exempt from income tax; true is refused, as fbt-tax-exempt-body-entertainment applies",
            ),
        ],
        "income_tests" => vec![
            ("taxable_income", "AUD, whole dollars: taxable income, including any assessable FHSS released amount (nil for a loss)"),
            ("assessable_fhss_released_amount", "AUD: the amount ITAA 1997 s 313-20 includes in assessable income for released FHSS amounts"),
            ("family_trust_distribution_amounts", "AUD: amounts on which family trust distribution tax was paid that ITAA 1936 Sch 2F s 271-105(1) kept out of assessable income"),
            ("reportable_fringe_benefits_section_57a", "AUD: reportable fringe benefits amounts from employers exempt from FBT under FBTAA s 57A (return item IT1, label N)"),
            ("reportable_fringe_benefits_other", "AUD: reportable fringe benefits amounts from every other employer (return item IT1, label W)"),
            ("reportable_employer_super_contributions", "AUD: reportable employer superannuation contributions (return item IT2)"),
            ("deductible_personal_super_contributions", "AUD: personal superannuation contributions deducted under ITAA 1997 Subdivision 290-C (return item D12)"),
            ("excess_concessional_contributions", "AUD: excess concessional contributions for the corresponding financial year (ITAA 1997 s 291-20)"),
            ("net_financial_investment_loss", "AUD: deductions attributable to financial investments less the gross income from them, nil if none (return item IT5)"),
            ("net_rental_property_loss", "AUD: deductions attributable to rental property less the gross income from it, nil if none (return item IT6)"),
            ("tax_free_government_pensions", "AUD: tax free pensions or benefits under A New Tax System (Family Assistance) Act 1999 Sch 3 cl 7 as in force for the year (return item IT3)"),
            ("target_foreign_income", "AUD: target foreign income under A New Tax System (Family Assistance) Act 1999 Sch 3 cl 5, including exempt foreign employment income (return item IT4)"),
            ("exempt_foreign_employment_income", "AUD: income exempt under ITAA 1936 s 23AF or 23AG less the losses and outgoings, other than capital ones, incurred in deriving it (HESA s 154-5(4) and (5))"),
            ("child_support_paid", "AUD: deductible child maintenance expenditure under A New Tax System (Family Assistance) Act 1999 Sch 3 cl 8 (return item IT7)"),
            ("assessable_income", "AUD: assessable income, including any assessable FHSS released amount and excess concessional contributions"),
            ("business_deductions", "AUD: deductions the person is entitled to as a result of carrying on a business"),
            ("residency", "full_year_resident; part_year_resident, working_holiday_maker and foreign_resident are refused"),
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
        // 10,000 litres of diesel in a heavy vehicle on public roads and
        // 2,000 litres off road, acquired from 3 August 2026: $2,130 + $1,074.
        "fuel_tax_credits" => vec![
            ("year", Text("2026-27")),
            (
                "lines",
                Json(
                    r#"[{"fuel": "diesel", "use": "heavy_vehicle_public_road", "acquired": "2026-09-15", "quantity": "10000"}, {"fuel": "diesel", "use": "other_business", "acquired": "2026-09-15", "quantity": "2000"}]"#,
                ),
            ),
        ],
        "resident_tax" | "foreign_resident_tax" => vec![("taxable_income", Text("45000"))],
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
            ("year", Text("2024-25")),
        ],
        // Peter's loan (ATO, Borrowing expenses) from 3 July 2024, refinanced
        // on 2 March 2026 by a second loan whose money is half used to produce
        // income from 2026-27.
        "borrowing_expenses_schedule" => vec![
            ("year", Text("2026-27")),
            (
                "loans",
                Json(
                    r#"[{"borrowing_expenses": "1600.00", "loan_start": "2024-07-03", "loan_term_end": "2049-07-02", "repaid_on": "2026-03-02", "income_use": "1", "section_26_50_applies": false}, {"borrowing_expenses": "900.00", "loan_start": "2026-03-02", "loan_term_end": null, "repaid_on": null, "income_use": {"2025-26": "1", "2026-27": "0.5", "2027-28": "0.5", "2028-29": "0.5", "2029-30": "0.5", "2030-31": "0.5"}, "section_26_50_applies": false, "refinances": 1}]"#,
                ),
            ),
        ],
        "business_capital_expenditure" => vec![
            ("year", Text("2026-27")),
            (
                "items",
                Json(
                    r#"[{"amount": "1500.00", "incurred_in": "2025-26", "capital": true, "business": "current", "taxable_purpose": "1", "exclusions": []}, {"amount": "576.00", "incurred_in": "2026-27", "capital": true, "business": "proposed", "carried_on_by": "another_entity", "assessable_income_connection": true, "within_reasonable_time": true, "start_up_expense": "government_fees", "entity_test": "not_in_business_and_unconnected", "taxable_purpose": "1", "exclusions": []}]"#,
                ),
            ),
        ],
        "balancing_adjustment" => vec![
            ("year", Text("2026-27")),
            ("cost", Text("1000.00")),
            ("termination_value", Text("600.00")),
            ("asset", Text("other")),
            (
                "years",
                Json(
                    r#"[{"income_year": "2025-26", "decline_in_value": "300.00", "deduction": "180.00"}, {"income_year": "2026-27", "decline_in_value": "0.00", "deduction": "0.00"}]"#,
                ),
            ),
        ],
        // The Library's Cleancut example (Tax Examples, FBT, 3-045): an
        // electric car bought for $72,000 plus GST on 1 July 2025.
        "fbt_ev_exemption_check" => vec![
            ("vehicle", Text("battery_electric")),
            ("car", Flag(true)),
            ("current_employee", Flag(true)),
            ("first_held_and_used", Text("2025-07-01")),
            ("first_retail_sale", Text("2025-07-01")),
            ("luxury_car_tax_value", Text("79200.00")),
            ("luxury_car_tax_on_other_supply", Flag(false)),
        ],
        "payroll_tax" => vec![
            ("year", Text("2026-27")),
            ("jurisdiction", Text("nsw")),
            ("taxable_wages", Text("900000.00")),
            ("australian_wages", Text("3000000.00")),
            ("group_member", Flag(false)),
            ("employed_all_year", Flag(true)),
        ],
        // The Library's Joanne Black example (Tax Examples, GST, 12-200):
        // land bought in January 2000, worth $448,000 on 1 July 2000, sold
        // with a house in January 2026 for $640,000.
        "gst_margin_scheme" => vec![
            ("interest", Text("freehold_interest")),
            ("agreement", Text("written_on_or_before_supply")),
            ("acquisition", Text("before_1_july_2000")),
            ("consideration", Text("640000.00")),
            ("method", Text("approved_valuation")),
            ("recipient_associate", Flag(false)),
            ("approved_valuation", Text("448000.00")),
            ("valuation_day", Text("2000-07-01")),
            ("registered_on_1_july_2000", Flag(true)),
            ("government_unimproved_on_1_july_2000", Flag(false)),
        ],
        // A dozen bottles sold by wholesale for $120 and a dozen at the
        // cellar door for $140: $34.80 + $20.30 of wine tax.
        "wine_equalisation_tax" => vec![
            ("year", Text("2026-27")),
            (
                "dealings",
                Json(
                    r#"[{"kind": "wholesale_sale", "price_excluding_wet_and_gst": "120.00"}, {"kind": "retail_sale", "retail_price_including_wet_and_gst": "140.00"}]"#,
                ),
            ),
        ],
        // The ATO's QC22107: a car sold for $88,000 including GST in 2026-27
        // bears $2,157.30 of luxury car tax.
        "luxury_car_tax" => vec![
            ("year", Text("2026-27")),
            (
                "lines",
                Json(
                    r#"[{"kind": "sale", "luxury_car_tax_value": "88000.00", "fuel_consumption_litres_per_100km": "8.5"}]"#,
                ),
            ),
        ],
        "trade_in" => vec![
            ("year", Text("2026-27")),
            ("cost", Text("40800.00")),
            ("asset", Text("car")),
            ("car_limit", Json("null")),
            (
                "years",
                Json(
                    r#"[{"income_year": "2025-26", "decline_in_value": "10200.00", "method": "log_book", "deduction": "8160.00", "days_held": 365}, {"income_year": "2026-27", "decline_in_value": "0.00", "method": "log_book", "deduction": "0.00", "days_held": 1}]"#,
                ),
            ),
            ("trade_in_allowance", Text("22000.00")),
            ("trade_in_taxable", Flag(true)),
            ("adjusted_input_tax_credit", Text("3200.00")),
            ("full_input_tax_credit", Text("4000.00")),
            ("replacement_price", Text("91242.80")),
            ("replacement_asset", Text("passenger_car")),
            ("replacement_luxury_car_tax", Text("2407.80")),
            ("replacement_creditable_acquisition", Flag(true)),
            ("creditable_purpose", Text("0.8")),
            ("arms_length", Flag(true)),
            ("involuntary_disposal", Flag(false)),
        ],
        "chattel_mortgage_schedule" => vec![
            ("year", Text("2026-27")),
            ("amount_financed", Text("45000.00")),
            ("annual_rate", Text("0.0799")),
            ("instalments", Whole(60)),
            ("balloon", Text("5000.00")),
            ("timing", Text("in_arrears")),
            ("start_date", Text("2026-07-15")),
        ],
        "loan_transactions" => vec![
            ("year", Text("2026-27")),
            ("opening_balance", Text("0.00")),
            ("annual_rate", Text("0.073")),
            ("start_date", Text("2026-07-01")),
            ("end_date", Text("2026-09-30")),
            (
                "transactions",
                Json(
                    r#"[{"date": "2026-07-01", "kind": "drawing", "amount": "10000.00"}, {"date": "2026-08-31", "kind": "repayment", "amount": "1000.00"}]"#,
                ),
            ),
        ],
        "hire_purchase_schedule" => vec![
            ("year", Text("2026-27")),
            ("amount_financed", Text("45000.00")),
            ("instalment", Text("850.00")),
            ("instalments", Whole(60)),
            ("balloon", Text("5000.00")),
            ("timing", Text("in_arrears")),
            ("start_date", Text("2026-07-15")),
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
        "fbt_loan" => vec![
            ("daily_balance_total", Text("7300000.00")),
            ("interest_accrued", Text("0.00")),
            ("income_use", Text("0")),
            ("accrued_interest_deduction", Text("0.00")),
        ],
        "fbt_meal_entertainment" => vec![
            ("method", Text("50_50_split")),
            ("meal_entertainment_expenditure", Text("4000.00")),
        ],
        // A 12 week register kept from 1 May to 23 July 2025, so first valid
        // for the FBT year ended 31 March 2026, checked for that year.
        "fbt_meal_entertainment_register" => vec![
            ("register_start", Text("2025-05-01")),
            ("register_end", Text("2025-07-23")),
            (
                "entries",
                Json(
                    r#"[{"date": "2025-05-14", "recipients": {"employees": 3, "associates": 0, "others": 2}, "cost": "500.00", "kind": "client dinner", "location": "restaurant", "on_premises": false, "fringe_benefits_value": "300.00"}, {"date": "2025-07-02", "recipients": {"employees": 10, "associates": 2, "others": 0}, "cost": "1200.00", "kind": "end of year party", "location": "the employer's office", "on_premises": true, "in_house_dining_facility": false, "fringe_benefits_value": "1200.00"}]"#,
                ),
            ),
        ],
        "fbt_car_parking" => vec![
            ("method", Text("commercial_parking_station")),
            ("first_business_day_fee", Text("18.00")),
            ("benefits", Whole(230)),
            ("lowest_fee", Text("18.00")),
            ("employee_contribution", Text("1000.00")),
        ],
        "fbt_debt_waiver" => vec![("amount_waived", Text("500.00"))],
        "fbt_expense_payment" => vec![
            ("method", Text("external")),
            ("recipients_expenditure", Text("1200.00")),
            ("amount_paid", Text("1200.00")),
            ("employee_contribution", Text("0.00")),
            ("income_use", Text("0.6")),
            ("recipients_portion_deduction", Text("0.00")),
        ],
        "fbt_property" => vec![
            ("method", Text("in_house_public_price")),
            ("lowest_price", Text("1200.00")),
            ("employee_contribution", Text("600.00")),
            ("income_use", Text("0")),
            ("contribution_deduction", Text("0.00")),
            ("in_house_reduction_available", Text("0.00")),
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
        // The net of the payg_withholding example, whose gross is its only one.
        "payg_gross_up" => vec![
            ("year", Text("2026-27")),
            ("net_pay", Text("862.00")),
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
        // Shaped on the ATO's Jackson example: $102,000, terminated for
        // invalidity a year before the 65th birthday, moved into 2026-27.
        "etp_tax_free_component" => vec![
            ("payment", Text("102000.00")),
            ("employment_start", Text("2013-09-02")),
            ("termination_date", Text("2026-09-25")),
            ("last_retirement_day", Text("2027-09-25")),
            ("ill_health", Flag(true)),
            ("medical_certification", Flag(true)),
        ],
        // Schedule 7's example (Norman), which gives no accrual start: any day
        // before 16 August 1978 gives its 3 long service leave periods.
        "unused_leave_payment" => vec![
            ("annual_leave", Text("0.00")),
            ("leave_loading", Text("0.00")),
            ("long_service_leave", Text("22020.00")),
            ("long_service_leave_before_16_august_1978", Text("780.00")),
            (
                "long_service_leave_16_august_1978_to_17_august_1993",
                Text("6420.00"),
            ),
            ("leave_accrual_start", Text("1977-01-01")),
            ("termination_reason", Text("normal")),
            ("working_holiday_maker", Flag(false)),
            ("earnings", Text("1155.00")),
            ("pay_period", Text("weekly")),
            ("scale", Whole(2)),
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
        // The ATO's Yan example's combined breakdown, received in 2025-26, with
        // made-up taxable incomes.
        "lump_sum_arrears_tax_offset" => vec![
            ("year", Text("2025-26")),
            ("taxable_income", Text("100000")),
            ("net_capital_gain", Text("0")),
            (
                "accrual_years",
                Json(
                    r#"[{"year": "2023-24", "arrears": "3000", "taxable_income": "30000"}, {"year": "2022-23", "arrears": "2500", "taxable_income": "17000"}, {"year": "2021-22", "arrears": "1400"}, {"year": "2020-21", "arrears": "1700"}, {"year": "2019-20", "arrears": "1400"}, {"year": "2018-19", "arrears": "1000"}]"#,
                ),
            ),
        ],
        // FBT return 2026 instructions, item 23M example 16.
        "fbt_residual" => vec![
            ("method", Text("external_notional_value")),
            ("notional_value", Text("8000.00")),
            ("employee_contribution", Text("0.00")),
            ("income_use", Text("0.75")),
            ("contribution_deduction", Text("0.00")),
        ],
        // Item 23F example 11.
        "fbt_housing" => vec![
            ("method", Text("base_year")),
            ("market_value", Text("41600.00")),
            ("tenancy_days", Whole(365)),
            ("employee_contribution", Text("5200.00")),
        ],
        // Item 23G example 12's first 40 weeks.
        "fbt_lafha" => vec![
            ("method", Text("maintains_home")),
            ("allowance", Text("23640.00")),
            ("accommodation_component", Text("14000.00")),
            ("accommodation_expenses", Text("14000.00")),
            ("food_component", Text("9640.00")),
            ("food_expenses", Text("9640.00")),
            ("normal_food_amount", Text("0.00")),
            ("adult_days", Whole(280)),
            ("child_days", Whole(0)),
        ],
        // Item 23J example 13.
        "fbt_board" => vec![
            ("adult_meals", Whole(10950)),
            ("child_meals", Whole(0)),
            ("employee_contribution", Text("0.00")),
            ("income_use", Text("0")),
            ("contribution_deduction", Text("0.00")),
        ],
        // Item 23L example 15.
        "fbt_tax_exempt_body_entertainment" => vec![
            ("method", Text("actual")),
            ("entertainment_expenditure", Text("5000.00")),
        ],
        // TR 97/17 example 1.1 (paras 49 to 52): an end of financial year
        // function on the employer's premises, valued per head.
        // The Library's birthday flowers (Tax Examples, FBT, 3-060): an $80
        // bouquet on the employee's birthday, a property benefit.
        "fbt_minor_benefit_check" => vec![
            ("category", Text("property")),
            ("in_house", Flag(false)),
            ("notional_taxable_value", Text("80.00")),
            ("salary_sacrifice", Flag(false)),
            ("frequency", Text("infrequent_and_irregular")),
            ("similar_benefits_substantial", Flag(false)),
            ("other_associated_benefits_substantial", Flag(false)),
            ("valuation_difficult", Flag(false)),
            ("unexpected_event", Flag(false)),
            ("reward_for_services", Flag(false)),
        ],
        "fbt_entertainment_classifier" => vec![
            (
                "lines",
                Json(
                    r#"[{"cost": "1100.00", "gst_status": "includes_gst", "provided": "food_or_drink", "occasion": "social_function", "venue": "business_premises_working_day", "paid_by": "employer", "attendees": {"employees": 6, "associates": 2, "others": 2}}]"#,
                ),
            ),
            ("income_tax_exempt", Flag(false)),
            ("method", Text("actual")),
        ],
        // The ATO's Christina example for repayment income; the split of her
        // investment loss, fringe benefits and super contributions, her target
        // foreign income and her assessable income are fabricated.
        "income_tests" => vec![
            ("year", Text("2025-26")),
            ("taxable_income", Text("50420")),
            ("assessable_fhss_released_amount", Text("0")),
            ("family_trust_distribution_amounts", Text("0")),
            ("reportable_fringe_benefits_section_57a", Text("0")),
            ("reportable_fringe_benefits_other", Text("4560")),
            ("reportable_employer_super_contributions", Text("15000")),
            ("deductible_personal_super_contributions", Text("0")),
            ("excess_concessional_contributions", Text("0")),
            ("net_financial_investment_loss", Text("1250")),
            ("net_rental_property_loss", Text("0")),
            ("tax_free_government_pensions", Text("0")),
            ("target_foreign_income", Text("2580")),
            ("exempt_foreign_employment_income", Text("2580")),
            ("child_support_paid", Text("0")),
            ("assessable_income", Text("50420")),
            ("business_deductions", Text("0")),
            ("residency", Text("full_year_resident")),
        ],
        _ => Vec::new(),
    }
}
