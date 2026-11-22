// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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

/// The same singles thresholds for the private health insurance rebate's
/// earlier years, from the ATO's 'Income thresholds and rates for the private
/// health insurance rebate' pages for 2021 to 2024 (published 14 May 2025) and
/// its Medicare levy surcharge pages, read on 22 November 2026. PHI Act s 22-35
/// states the 2021-22 and 2022-23 amounts; s 22-45 indexes them from 2023-24.
pub const PHI_EARLIER_THRESHOLDS: &[(&str, [&str; 3])] = &[
    ("2020-21", ["90000", "105000", "140000"]),
    ("2021-22", ["90000", "105000", "140000"]),
    ("2022-23", ["90000", "105000", "140000"]),
    ("2023-24", ["93000", "108000", "144000"]),
];

/// The private health insurance rebate percentages for each adjustment year,
/// keyed by the calendar year of the 1 April it starts on (PHI Act s 22-15(5A)
/// to (5D)): for the oldest insured person under 65, 65 to 69 and 70 or over,
/// the base tier, tier 1 and tier 2 percentages; tier 3 is nil (s 22-15(4)).
/// From the ATO's rates pages for 2020-21 to 2023-24 (published 14 May 2025)
/// and its current page (last updated 22 June 2026), read on 22 November 2026.
/// The percentages from 1 April 2027 were not published.
pub const PHI_REBATE_PERCENTAGES: &[(&str, [[&str; 3]; 3])] = &[
    (
        "2020",
        [
            ["25.059", "16.706", "8.352"],
            ["29.236", "20.883", "12.529"],
            ["33.413", "25.059", "16.706"],
        ],
    ),
    ("2021", PHI_2021_TO_2024),
    ("2022", PHI_2021_TO_2024),
    ("2023", PHI_2021_TO_2024),
    ("2024", PHI_2021_TO_2024),
    (
        "2025",
        [
            ["24.288", "16.192", "8.095"],
            ["28.337", "20.240", "12.143"],
            ["32.385", "24.288", "16.192"],
        ],
    ),
    (
        "2026",
        [
            ["24.118", "16.079", "8.038"],
            ["28.139", "20.098", "12.058"],
            ["32.158", "24.118", "16.079"],
        ],
    ),
];

/// Unchanged from 1 April 2021 to 31 March 2025.
const PHI_2021_TO_2024: [[&str; 3]; 3] = [
    ["24.608", "16.405", "8.202"],
    ["28.710", "20.507", "12.303"],
    ["32.812", "24.608", "16.405"],
];

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

/// The low rate cap amount (ITAA 1997 s 307-345) for the years anyone can be
/// at or above preservation age and under 60: from 1 July 2024 the
/// preservation age is 60 for everyone born after 30 June 1964. The ATO's
/// 'Payments from super', Table 9 (last updated 16 September 2026).
pub const SUPER_LOW_RATE_CAP: [(&str, &str); 4] = [
    ("2020-21", "215000"),
    ("2021-22", "225000"),
    ("2022-23", "230000"),
    ("2023-24", "235000"),
];

/// The untaxed plan cap amount (ITAA 1997 s 307-350): the same page,
/// Table 10; Schedule 12 (2026) gives $1,935,000 for 2026-27.
pub const SUPER_UNTAXED_PLAN_CAP: [(&str, &str); 7] = [
    ("2020-21", "1565000"),
    ("2021-22", "1615000"),
    ("2022-23", "1650000"),
    ("2023-24", "1705000"),
    ("2024-25", "1780000"),
    ("2025-26", "1865000"),
    ("2026-27", "1935000"),
];

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

/// The beneficiary tax offset's amounts (Income Tax Assessment (1936 Act)
/// Regulations 2025 s 13(2) and (3), and the 2015 Regulation s 13): $6,000,
/// and the threshold at the upper conclusion of the lowest marginal tax rate,
/// the top of item 1 of Income Tax Rates Act 1986 Sch 7 Part I, $45,000 for
/// each year 2020-21 to 2026-27 (compilations 61 and 66).
pub const BENEFICIARY_LOWER_THRESHOLD: i64 = 6_000;
pub const BENEFICIARY_UPPER_THRESHOLD: i64 = 45_000;

/// The invalid and invalid carer tax offset by year: the indexed amount
/// (ITAA 1997 s 61-30), the ATO's daily rate for part of the year and the
/// FTB Part B income limit (s 61-20), from the ATO's T5 Invalid and invalid
/// carer instructions for 2021 to 2026, read on 22 November 2026.
pub const INVALID_CARER_OFFSET: &[(&str, [&str; 3])] = &[
    ("2020-21", ["2816", "7.72", "100000"]),
    ("2021-22", ["2833", "7.76", "100900"]),
    ("2022-23", ["2943", "8.06", "104432"]),
    ("2023-24", ["3152", "8.61", "112578"]),
    ("2024-25", ["3300", "9.04", "117194"]),
    ("2025-26", ["3396", "9.30", "120007"]),
];

