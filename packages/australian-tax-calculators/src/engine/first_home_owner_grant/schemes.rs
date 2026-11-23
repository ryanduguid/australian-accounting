// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Each jurisdiction's grant rules for `first_home_owner_grant`, with the
//! provision or page each rests on.

/// One amount for contracts made from `from` until the next rule's `from`. `bought`,
/// `built` and `north` are the highest eligible whole-dollar value; `None`
/// is no cap. `established` is the amount for an established home.
pub(super) struct Rule {
    pub(super) from: &'static str,
    pub(super) amount: i64,
    pub(super) established: i64,
    pub(super) bought: Option<i64>,
    pub(super) built: Option<i64>,
    pub(super) north: Option<i64>,
    pub(super) authority: &'static str,
}

pub(super) struct Scheme {
    pub(super) code: &'static str,
    pub(super) name: &'static str,
    pub(super) residence: &'static str,
    /// Whether the source read gives the lesser of the consideration and
    /// the amount.
    pub(super) lesser: bool,
    pub(super) rules: &'static [Rule],
}

const fn rule(
    from: &'static str,
    amount: i64,
    (bought, built, north): (Option<i64>, Option<i64>, Option<i64>),
    authority: &'static str,
) -> Rule {
    Rule {
        from,
        amount,
        established: 0,
        bought,
        built,
        north,
        authority,
    }
}

pub(super) static SCHEMES: [Scheme; 8] = [
    Scheme {
        code: "nsw",
        name: "New South Wales",
        residence: "each applicant occupies the home as their principal place of residence for at least 12 continuous months, starting within 12 months of the purchase or construction",
        lesser: false,
        rules: &[rule(
            "2016-01-01",
            10_000,
            (Some(600_000), Some(750_000), None),
            "Revenue NSW, First Home Owner (New Homes) Grant: $10,000; a price not over $600,000 for a home bought, and land, building contract and variations not over $750,000 for a home built",
        )],
    },
    Scheme {
        code: "vic",
        name: "Victoria",
        residence: "at least one applicant occupies the home as their principal place of residence for at least 12 months, starting within 12 months of settlement or of the build finishing",
        lesser: true,
        rules: &[rule(
            "2013-07-01",
            10_000,
            (Some(750_000), Some(750_000), None),
            "First Home Owner Grant and Home Buyer Schemes Act 2000 (Vic) s 18(1A): the lesser of the consideration and $10,000; s 13(1A): consideration not over $750,000",
        )],
    },
    Scheme {
        code: "qld",
        name: "Queensland",
        residence: "each applicant occupies the home as their principal place of residence for a continuous period of at least 6 months, starting within 1 year after completion of the transaction (s 15)",
        lesser: true,
        rules: &[rule(
            "2023-11-20",
            30_000,
            (Some(749_999), Some(749_999), None),
            "First Home Owner Grant and Other Home Owner Grants Act 2000 (Qld) s 20(1): the lesser of the consideration and $30,000; s 5(6) to (8): not eligible at $750,000 or more",
        )],
    },
    Scheme {
        code: "wa",
        name: "Western Australia",
        residence: "each applicant occupies the home as their principal place of residence for a continuous period of at least 6 months, starting within 12 months of completion of the transaction",
        lesser: true,
        rules: &[
            rule(
                "2000-07-01",
                10_000,
                (Some(750_000), Some(750_000), Some(1_000_000)),
                "First Home Owner Grant Act 2000 (WA) s 19(2CA): the lesser of $10,000 and the consideration; s 14AD: $750,000 south and $1,000,000 north of the 26th parallel",
            ),
            rule(
                "2026-05-07",
                10_000,
                (Some(800_000), Some(800_000), Some(1_000_000)),
                "First Home Owner Grant Act 2000 (WA) s 19(2CA): the lesser of $10,000 and the consideration; DTF fact sheet as at 28 July 2026: $800,000 south of the 26th parallel from 7 May 2026, $1,000,000 north (s 14AD)",
            ),
        ],
    },
    Scheme {
        code: "sa",
        name: "South Australia",
        residence: "all applicants occupy the home as their principal place of residence for a continuous period of at least 6 months, starting within 12 months after completion of the transaction",
        lesser: true,
        rules: &[rule(
            "2024-06-06",
            15_000,
            (None, None, None),
            "First Home and Housing Construction Grants Act 2000 (SA) s 18(1): the lesser of the consideration and $15,000; s 7(1a): no value cap from 6 June 2024",
        )],
    },
    Scheme {
        code: "tas",
        name: "Tasmania",
        residence: "each applicant occupies the home as their principal place of residence for a continuous period of at least 6 months, starting within 12 months of completion of the transaction",
        lesser: true,
        rules: &[
            rule(
                "2025-07-01",
                30_000,
                (None, None, None),
                "First Home Owner Grant Act 2000 (Tas) s 18(1) and (2)(al): the lesser of the consideration and $30,000 for a transaction satisfying s 18L; s 7 sets no value cap",
            ),
            rule(
                "2026-07-01",
                20_000,
                (None, None, None),
                "First Home Owner Grant Act 2000 (Tas) s 18(1) and (2)(am): the lesser of the consideration and $20,000 for a transaction satisfying s 18M; s 7 sets no value cap",
            ),
        ],
    },
    Scheme {
        code: "act",
        name: "Australian Capital Territory",
        residence: "none, as no grant is paid",
        lesser: true,
        rules: &[rule(
            "2019-07-01",
            0,
            (None, None, None),
            "First Home Owner Grant Act 2000 (ACT) s 18: the amount the Minister determines; ACT Revenue Office: no grant for a commencement date from 1 July 2019",
        )],
    },
    Scheme {
        code: "nt",
        name: "Northern Territory",
        residence: "each applicant occupies the home as their principal place of residence for a continuous period of at least 12 months, starting within 12 months after completion of the transaction (s 12(2))",
        lesser: true,
        rules: &[
            Rule {
                established: 10_000,
                ..rule(
                    "2024-10-01",
                    50_000,
                    (None, None, None),
                    "First Home Owner Grant Act 2000 (NT) s 18(1) and (3D): the lesser of the consideration and $50,000 for a new home or $10,000 for an established one (s 13(1)(a)(iii)); no value cap (s 13AA)",
                )
            },
            rule(
                "2025-10-01",
                50_000,
                (None, None, None),
                "First Home Owner Grant Act 2000 (NT) s 18(1) and (3E): the lesser of the consideration and $50,000, the HomeGrown Territory Grant, for a new home (s 13(1)(a)(iv)); no value cap (s 13AA)",
            ),
        ],
    },
];
