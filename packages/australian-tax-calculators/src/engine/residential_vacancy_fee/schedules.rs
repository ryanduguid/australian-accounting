// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use crate::dates;

/// De minimis consideration threshold (Fees Regs s 53(1)).
pub(super) const DE_MINIMIS_THRESHOLD: i64 = 75_000;

/// Fee constant for acquiring an interest in residential land (Fees Regs s 9
/// item 1).
pub(super) const RESIDENTIAL_FEE_CONSTANT: i64 = 1_000_000;

/// Verified floor for the Fees Imposition Act 2015 s 6(3) statutory fee cap in
/// 2026-27 (and later unindexed years, since s 7(3) prevents downward
/// indexation): $7,000,000 x 1.113 = $7,791,000.
pub(super) const CAP_FLOOR_2026_27: i64 = 7_791_000;

/// Verified Fees Imposition Regulations 2020 residential fee parameters by
/// inclusive effective date range (`2021-01-01` to `2027-06-30`).
#[derive(Clone, Copy)]
pub(super) struct Schedule {
    pub(super) de_minimis: i64,
    pub(super) tier_one: i64,
    pub(super) tier_step: i64,
    pub(super) residential_cap: i64,
    pub(super) established_triple: bool,
}

// Each tuple: (from, to, [de_minimis, tier_one, tier_step, residential_cap], established_triple).
const SCHEDULE_ROWS: [(&str, &str, [i64; 4], bool); 9] = [
    (
        "2021-01-01",
        "2021-06-30",
        [2_000, 6_350, 12_700, 500_000],
        false,
    ),
    (
        "2021-07-01",
        "2022-06-30",
        [2_000, 6_350, 12_700, 503_000],
        false,
    ),
    (
        "2022-07-01",
        "2022-07-28",
        [2_000, 6_600, 13_200, 522_500],
        false,
    ),
    (
        "2022-07-29",
        "2023-06-30",
        [4_000, 13_200, 26_400, 1_045_000],
        false,
    ),
    (
        "2023-07-01",
        "2024-04-08",
        [4_200, 14_100, 28_200, 1_119_100],
        false,
    ),
    (
        "2024-04-09",
        "2024-06-30",
        [4_200, 14_100, 28_200, 1_119_100],
        true,
    ),
    (
        "2024-07-01",
        "2025-06-30",
        [4_300, 14_700, 29_500, 1_171_600],
        true,
    ),
    (
        "2025-07-01",
        "2026-06-30",
        [4_500, 15_100, 30_300, 1_205_200],
        true,
    ),
    (
        "2026-07-01",
        "2027-06-30",
        [4_600, 15_600, 31_300, 1_245_500],
        true,
    ),
];

/// Verified Fees Imposition Act 2015 s 6(3) statutory fee cap (enforced by
/// Fees Regulations s 61A) by inclusive effective date range.
const STATUTORY_CAPS: [(&str, &str, i64); 8] = [
    ("2021-01-01", "2021-06-30", 1_000_000),
    ("2021-07-01", "2022-06-30", 1_006_000),
    ("2022-07-01", "2023-06-30", 1_045_000),
    ("2023-07-01", "2024-04-08", 1_119_100),
    ("2024-04-09", "2024-06-30", 7_000_000),
    ("2024-07-01", "2025-06-30", 7_329_000),
    ("2025-07-01", "2026-06-30", 7_539_000),
    ("2026-07-01", "2027-06-30", CAP_FLOOR_2026_27),
];

pub(super) fn parse_day(text: &str) -> i64 {
    dates::parse(text).expect("valid date literal")
}

pub(super) fn schedule_for(day: i64) -> Option<Schedule> {
    SCHEDULE_ROWS
        .iter()
        .find(|(from, to, _, _)| (parse_day(from)..=parse_day(to)).contains(&day))
        .map(|(_, _, fees, established_triple)| Schedule {
            de_minimis: fees[0],
            tier_one: fees[1],
            tier_step: fees[2],
            residential_cap: fees[3],
            established_triple: *established_triple,
        })
}

pub(super) fn statutory_cap_for(day: i64) -> Option<i64> {
    STATUTORY_CAPS
        .iter()
        .find(|(from, to, _)| (parse_day(from)..=parse_day(to)).contains(&day))
        .map(|(_, _, cap)| *cap)
}