/// The net medical expenses tax offset's lower and higher phase-in limits
/// (ITAA 1936 s 159Q(5) and (6), indexed), from the ATO's T5 Total net medical
/// expenses instructions for 2017, 2018 and 2019, read on 22 November 2026.
pub const NET_MEDICAL_LIMITS: &[(&str, [&str; 2])] = &[
    ("2016-17", ["2299", "5423"]),
    ("2017-18", ["2333", "5504"]),
    ("2018-19", ["2377", "5609"]),
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
        // s 5 sets fixed rates; the temporary budget repair levy of s 6 ended
        // with 2016-17. The years are the other superannuation worksheets'.
        "dasp_tax" => vec!["2025-26", "2026-27", "2027-28"],
        // The years the map asks for; the untaxed plan cap for 2027-28 is not
        // yet published (s 307-350, indexed each February).
        "super_benefit_tax" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        // Division 75 sets no figure by year; the years are gst's.
        "gst_margin_scheme" => vec!["2025-26", "2026-27", "2027-28"],
        // ITAA 1997 Division 205 sets no figure by year; the years the
        // Act as compiled on 19 November 2026 and the ATO's Franking account
        // tax return 2026 instructions were read for, with the 25% and 30%
        // rates for imputation purposes.
        "franking_account" => vec!["2025-26", "2026-27"],
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
        "cgt_parcels" => vec!["2025-26", "2026-27"],
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
        // The years whose rates the ATO's return instructions publish (the
        // beneficiary and zone amounts are fixed in the law); the invalid carer
        // amount for 2026-27 is unpublished, and the net medical expenses
        // offset ran to 2018-19 (ITAA 1936 s 159P(1A)).
        "beneficiary_tax_offset" | "zone_tax_offset" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        "invalid_carer_tax_offset" => keys(INVALID_CARER_OFFSET),
        "net_medical_expenses_tax_offset" => keys(NET_MEDICAL_LIMITS),
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
        // The financial years of the determination day: determinations began
        // on 1 July 2018, and the last published SIC quarter ends on
        // 31 December 2026.
        "fhss" => vec![
            "2018-19", "2019-20", "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26",
            "2026-27",
        ],
        // The year of the registration, possession or attachment, or
        // agreement: the fees are unchanged since 1 August 2018 and the law
        // read is in force from 14 October 2024 (PPSA), and the Corporations
        // Act's ss 9 and 588FL to 588FN read the same from 1 July 2025 with no
        // enacted amendment; critical times are taken within the same years.
        "ppsr_timing" => vec!["2025-26", "2026-27"],
        "fbt_entertainment_classifier" => vec!["2026", "2027"],
        "fbt_housing" => keys(FBT_HOUSING_INDEXATION),
        // The years whose definitions were read: the 1 July 2024 compilations
        // define them as the current ones do, and the ATO's 2025 and 2026
        // return instructions and its current pages apply them.
        // Subdivision 328-F reads the same in compilations 210 (1 July 2020),
        // 223 (1 July 2021) and 268 (1 October 2026) but for the rate.
        "small_business_income_offset" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        // Division 35 reads the same in compilations 210 (1 July 2020), 223
        // (1 July 2021) and 268 (1 October 2026).
        "non_commercial_losses" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        // Part 2-42 reads the same in compilations 210 (1 July 2020), 223
        // (1 July 2021) and 268 (1 October 2026).
        "psi_test" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        // ITAA 1936 s 6(1) 'resident' reads the same in compilations 169
        // (1 July 2020) and 193 (1 October 2026).
        "residency_tests" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        // Each year's lodgment instrument, read on the register; none for the
        // 2027 year yet.
        "lodgment_required" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26",
        ],
        "income_tests" => vec!["2024-25", "2025-26", "2026-27"],
        // Each year's rates were read at the ATO, the Commissioner's
        // instruments and PCG 2023/1 on 22 November 2026; 2027-28 has no cents
        // per kilometre rate yet.
        "work_expenses" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        // The years whose thresholds and percentages the ATO publishes, from
        // 2020-21; the percentages from 1 April 2027 are refused line by line.
        "private_health_insurance_rebate" => {
            [keys(PHI_EARLIER_THRESHOLDS), keys(SURCHARGE_THRESHOLDS)].concat()
        }
        // The pools' rates and the capital works rates do not change in these
        // years; the small business pool's low pool value threshold does, by year.
        "depreciation_low_value_pool" | "depreciation_small_business_pool" | "capital_works" => {
            vec![
                "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
            ]
        }
        // The ATO's co-contribution thresholds from 2020-21, under the Act's
        // compilation 28 (1 July 2018 to 30 June 2027); income-tests'
        // definitions read the same in the compilations of 1 July 2020.
        "government_super_amounts" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        // Act No. 92 of 2020's rates, cap and clawback rules, for income years
        // starting on or after 1 July 2021, are unchanged to 2026-27.
        "rd_tax_offset" => vec![
            "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        "sb_cgt_concessions" => vec!["2025-26", "2026-27"],
        // The question's end date. GIC days run back to 1 July 1999 and SIC
        // days to 1 April 2005; the penalty unit holds at $364 until the next
        // indexation day, 1 July 2029 (Crimes Act s 4AA(3)); GIC days after
        // 31 December 2026 need an assumed rate.
        "ato_interest_penalties" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27", "2027-28",
        ],
        // ITAA 1997 ss 8-1, 26-31, 40-27 and Division 43 read the same from
        // 2020-21 (no amendment after Act No. 126 of 2017 in compilation 264's
        // history but s 43-145's build to rent manner, out of scope); the tax
        // effect is refused before 2024-25, the first year resident-tax holds.
        "rental_property" => vec![
            "2020-21", "2021-22", "2022-23", "2023-24", "2024-25", "2025-26", "2026-27",
        ],
        // The years payg_withholding's Schedule 1 holds: Schedules 6 and 14 and
        // the daily and casual table use it, and Schedules 2, 3, 4 and 10 and
        // s 12-190 were read for the same years only.
        "horticulture_shearing_withholding"
        | "upward_variation_withholding"
        | "daily_casual_withholding"
        | "no_abn_withholding"
        | "entertainer_withholding"
        | "return_to_work_withholding"
        | "annuity_withholding"
        | "voluntary_agreement_withholding" => keys(PAYG_WITHHOLDING_COEFFICIENTS),
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
        "super_benefit_tax" => (
            "2026-11-22",
            "ITAA 1997 ss 301-10 to 301-40, 301-90 to 301-115, 301-225, 302-60, 302-140, 302-145, 302-195, 303-10, 307-120, 307-125, 307-145, 307-150, 307-205, 307-275 to 307-290, 307-345, 307-350 and 960-285, from the ATO's legal database; the ATO's 'Payments from super' (last updated 16 September 2026, Tables 9, 10, 12 and 13), 'Tax on super benefits' (2 August 2023) and 'Tax rates – Australian resident' (13 August 2026); Schedule 12 of the Taxation Administration (Withholding Schedules) Instrument 2026 (F2026L00716) and the ATO's Schedule 12 from 13 October 2020 to 30 June 2024",
        ),
        "dasp_tax" => (
            "2026-11-19",
            "Superannuation (Departing Australia Superannuation Payments Tax) Act 2007 ss 4 to 6 (compilation 5, 2 December 2016, Federal Register of Legislation); ITAA 1997 ss 301-170 and 301-175, from the ATO's legal database; the ATO's 'Payments from super' rates (last updated 16 September 2026) and 'Departing Australia superannuation payment (DASP)' (last updated 21 April 2026)",
        ),
        "payroll_tax" => (
            "2026-11-19",
            "Payroll Tax Act 2007 (NSW) Sch 1 cl 1 (version current from 10 July 2026); Payroll Tax Act 2007 (Vic) s 3A and Sch 1 cl 1, 1A, 1B and 5 (authorised version 041, 1 May 2026); Payroll Tax Act 1971 (Qld) ss 10, 29 and 43I (reprint current as at 22 October 2026); Pay-roll Tax Assessment Act 2002 (WA) ss 8 and 13 (04-d0-03) and Pay-roll Tax Act 2002 (WA) s 5(4) (00-d0-02); Payroll Tax Act 2009 (SA) Sch 1 cl 1, 4 and 5 (version 31.3.2026); Payroll Tax Act 2008 (Tas) Sch 1 cl 1 and 4; Payroll Tax Act 2011 (ACT) Sch 1 (R20) with DI2026-151 s 5; Payroll Tax Act 2009 (NT) s 7A and Sch 1 cl 1, 3 and 5 (as in force at 1 July 2026); and each revenue office's rates and thresholds page",
        ),
        "gst_margin_scheme" => (
            "2026-11-19",
            "GST Act ss 9-90, 11-5, 75-5, 75-10, 75-11, 75-12, 75-13, 75-14, 75-15, 75-20, 75-35 and 195-1 ('approved valuation', 'long-term lease', 'margin scheme'), from the ATO's legal database; GSTR 2006/8 (paras 15 to 39F: the agreement, a conditional agreement, ineligible supplies and examples 1 and 2); the ATO's 'GST and the margin scheme' (last updated 19 August 2021)",
        ),
        "franking_account" => (
            "2026-11-19",
            "ITAA 1997 ss 202-60, 202-65, 202-75, 203-20 to 203-55, 204-70, 204-75, 205-15, 205-25, 205-30, 205-40, 205-45, 205-50, 205-70, 214-15 to 214-25, 214-150 and 214-155, on the ATO's legal database; New Business Tax System (Franking Deficit Tax) Act 2002 s 5 (C2004A00982) and (Over-franking Tax) Act 2002 s 5 (C2004A00981), latest versions on the Federal Register of Legislation; the ATO's Franking account tax return 2026 instructions (published 30 May 2026): How to get, About (example 2), How to lodge and pay, and sections A, B (examples 4 and 5) and C (examples 7 to 9)",
        ),
        "wine_equalisation_tax" => (
            "2026-11-19",
            "A New Tax System (Wine Equalisation Tax) Act 1999 ss 5-5, 5-50, 7-10, 9-5, 9-25, 9-30, 9-35, 9-40, 9-45, 13-5, 19-5, 19-15, 19-20, 19-25, 31-1 and 33-1 at the ATO's legal database, and compilation 32 (1 April 2026) at the Federal Register of Legislation for ss 19-15 and 19-25; Treasury Laws Amendment (Supporting Choice in Superannuation and Other Measures) Act 2026 (No. 12, 2026) Schedule 6 (the $400,000 cap for assessable dealings from 1 July 2026); WETR 2009/1 paras 143 to 153 and 159; WETR 2009/2 para 67 and note 37; the ATO's 'Wholesale sales' (QC22750) and 'Retail sales and own use' (QC22751), last updated 19 March 2025; 'Producer rebate' (QC22756, 30 June 2026); 'Calculate and claim your rebate' (QC22758, 2 July 2026); 'Reporting and paying WET' (QC22762, 19 March 2025); 'Wine equalisation tax (WET)' (QC33703, 1 May 2025); 'Instructions for completing your BAS' (QC17458, 14 September 2026)",
        ),
        "luxury_car_tax" => (
            "2026-11-19",
            "A New Tax System (Luxury Car Tax) Act 1999 ss 5-15, 5-20, 7-15, 9-5, 9-15, 15-20, 15-25, 15-30, 18-5, 18-10, 25-1 and 27-1 at the ATO's legal database, with their history notes (ss 18-5(3) and 18-10(3) substituted by Act No. 94 of 2019 for cars supplied or imported from 1 July 2019), and compilation 27 (1 April 2025) at the Federal Register of Legislation; Treasury Laws Amendment (Tax Incentives and Integrity) Act 2025 (No. 29, 2025) Schedule 1 (the 3.5 litre fuel-efficient rating from 1 July 2025 and item 4's continued 7 litre rating); Luxury Car Tax Imposition (General) Act 1999 and (Customs) Act 1999 s 4 (33%); the ATO's 'Luxury car tax rate and thresholds' (QC38161, 1 June 2026), 'Working out the LCT on a sale' (QC22107, 1 June 2026), 'Working out the LCT on an import' (QC22108, 1 June 2026), 'Luxury car tax credits and refunds' (QC39467, 16 July 2025) and the BAS page 'Luxury car tax (LCT)' (QC33694, 1 May 2025); 'Instructions for completing your BAS' (QC17458, 14 September 2026)",
        ),
        "cgt_parcels" => (
            "2026-11-19",
            "ITAA 1997 ss 102-5, 102-10, 102-15, 104-10, 109-5, 110-25, 110-35, 110-55, 112-30, 114-1, 114-10, 115-5 to 115-25, 115-100 and 116-40, compilation 264 (21 May 2026), the text for 2025-26 and 2026-27 under Act No. 49 of 2026 Schedule 1 items 82 to 84; TD 33 (identical shares); the ATO's 'Keeping records of shares and units' (last updated 22 June 2026: Boris's example), 'CGT discount' (last updated 29 June 2026: the 12 months exclude the days of acquisition and the event) and the individual tax return instructions supplement 2026, question 18 (last updated 30 May 2026: Kathleen's examples 1 to 4)",
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
            "2026-11-20",
            "Study and training loan repayment thresholds and rates: tables 1 and 2, repayment income, examples 1 and 2; Gazette C2026G00249 for the 2026-27 minimum repayment income and replacement indexable amount",
        ),
        "study_loan_withholding" => (
            "2026-11-14",
            "Schedule 8 (NAT 3539) for payments from 1 July 2026, and Schedule 8 of the Taxation Administration (Withholding Schedules) Instrument (No. 2) 2025 (F2025L01016) for payments from 24 September 2025: study and training support loans component rates, calculating fortnightly, monthly and quarterly amounts, examples 1 to 3, rounding",
        ),
        "medicare_levy" => (
            "2026-11-14",
            "Medicare Levy Act 1986 ss 3(1) and (3), 6(1), 7(1) and (2) and 8, compilations 52 (2024-25) and 53; Treasury Laws Amendment (More Cost of Living Relief) Act 2025 Sch 2 item 14 (2024-25 and later years); Treasury Laws Amendment (Delivering an Efficient and Trusted Tax System) Act 2026 Sch 5 item 14 (2025-26 and later years); ATO Medicare levy reduction for low-income earners and Medicare levy reduction - family income, with the Ashton example (2025-26); myTax 2025 tables 2 and 3 and worksheet 2 (2024-25); exemptions: Medicare Levy Act 1986 s 9(1), ITAA 1936 ss 251T and 251U(1) to (3), ATO Medical exemption from Medicare levy (the Paul example), Not entitled to Medicare benefits (the Priya example) and M1 Medicare levy reduction or exemption 2026, part B",
        ),
        "private_health_insurance_rebate" => (
            "2026-11-22",
            "Private Health Insurance Act 2007 ss 22-5, 22-15, 22-20, 22-25, 22-30, 22-35, 22-40, 22-45 and 282-18, compilation 42 (19 September 2026); ITAA 1997 ss 61-205, 61-210, 61-215 and 67-23, compilation 268 (1 October 2026); ATO Income thresholds and rates for the private health insurance rebate (22 June 2026) and its 2021 to 2024 pages (14 May 2025), with the Narelle, Peta, Mike and Elle, and Zoe and Charlie examples; Claiming the private health insurance rebate (QC72945), with the Toby and Donna examples; Your private health insurance statement (benefit codes)",
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
        "beneficiary_tax_offset" => (
            "2026-11-22",
            "ITAA 1936 s 160AAA(1) to (4), compilation 193 (1 October 2026); Income Tax Assessment (1936 Act) Regulations 2025 ss 4, 13 and 23 (F2025L01060) and Regulation 2015 s 13, compilation 8 (9 December 2022); Income Tax Rates Act 1986 Sch 7 Part I, compilations 61 (20 October 2023) and 66 (1 July 2026); ITAA 1997 s 63-10(1) item 15; ATO Beneficiary tax offset (QC72199, 8 June 2026)",
        ),
        "invalid_carer_tax_offset" => (
            "2026-11-22",
            "ITAA 1997 Subdivision 61-A, ss 61-10 to 61-45, compilation 268 (1 October 2026); ATO T5 Invalid and invalid carer, supplementary tax return instructions 2021 to 2026 (worksheets 1A to 3C, the amounts, daily rates and FTB Part B limits); Offset for maintaining an invalid or invalid carer (QC72202, 8 June 2026)",
        ),
        "zone_tax_offset" => (
            "2026-11-22",
            "ITAA 1936 ss 79A and 79B and Schedule 2, compilation 193 (1 October 2026); ITAA 1997 Subdivisions 961-A and 961-B, compilation 268; ATO T4 Zone or overseas forces, supplementary tax return instructions 2021 to 2026 (tables 1 to 3, worksheets 1 to 7, the Garry and Sharon examples); Zone tax offset (QC105018, 8 June 2026, the Narendra example); Australian zone list (1 July 2026)",
        ),
        "net_medical_expenses_tax_offset" => (
            "2026-11-22",
            "ITAA 1936 ss 159P and 159Q, compilation 161 (6 April 2019); ATO T5 Total net medical expenses for disability aids, attendant care or aged care, supplementary tax return instructions 2017, 2018 and 2019 (tables 1 and 2); Medical expenses tax offset (16 June 2025)",
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
        "ppsr_timing" => (
            "2026-11-20",
            "Personal Property Securities Act 2009 ss 10 ('business day'), 62, 153 and 294, compilation 22 (14 October 2024); Personal Property Securities Regulations 2010, compilation 6 (1 July 2022), which prescribe no day for the s 10 definition; Personal Property Securities (Fees) Determination 2018 ss 2, 4, 5 and 6, as made (from 1 August 2018); Corporations Act 2001 ss 9 ('business day'), 588FL and 588FM, compilation 149 (19 September 2026), the same text in compilation 142 (1 July 2025) and unamended by the Acts listed for its versions from 1 December 2026 to 1 July 2027 (Nos. 61, 35, 38 and 57 of 2026 and 72 of 2025); the Registrar's 'Fees for using the PPSR' page",
        ),
        "fbt_minor_benefit_check" => (
            "2026-11-19",
            "FBTAA ss 58P, 136(1) ('fringe benefit' para (g) and 'eligible premises') and 152B, from the ATO's legal database (s 58P(1)(b) repealed by Act No. 88 of 2013); TR 2007/12 (paras 8 to 23 and examples 1 to 13)",
        ),
        "fhss" => (
            "2026-11-20",
            "TAA 1953 Sch 1 ss 12-460, 131-5, 131-10, 138-10, 138-25, 138-30, 138-35, 138-40 and 280-105, and ITAA 1997 ss 313-20, 313-25, 313-50 and 313-60, from the ATO's legal database; Taxation Administration Regulations 2017 reg 53A, compilation 18 (14 November 2026); Treasury Laws Amendment (2023 Measures No. 3) Act 2023 Sch 4 items 28 and 30, as made; GN 2024/1 sections 1 to 7 and examples 6, 7 and 11 to 13; the ATO's 'About FHSS release amounts' (Mary), 'FHSS tax assessment' and 'Shortfall interest charge (SIC) rates' (last updated 4 September 2026) pages",
        ),
        "fbt_entertainment_classifier" => (
            "2026-11-16",
            "ITAA 1997 Division 32, s 27-5 and s 995-1, compilation 268 (1 October 2026); FBTAA ss 5E, 20, 37A to 37CF, 40, 41, 45, 58P and 135P(1) and s 136(1), compilation 97 (1 July 2026); Fringe Benefits Tax Assessment Regulations 2018, regulations 6 to 11, as made; ITAA 1936 ss 51AEA and 51AEB, compilation 193 (1 October 2026); GST Act s 69-5, compilation 96 (1 January 2026); TR 97/17, paras 7 to 143 and the Part A table, and TR 2007/12, paras 8 to 22 and examples 1 to 3, as the Commissioner's view",
        ),
        "work_expenses" => (
            "2026-11-22",
            "ITAA 1997 ss 28-12, 28-25, 28-90, 28-95, 28-100, 28-110, 28-115, 28-120 and 28-125 (ATO legal database); the cents per kilometre instruments of 2020 (F2020L00676), 2022 (F2022L00813), 2023 (F2023L00767), 2024 (F2024L00697) and 2026 (F2026L00785); PCG 2023/1 as updated 16 April 2025, paras 14 to 28, 45 to 89 and Table 1; the ATO's 52 cent fixed rate, shortcut (PCG 2020/3), fixed rate, actual cost, cents per kilometre, log book and self-education pages; the 2022 D4 instructions; TR 98/9 paras 25 to 30 and 119 to 155 on ITAA 1936 s 82A; Act No. 84, 2022, Schedule 3 items 26 and 33",
        ),
        "ato_interest_penalties" => (
            "2026-11-22",
            "TAA 1953 ss 8AAC and 8AAD and Sch 1 ss 280-100, 280-105, 284-75, 284-80, 284-85, 284-90, 284-220, 284-224, 284-225, 286-75, 286-80 and 298-20, ITAA 1997 s 5-15 and Crimes Act 1914 s 4AA at the ATO's legal database; Taxation (Interest on Overpayments and Early Payments) Act 1983 ss 3, 3C, 8A to 8C, 9, 10, 12D and 12E (compilation 58, 14 October 2024, at the Federal Register of Legislation, and the ATO's legal database); the ATO's 'General interest charge (GIC) rates' and 'Shortfall interest charge (SIC) rates' (last updated 4 September 2026), 'Credit interest rates and calculation' (QC58284, 19 June 2026), 'Penalty units' (QC71196, 26 June 2026) and 'Significant global entities - penalties' (1 July 2026); PS LA 2006/8, PS LA 2011/19 paras 15 to 30, PS LA 2011/23 para 4, PS LA 2012/5 paras 95 to 137, MT 2008/2 paras 33 and 55 to 63 and MT 2012/3 paras 35 and 88 to 89",
        ),
        "depreciation_low_value_pool" => (
            "2026-11-22",
            "ITAA 1997 ss 40-425, 40-430, 40-435, 40-440 and 40-445 (ATO legal database); ATO Low-value pool (last updated 26 June 2025), examples 1 to 3 and the closing balance example; Guide to depreciating assets 2026, Working out the decline in value of depreciating assets in a low-value pool and its examples",
        ),
        "depreciation_small_business_pool" => (
            "2026-11-22",
            "ITAA 1997 ss 328-185, 328-190, 328-195, 328-200, 328-210 and 328-215 (ATO legal database); IT(TP)A 1997 ss 328-180(6) and 328-181(5); Treasury Laws Amendment (Tax Reform No. 2) Act 2026 (No. 71 of 2026) Sch 2 items 7 and 15; ATO Small business pool calculations (last updated 16 June 2025), examples 2 and 3; Instant asset write-off for eligible businesses (last updated 28 August 2026), table 1; LCR 2021/3, example 13",
        ),
        "capital_works" => (
            "2026-11-22",
            "ITAA 1997 ss 43-140, 43-145, 43-210 and 43-215 (ATO legal database); ATO Rental properties guide 2026, Capital works deductions, tables 1 and 2 and example 29; Guide to depreciating assets 2026, days held in a leap year",
        ),
        "horticulture_shearing_withholding" => (
            "2026-11-22",
            "Schedule 2 (NAT 1013, horticultural or shearing industry) of F2024L00664, F2025L00647, F2025L01016 and F2026L00716: using this schedule, working out the withholding amount and its example, resident and foreign resident workers, pay period, using a formula, Table A, rounding and TFN declarations",
        ),
        "upward_variation_withholding" => (
            "2026-11-22",
            "Schedule 14 (NAT 5441, an agreement to increase withholding) of F2024L00664, F2025L00647, F2025L01016 and F2026L00716: using this schedule, how to work out the withholding amount and its example; Schedule 1 as payg_withholding records",
        ),
        "daily_casual_withholding" => (
            "2026-11-22",
            "Schedule 1, 'Calculating withholding amounts for payments made on a daily or casual basis', of F2024L00664, F2025L01016 and F2026L00716; the tax table for daily and casual workers (NAT 1024) from 1 July 2024 and from 1 July 2026: using this tax table, working out the withholding amount and its example, using a formula, TFN declarations, claiming tax offsets and foreign residents",
        ),
        "no_abn_withholding" => (
            "2026-11-22",
            "TAA 1953 Sch 1 ss 12-190 and 15-10 (ATO legal database); Taxation Administration Regulations 2017 ss 4 ('top rate'), 26 and 38, compilations 12 (1 July 2024) and 18 (14 November 2026); A New Tax System (Goods and Services Tax) Regulations 2019 s 29-80.01, compilations 4 and 5; ATO Payments you don't withhold from (last updated 11 June 2025); Rental expenses (Rental properties 2025)",
        ),
        "entertainer_withholding" => (
            "2026-11-22",
            "Schedule 3 (NAT 1023, actors, variety artists and other entertainers) of F2024L00664, F2025L00647, F2025L01016 and F2026L00716: using this schedule, working out the withholding amount and its example, using a formula, Tables A and B and Sandra's example, TFN declarations, foreign residents and claiming tax offsets",
        ),
        "return_to_work_withholding" => (
            "2026-11-22",
            "Schedule 4 (return to work payments) of F2024L00664, F2025L00647, F2025L01016 and F2026L00716: using this schedule, working out the withholding amount and George's example, rounding and TFN declarations; TAA 1953 Sch 1 s 12-50 (ATO legal database)",
        ),
        "annuity_withholding" => (
            "2026-11-22",
            "Schedule 6 (NAT 1005, annuities) of F2024L00664, F2025L00647, F2025L01016 and F2026L00716: using this schedule, working out the withholding amount, rounding, Barbara's and Kenneth's examples and TFN declarations; TAA 1953 Sch 1 s 12-80 (ATO legal database); Schedule 1 as payg_withholding records",
        ),
        "voluntary_agreement_withholding" => (
            "2026-11-22",
            "Schedule 10 (voluntary agreements) of F2024L00664, F2025L00647, F2025L01016 and F2026L00716: using this schedule, how to work out the withholding amount, the CIR and GST rules, Francesca's example and rounding; TAA 1953 Sch 1 s 12-55 (ATO legal database)",
        ),
        "rd_tax_offset" => (
            "2026-11-22",
            "ITAA 1997 ss 355-100, 355-105, 355-115, 355-435, 355-440, 355-445, 355-450, 63-10(1) item 35, 67-30 and s 995-1 'corporate tax rate', and Income Tax Rates Act 1986 s 23(2) (ATO legal database); Act No. 92 of 2020 Sch 4 items 1 and 14 and Sch 5 items 29 and 56; ATO Rates of R&D tax incentive offset and Calculate the clawback adjustment (last updated 13 May 2026), its examples for Companies V, F, Y, Q, L and J; R&D tax incentive schedule 2022 instructions, Part E",
        ),
        "sb_cgt_concessions" => (
            "2026-11-19",
            "ITAA 1997 Division 152 (ss 152-5 to 152-35, 152-50, 152-55, 152-105, 152-110, 152-205 to 152-220, 152-305, 152-320, 152-325, 152-330, 152-410 and 152-430), s 104-190(1A) and s 328-110, compilation 264 (21 May 2026), the text for 2025-26 and 2026-27 under Act No. 49 of 2026 Schedule 1 item 82",
        ),
        "small_business_income_offset" => (
            "2026-11-22",
            "ITAA 1997 ss 328-110, 328-355 to 328-375, 4-10(3) and 63-10, compilation 268 (1 October 2026), against compilations 210 (1 July 2020; s 328-360(1) at 13%) and 223 (1 July 2021; 16%); the ATO's 'Small business income tax offset' page (last updated 7 May 2025)",
        ),
        "non_commercial_losses" => (
            "2026-11-22",
            "ITAA 1997 Division 35 (ss 35-10 to 35-55), compilation 268 (1 October 2026), against compilations 210 (1 July 2020) and 223 (1 July 2021), which read the same; the ATO's non-commercial loss pages 'Four tests', 'The income requirement', 'Excepted business activities' and 'How to defer your losses' (each last updated 7 April 2025)",
        ),
        "psi_test" => (
            "2026-11-22",
            "ITAA 1997 Part 2-42 (ss 84-5, 85-30, 85-35, 87-15 to 87-40, 87-60 and 87-65), compilation 268 (1 October 2026), against compilations 210 (1 July 2020) and 223 (1 July 2021), which read the same; TR 2022/3, examples 20, 29, 30 and 40 and paragraph 118; the site's PSI page and its 19 cases (checked 17 November 2026)",
        ),
        "rental_property" => (
            "2026-11-22",
            "ITAA 1997 ss 6-5, 8-1, 26-31, 26-102, 40-27, 43-10, 43-15, 43-20, 43-25, 43-80, 43-140, 43-145, 43-210, 43-215, 43-220, 43-230, 43-235 and 43-240 on the ATO's legal database, with the amendment history of compilation 264 (21 May 2026); Treasury Laws Amendment (Housing Tax Integrity) Act 2017 (No. 126, 2017) Schedule 1 and Schedule 2 item 13; TR 2000/2 paragraphs 13 to 26; TR 93/32 paragraphs 6 and 11 to 13; ATO Rental properties guide 2026 (NAT 1729-06.2026, current at 1 June 2026): the time-based method, Tables 1 and 2 and Examples 1, 2, 12, 20 to 23, 29, 33, 34 and 40; s 25-25 through borrowing-expenses-schedule (checked 19 November 2026) and the resident scale through resident-tax",
        ),
        "residency_tests" => (
            "2026-11-22",
            "ITAA 1936 s 6(1), 'resident or resident of Australia' paragraph (a), compilation 193 (1 October 2026), against compilation 169 (1 July 2020), which reads the same; TR 2023/1 paragraphs 3, 55, 63 to 65, 83 to 87 and footnote 51, 96, 105 to 108 and examples 5, 7, 10, 11, 13, 14, 15, 16 and 18",
        ),
        "lodgment_required" => (
            "2026-11-22",
            "Notices of Requirement to Lodge a Return for the Income Year Ended 30 June 2021 (F2021L00577), 2022 (F2022L00508) and 2023 (F2023L00533), Tables A, B, J, K and P and sections 3.3 and 4; Taxation Laws (Requirement to Lodge a Return for the 2024 Year) Instrument 2024 (F2024L00550), 2025 Year (F2025L00570) and 2026 Year (F2026L00618), ss 6(2), 6(3), 6(10), 7(1)(f), 7(2), 7(3) and 8(1); the 2024 explanatory statement, paragraph 17",
        ),
        "income_tests" => (
            "2026-11-16",
            "ITAA 1997 s 995-1 (income for surcharge purposes, reportable superannuation contributions, total net investment loss, adjusted taxable income for offsets, exempt foreign employment income, assessable FHSS released amount) and ss 4-15, 61-580, 291-15, 293-20, 301-20 and 313-20, compilation 268 (1 October 2026); Medicare Levy Act 1986 s 3(1), compilation 53 (1 July 2026); A New Tax System (Medicare Levy Surcharge—Fringe Benefits) Act 1999 s 3(1), compilation 42 (1 July 2026); ITAA 1936 s 6(1) (rebate income, adjusted taxable income for rebates, adjusted fringe benefits total) and s 251S, compilation 193 (1 October 2026); HESA ss 154-5, 154-16 and 154-17, compilation 105 (17 October 2026); A New Tax System (Family Assistance) Act 1999 Sch 3 cls 2, 4, 5, 7 and 8, compilation 124 (1 July 2026); FBTAA ss 135N, 135P and 135Q (1 July 2026); SISR reg 6.01(2) preservation age, compilation 159 (1 July 2026); Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 8, compilation 28 (1 July 2018); and the 1 July 2024 compilations of the ITAA 1997 (250), ITAA 1936 (184) and Family Assistance Act (119), compared; ATO Income tests (last updated 8 June 2026), Rebate income and Adjusted taxable income for you and your dependants for 2025 and 2026, IT1 Total reportable fringe benefits amounts 2025 and 2026, A3 Government super contributions 2025 and 2026, Medicare levy surcharge income, thresholds and rates (22 June 2026), When you must repay your loan (13 October 2026), Division 293 tax on concessional contributions by high-income earners (24 August 2026), Exempt income and other tax obligations (3 June 2026) and Fringe benefits tax - rates and thresholds (20 May 2026)",
        ),
        "government_super_amounts" => (
            "2026-11-22",
            "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 ss 6 to 11, 12C and 12E, compilation 28 (1 July 2018 to 30 June 2027); its Regulations 2004 reg 24(2), compilation 8, and Regulations 2022 reg 15(2); ITAA 1997 ss 290-230 and 290-235, compilation 268 (1 October 2026), compared with compilation 210 (1 July 2020); income-tests' definitions in A New Tax System (Family Assistance) Act 1999 Sch 3 and ITAA 1997 s 995-1 compared with the 1 July 2020 compilations (101 and 210); Treasury Laws Amendment (Building a Stronger and Fairer Super System) Act 2026 Sch 4; ATO Government contributions (Table 25, 27 April 2026), Super co-contribution and Low income super tax offset (2 August 2023), Superannuation-related tax offsets (8 June 2026), myTax 2026 Government super contributions (1 June 2026) and the FBT rates pages (20 May 2026 and 1 April 2026)",
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
        // No Library volume covers the PPSR; the Acts themselves were read.
        "ppsr_timing" => (
            "Personal Property Securities Act 2009, compilation 22; Corporations Act 2001, compilation 149",
            "PPSA ss 10, 62 and 153; Corporations Act ss 9 and 588FL",
            "2026-09-19",
            "2026-11-20",
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
        "dasp_tax" => (
            "superannuation-superannuation-benefits-termination-payments",
            "8-420",
            "2026-06-30",
            "2026-11-19",
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
        "franking_account" => (
            "ATO Franking account tax return 2026 instructions",
            "About example 2; section B examples 4 and 5; section C examples 7 to 9",
            "2026-05-30",
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
        "cgt_parcels" => (
            "tax-examples-capital-gains-tax-cgt",
            "2-240",
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
            "ATO study and training loan repayment thresholds and rates",
            "tables 1 and 2, examples 1 and 2",
            "2026-06-30",
            "2026-11-20",
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
        "private_health_insurance_rebate" => (
            "ATO Income thresholds and rates for the private health insurance rebate; Claiming the private health insurance rebate",
            "thresholds and rates for 2020-21 to 2026-27, examples (Narelle, Peta, Mike and Elle, Zoe and Charlie); examples (Toby, Donna)",
            "2026-06-22",
            "2026-11-22",
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
        "beneficiary_tax_offset" => (
            "Income Tax Assessment (1936 Act) Regulations 2025 s 13; ATO Beneficiary tax offset",
            "the formula and its rounding; no worked example is published",
            "2026-06-08",
            "2026-11-22",
        ),
        "invalid_carer_tax_offset" => (
            "ATO T5 Invalid and invalid carer 2021 to 2026",
            "the amounts, daily rates, FTB Part B limits and worksheets 1A to 3C",
            "2026-05-30",
            "2026-11-22",
        ),
        "zone_tax_offset" => (
            "ATO T4 Zone or overseas forces 2021 to 2026; Zone tax offset",
            "tables 1 to 3, worksheets 1 to 7, examples (Garry, Sharon, Narendra)",
            "2026-05-30",
            "2026-11-22",
        ),
        "net_medical_expenses_tax_offset" => (
            "ATO T5 Total net medical expenses for disability aids, attendant care or aged care 2017 to 2019",
            "tables 1 and 2: the phase-in limits and thresholds",
            "2019-05-30",
            "2026-11-22",
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
        "work_expenses" => (
            "ATO working from home, car and self-education pages; PCG 2023/1; TR 98/9",
            "Keisha, Yang and Wanda (fixed rate method), PCG 2023/1 examples 1 (Gerry) and 8 (Piruntha), Johan (cents per kilometre), Lana (log book), Maureen (self-education reduction) and TR 98/9 paras 145 to 155 (Con)",
            "2026-08-12",
            "2026-11-22",
        ),
        "depreciation_low_value_pool" => (
            "ATO Low-value pool page and Guide to depreciating assets 2026",
            "John's printer: $111 and $1,875, a $1,986 decline and a $3,608 closing balance; then $1,353, a $300 disposal and $1,955",
            "2025-06-26",
            "2026-11-22",
        ),
        "depreciation_small_business_pool" => (
            "ATO Small business pool calculations, s 328-210(3) and LCR 2021/3",
            "Loretta's 2018-19 pool ($30,000 and $4,200, closing $85,800), Cassidy's Gardens ($15,700 written off) and example 13 ($200,000 written off in 2020-21)",
            "2025-06-16",
            "2026-11-22",
        ),
        "capital_works" => (
            "ATO Rental properties guide 2026",
            "Example 29: Greg and Suzie's $115,800 at 2.5% for 346 days, $2,744",
            "2026-06-30",
            "2026-11-22",
        ),
        "rental_property" => (
            "ATO Rental properties guide 2026 (NAT 1729-06.2026)",
            "Examples 1, 2, 12, 20 to 23, 29, 33, 34 and 40 and Tables 1 and 2",
            "2026-05-30",
            "2026-11-22",
        ),
        "horticulture_shearing_withholding" => (
            "ATO Schedule 2 tax table for the horticultural or shearing industry (NAT 1013)",
            "Example: earnings of $231.50, $30, $108, $69 and $103 by residency and TFN",
            "2026-06-17",
            "2026-11-22",
        ),
        "upward_variation_withholding" => (
            "ATO Schedule 14 table of additional amounts to withhold (NAT 5441)",
            "Example: $563.00 weekly, $33 plus $50, $83 (2026-27); $39 plus $50 in the 2024 and 2025 schedules",
            "2026-06-17",
            "2026-11-22",
        ),
        "daily_casual_withholding" => (
            "ATO Tax table for daily and casual workers (NAT 1024)",
            "Example: $193.62 with $500 of tax offsets, $25 less $2 (2026-27); $26 less $2 from 1 July 2024",
            "2026-06-17",
            "2026-11-22",
        ),
        "no_abn_withholding" => (
            "ATO Rental expenses (Rental properties 2025) and Payments you don't withhold from",
            "Derek's $1,800 cash job, $846 withheld; Ana's $150 sign and $40 flowers",
            "2025-06-11",
            "2026-11-22",
        ),
        "entertainer_withholding" => (
            "ATO Schedule 3 tax table for actors, variety artists and other entertainers (NAT 1023)",
            "Sandra's 2 performances at $500.35, $42 each (2026-27), $44 in the 2024 and 2025 schedules; $279.35 a day, 3 performances and $500 of tax offsets, $17 ($18)",
            "2026-06-17",
            "2026-11-22",
        ),
        "return_to_work_withholding" => (
            "ATO Schedule 4 tax table for return to work payments",
            "George's $18,000 return to work payment, $5,760 withheld",
            "2026-06-17",
            "2026-11-22",
        ),
        "annuity_withholding" => (
            "ATO Schedule 6 tax table for annuities (NAT 1005)",
            "Barbara's $1,000 a week less $5,200 over 52, $106 on $900 (2026-27), $111 in the 2024 and 2025 schedules; Kenneth's $850 a fortnight less $1,000 over 10, $4 on $750",
            "2026-06-17",
            "2026-11-22",
        ),
        "voluntary_agreement_withholding" => (
            "ATO Schedule 10 tax table for payments made under voluntary agreements",
            "Francesca's CIR of 16.44% applied by agreement, otherwise 20%",
            "2026-06-17",
            "2026-11-22",
        ),
        "rd_tax_offset" => (
            "ATO Calculate the clawback adjustment",
            "Company F: $1,000,000 at 43.5% ($435,000), $500,000 reimbursed, adjusted offset $217,500, deduction amount $125,000, $370,000 included in assessable income",
            "2026-05-13",
            "2026-11-22",
        ),
        "sb_cgt_concessions" => (
            "tax-examples-capital-gains-tax-cgt",
            "2-280",
            "2026-06-30",
            "2026-11-19",
        ),
        "small_business_income_offset" => (
            "ATO Small business income tax offset page and the site's sole trader vectors",
            "the rate table (13% for 2020-21, 16% from 2021-22, $1,000 cap, $5 million turnover) and the offset as the proportion of the basic income tax liability",
            "2025-05-07",
            "2026-11-22",
        ),
        "non_commercial_losses" => (
            "ATO non-commercial loss pages",
            "Slaide's estimate, Nguyen's profits test, Joe's, Fred's and Alex's income requirement and Michael's deferred loss",
            "2025-04-07",
            "2026-11-22",
        ),
        "psi_test" => (
            "TR 2022/3",
            "Example 20 (Joe's ducts, the results test), 29 (Tom's 23.07%), 30 (Ian's 18.57%) and 40 (Chris, an agent on 70% commission)",
            "2022-08-10",
            "2026-11-22",
        ),
        "residency_tests" => (
            "TR 2023/1",
            "Examples 5 (Conchita), 7 (Mark), 10 (Brian), 11 (Stuart), 13 (Aiko), 14 (Alani), 15 (Grace), 16 (Megan) and 18 (Ryan); paragraph 86 and footnote 51 for the day count",
            "2023-06-07",
            "2026-11-22",
        ),
        "lodgment_required" => (
            "Each year's lodgment instrument",
            "The thresholds of s 6(2)(i) and (o), 6(3)(a), 6(10)(b), 7(1)(f)(ii), 7(2)(b) and 7(3)(b), and Tables A, B, J, K and P of the Notices",
            "2026-05-22",
            "2026-11-22",
        ),
        "income_tests" => (
            "ATO income test pages",
            "Christina's repayment income (When you must repay your loan) and Tom's income for MLS purposes (Medicare levy surcharge income, thresholds and rates)",
            "2026-10-13",
            "2026-11-16",
        ),
        "government_super_amounts" => (
            "ATO super pages",
            "Angelo (Super co-contribution), Julie (Low income super tax offset) and Examples 1 to 3 (Superannuation-related tax offsets)",
            "2026-06-08",
            "2026-11-22",
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
        "government_super_amounts" => vec![
            (
                "spouse_contributions",
                "AUD, with the other spouse inputs: the contributions you made in the income year to a super fund or RSA for your spouse that s 290-230 counts: made whilst married or de facto, both Australian residents and not living separately and apart on a permanent basis, to a complying fund, not deductible under s 290-60 and not a Family Law Act payment (ITAA 1997 s 290-230(1) to (4))",
            ),
            (
                "spouse_assessable_income",
                "AUD, with the other spouse inputs: the spouse's assessable income for the year, including any assessable FHSS released amount",
            ),
            (
                "spouse_assessable_fhss_released_amount",
                "AUD, with the other spouse inputs: the spouse's assessable FHSS released amount (s 290-230(2)(c)(i))",
            ),
            (
                "spouse_reportable_fringe_benefits_total",
                "AUD, with the other spouse inputs: the spouse's reportable fringe benefits total (s 290-230(2)(c)(ii))",
            ),
            (
                "spouse_reportable_employer_super_contributions",
                "AUD, with the other spouse inputs: the spouse's reportable employer superannuation contributions (s 290-230(2)(c)(iii))",
            ),
            (
                "spouse_excess_concessional_contributions",
                "AUD, with the other spouse inputs: the spouse's excess concessional contributions for the financial year (s 290-230(5))",
            ),
            (
                "spouse_non_concessional_within_cap",
                "boolean, or null if not established, with the other spouse inputs: the spouse's non-concessional contributions for the financial year do not exceed the spouse's cap (s 290-230(4A)(a)); null is refused where it decides the offset",
            ),
            (
                "spouse_total_super_balance_below_cap",
                "boolean, or null if not established, with the other spouse inputs: the spouse's total superannuation balance immediately before the start of the financial year is less than the general transfer balance cap (s 290-230(4A)(b)); null is refused where it decides the offset",
            ),
        ],
        "super_benefit_tax" => vec![
            (
                "date_of_birth",
                "YYYY-MM-DD, the member's, for recipient member only, and then required: sets the age on payment_date (an age is reached on the birthday; 29 February falls on 1 March in a common year) and the preservation age (SISR reg 6.01(2))",
            ),
            (
                "terminal_medical_condition",
                "boolean, for recipient member only, and then required: a terminal medical condition (s 303-10: two registered medical practitioners' certificates, one a specialist, within their 24 month certification periods) exists when the benefit is received or within 90 days after",
            ),
            (
                "disability_superannuation_benefit",
                "boolean, for recipient member only, and then required: true is refused, as s 307-145 increases a disability superannuation benefit's tax free component",
            ),
            (
                "service_before_july_1983",
                "boolean, required with an untaxed_element above nil and refused otherwise: true (the benefit is attributable to an interest that existed before 1 July 2007 and its service period began before 1 July 1983) is refused, as s 307-150 then moves part of the element untaxed in the fund to the tax free component",
            ),
            (
                "low_rate_cap_used",
                "AUD, for a member at or above preservation age and under 60 (possible only to 2023-24), and then required: the amounts counted towards the s 301-20(2) and 301-105(4) offsets for lump sums in earlier income years (each year's low_rate_cap_counted), which reduce the cap for later years (s 307-345(2)); at most the year's low rate cap",
            ),
            (
                "untaxed_plan_cap_used",
                "AUD, for a member benefit with an untaxed_element above nil (a wholly tax free one too, as s 307-350 still reduces the cap), and then required: the total by which earlier lump sums from the same plan reduced the untaxed plan cap (each one's untaxed_plan_cap_counted: its element untaxed in the fund up to the cap left then, as s 307-350(2)(b) reduces the cap to nil rather than counting an excess), in this and earlier years; at most the year's untaxed plan cap",
            ),
            (
                "other_lump_sums_this_year",
                "boolean, for a member at or above preservation age and under 60, and then required: true (the member receives another superannuation lump sum in the same income year in that band) is refused while any of the low rate cap is left, as ss 301-20(3) and 301-105(5) and (6) share the year's low rate cap across them, the element taxed in the fund first",
            ),
        ],
        "upward_variation_withholding" | "annuity_withholding" => {
            optional_inputs("payg_withholding")
        }
        "entertainer_withholding" => vec![(
            "tax_offsets",
            "AUD, with a TFN and the tax-free threshold only, and one performance a day: the yearly total the performer claimed on their Withholding declaration",
        )],
        "voluntary_agreement_withholding" => vec![(
            "instalment_rate",
            "decimal string from 0 to 100: the payee's Commissioner's instalment rate as a percentage, absent when they have told you they have none",
        )],
        "daily_casual_withholding" => vec![(
            "tax_offsets",
            "AUD, with a TFN only: the yearly total the payee claimed at the tax offsets questions on their Withholding declaration; the daily value is 0.38% of it",
        )],
        "fuel_tax_credits" => vec![
            (
                "small_claimant",
                "boolean, required with a basic_heavy_vehicle or litres_from_cost line or rate_at_period_end and refused otherwise: true states that the fuel tax credit entitlement for the previous 12 months was $10,000 or less (for the basic method alone, PCG 2021/2 footnote 3 also accepts no claim in those 12 months) and is reasonably expected to be $10,000 or less for the current 12 months (PCG 2016/2 para 4; PCG 2021/2 para 6); false is refused",
            ),
            (
                "rate_at_period_end",
                "object with start and end (YYYY-MM-DD), a tax period inside the income year that holds every line's acquisition: PCG 2016/2 Method 2 takes every line at the rate in force on end if indexation changes an applicable rate (para 11). If the applicable rates stay flat, the worksheet uses actual acquisition dates. Refused across a rate change other than indexation alone, when end is after the last day a rate is held for, and for lpg or lng_cng lines",
            ),
            (
                "return_date",
                "YYYY-MM-DD, the day the return taking these credits into account is given, not before any line's acquisition: lines acquired more than 4 years before it are listed in time_limit_lines, with the s 47-5(2) warning",
            ),
        ],
        "work_expenses" => vec![
            (
                "hours",
                "hours, at most 2 decimal places, for home_office only, and then required: the actual hours worked from home in the year, from a record kept at the time (PCG 2023/1 paras 56 to 59), at most 24 x the days in the year",
            ),
            (
                "energy",
                "AUD, for home_office only, and then required: the work-related portion of electricity and gas for heating, cooling, lighting and electronic items used whilst working from home; the 52c rate before 2022-23 is taken to cover all of it, as the ATO lists no energy claimed beside that rate",
            ),
            (
                "phone",
                "AUD, for home_office only, and then required: the work-related portion of mobile and home phone usage",
            ),
            (
                "internet",
                "AUD, for home_office only, and then required: the work-related portion of home and mobile internet and data",
            ),
            (
                "stationery",
                "AUD, for home_office only, and then required: the work-related portion of stationery and computer consumables",
            ),
            (
                "cleaning",
                "AUD, for home_office only, and then required: the work-related portion of cleaning a dedicated home office",
            ),
            (
                "furniture_decline",
                "AUD, for home_office only, and then required: the work-related decline in value (or immediate deduction at $300 or less) of home office furniture and furnishings",
            ),
            (
                "asset_decline",
                "AUD, for home_office only, and then required: the work-related decline in value of other depreciating assets used whilst working from home, such as computers and phones",
            ),
            (
                "repairs",
                "AUD, for home_office only, and then required: the work-related portion of repairs to those depreciating assets",
            ),
            (
                "running_expenses_incurred",
                "boolean, for home_office only, and then required: the taxpayer paid or became liable for at least one additional running expense as a result of working from home, which every hourly rate needs; from 2022-23 it must be energy, phone, internet, stationery or computer consumables, the kinds the fixed rate covers (PCG 2023/1 paras 14, 18(b) and 49 to 55), and before then any running expense (the ATO's 52 cent and shortcut pages); false leaves only the actual cost, and conflicts with an amount for such an expense",
            ),
            (
                "dedicated_home_office",
                "boolean, for home_office in 2020-21 or 2021-22 only, and then required: the taxpayer had a dedicated home office, which the 52c fixed rate needs",
            ),
            (
                "owned_or_leased",
                "boolean, for car_cents_per_kilometre and car_log_book only, and then required: the taxpayer owned or leased the car (ITAA 1997 s 28-12(1)); false is refused",
            ),
            (
                "business_kilometres",
                "kilometres, at most 2 decimal places, for car_cents_per_kilometre and car_log_book only, and then required: the reasonable estimate of business kilometres the car travelled in the year, or for car_log_book in the period held in the year (ss 28-25(3) and 28-90(5))",
            ),
            (
                "total_kilometres",
                "kilometres above 0, for car_log_book only, and then required: the total kilometres the car travelled in the period held in the year, from odometer records (s 28-90(3))",
            ),
            (
                "car_expenses",
                "AUD, for car_log_book only, and then required: the car expenses for the period held in the year that are deductible outside Division 28, decline in value included (s 28-90(2))",
            ),
            (
                "log_book_year",
                "income year YYYY-YY, for car_log_book only, and then required: the year the log book was kept for, the year claimed or one of the 4 before it (s 28-115(2))",
            ),
            (
                "log_book_weeks",
                "integer, for car_log_book only, and then required: the continuous weeks the log book covers, at least 12 (s 28-120(1))",
            ),
            (
                "log_book_representative",
                "boolean, for car_log_book only, and then required: the log book still represents the car's business use in the year claimed; false, as after a change of job or home, is refused (ATO 'Logbook method')",
            ),
            (
                "connection",
                "for self_education only, and then required: maintains_or_improves_skills, increases_income, general_only or new_employment, the self-education's connection to current employment when incurred (TR 2024/3)",
            ),
            (
                "employed_when_incurred",
                "boolean, for self_education only, and then required: the taxpayer was employed when the expenses were incurred",
            ),
            (
                "prescribed_course",
                "boolean, for self_education in 2020-21 or 2021-22 only, and then required: the expenses were for a course of education at a school, college, university or other place of education, which ITAA 1936 s 82A reduces; a short-term refresher or staff development course, seminar, conference or workshop is not one (TR 98/9 para 129)",
            ),
            (
                "category_a",
                "AUD, for self_education only, and then required: category A, deductible general expenses such as fees, textbooks, stationery, fares and log book car expenses other than decline in value",
            ),
            (
                "category_b",
                "AUD, for self_education only, and then required: category B, decline in value of depreciating assets used for the self-education",
            ),
            (
                "category_c",
                "AUD, for self_education only, and then required: category C, repairs to assets used for the self-education, not car repairs",
            ),
            (
                "category_d",
                "AUD, for self_education only, and then required: category D, car expenses by the cents per kilometre method",
            ),
            (
                "log_book_car_in_category_a",
                "boolean, for self_education with category_d above 0 only, and then required: category_a includes log book car expenses; true is refused, as a car's expenses take one method",
            ),
            (
                "category_e",
                "AUD, for self_education with prescribed_course true only, and then required: category E, expenses incurred for the course that are not deductible, such as child care for attendance or the capital cost of a computer",
            ),
        ],
        "ato_interest_penalties" => vec![
            (
                "amount",
                "AUD: gic, the unpaid amount; sic, the additional income tax from the amendment; early_payment_interest, the payment (no more than the amount due)",
            ),
            (
                "due_date",
                "YYYY-MM-DD: gic, the day the amount was due, the first day of GIC (ITAA 1997 s 5-15); sic, the day the first assessment's tax was due (or the earlier amended assessment's for a reinstated liability, s 280-100(3)); failure_to_lodge, the day the document was due (the deferred day if deferred); early_payment_interest, the day the amount is due and payable",
            ),
            (
                "to",
                "YYYY-MM-DD, gic only and required: the last day of GIC, the day before payment in full or an as-at day",
            ),
            (
                "payments",
                "gic: optional list of objects with date (YYYY-MM-DD, from due_date to to) and amount (AUD), each applied before that day's charge; overpayment_interest: required list of objects with paid_on and amount, each instalment of the relevant tax paid",
            ),
            (
                "assumed_annual_rate",
                "percentage above 0 and at most 100, gic only: the annual rate for days after 31 December 2026, the last published quarter; without it such days are refused",
            ),
            (
                "notice_date",
                "YYYY-MM-DD, sic only and required: the day the Commissioner gave notice of the amended assessment; SIC runs to the day before",
            ),
            (
                "given_on",
                "YYYY-MM-DD, failure_to_lodge only and required: the day the document was given, or an as-at day for one not yet given (s 286-80(6))",
            ),
            (
                "size",
                "failure_to_lodge only and required: small, medium (x2, s 286-80(3)), large (x5, s 286-80(4)) or significant_global (x500, s 286-80(4A) or (4C)), as established",
            ),
            (
                "statement_date",
                "YYYY-MM-DD, statement_penalty only and required: the day the statement was made; it sets the penalty unit for a statement with no shortfall amount",
            ),
            (
                "significant_global_entity",
                "boolean, statement_penalty only and required: true when s 284-90(1A) doubles the base penalty amount, as established",
            ),
            (
                "trust_or_partnership",
                "boolean, statement_penalty only: true for a trust or partnership, whose not reasonably arguable positions (items 5 and 6) are refused",
            ),
            (
                "other_disclosed_before_examination",
                "AUD, statement_penalty with a part with a shortfall_amount disclosed before_examination only, and then required (0.00 when none): the shortfall amounts for the same accounting period disclosed before an examination in other statements, added to these parts' for s 284-225(3) (MT 2012/3 para 35)",
            ),
            (
                "income_tax_payable",
                "AUD, statement_penalty with a not_reasonably_arguable part only, and then required: the income tax payable for the year on the basis of the return (s 284-90(3)(a))",
            ),
            (
                "parts",
                "list of 1 to 100 objects, statement_penalty only and required: the matters in one statement for one accounting period, whose before-examination disclosures are added for s 284-225(3) (a statement for another period is its own question), one matter each, with shortfall_amount (AUD, or null for a false or misleading statement with no shortfall amount), behaviour (intentional_disregard, recklessness, failure_to_take_reasonable_care or reasonable_care) and optionally not_reasonably_arguable (boolean), accepted_way_amount (AUD, the part of the shortfall from treating the law in an accepted way, s 284-224), increase (boolean, a s 284-220 ground) and disclosure (none, before_examination or during_examination, the latter for one that saved significant time or resources, s 284-225)",
            ),
            (
                "liability",
                "early_payment_interest only and required: income_tax, shortfall_interest_charge, compulsory_study_loan_repayment or fs_assessment_debt (Overpayments Act s 8A(1)(a))",
            ),
            (
                "paid_on",
                "YYYY-MM-DD, early_payment_interest only and required: the day the payment was made",
            ),
            (
                "full_self_assessment",
                "boolean, early_payment_interest only and required: true for a full self-assessment taxpayer (ITAA 1936 s 6(1)), such as a company",
            ),
            (
                "notice_issued",
                "YYYY-MM-DD: early_payment_interest when full_self_assessment is false, and then required, the day the notice of the amount was issued; overpayment_interest, required, the day the notice of the assessment, determination or decision the decision was made about was issued (s 10(1)(a)(i))",
            ),
            (
                "refunded",
                "AUD, overpayment_interest only and required: the one amount refunded or applied as a result of the decision, attributed to the payments in reverse order, no more than the payments (a later refund of the same payments is not worked)",
            ),
            (
                "refunded_on",
                "YYYY-MM-DD, overpayment_interest only and required: the day it was refunded or applied",
            ),
            (
                "relevant_tax",
                "boolean, overpayment_interest only and required: true when the amount paid is relevant tax (Overpayments Act s 3C); false is refused",
            ),
            (
                "decision_to_which_act_applies",
                "boolean, overpayment_interest only and required: true when the overpayment results from a decision to which the Act applies (s 3(1)); false is refused",
            ),
        ],
        "sb_cgt_concessions" => vec![
            (
                "small_business_when_stopped",
                "boolean, for business winding_up only, and then required: the business entity was a small business entity for the income year in which it stopped carrying on the business, with $2 million in place of each $10 million in s 328-110 (ITAA 1997 ss 152-10(1AA)(b) and 328-110(5)(b))",
            ),
            (
                "longer_period_allowed",
                "boolean, with a business_ceased date only, and then required: the Commissioner allows longer than 12 months after the cessation (s 152-35(2)(b)(ii))",
            ),
            (
                "carries_on_own_business",
                "boolean, for a passively held asset only, and then required: you carry on a business in the year other than in partnership (s 152-10(1A)(b) and (1B)(c))",
            ),
            (
                "significant_individual_before_event",
                "boolean, for a company or trust only, and then required: it had a significant individual just before the CGT event (ss 152-50 and 152-55)",
            ),
            (
                "significant_individual_15_years",
                "boolean, for a company or trust only, and then required: it had a significant individual for a total of at least 15 years during which it owned the asset (s 152-110(1)(c))",
            ),
            (
                "retirement_exemption_used",
                "AUD, for an individual only, and then required: the CGT exempt amounts of the individual's earlier choices of the retirement exemption (s 152-320(1))",
            ),
            (
                "choice_date",
                "YYYY-MM-DD, for an individual only, and then required: when the individual chooses the retirement exemption, on or after the CGT event (s 152-305(1)(b))",
            ),
        ],
        "small_business_income_offset" => vec![
            (
                "business",
                "carried_on or winding_up, with net_small_business_income above nil only, and then required: your business in the income year (s 328-110(1)(a) and (5))",
            ),
            (
                "small_business_when_stopped",
                "boolean, for business winding_up only, and then required: you were a small business entity, at $5 million, for the income year in which you stopped carrying on the business (s 328-110(5)(b))",
            ),
            (
                "previous_year_turnover",
                "AUD, or null for no business carried on, with net_small_business_income above nil only, and then required: your aggregated turnover for the previous income year (s 328-110(1)(b)(i))",
            ),
            (
                "year_before_turnover",
                "AUD, or null for no business carried on, with net_small_business_income above nil only, and then required: your aggregated turnover for the income year before that (s 328-110(3))",
            ),
            (
                "likely_turnover",
                "AUD, with net_small_business_income above nil only, and then required: your likely aggregated turnover for the year, worked out as at its first day, or as at the day you started the business if that was during the year (s 328-110(1)(b)(ii) and (2))",
            ),
            (
                "year_turnover",
                "AUD, with net_small_business_income above nil only, and then required: your aggregated turnover for the year, worked out at its end (s 328-110(4))",
            ),
        ],
        "non_commercial_losses" => vec![
            (
                "assessable_income_estimate",
                "AUD, with part_year true only, and then required: a reasonable estimate of the activity's assessable income had it been carried on for the whole year (s 35-30(b))",
            ),
            (
                "other_income",
                "AUD, for a primary_production or professional_arts activity only, and then required: your assessable income for the year from other sources not related to the activity, except net capital gain (s 35-10(4)(b))",
            ),
        ],
        "psi_test" => vec![
            (
                "commission",
                "AUD, with agent true only, and then required: the part of the principal's payments to you that is commissions or fees based on your performance (s 87-40(2)(c))",
            ),
            (
                "principal_payments",
                "AUD, with agent true only, and then required: the principal's payments to you for services to its customers (s 87-40(2)(b))",
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
                "improved_on_1_july_2000",
                "boolean, required for later registration under item 2: true if the land or premises had improvements on 1 July 2000. Overlapping items 2 and 3 need separate review",
            ),
            (
                "registration_effective_day",
                "YYYY-MM-DD date, required for later registration under item 2: the date your GST registration took effect",
            ),
            (
                "applied_for_registration",
                "boolean, required for later registration under item 2: true if you applied for GST registration, false if you did not apply",
            ),
            (
                "registration_application_day",
                "YYYY-MM-DD date, required under item 2 when applied_for_registration is true: the day you applied for GST registration, which may be on or before 1 July 2000",
            ),
            (
                "supply_day",
                "YYYY-MM-DD date, required for government land under item 4: the day the taxable supply took place, in year",
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
        "franking_account" => vec![
            (
                "income_tax",
                "object, to apply the offset, and required for a private company whose offset reduction would otherwise apply: before_fdt_offset (AUD, the income tax for the year after every tax offset s 63-10 applies before this one, for s 205-70(5)(b) and (d)), offset_brought_forward (AUD, the offset left after s 63-10 the last year the entity was entitled to one, s 205-70(1)(c)) and, for a private company, earlier_year_liability (boolean, true when it had an income tax liability for an earlier income year, s 205-70(5)(c))",
            ),
            (
                "commissioner_determination",
                "boolean, true when the Commissioner has determined in writing under s 205-70(6) that the excess was due to events outside the entity's control, so the offset is not reduced",
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
        "rental_property" => vec![(
            "other_taxable_income",
            "AUD, whole dollars, 2024-25 to 2026-27 only: the taxable income apart from this property, for the tax at the resident scale without and with the net rent",
        )],
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
        "private_health_insurance_rebate" => vec![
            (
                "spouse_income_for_surcharge_purposes",
                "AUD, the income for surcharge purposes of the person's spouse, for a person married on 30 June (PHI Act s 22-30(1)(a) and (2); the ATO counts a spouse who died in the year); tests the combined income against the family thresholds; omit for a person without a spouse on 30 June",
            ),
            (
                "dependent_children",
                "integer from 0, the dependants who are children (Fringe Benefits Act meaning: under 21, or a full-time student under 25, maintained by the person); each after the first raises the family thresholds by $1,500 (s 22-40(4)); only with spouse_income_for_surcharge_purposes or maintained_dependent_person true; omit for none",
            ),
            (
                "maintained_dependent_person",
                "boolean, true when on any day in the year the person contributed in a substantial way to the maintenance of a dependent person (PHI Act Schedule 1: a person under 18 without a partner; a person aged 18 to 31 without a partner who is either in full-time education and a dependent student under the insurer's rules, or not in full-time education and a dependent non-student under the insurer's rules; or a person 18 or over, partnered or not, with a disability as the Private Health Insurance (Complying Product) Rules or the insurer's rules define it) who is their child or a sibling dependent on them; puts a person without a spouse on 30 June on the family thresholds (s 22-30(1)(b)); omit or false otherwise",
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
                "Required for commercial_parking_station, market_value, average_cost and spaces. AUD net of reimbursements. For the first 3 methods, give the amount for one benefit, or a total no greater than one benefit's value. For a larger combined amount, give an object with capped_total (decimal string, up to 3 decimal places to retain half-cents in average cost), the total after limiting each payment to its own benefit's value (FBTAA ss 39C, 39D and 39DA). For spaces give every relevant recipients contribution for the employees the election covers, for the FBT year (s 39FE)",
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
        "invalid_carer_tax_offset" => vec![
            (
                "days",
                "integer from 0 to the days in the year: the days that count for part of the year (ITAA 1997 s 61-40): for a spouse, days the spouse was an invalid or carer when neither of you was eligible for FTB Part B or received parental leave pay; for another dependant, days you maintained them while they qualified; omit for the whole year",
            ),
            (
                "spouse_adjusted_taxable_income",
                "AUD, the adjusted taxable income for offsets of a spouse who is not the dependant, added for the income limit (s 61-20(1)(b)); not taken when dependant is spouse; omit for no spouse",
            ),
            (
                "spouse_days",
                "integer, with spouse_adjusted_taxable_income only: the days you had that spouse, which scale their income (s 61-20(2)); omit for the whole year",
            ),
            (
                "maintained_by_others",
                "boolean: another person also contributed to the dependant's maintenance; true is refused (s 61-40(1)(b))",
            ),
            (
                "ftb_b_shared_care",
                "boolean: you or your spouse were eligible for FTB Part B at a shared-care rate while the spouse was the dependant; true is refused (s 61-35)",
            ),
            (
                "different_spouses",
                "boolean: you had a different spouse during different parts of the year; true is refused (s 61-20(3))",
            ),
        ],
        "zone_tax_offset" => vec![
            (
                "prior_year",
                "income year string, zones only, with prior_days: the immediately preceding year (ITAA 1936 s 79A(3B)(d)) or a relevant preceding year 2 to 4 years before (s 79A(3B)(e)) in which you were not a resident of the area",
            ),
            (
                "prior_days",
                "integer, with prior_year: the days your usual place of residence was in the area in prior_year, not more than one-half of it, without days in a period s 23AB(8) or 79B(3) applied to",
            ),
            (
                "continuous_residence",
                "boolean, with a relevant preceding year only: your usual place of residence was in the area continuously from the start of prior_days' period to the end of this year's (s 79A(3B)(e)(v))",
            ),
            (
                "includes_first_day",
                "boolean, with a relevant preceding year only: this year's period in the area included 1 July (s 79A(3B)(e)(i))",
            ),
            (
                "dependants",
                "list of up to 50 objects, each with kind (\"student\": full-time, under 25; \"child\": not a student, under 21), adjusted_taxable_income (AUD decimal string, for the period you maintained them) and optional days (integer, days you maintained them while they qualified; omit for the whole year): your dependants under ITAA 1997 Subdivision 961-A, non-student children oldest first; a dependant also maintained by another person is refused",
            ),
            (
                "sole_parent_days",
                "integer: the days you had sole care of a dependant listed in dependants, with no spouse in the year (ITAA 1997 s 961-55)",
            ),
            (
                "invalid_carer_tax_offset",
                "AUD: the invalid and invalid carer tax offset you are entitled to for the year (ITAA 1997 Subdivision 61-A), as invalid-carer-tax-offset works it out",
            ),
            (
                "remote_area_allowance",
                "AUD, zones only: the remote area allowance paid to you for the year under the Social Security Act 1991 or the Veterans' Entitlements Act 1986 (s 79A(2A))",
            ),
        ],
        "net_medical_expenses_tax_offset" => vec![
            (
                "spouse_adjusted_taxable_income",
                "AUD, for a person married on 30 June, or on the day the person died for one who died in the year (s 159Q(2)): the spouse's adjusted taxable income for rebates, added and tested against the family threshold (ITAA 1936 s 159Q(1) item 1)",
            ),
            (
                "dependent_children",
                "integer from 0: dependent children (A New Tax System (Medicare Levy Surcharge - Fringe Benefits) Act 1999 meaning) on any day; one or more puts a person on the family threshold, raised by $1,500 for each after the first",
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
        "ppsr_timing" => vec![
            (
                "registration_time",
                "YYYY-MM-DDTHH:MM, ACT legal time (PPSA s 294): for registration, required, the registration time; for pmsi and company_grantor, optional, a registration time to test",
            ),
            (
                "collateral",
                "registration only and required: consumer or commercial (PPSA s 153 item 4(a))",
            ),
            (
                "serial_numbered",
                "boolean, registration only and required: true when the collateral is described by serial number",
            ),
            (
                "end_date",
                "YYYY-MM-DD, registration only: the requested end date (the registration ends at the end of that day); absent for no stated end time",
            ),
            (
                "inventory",
                "boolean, pmsi only and required: true when the collateral is inventory (PPSA s 10)",
            ),
            (
                "goods",
                "boolean, pmsi only and required: true when the collateral is goods",
            ),
            (
                "trigger_time",
                "YYYY-MM-DDTHH:MM, pmsi only and required: for goods, when the grantor, or another person at its request, obtained possession; otherwise when the interest attached (PPSA s 62(2)(b) and (3)(b))",
            ),
            (
                "excluded_days",
                "list of YYYY-MM-DD, pmsi on collateral that is not inventory, and then required: every day from the trigger to the deadline that is a public holiday for the whole of any State, the ACT or the NT, or a day the Registrar refused access to or suspended the register (PPSA s 10 'business day' (b) and (d)); weekends and the days between Christmas Day and New Year's Day are excluded without listing",
            ),
            (
                "pmsi_stated",
                "boolean, pmsi with registration_time and then required: true when the registration states that the interest is a purchase money security interest (PPSA s 153 item 7)",
            ),
            (
                "agreement_in_force_time",
                "YYYY-MM-DDTHH:MM, company_grantor only and required: when the security agreement came into force (Corporations Act s 588FL(2)(b)(ii))",
            ),
            (
                "place_holidays",
                "list of YYYY-MM-DD, company_grantor only and required: every public or bank holiday in the place concerned within the count (Corporations Act s 9 'business day')",
            ),
            (
                "foreign_law_agreement",
                "boolean, company_grantor only and required: true when the agreement came into force under the law of a foreign jurisdiction (s 588FL(2)(b)(iii)), which is refused",
            ),
            (
                "critical_time",
                "YYYY-MM-DDTHH:MM, company_grantor only: the critical time (s 588FL(7)), from 1 July 2025 to 30 June 2027",
            ),
            (
                "court_ordered_time",
                "YYYY-MM-DDTHH:MM, company_grantor with critical_time only: a later time the Court ordered (ss 588FL(2)(b)(iv) and 588FM)",
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
        "fhss" => vec![
            (
                "contributions",
                "list of objects, to estimate the determination; give this or determination: date (YYYY-MM-DD, from 1 July 2017 to the determination day), amount (AUD decimal string) and kind, employer_voluntary (salary sacrifice or another employer contribution that is not mandated: concessional) or personal (made by you: non-concessional unless deducted)",
            ),
            (
                "determination_requested",
                "YYYY-MM-DD, with contributions and then required: the day the determination was requested; before 1 July 2022 the limit in all is $30,000, otherwise $50,000 (GN 2024/1 section 3)",
            ),
            (
                "personal_deductions",
                "list of objects, with contributions: financial_year (such as \"2024-25\") and amount (AUD), the personal contributions deducted for that year, which makes them concessional; the year's latest personal contributions are taken to be the deducted ones (s 138-30(3)(b))",
            ),
            (
                "determination",
                "object, the determination's stated components; give this or contributions: concessional (the 85% amount it states), non_concessional and associated_earnings (AUD)",
            ),
            (
                "amount_requested",
                "AUD, the amount requested for release; the FHSS maximum release amount when left out, and never more (s 131-10(1) item 4)",
            ),
            (
                "estimated_marginal_rate",
                "fraction from 0 to 0.45, the marginal rate the Commissioner estimates; withholding is it plus 2% Medicare levy less 30%, not below nil (reg 53A(1)(a); GN 2024/1 section 7); required unless commissioner_cannot_estimate is true",
            ),
            (
                "commissioner_cannot_estimate",
                "boolean: true when the Commissioner cannot estimate the tax, so 17% is withheld (reg 53A(1)(b)); not with estimated_marginal_rate",
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
        "rental_property" => vec![
            ("ownership_share", "decimal fraction above 0 and at most 1: your legal interest in the property (TR 93/32); income, expenses, decline in value and construction expenditure are the whole property's and are multiplied by it"),
            ("acquired", "YYYY-MM-DD: when you acquired the property, on or before the last day of the year"),
            ("disposed", "YYYY-MM-DD or null: when you stopped holding it, in the year"),
            ("days_available", "integer: the days in the year you held it that it was let, or available for rent on commercial terms (Rental properties guide 2026, 'Time-based method')"),
            ("section_26_50_applies", "boolean: s 26-50 denies the costs of owning it as a holiday home; true is refused"),
            ("income", "object of AUD amounts, the whole property's: rental_income and other_rental_income (ITAA 1997 s 6-5)"),
            ("expenses", "object of AUD amounts, the whole property's, for expenses relating to it for the whole period held, which the time-based share apportions, by line: advertising_for_tenants, body_corporate_fees, cleaning, council_rates, gardening, insurance, land_tax, legal_expenses, pest_control, property_agent_fees, repairs_and_maintenance, stationery_telephone_postage, travel (denied, s 26-31), water_charges and sundry"),
            ("rental_only_expenses", "object of AUD amounts by the same lines, for expenses relating solely to renting (agent commission, advertising for tenants), which are not apportioned"),
            ("loans", "list of objects, your own share of each loan, each with interest (AUD charged in the year), principal_for_rental and principal_total (AUD, the outstanding principal used for the rental and in total, TR 2000/2 paragraph 16) and redrawn_in_year (boolean; true is refused)"),
            ("borrowing_loans", "list of objects as borrowing-expenses-schedule takes loans, your own borrowing expenses only; empty for none"),
            ("capital_works", "list of objects, the whole property's, each with construction_expenditure (AUD, given, never derived), kind (building or structural_improvement), began and first_used (YYYY-MM-DD) and taken_to_begin_before_16_september_1987 (boolean, s 43-220(1)(a) or (b); required for a building begun after 15 September 1987)"),
            ("depreciating_assets", "list of objects, the whole property's, each with decline_in_value (AUD from its start time to the end of the days held, before any private use reduction), acquired (YYYY-MM-DD, when you acquired it or entered into the contract under which you did, if earlier: Act No. 126 of 2017 Schedule 2 item 13), first_used (YYYY-MM-DD, its start time, first used or installed ready for use), held_when_first_used, residence_or_private_use and low_value_pool (booleans), and new_premises_exception (boolean, for an asset not held when first used) and deductible_in_2016_17 (boolean, for an asset acquired before 9 May 2017) where needed (s 40-27)"),
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
        "super_benefit_tax" => vec![
            (
                "benefit",
                "AUD, the superannuation lump sum paid to the recipient, not rolled over",
            ),
            (
                "interest_value",
                "AUD, the value of the superannuation interest it is paid from (s 307-205) just before it is paid (s 307-125(3)(b)); at least the benefit. A commutation of an income stream, valued when the stream commenced (s 307-125(3)(c)), is outside the command",
            ),
            (
                "interest_tax_free_component",
                "AUD, the interest's tax free component (s 307-210: its contributions segment and crystallised segment) at the same time",
            ),
            (
                "untaxed_element",
                "AUD, the element untaxed in the fund of the benefit's taxable component as the fund states it under ss 307-275 to 307-290; 0.00 for a taxed fund that claims no s 295-465 or 295-470 deduction for a death benefit",
            ),
            (
                "recipient",
                "member, death_benefits_dependant (s 302-195, including any recipient where the deceased died in the line of duty) or non_dependant",
            ),
            (
                "payment_date",
                "YYYY-MM-DD, the day the recipient receives the benefit, in the income year given",
            ),
        ],
        "dasp_tax" => vec![
            (
                "tax_free_component",
                "AUD, the payment's tax free component, as the paying fund states it",
            ),
            (
                "taxed_element",
                "AUD, the element taxed in the fund of the payment's taxable component",
            ),
            (
                "untaxed_element",
                "AUD, the element untaxed in the fund of the payment's taxable component, 0.00 for most funds",
            ),
            (
                "working_holiday_maker_contributions",
                "boolean, true when the payment includes amounts attributable to superannuation contributions made whilst the person was a working holiday maker: the holder of a subclass 417 (Working Holiday) or 462 (Work and Holiday) visa or an associated bridging visa (DASP Tax Act s 5(3); Income Tax Rates Act 1986 s 3A). The 65% rate then applies to the whole taxable component, whatever visa the person held when other contributions were made",
            ),
            (
                "unclaimed_money_rollover",
                "boolean, true when the payment is a roll-over superannuation benefit paid under s 20H(2), (2AA) or (2A) of the Superannuation (Unclaimed Money and Lost Members) Act 1999; true with an element untaxed in the fund is refused, as s 5(2) taxes any excess untaxed roll-over amount in it at nil",
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
        "franking_account" => vec![
            (
                "private_company",
                "boolean, true when the entity is a private company for the income year (ITAA 1997 s 995-1(1)): its franking period is then the income year (s 203-45)",
            ),
            (
                "benchmark_rule_exempt",
                "boolean, true when s 203-20 exempts the entity from the benchmark rule for the year's franking periods: a listed public company meeting s 203-20(1)(a) throughout, or its 100% subsidiary; false for a private company",
            ),
            (
                "corporate_tax_rate",
                "decimal string, 0.25 or 0.30: the entity's corporate tax rate for imputation purposes for the income year",
            ),
            (
                "opening_balance",
                "AUD, may be negative: the franking account balance at the end of the previous income year, a deficit as a negative amount, before the credit for that year's franking deficit tax",
            ),
            (
                "rd_deferred_debit_brought_forward",
                "AUD, 0.00 when none: the parts of refunds of income tax for earlier income years attributable to a refundable R&D tax offset (s 67-30), less the reductions of earlier credits under s 205-15(4)",
            ),
            (
                "last_benchmark",
                "null when no frankable distribution was made in a franking period before the income year, or an object with percentage (decimal string from 0 to 100, the benchmark franking percentage of the last such period) and periods_since (whole number from 1: the franking periods from the one after it to the income year's first, counted, for s 204-70)",
            ),
            (
                "events",
                "list of at most 10000 objects, each with date (YYYY-MM-DD in the income year; for income_tax_refund up to 3 months after it, s 205-50(2)), type and amount (AUD above nil, the credit or debit the provision gives, or for payg_instalment and income_tax_paid the payment): credits payg_instalment (s 205-15(1) item 1), income_tax_paid (item 2), franked_distribution_received (item 3: the franking credit), franked_distribution_flowed_indirectly (item 4: the share of the franking credit, dated the end of the last interposed entity's income year), tax_offset_refund_excess_paid (item 4A), exploration_credit (item 7), dmt_paid (item 9); debits income_tax_refund (s 205-30(1) item 2, with rd_offset_part, AUD from 0.00 to the amount: the part attributable to a refundable R&D tax offset), linked_distribution (item 5), tax_exempt_bonus_shares (item 6), streaming_determination (item 7), share_capital_tainting (item 7A), untainting_choice (item 7B), on_market_buy_back (item 9), off_market_buy_back (item 9A), selective_capital_reduction (item 9B), dmt_refund (item 14)",
            ),
            (
                "distributions",
                "list of at most 10000 objects in date order, one for each frankable distribution the entity made in the income year, each with date (YYYY-MM-DD), amount (AUD above nil, the frankable distribution) and franking_credit (AUD, the franking credit its distribution statement allocates), and optional permitted_percentage (decimal string from 0 to 100: a franking percentage the Commissioner permitted for it under s 203-55; not for the first distribution in its franking period) and statement_after_distribution (boolean, a private company's only: the statement was given after the distribution, s 202-75(3))",
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
        "cgt_parcels" => vec![
            (
                "entity",
                "individual, trust, complying_superannuation_entity or company (not a life insurance company): the entity that made the gains (ITAA 1997 ss 115-10 and 115-100)",
            ),
            (
                "identification",
                "first_in_first_out (the earliest acquired parcel of the security first) or selected (each disposal lists its parcels, as TD 33 para 4 accepts)",
            ),
            (
                "parcels",
                "list of 1 to 1,000 objects, each with parcel (a unique name), security (the same text for identical shares or units), acquired (YYYY-MM-DD, the contract date, s 109-5; not after the year), units (a decimal string above 0, at most 6dp), acquisition_cost (AUD, the first element, s 110-25(2)) and incidental_costs (AUD, the second element on acquiring, such as brokerage and stamp duty, ss 110-25(3) and 110-35), and optionally ownership_costs (AUD, the third element, s 110-25(4)), capital_expenditure (AUD, the fourth, s 110-25(5)) and title_costs (AUD, the fifth, s 110-25(6))",
            ),
            (
                "disposals",
                "list of 1 to 1,000 objects, each with security, date (YYYY-MM-DD, the contract date, s 104-10(3); not after the year, and one before it uses up units without being counted), units, capital_proceeds (AUD, before selling costs) and incidental_costs (AUD, the second element on disposing, such as brokerage); for selected identification also parcels, a list of objects each with parcel and units, adding up to the disposal's units",
            ),
            (
                "other_discount_capital_gains",
                "AUD, the year's other discount capital gains, before losses and the discount; 0.00 for a company",
            ),
            (
                "other_capital_gains",
                "AUD, the year's other capital gains that are not discount capital gains",
            ),
            (
                "other_capital_losses",
                "AUD, the year's other capital losses, none from collectables or personal use assets",
            ),
            (
                "prior_net_capital_losses",
                "AUD, net capital losses of earlier years not yet applied, none from collectables",
            ),
        ],
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
        "private_health_insurance_rebate" => vec![
            (
                "income_for_surcharge_purposes",
                "AUD, the person's own income for surcharge purposes (ITAA 1997 s 995-1), as income-tests works it out for 2024-25 to 2026-27",
            ),
            (
                "lines",
                "list of 1 to 100 lines of the private health insurance statement, each an object with premiums_eligible (AUD decimal string, label J: the person's share of the premiums paid, without lifetime health cover loading), rebate_received (AUD decimal string, label K: the person's share of the premium reduction received, \"0\" for none) and benefit_code (integer, label L: 30, 35 or 40 for premiums paid 1 July to 31 March, 31, 36 or 41 for 1 April to 30 June, with the oldest insured person under 65, 65 to 69 or 70 or over)",
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
        "beneficiary_tax_offset" => vec![(
            "rebatable_benefit",
            "AUD: the rebatable benefit included in assessable income (ITAA 1936 s 160AAA(1)), such as a Social Security Act 1991 Part 2.11, 2.11A, 2.12, 2.15 or 2.23B benefit, as established",
        )],
        "invalid_carer_tax_offset" => vec![
            (
                "dependant",
                "spouse, or other (a parent or the spouse's parent, or a child, brother or sister of you or your spouse aged 16 or over): the person maintained, who meets ITAA 1997 s 61-10(2), (3) or (4)",
            ),
            (
                "adjusted_taxable_income",
                "AUD: your adjusted taxable income for offsets for the year, as income-tests works it out for 2024-25 to 2026-27",
            ),
            (
                "dependant_adjusted_taxable_income",
                "AUD: the dependant's adjusted taxable income for offsets, including their pension or carer payment, for the period you maintained them",
            ),
        ],
        "zone_tax_offset" => vec![
            (
                "area",
                "special_area, zone_a or zone_b (the zone of your usual place of residence: ITAA 1936 Schedule 2 and the ATO's Australian zone list), or overseas_forces",
            ),
            (
                "days",
                "integer from 0 to the days in the year: days your usual place of residence was in the area, or for overseas_forces days of service as a Defence Force member at overseas localities, counting service in the prescribed area (s 79B(3)) and not service whose income s 23AD or 23AG exempts",
            ),
        ],
        "net_medical_expenses_tax_offset" => vec![
            (
                "net_medical_expenses",
                "AUD: expenses paid in the year for disability aids, attendant care or aged care for you or a resident dependant, less refunds from the NDIS, a private health insurer or another government, public authority, society, association or fund (ITAA 1936 s 159P(1) and (1B))",
            ),
            (
                "adjusted_taxable_income",
                "AUD: your adjusted taxable income for rebates for the year",
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
        "ppsr_timing" => vec![(
            "question",
            "registration (end time and fee), pmsi (registration deadline for super-priority) or company_grantor (Corporations Act s 588FL time)",
        )],
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
        "fhss" => vec![
            (
                "determination_date",
                "YYYY-MM-DD, the day the Commissioner makes the first home super saver determination, from 1 July 2018 to 31 December 2026; associated earnings run to it (s 138-40(3))",
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
        "work_expenses" => vec![
            ("method", "home_office, car_cents_per_kilometre, car_log_book or self_education"),
            ("reimbursed", "boolean: any of the expenses was reimbursed or paid for by an employer or anyone else; true is refused, so an unknown reimbursement is too"),
            ("allowance_received", "boolean: an allowance was received for the expenses; true is refused"),
        ],
        "ato_interest_penalties" => vec![(
            "question",
            "gic, sic, failure_to_lodge, statement_penalty, early_payment_interest or overpayment_interest; remission and scheme_penalty are refused",
        )],
        "depreciation_low_value_pool" => vec![
            ("previous_closing_pool_balance", "AUD: the pool's closing pool balance for the previous income year (ITAA 1997 s 40-440(1) step 3(a)); 0.00 in the year the pool starts"),
            ("low_cost_assets", "AUD: the taxable use percentage of the cost of the low-cost assets (cost under $1,000, s 40-425(2)) allocated to the pool for the year"),
            ("low_value_assets", "AUD: the taxable use percentage of the opening adjustable values of the low-value assets (s 40-425(5)) allocated to the pool for the year"),
            ("second_element_costs", "AUD: the taxable use percentage of amounts included for the year in the second element of cost of assets allocated for an earlier year and of low-value assets allocated for the year"),
            ("termination_values", "AUD: the taxable use percentage of the termination values of pooled assets for which a balancing adjustment event happened in the year (s 40-445)"),
        ],
        "depreciation_small_business_pool" => vec![
            ("opening_pool_balance", "AUD: the general small business pool's opening pool balance (ITAA 1997 s 328-195), including any s 328-225 adjustment"),
            ("additions", "AUD: the taxable purpose proportions of the adjustable values of assets first used, or installed ready for use, for a taxable purpose in the year and allocated to the pool (s 328-185(4)), not those written off under s 328-180 or temporary full expensing"),
            ("cost_additions", "AUD: the taxable purpose proportion of cost addition amounts for the year for pooled assets (s 328-190(3))"),
            ("termination_values", "AUD: the taxable purpose proportions of the termination values of pooled assets for which a balancing adjustment event happened in the year"),
        ],
        "capital_works" => vec![
            ("construction_expenditure", "AUD: your construction expenditure attributable to the part of your area this deduction is for (ITAA 1997 s 43-210 'portion of your CE')"),
            ("construction_start", "YYYY-MM-DD: when the capital works began, from 16 September 1987"),
            ("four_percent_manner", "boolean: the part was used in the 4% manner (s 43-145, including the 4% build to rent manner); only for works begun after 26 February 1992"),
            ("days_used", "integer from 0 to the days of the income year from the later of 1 July and construction_start (365, or 366 in 2023-24, for works begun before the year): the days in the year you owned or leased the part and used it in that manner (the denominator stays 365)"),
            ("taxable_use", "fraction, decimal string from 0 to 1: the extent of the part's use for producing assessable income over those days; use for R&D activities counts too only at 2.5% for capital works other than a hotel or apartment building (s 43-210 step 4(b), s 43-215 step 3(b)), never in the 4% manner (s 43-210 step 2)"),
            ("undeducted_construction_expenditure", "AUD: the undeducted construction expenditure for your area (s 43-235), at most construction_expenditure; the deduction cannot exceed it"),
        ],
        "horticulture_shearing_withholding" => vec![
            ("earnings", "AUD: the worker's total earnings for the pay period, whatever its length"),
            ("tfn_quoted", "boolean: the worker has given a valid TFN (false when no TFN, exemption or application has been given, or 28 days have passed since an application)"),
            ("foreign_resident", "boolean: the worker answered that they are a foreign resident for tax purposes"),
            ("tax_free_threshold_claimed", "boolean: the worker's TFN declaration claims the tax-free threshold; false with a TFN is refused (Schedule 1 applies)"),
            ("working_holiday_maker", "boolean: the worker is a working holiday maker; true is refused (Schedule 15)"),
            ("labour_mobility_scheme", "boolean: the worker is employed under the Seasonal Worker Programme, Pacific Labour Scheme or Pacific Australia Labour Mobility scheme; true is refused"),
        ],
        "upward_variation_withholding" => {
            let mut units = input_units("payg_withholding");
            units.push(("additional_amount", "AUD: the additional amount for the pay period the payee asked, in writing, to have withheld"));
            units
        }
        "daily_casual_withholding" => vec![
            ("earnings", "AUD: the payee's earnings for the day"),
            ("tfn_quoted", "boolean: the payee has given a valid TFN"),
            ("foreign_resident", "boolean: the payee answered that they are a foreign resident; true with a TFN is refused"),
            ("tax_free_threshold_claimed", "boolean: the payee has claimed the tax-free threshold, which this table requires when a TFN is quoted"),
            ("working_holiday_maker", "boolean: the payee is a working holiday maker; true is refused (Schedule 15)"),
        ],
        "no_abn_withholding" => vec![
            ("payment", "AUD: the payment for the supply, including any GST in it"),
            ("gst_included", "AUD: the GST included in the payment, 0.00 for none; at most the payment"),
            ("supply_total_excluding_gst", "AUD: the total of all payments made or to be made for the supply, this one included, disregarding GST (TAA 1953 Sch 1 s 12-190(4)(b))"),
        ],
        "entertainer_withholding" => vec![
            ("performances_per_week", "integer from 1: the performer's performances for the week"),
            ("performances_per_day", "integer from 1 to performances_per_week: the performances on the day this payment covers; 1 when each performance is paid separately"),
            ("earnings", "AUD: the performer's earnings for one performance"),
            ("tfn_quoted", "boolean: the performer has given a valid TFN"),
            ("foreign_resident", "boolean: the performer answered that they are a foreign resident; true with a TFN is refused"),
            ("tax_free_threshold_claimed", "boolean: the performer has claimed the tax-free threshold (Table A); false uses Table B"),
            ("working_holiday_maker", "boolean: the performer is a working holiday maker; true is refused (Schedule 15)"),
        ],
        "return_to_work_withholding" => vec![
            ("payment", "AUD: the return to work payment"),
            ("tfn_quoted", "boolean: the payee has given a valid TFN"),
            ("foreign_resident", "boolean: the payee is a foreign resident; it matters only without a TFN"),
            ("working_holiday_maker", "boolean: the payee is a working holiday maker; true is refused (Schedule 15)"),
        ],
        "annuity_withholding" => vec![
            ("payment", "AUD: the annuity payment"),
            ("deductible_amount", "AUD: the deductible amount for the income year (ITAA 1936 s 27H), the payee's share where 2 or more people hold the annuity"),
            ("instalments", "integer 1 to 366: the instalments of the annuity payable in the income year"),
            ("pay_period", "weekly, fortnightly, monthly or quarterly: how often the annuity is paid"),
            ("scale", "integer 1 to 6, as payg_withholding; scale 4 (no TFN) also needs foreign_resident"),
        ],
        "voluntary_agreement_withholding" => vec![
            ("payment", "AUD: the payment, including any GST in it"),
            ("gst_included", "AUD: the GST included in the payment, 0.00 for none; at most the payment"),
            ("agreement_in_force", "boolean: a voluntary agreement in the approved form covering the payment is in force (not terminated under s 12-55(3)); false is refused"),
            ("abn_quoted", "boolean: the payee has an ABN in force and it is quoted in the agreement; false is refused"),
            ("agreement_applies_instalment_rate", "boolean: the agreement says the payee's Commissioner's instalment rate applies; true needs instalment_rate"),
        ],
        "rd_tax_offset" => vec![
            ("notional_deductions", "AUD: the total of the amounts the R&D entity can deduct for the year under ITAA 1997 ss 355-205, 355-305, 355-480, 355-520 and 355-580 (s 355-100(1)), as established"),
            ("rsp_crc_expenditure", "AUD: the part of notional_deductions s 355-100(2) keeps when the total is under $20,000: s 355-205 expenditure incurred to a research service provider that is not an associate, for services within a research field it is registered for and applicable to the R&D activities, and s 355-580 CRC contributions; at most notional_deductions"),
            ("aggregated_turnover", "AUD: the entity's aggregated turnover for the year (s 328-115)"),
            ("exempt_entity_controlled", "boolean: exempt entities would control the entity at any time in the year as s 355-100(1) item 2 describes (s 328-125 at 50%, without s 328-125(6))"),
            ("base_rate_entity", "boolean: Income Tax Rates Act 1986 s 23(2)(a) applies to the company for the year (a base rate entity to which ss 23(3) to (5) and 23A do not apply), so its corporate tax rate is 25%; otherwise 30% (ITAA 1997 s 995-1 'corporate tax rate')"),
            ("total_expenses", "AUD: the entity's total expenses for the year under s 355-115, used for the premium tiers where item 1 does not apply"),
            ("recoupment_clawback", "AUD: the s 355-440 amount attributable to this offset year, after repayments (s 355-440(3)) and the grant cap (s 355-440(4)); 0.00 if none"),
            ("feedstock_revenue", "AUD: the feedstock revenue (s 355-445(4)) of feedstock outputs supplied or applied to own use in the present year; 0.00 if none"),
            ("feedstock_expenditure", "AUD: the amounts deducted for this offset year that are reasonably attributable to producing those feedstock outputs (s 355-445(2)(b)); at most notional_deductions; 0.00 if none"),
            ("present_year_base_rate_entity", "boolean: as base_rate_entity, for the present year, the year the clawback amount arises, whose corporate tax rate divides the s 355-450 amount"),
        ],
        "sb_cgt_concessions" => vec![
            ("entity", "individual, company or trust: the entity that made the capital gain"),
            ("share_or_trust_interest", "boolean: the asset is a share in a company or an interest in a trust; true is refused (ITAA 1997 s 152-10(2))"),
            ("acquired", "YYYY-MM-DD: when the asset was acquired, the contract date (s 109-5), from 20 September 1985"),
            ("event_date", "YYYY-MM-DD: the CGT event, the sale contract date (s 104-10(3)), in the year"),
            ("partnership_asset", "boolean: the asset is your interest in an asset of a partnership you are a partner in (s 152-10(1)(c)(iii))"),
            ("passively_held", "no, affiliate_or_connected_entity or partnership: the asset is used in the business of your affiliate or an entity connected with you (s 152-10(1A)), or of a partnership you are a partner in (s 152-10(1B)), not in your own"),
            ("business", "carried_on, winding_up or not_carried_on: the business entity's business in the year (you, the partnership, or the entity using a passively held asset; s 328-110(1)(a) and (5))"),
            ("previous_year_turnover", "AUD, or null for no business carried on: the business entity's aggregated turnover for the previous year (s 328-110(1)(b)(i))"),
            ("year_before_turnover", "AUD, or null for no business carried on: its aggregated turnover for the year before the previous year (s 328-110(3))"),
            ("likely_turnover", "AUD: its likely aggregated turnover for the year, worked out as at its first day, or as at the day it started the business if that was during the year (s 328-110(1)(b)(ii) and (2))"),
            ("year_turnover", "AUD: its aggregated turnover for the year, as at the end of the year (s 328-110(4))"),
            ("net_asset_value", "AUD, may be negative: the sum s 152-15 counts just before the CGT event, the net value of the CGT assets (s 152-20) of yours, your connected entities, your affiliates and entities connected with them"),
            ("active_days", "integer: the total days in the test period on which the asset was an active asset (s 152-40)"),
            ("business_ceased", "YYYY-MM-DD, or null: when the relevant business ceased to be carried on, if it did before the CGT event (s 152-35(2)(b)(ii))"),
            ("date_of_birth", "YYYY-MM-DD: the individual's, or for a company or trust that of the significant individual relied on (ss 152-105(d) and 152-110(1)(d))"),
            ("retirement", "boolean: the CGT event happens in connection with that individual's retirement"),
            ("permanently_incapacitated", "boolean: that individual is permanently incapacitated at the time of the CGT event"),
            ("last_rollover_event", "YYYY-MM-DD, or null for this one: the last CGT event in the year for which you obtain the small business roll-over, from event_date to the end of the year; the replacement asset period runs from it (s 104-190(1A))"),
            ("earnout_latest_due", "YYYY-MM-DD, or null for no look-through earnout right: the latest time a possible financial benefit becomes or could become due under a look-through earnout right relating to the asset and this disposal, on or after event_date; the replacement asset period ends no earlier than 6 months after it (s 104-190(1A)(b)(ii))"),
        ],
        "small_business_income_offset" => vec![
            ("taxable_income", "AUD, whole dollars: your taxable income for the income year"),
            ("basic_income_tax_liability", "AUD: your basic income tax liability for the year, the tax on your taxable income at your rates before tax offsets and without the Medicare levy (ITAA 1997 s 4-10(3) step 2)"),
            ("offsets_applied_first", "AUD: the tax offsets you apply against that liability before this one (s 63-10), such as the low income tax offset; nil for none"),
            ("net_small_business_income", "AUD: your net small business income for the year as a sole trader (s 328-365: business income less s 328-370 deductions, without net capital gains or personal services income not from a personal services business; nil for none or a loss); it counts only if you are a small business entity at $5 million"),
            ("shares", "list of up to 50 objects, each {amount, deductions} in AUD: a share of a small business entity's net small business income, or an amount s 328-355(c) covers, included in your assessable income (s 328-355(b), (c)), and your deductions attributable to it that s 328-370 covers; [] for none"),
            ("prescribed_person", "boolean: you are a prescribed person under s 102AC of the ITAA 1936 for the year (broadly, under 18 at its end); true is refused (s 328-375)"),
        ],
        "non_commercial_losses" => vec![
            ("activity", "other, primary_production or professional_arts: the business activity, or group of similar activities (s 35-10(3)), you carry on as an individual alone or in partnership (s 35-10(1), (4) and (5))"),
            ("assessable_income", "AUD: your assessable income from the activity for the year, or your share as a partner without the parts s 35-25 ignores"),
            ("deductions", "AUD: the deductions attributable to the activity for the year apart from s 35-10(2) and (2C), or your share as a partner (s 35-35)"),
            ("brought_forward", "AUD: amounts from earlier years that s 35-10(2)(b) defers to this year, and pre-business amounts s 35-10(2C) brings into it; nil for none"),
            ("part_year", "boolean: you started or stopped carrying on the activity during the year (s 35-30(b))"),
            ("earlier_years", "list of the 4 income years before this one, latest first, each null for a year the activity was not carried on or {assessable_income, deductions} in AUD, the deductions apart from s 35-10(2) and (2C) (s 35-35)"),
            ("real_property_value", "AUD: the total reduced cost bases, or market values where higher, of real property used on a continuing basis in the activity at the end of the year or when it stopped, without a private dwelling and its land or a tenant's fixtures, apportioned for other use (ss 35-40 and 35-50)"),
            ("other_assets_value", "AUD: the total values of the assets s 35-45(2) counts used on a continuing basis in the activity, without real property and cars, motor cycles and similar vehicles, apportioned for other use (ss 35-45 and 35-50)"),
            ("taxable_income", "AUD, may be negative: your taxable income for the year, or the loss as a negative amount, as worked out with business_losses deducted"),
            ("business_losses", "AUD: the excesses of deductions over assessable income for each business activity that taxable_income deducts (s 35-10(2E), last sentence)"),
            ("assessable_fhss_released_amount", "AUD: the assessable FHSS released amount included in taxable_income (s 35-10(2E)(a))"),
            ("reportable_fringe_benefits", "AUD: your reportable fringe benefits total (s 35-10(2E)(b))"),
            ("reportable_super_contributions", "AUD: your reportable superannuation contributions (s 35-10(2E)(c))"),
            ("total_net_investment_loss", "AUD: your total net investment losses (s 35-10(2E)(d))"),
            ("net_exempt_income", "AUD: your net exempt income for the year left after ss 36-10 and 36-15, which reduces an amount deferred (s 35-15); nil for none"),
        ],
        "psi_test" => vec![
            ("earner", "employee, religious_practitioner, sole_trader or entity: who receives the income: the individual as an employee or office holder (s 85-35(1)), as a religious practitioner paid under s 12-47 of Schedule 1 to the TAA 1953 (s 85-35(2)), or as a contractor, or a company, partnership or trust (a personal services entity)"),
            ("mainly_personal_efforts", "boolean: the income is mainly a reward for the individual's personal efforts or skills (s 84-5(1))"),
            ("determination_in_force", "boolean: a personal services business determination is in force for the individual's PSI (s 87-15(1)(a) or (b))"),
            ("payers", "list of 1 to 50 objects, one for each payer with its associates, or for each contract when a payer's contracts differ: amount (AUD, the PSI from it in the year), associate (it is an associate of the individual or entity), offer (it engaged the individual or entity as a direct result of offers to the public, s 87-20), and result, tools and defects (the income is for producing a result, the individual or entity supplies the tools and is liable for defects, s 87-18), each a boolean; and source, an optional label of 1 to 100 characters: rows with the same source are one payer with its associates for the 80% rule and the unrelated clients test (s 87-15(3))"),
            ("agent", "boolean: the individual or entity is an agent of a principal, not its employee, paid for services to the principal's customers (s 87-40(2)); then commission and principal_payments are required"),
            ("principal_work_by_others", "AUD: the market value of the principal work performed in the year by entities the individual or entity engaged, not counting associates that are not individuals and, for a personal services entity, individuals whose PSI is in its income; a partner's work counts for a partnership (s 87-25(1), (2) and (2A)); TR 2022/3 paragraph 118 measures it against the PSI"),
            ("apprentice", "boolean: the individual or entity had one or more apprentices for at least half the year (s 87-25(3))"),
            ("premises", "object of 5 booleans for the business premises test (s 87-30): at_all_times (maintained and used at all times in the year), mainly_for_psi, exclusive_use, separate_from_private and separate_from_clients"),
        ],
        "government_super_amounts" => {
            let mut units = input_units("income_tests");
            units.extend([
                ("eligible_income", "AUD: the part of total income before business deductions attributable to activities that make the person an employee for super guarantee purposes or to carrying on a business (Co-contribution Act s 6(1)(b), (2) and s 8(3)): salary, wages, director fees and business income, not trust or non-business partnership distributions, interest, rent, dividends or lump sums for another year of employment"),
                ("personal_contributions", "AUD: eligible personal superannuation contributions the fund received in the income year, not deducted and not a downsizer contribution (Co-contribution Act s 7)"),
                ("concessional_contributions", "AUD: concessional contributions for the financial year, including super guarantee, salary sacrifice and deducted personal contributions (Co-contribution Act s 12C(1)(a) and s 12E(2)(a)); a person with contributions or amounts s 12E(3) disregards (constitutionally protected funds, ITAA 1997 s 291-370(1)(a), and other s 291-165(1)(c) amounts) is outside scope"),
                ("return_lodged", "boolean: an income tax return for the year is lodged (Co-contribution Act s 6(1)(d))"),
                ("under_71", "boolean: less than 71 years old at the end of the income year (Co-contribution Act s 6(1)(e))"),
                ("temporary_visa_held", "boolean: held a temporary visa at any time in the income year; true is refused (Co-contribution Act s 6(1)(f))"),
                ("non_concessional_within_cap", "boolean, or null if not established: non-concessional contributions for the financial year do not exceed the cap (Co-contribution Act s 6(1)(da)); null is refused where it decides the co-contribution"),
                ("total_super_balance_below_cap", "boolean, or null if not established: total superannuation balance immediately before the start of the financial year is less than the general transfer balance cap (Co-contribution Act s 6(1)(db)); null is refused where it decides the co-contribution"),
            ]);
            units
        }
        "residency_tests" => vec![
            ("resides", "boolean, your judgement: you resided in Australia according to the ordinary meaning of the word at some time in the income year (s 6(1), resident (a); TR 2023/1 paragraphs 12 and 17 to 54); never inferred from the dates"),
            ("domicile_in_australia", "boolean, your judgement: your domicile (of origin, dependence or choice, under the common law and the Domicile Act 1982) was in Australia (s 6(1)(a)(i); TR 2023/1 paragraphs 56 to 62)"),
            ("permanent_place_of_abode_outside", "boolean, your judgement: your permanent place of abode was outside Australia, having definitely abandoned residence in Australia and begun living permanently in one other country (s 6(1)(a)(i); TR 2023/1 paragraphs 63 to 82); read only when domicile_in_australia is true"),
            ("presence", "list of up to 400 objects {from, to}, each a YYYY-MM-DD date: the periods you were in Australia, both days counted; the days inside the income year are counted once each, part days as whole days (TR 2023/1 paragraphs 86 and 87); an empty list for none"),
            ("usual_place_of_abode_outside", "boolean, your judgement: your usual place of abode was outside Australia (s 6(1)(a)(ii); TR 2023/1 paragraphs 88 to 91); read only at 183 days or more (184 in 2023-24)"),
            ("intends_to_take_up_residence", "boolean, your judgement: you intended to take up residence in Australia (s 6(1)(a)(ii); TR 2023/1 paragraphs 92 to 95); read only at 183 days or more (184 in 2023-24)"),
            ("commonwealth_super", "none, pss_member (a member of the PSS, the scheme established by deed under the Superannuation Act 1990), css_eligible_employee (an eligible employee for the Superannuation Act 1976), partner_of_member (the spouse, including a de facto partner, of either) or child_under_16_of_member (s 6(1)(a)(iii)); TR 2023/1 paragraph 96: active members only"),
            ("resident_elsewhere", "boolean: you were also a resident of another country under that country's law; with a resident result this is refused, as a tax treaty's tie-breaker may apply (TR 2023/1 paragraph 108)"),
        ],
        "lodgment_required" => vec![
            ("residency", "object: {status: whole_year}, {status: part_year, months: 1 to 12, counting the months you became and stopped being a resident} or {status: none, taxable_australian_income: boolean, income including capital gains taxable in Australia other than the withholding payments s 6(3)(b) excludes}; residency-tests gives the status"),
            ("taxable_income", "AUD: your taxable income for the year"),
            ("dividends_and_franking_credits", "AUD: your assessable income from dividends or distributions and franking credits (s 6(2)(i) and (o))"),
            ("under_18", "null, or for a person under 18 at all times in the year {income_excluding_work: AUD, income other than salary, wages or other payments for work personally performed} (s 6(2)(i))"),
            ("triggers", "list of the conditions that applied in the year, each a stated fact: payg_withheld (s 6(2)(a), other than the payments it excludes), tax_loss, net_capital_loss, business, primary_production_trust, primary_production_partnership, exempt_overseas_income (s 23AF or 23AG income and $1 or more of other income), payg_instalment, special_professional, private_health_offset (s 6(2)(m)), reportable_amounts (s 6(2)(n)), co_contribution, contributions_cap and super_lump_sum (s 6(2)(r))"),
            ("study_loan", "null, or with an accumulated HELP, TSL or AASL, or VSL debt on the 1 June before the year {income: AUD, income including foreign-sourced income} (s 6(10)); read only for a foreign resident at some time in the year"),
            ("working_holiday_maker", "null, or when you derived assessable income from Australian sources as a working holiday maker {other_assessable_income: boolean, taxable_income_excluding_remainders: AUD} (s 7(1)(f))"),
            ("listed_payment", "null, or when your assessable income included a payment the year's instrument lists, such as jobseeker or austudy payment {other_assessable_income: boolean} (s 7(2))"),
            ("sapto", "null, or when you qualified for the seniors and pensioners tax offset {rebate_income: AUD, category: single (single, widowed or separated at any time), apart (otherwise, living apart from your spouse due to illness, or either of you in a nursing home, at any time) or together} (s 7(3))"),
            ("child_support_parent", "boolean: you were a parent liable for or entitled to child support; refused when this instrument does not require a return, as the separate instrument for parents may"),
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
        // The ATO's Rental properties guide 2026, Example 34: a holiday home
        // let mainly for rent, held all year and available 330 days, owned in
        // equal shares; the $37,000 of costs is entered on one line.
        "rental_property" => vec![
            ("year", Text("2025-26")),
            ("ownership_share", Text("0.5")),
            ("acquired", Text("2024-07-01")),
            ("disposed", Json("null")),
            ("days_available", Whole(330)),
            ("section_26_50_applies", Flag(false)),
            ("income", Json(r#"{"rental_income": "50000.00"}"#)),
            ("expenses", Json(r#"{"sundry": "37000.00"}"#)),
            (
                "rental_only_expenses",
                Json(r#"{"property_agent_fees": "3000.00"}"#),
            ),
            ("loans", Json("[]")),
            ("borrowing_loans", Json("[]")),
            ("capital_works", Json("[]")),
            ("depreciating_assets", Json("[]")),
        ],
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
        // The Library's example (Superannuation Benefits, 8-420): a $42,000
        // benefit of a $25,000 tax free component and a $17,000 element
        // taxed in the fund, applied for on 1 August 2026.
        // The ATO's 'Tax on super benefits' example: Tony, 58 and retired,
        // receives $350,000 on 25 July 2021, a $100,000 tax free component
        // and a $250,000 element taxed in the fund; $25,000 is above the
        // 2021-22 low rate cap of $225,000.
        "super_benefit_tax" => vec![
            ("year", Text("2021-22")),
            ("benefit", Text("350000.00")),
            ("interest_value", Text("350000.00")),
            ("interest_tax_free_component", Text("100000.00")),
            ("untaxed_element", Text("0.00")),
            ("recipient", Text("member")),
            ("payment_date", Text("2021-07-25")),
            ("date_of_birth", Text("1963-03-01")),
            ("terminal_medical_condition", Flag(false)),
            ("disability_superannuation_benefit", Flag(false)),
            ("low_rate_cap_used", Text("0.00")),
            ("other_lump_sums_this_year", Flag(false)),
        ],
        "dasp_tax" => vec![
            ("tax_free_component", Text("25000.00")),
            ("taxed_element", Text("17000.00")),
            ("untaxed_element", Text("0.00")),
            ("working_holiday_maker_contributions", Flag(false)),
            ("unclaimed_money_rollover", Flag(false)),
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
        // Shaped on the ATO's EKW Ltd example (section B, example 4):
        // FDT $80,000 and an offset of $62,000.
        "franking_account" => vec![
            ("private_company", Flag(false)),
            ("benchmark_rule_exempt", Flag(false)),
            ("corporate_tax_rate", Text("0.30")),
            ("opening_balance", Text("10000.00")),
            ("rd_deferred_debit_brought_forward", Text("0.00")),
            ("last_benchmark", Json("null")),
            (
                "events",
                Json(
                    r#"[{"date": "2025-10-28", "type": "payg_instalment", "amount": "125000.00"}, {"date": "2026-02-28", "type": "payg_instalment", "amount": "125000.00"}, {"date": "2026-04-28", "type": "payg_instalment", "amount": "250000.00"}, {"date": "2025-11-15", "type": "income_tax_refund", "amount": "60000.00", "rd_offset_part": "0.00"}, {"date": "2026-03-31", "type": "streaming_determination", "amount": "20000.00"}]"#,
                ),
            ),
            (
                "distributions",
                Json(
                    r#"[{"date": "2025-09-30", "amount": "1190000.00", "franking_credit": "510000.00"}]"#,
                ),
            ),
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
        // The ATO's Kathleen (question 18 of the 2026 supplement, examples 1
        // to 4), her collectables left out.
        "cgt_parcels" => vec![
            ("year", Text("2025-26")),
            ("entity", Text("individual")),
            ("identification", Text("first_in_first_out")),
            (
                "parcels",
                Json(
                    r#"[{"parcel": "K1", "security": "Company A", "acquired": "2000-11-17", "units": "1000", "acquisition_cost": "3000.00", "incidental_costs": "0.00"}, {"parcel": "K2", "security": "Company B", "acquired": "2025-10-10", "units": "130", "acquisition_cost": "520.00", "incidental_costs": "0.00"}, {"parcel": "K3", "security": "Company C", "acquired": "2025-10-10", "units": "600", "acquisition_cost": "2400.00", "incidental_costs": "0.00"}]"#,
                ),
            ),
            (
                "disposals",
                Json(
                    r#"[{"security": "Company A", "date": "2025-12-17", "units": "1000", "capital_proceeds": "6000.00", "incidental_costs": "0.00"}, {"security": "Company B", "date": "2026-02-27", "units": "130", "capital_proceeds": "1040.00", "incidental_costs": "0.00"}, {"security": "Company C", "date": "2026-06-25", "units": "600", "capital_proceeds": "1800.00", "incidental_costs": "0.00"}]"#,
                ),
            ),
            ("other_discount_capital_gains", Text("0.00")),
            ("other_capital_gains", Text("0.00")),
            ("other_capital_losses", Text("0.00")),
            ("prior_net_capital_losses", Text("400.00")),
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
            (
                "employee_contribution",
                Json(r#"{"capped_total":"1000.00"}"#),
            ),
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
        // The ATO's Narelle example, 2025-26 (Income thresholds and rates for
        // the private health insurance rebate).
        "private_health_insurance_rebate" => vec![
            ("year", Text("2025-26")),
            ("income_for_surcharge_purposes", Text("83000")),
            (
                "lines",
                Json(
                    r#"[{"premiums_eligible": "900", "rebate_received": "146", "benefit_code": 30}, {"premiums_eligible": "300", "rebate_received": "48", "benefit_code": 31}]"#,
                ),
            ),
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
        // Fabricated: $20,000 of a rebatable benefit in 2025-26.
        "beneficiary_tax_offset" => vec![
            ("year", Text("2025-26")),
            ("rebatable_benefit", Text("20000")),
        ],
        // Fabricated: a parent on a disability support pension, 2025-26.
        "invalid_carer_tax_offset" => vec![
            ("year", Text("2025-26")),
            ("dependant", Text("other")),
            ("adjusted_taxable_income", Text("80000")),
            ("dependant_adjusted_taxable_income", Text("4282")),
        ],
        // The ATO's Narendra example (Zone tax offset, QC105018): Lord Howe
        // Island for 100 days in 2024-25 and 124 in 2025-26.
        "zone_tax_offset" => vec![
            ("year", Text("2025-26")),
            ("area", Text("special_area")),
            ("days", Whole(124)),
            ("prior_year", Text("2024-25")),
            ("prior_days", Whole(100)),
        ],
        // Fabricated: $10,000 of net aged care expenses in 2018-19.
        "net_medical_expenses_tax_offset" => vec![
            ("year", Text("2018-19")),
            ("net_medical_expenses", Text("10000")),
            ("adjusted_taxable_income", Text("90000")),
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
        // Constructed: consumer goods described by serial number, registered
        // on 2 March 2026 with no end date asked for.
        "ppsr_timing" => vec![
            ("question", Text("registration")),
            ("registration_time", Text("2026-03-02T10:30")),
            ("collateral", Text("consumer")),
            ("serial_numbered", Flag(true)),
        ],
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
        // GN 2024/1 example 11 (Sue): the determination's components, released
        // in full, with no marginal rate estimate.
        "fhss" => vec![
            ("year", Text("2024-25")),
            ("determination_date", Text("2025-06-20")),
            (
                "determination",
                Json(
                    r#"{"concessional": "8500", "non_concessional": "5000", "associated_earnings": "850"}"#,
                ),
            ),
            ("commissioner_cannot_estimate", Flag(true)),
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
        // The Library's Neil Swan (Tax Examples ¶2-280): the days, his
        // birthday, the turnover 2 years back and the net value are fabricated.
        // Keisha, the ATO's fixed rate method example: 843 hours in 2025-26.
        // Her actual expenses are fabricated.
        "work_expenses" => vec![
            ("year", Text("2025-26")),
            ("method", Text("home_office")),
            ("reimbursed", Flag(false)),
            ("allowance_received", Flag(false)),
            ("hours", Text("843")),
            ("energy", Text("180.00")),
            ("phone", Text("240.00")),
            ("internet", Text("300.00")),
            ("stationery", Text("0.00")),
            ("cleaning", Text("0.00")),
            ("furniture_decline", Text("0.00")),
            ("asset_decline", Text("0.00")),
            ("repairs", Text("0.00")),
            ("running_expenses_incurred", Flag(true)),
        ],
        // The site's GIC payment plan oracle, hand-worked: $1,000 due 1 October
        // 2026, 2 instalments of $336.56 and a final $336.56 on 1 December.
        "ato_interest_penalties" => vec![
            ("question", Text("gic")),
            ("year", Text("2026-27")),
            ("amount", Text("1000.00")),
            ("due_date", Text("2026-10-01")),
            ("to", Text("2026-11-30")),
            (
                "payments",
                Json(
                    r#"[{"date": "2026-10-01", "amount": "336.56"}, {"date": "2026-11-01", "amount": "336.56"}]"#,
                ),
            ),
        ],
        // The Guide to depreciating assets 2026's John: $5,000 closing balance
        // for 2024-25 and a $990 printer used 60% for taxable purposes.
        "depreciation_low_value_pool" => vec![
            ("year", Text("2025-26")),
            ("previous_closing_pool_balance", Text("5000.00")),
            ("low_cost_assets", Text("594.00")),
            ("low_value_assets", Text("0.00")),
            ("second_element_costs", Text("0.00")),
            ("termination_values", Text("0.00")),
        ],
        // The ATO's Loretta (Small business pool calculations, example 2,
        // 2018-19) worked under the same rules in 2025-26.
        "depreciation_small_business_pool" => vec![
            ("year", Text("2025-26")),
            ("opening_pool_balance", Text("100000.00")),
            ("additions", Text("28000.00")),
            ("cost_additions", Text("0.00")),
            ("termination_values", Text("8000.00")),
        ],
        // Rental properties guide 2026, example 29; the day in August 1991 and
        // the undeducted construction expenditure are fabricated.
        "capital_works" => vec![
            ("year", Text("2025-26")),
            ("construction_expenditure", Text("115800.00")),
            ("construction_start", Text("1991-08-01")),
            ("four_percent_manner", Flag(false)),
            ("days_used", Whole(346)),
            ("taxable_use", Text("1")),
            ("undeducted_construction_expenditure", Text("20000.00")),
        ],
        // Schedule 2's example: a resident with a TFN earning $231.50.
        "horticulture_shearing_withholding" => vec![
            ("year", Text("2026-27")),
            ("earnings", Text("231.50")),
            ("tfn_quoted", Flag(true)),
            ("foreign_resident", Flag(false)),
            ("tax_free_threshold_claimed", Flag(true)),
            ("working_holiday_maker", Flag(false)),
            ("labour_mobility_scheme", Flag(false)),
        ],
        // Schedule 14's example: $563.00 weekly on scale 2 and $50 more.
        "upward_variation_withholding" => vec![
            ("year", Text("2026-27")),
            ("earnings", Text("563.00")),
            ("pay_period", Text("weekly")),
            ("scale", Whole(2)),
            ("additional_amount", Text("50.00")),
        ],
        // The daily and casual table's example, without its tax offsets.
        "daily_casual_withholding" => vec![
            ("year", Text("2026-27")),
            ("earnings", Text("193.62")),
            ("tfn_quoted", Flag(true)),
            ("foreign_resident", Flag(false)),
            ("tax_free_threshold_claimed", Flag(true)),
            ("working_holiday_maker", Flag(false)),
        ],
        // Rental expenses: Derek's $1,800 cash job with no ABN quoted.
        "no_abn_withholding" => vec![
            ("year", Text("2025-26")),
            ("payment", Text("1800.00")),
            ("gst_included", Text("0.00")),
            ("supply_total_excluding_gst", Text("1800.00")),
        ],
        // Schedule 3's example: Sandra's 2 performances at $500.35, one a day.
        "entertainer_withholding" => vec![
            ("year", Text("2026-27")),
            ("performances_per_week", Whole(2)),
            ("performances_per_day", Whole(1)),
            ("earnings", Text("500.35")),
            ("tfn_quoted", Flag(true)),
            ("foreign_resident", Flag(false)),
            ("tax_free_threshold_claimed", Flag(true)),
            ("working_holiday_maker", Flag(false)),
        ],
        // Schedule 4's example: George's $18,000 return to work payment.
        "return_to_work_withholding" => vec![
            ("year", Text("2026-27")),
            ("payment", Text("18000.00")),
            ("tfn_quoted", Flag(true)),
            ("foreign_resident", Flag(false)),
            ("working_holiday_maker", Flag(false)),
        ],
        // Schedule 6's example 1: Barbara's $1,000 a week, $5,200 over 52.
        "annuity_withholding" => vec![
            ("year", Text("2026-27")),
            ("payment", Text("1000.00")),
            ("deductible_amount", Text("5200.00")),
            ("instalments", Whole(52)),
            ("pay_period", Text("weekly")),
            ("scale", Whole(2)),
        ],
        // Schedule 10's example without the term applying Francesca's CIR: 20%.
        "voluntary_agreement_withholding" => vec![
            ("year", Text("2026-27")),
            ("payment", Text("1000.00")),
            ("gst_included", Text("0.00")),
            ("agreement_in_force", Flag(true)),
            ("abn_quoted", Flag(true)),
            ("agreement_applies_instalment_rate", Flag(false)),
        ],
        // Calculate the clawback adjustment, Company F; its turnover, given
        // only as under $20 million, is fabricated.
        "rd_tax_offset" => vec![
            ("year", Text("2022-23")),
            ("notional_deductions", Text("1000000.00")),
            ("rsp_crc_expenditure", Text("0.00")),
            ("aggregated_turnover", Text("10000000.00")),
            ("exempt_entity_controlled", Flag(false)),
            ("base_rate_entity", Flag(true)),
            ("total_expenses", Text("0.00")),
            ("recoupment_clawback", Text("500000.00")),
            ("feedstock_revenue", Text("0.00")),
            ("feedstock_expenditure", Text("0.00")),
            ("present_year_base_rate_entity", Flag(true)),
        ],
        "sb_cgt_concessions" => vec![
            ("year", Text("2025-26")),
            ("entity", Text("individual")),
            ("share_or_trust_interest", Flag(false)),
            ("acquired", Text("2007-07-02")),
            ("event_date", Text("2026-05-15")),
            ("partnership_asset", Flag(false)),
            ("passively_held", Text("no")),
            ("business", Text("carried_on")),
            ("previous_year_turnover", Text("1100000.00")),
            ("year_before_turnover", Text("1000000.00")),
            ("likely_turnover", Text("1200000.00")),
            ("year_turnover", Text("1200000.00")),
            ("net_asset_value", Text("3800000.00")),
            ("active_days", Whole(6893)),
            ("business_ceased", Json("null")),
            ("date_of_birth", Text("1972-04-10")),
            ("retirement", Flag(false)),
            ("permanently_incapacitated", Flag(false)),
            ("last_rollover_event", Json("null")),
            ("earnout_latest_due", Json("null")),
            ("retirement_exemption_used", Text("0.00")),
            ("choice_date", Text("2026-06-30")),
        ],
        // The site's sole trader vector for 2026-27 ($20,000 of profit on top
        // of $60,000 of wages); the turnovers are fabricated.
        "small_business_income_offset" => vec![
            ("year", Text("2026-27")),
            ("taxable_income", Text("79000")),
            ("basic_income_tax_liability", Text("14220.00")),
            ("offsets_applied_first", Text("0.00")),
            ("net_small_business_income", Text("20000.00")),
            ("shares", Json("[]")),
            ("prescribed_person", Flag(false)),
            ("business", Text("carried_on")),
            ("previous_year_turnover", Text("21000.00")),
            ("year_before_turnover", Text("18000.00")),
            ("likely_turnover", Text("22000.00")),
            ("year_turnover", Text("22000.00")),
        ],
        // The ATO's Nguyen (profits test) with fabricated income, deductions,
        // assets and income requirement figures.
        "non_commercial_losses" => vec![
            ("year", Text("2025-26")),
            ("activity", Text("primary_production")),
            ("assessable_income", Text("10000.00")),
            ("deductions", Text("17500.00")),
            ("brought_forward", Text("0.00")),
            ("part_year", Flag(false)),
            (
                "earlier_years",
                Json(
                    r#"[{"assessable_income": "40000.00", "deductions": "22500.00"}, {"assessable_income": "35000.00", "deductions": "20000.00"}, {"assessable_income": "20000.00", "deductions": "24000.00"}, {"assessable_income": "36000.00", "deductions": "20000.00"}]"#,
                ),
            ),
            ("real_property_value", Text("300000.00")),
            ("other_assets_value", Text("40000.00")),
            ("taxable_income", Text("52500.00")),
            ("business_losses", Text("7500.00")),
            ("assessable_fhss_released_amount", Text("0.00")),
            ("reportable_fringe_benefits", Text("0.00")),
            ("reportable_super_contributions", Text("0.00")),
            ("total_net_investment_loss", Text("0.00")),
            ("net_exempt_income", Text("0.00")),
            ("other_income", Text("60000.00")),
        ],
        // TR 2022/3 example 20: Joe installs ducts to plans with his own tools
        // and is liable for defects; the amount is the site's.
        "psi_test" => vec![
            ("year", Text("2025-26")),
            ("earner", Text("sole_trader")),
            ("mainly_personal_efforts", Flag(true)),
            ("determination_in_force", Flag(false)),
            (
                "payers",
                Json(
                    r#"[{"amount": "18000.00", "associate": false, "offer": false, "result": true, "tools": true, "defects": true}]"#,
                ),
            ),
            ("agent", Flag(false)),
            ("principal_work_by_others", Text("0.00")),
            ("apprentice", Flag(false)),
            (
                "premises",
                Json(
                    r#"{"at_all_times": false, "mainly_for_psi": false, "exclusive_use": false, "separate_from_private": false, "separate_from_clients": false}"#,
                ),
            ),
        ],
        // The ATO's Angelo (Super co-contribution): $35,000 of salary and
        // $1,040 of personal contributions; the year is the page's.
        "government_super_amounts" => vec![
            ("year", Text("2023-24")),
            ("taxable_income", Text("35000")),
            ("assessable_fhss_released_amount", Text("0")),
            ("family_trust_distribution_amounts", Text("0")),
            ("reportable_fringe_benefits_section_57a", Text("0")),
            ("reportable_fringe_benefits_other", Text("0")),
            ("reportable_employer_super_contributions", Text("0")),
            ("deductible_personal_super_contributions", Text("0")),
            ("excess_concessional_contributions", Text("0")),
            ("net_financial_investment_loss", Text("0")),
            ("net_rental_property_loss", Text("0")),
            ("tax_free_government_pensions", Text("0")),
            ("target_foreign_income", Text("0")),
            ("exempt_foreign_employment_income", Text("0")),
            ("child_support_paid", Text("0")),
            ("assessable_income", Text("35000")),
            ("business_deductions", Text("0")),
            ("residency", Text("full_year_resident")),
            ("eligible_income", Text("35000")),
            ("personal_contributions", Text("1040")),
            ("concessional_contributions", Text("0")),
            ("return_lodged", Flag(true)),
            ("under_71", Flag(true)),
            ("temporary_visa_held", Flag(false)),
            ("non_concessional_within_cap", Flag(true)),
            ("total_super_balance_below_cap", Flag(true)),
        ],
        // TR 2023/1 example 5: Conchita holidays in Australia for 9 months;
        // the dates are fabricated to fit.
        "residency_tests" => vec![
            ("year", Text("2025-26")),
            ("resides", Flag(false)),
            ("domicile_in_australia", Flag(false)),
            ("permanent_place_of_abode_outside", Flag(false)),
            (
                "presence",
                Json(r#"[{"from": "2025-08-01", "to": "2026-04-30"}]"#),
            ),
            ("usual_place_of_abode_outside", Flag(true)),
            ("intends_to_take_up_residence", Flag(false)),
            ("commonwealth_super", Text("none")),
            ("resident_elsewhere", Flag(false)),
        ],
        // A resident employee for the whole of 2025-26 whose employer withheld
        // PAYG; the figures are fabricated.
        "lodgment_required" => vec![
            ("year", Text("2025-26")),
            ("residency", Json(r#"{"status": "whole_year"}"#)),
            ("taxable_income", Text("15000.00")),
            ("dividends_and_franking_credits", Text("0.00")),
            ("under_18", Json("null")),
            ("triggers", Json(r#"["payg_withheld"]"#)),
            ("study_loan", Json("null")),
            ("working_holiday_maker", Json("null")),
            ("listed_payment", Json("null")),
            ("sapto", Json("null")),
            ("child_support_parent", Flag(false)),
        ],
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
