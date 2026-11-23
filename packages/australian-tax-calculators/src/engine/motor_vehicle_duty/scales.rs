// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Each jurisdiction's passenger vehicle duty scale as its Act, determination
//! or revenue office states it for 2026-27, read on 24 November 2026
//! (docs/calculation-evidence.md#motor-vehicle-duty). Western Australia's
//! sliding rate is worked in the parent module.

use std::borrow::Cow;

/// Values from `from` (whole dollars) up to the next band's `from` pay `base`
/// plus `rate` for every `per` dollars, or part, of the value over `over`. A
/// band whose rate is "0" is a fixed amount, `base`.
#[derive(Clone)]
pub(super) struct Band {
    pub from: i64,
    pub base: &'static str,
    pub rate: &'static str,
    pub over: i64,
}

pub(super) struct Scale {
    pub authority: &'static str,
    /// The dollars each `rate` is charged on: 100, or 200 in Victoria.
    pub per: i64,
    /// The least amount payable, where the Act sets one.
    pub minimum: Option<&'static str>,
    pub bands: Cow<'static, [Band]>,
}

pub(super) const fn band(from: i64, base: &'static str, rate: &'static str, over: i64) -> Band {
    Band {
        from,
        base,
        rate,
        over,
    }
}

pub(super) const NSW: Scale = Scale {
    authority: "Revenue NSW, 'Motor vehicle duty' (last updated 16 October 2025): passenger vehicles up to $44,999, $3 for every $100; $45,000 or more, $1,350 plus $5 for every $100; and Service NSW, 'Check motor vehicle stamp duty': $3 per $100, or part thereof, and for passenger vehicles valued over $45,000, $1,350 plus $5 per $100, or part thereof, of the value over $45,000; read on 24 November 2026 (Duties Act 1997 (NSW) Ch 9; the legislation site was not readable)",
    per: 100,
    minimum: None,
    bands: Cow::Borrowed(&[band(0, "0", "3", 0), band(45_001, "1350", "5", 45_000)]),
};

const VIC_TABLE: &str = "Duties Act 2000 (Vic) s 218(1) Table and (5) (authorised version 141, 24 June 2026): per $200, or part, of the dutiable value of a passenger car; the luxury car tax threshold for 2026-27 is $80,809 (ATO, 'Luxury car tax rate and thresholds', updated 1 June 2026; SRO Victoria, 'Motor vehicle duty (current rates)' from 1 July 2026), read on 24 November 2026";

/// Items 3 to 6: other passenger cars.
pub(super) const VIC: Scale = Scale {
    authority: VIC_TABLE,
    per: 200,
    minimum: None,
    bands: Cow::Borrowed(&[
        band(0, "0", "8.40", 0),
        band(80_810, "0", "10.40", 0),
        band(100_001, "0", "14.00", 0),
        band(150_001, "0", "18.00", 0),
    ]),
};

/// Items 1 and 2: a green car or a primary producer passenger car.
pub(super) const VIC_CONCESSION: Scale = Scale {
    authority: "Duties Act 2000 (Vic) s 218(1) Table items 1 and 2 and s 3(1) 'green car' (combined tailpipe CO2 emissions not exceeding 120 g/km) and 'primary producer passenger car' (authorised version 141, 24 June 2026): $8.40 per $200, or part, of the dutiable value, read on 24 November 2026",
    per: 200,
    minimum: None,
    bands: Cow::Borrowed(&[band(0, "0", "8.40", 0)]),
};

const QLD_AUTHORITY: &str = "Duties Act 2001 (Qld) s 383(1) and Sch 4C (current as at 22 October 2026): the Schedule 4C rate for each $100, and each part of $100, of the dutiable value, plus $2 for each $100, and each part of $100, of the whole dutiable value if it is more than $100,000 (a vehicle with a GVM of 4.5t or less, s 383(2)), read on 24 November 2026";

/// Queensland, by the Schedule 4C row: hybrid or electric; 1 to 4 cylinders,
/// 2 rotors or steam; 5 or 6 cylinders or 3 rotors; 7 or more cylinders.
pub(super) const QLD: [(&str, Scale); 4] = [
    (
        "hybrid_or_electric",
        Scale {
            authority: QLD_AUTHORITY,
            per: 100,
            minimum: None,
            bands: Cow::Borrowed(&[band(0, "0", "2", 0), band(100_001, "0", "4", 0)]),
        },
    ),
    (
        "up_to_4_cylinders",
        Scale {
            authority: QLD_AUTHORITY,
            per: 100,
            minimum: None,
            bands: Cow::Borrowed(&[band(0, "0", "3", 0), band(100_001, "0", "5", 0)]),
        },
    ),
    (
        "5_or_6_cylinders",
        Scale {
            authority: QLD_AUTHORITY,
            per: 100,
            minimum: None,
            bands: Cow::Borrowed(&[band(0, "0", "3.50", 0), band(100_001, "0", "5.50", 0)]),
        },
    ),
    (
        "7_or_more_cylinders",
        Scale {
            authority: QLD_AUTHORITY,
            per: 100,
            minimum: None,
            bands: Cow::Borrowed(&[band(0, "0", "4", 0), band(100_001, "0", "6", 0)]),
        },
    ),
];

pub(super) const WA: &str = "Duties Act 2008 (WA) s 236(1) and (2) (03-z0-00, as at 22 October 2026): for a vehicle that is not a heavy vehicle, 2.75% of a dutiable value to $25,000; over $25,000 to $50,000, R% where R = 2.75 + (DV - 25,000) / 6,666.66 rounded to 2 decimal places; over $50,000, 6.5%; the amount rounded down to the nearest 5 cents; read on 24 November 2026";

pub(super) const SA: Scale = Scale {
    authority: "Stamp Duties Act 1923 (SA) Sch 2 cl 2(1)(a) and (c) (version 15.9.2026): a motor vehicle that is not a commercial motor vehicle or trailer, $1 for every $100 or fractional part to $1,000; $10 plus $2 over $1,000; $30 plus $3 over $2,000; $60 plus $4 over $3,000; at least $5; read on 24 November 2026",
    per: 100,
    minimum: Some("5"),
    bands: Cow::Borrowed(&[
        band(0, "0", "1", 0),
        band(1_001, "10", "2", 1_000),
        band(2_001, "30", "3", 2_000),
        band(3_001, "60", "4", 3_000),
    ]),
};

pub(super) const TAS: Scale = Scale {
    authority: "Duties Act 2001 (Tas) s 197(2), (3) and (4) (in force 24 November 2026): a motor vehicle of $600 or less, $20; a passenger vehicle to $35,000, $3 per $100, or part; to $40,000, $1,050 plus $11 for every $100 or part over $35,000; over $40,000, $4 for every $100 or part of the whole value; read on 24 November 2026",
    per: 100,
    minimum: None,
    bands: Cow::Borrowed(&[
        band(0, "20", "0", 0),
        band(601, "0", "3", 0),
        band(35_001, "1050", "11", 35_000),
        band(40_001, "0", "4", 0),
    ]),
};

pub(super) const TAS_FLEET: Scale = Scale {
    authority: "Duties Act 2001 (Tas) s 197(3A) and (6) (in force 24 November 2026): a new motor vehicle, other than a heavy vehicle, for which a manufacturer's fleet discount has been provided, $3.50 per $100, or part, read on 24 November 2026",
    per: 100,
    minimum: None,
    bands: Cow::Borrowed(&[band(0, "0", "3.50", 0)]),
};

pub(super) const NT: Scale = Scale {
    authority: "Stamp Duty Act 1978 (NT) Sch 1 cl 6 (as in force at 1 July 2025): $3 for every $100 or fractional part of $100 of the dutiable value of the motor vehicle, read on 24 November 2026",
    per: 100,
    minimum: None,
    bands: Cow::Borrowed(&[band(0, "0", "3", 0)]),
};

pub(super) const NT_ELECTRIC: Scale = Scale {
    authority: "Stamp Duty Act 1978 (NT) s 92A and Sch 1 cl 6 (as in force at 1 July 2025): for an electric motor vehicle registered from 1 July 2022 to 30 June 2027, the first $50,000 of the dutiable value is exempt (at most $1,500), and the rest pays $3 for every $100 or fractional part; read on 24 November 2026",
    per: 100,
    minimum: None,
    bands: Cow::Borrowed(&[band(0, "0", "0", 0), band(50_001, "0", "3", 50_000)]),
};

/// One ACT row: the Table 1 rate per $100, the Table 2 base and rate over
/// $45,000, and the Table 3 base over $80,000 (whose rate is $8 in every row).
pub(super) struct ActRow {
    pub table_1: &'static str,
    pub table_2_base: &'static str,
    pub table_2_rate: &'static str,
    pub table_3_base: &'static str,
}

const fn row(
    table_1: &'static str,
    table_2_base: &'static str,
    table_2_rate: &'static str,
    table_3_base: &'static str,
) -> ActRow {
    ActRow {
        table_1,
        table_2_base,
        table_2_rate,
        table_3_base,
    }
}

/// An ACT determination's rows for categories AAA, AA, A, B, C and D; a
/// non-rated vehicle, and a used vehicle other than Category AAA, pays C's.
pub(super) struct ActPeriod {
    pub authority: &'static str,
    pub rows: [ActRow; 6],
}

/// Duty dates from 1 July 2026 to 31 January 2027.
pub(super) const ACT_DI2025_152: ActPeriod = ActPeriod {
    authority: "Duties Act 1999 (ACT) s 208 (republication 79) and the Taxation Administration (Amounts Payable—Motor Vehicle Duty) Determination 2025 (DI2025-152) s 6 Tables 1 to 3, column 3 (on or after 1 September 2025): for every $100, or part of $100, of the dutiable value under $45,000; from $45,000 a base plus a rate over $45,000; from $80,000 a base plus $8 over $80,000; read on 24 November 2026",
    rows: [
        row("2.50", "1125.00", "4.00", "2525.00"),
        row("2.67", "1201.50", "4.41", "2745.00"),
        row("2.84", "1278.00", "4.81", "2961.50"),
        row("3.00", "1350.00", "5.22", "3177.00"),
        row("3.17", "1426.50", "5.62", "3393.50"),
        row("4.53", "2038.50", "7.81", "4772.00"),
    ],
};

/// Duty dates from 1 February 2027.
pub(super) const ACT_DI2026_154: ActPeriod = ActPeriod {
    authority: "Duties Act 1999 (ACT) s 208 (republication 79) and the Taxation Administration (Amounts Payable—Motor Vehicle Duty) Determination 2026 (DI2026-154, commencing 1 February 2027) s 6 Tables 1 to 3: for every $100, or part of $100, of the dutiable value under $45,000; from $45,000 a base plus a rate over $45,000; from $80,000 a base plus $8 over $80,000; read on 24 November 2026",
    rows: [
        row("2.50", "1125.00", "4.00", "2525.00"),
        row("2.67", "1201.50", "4.41", "2745.00"),
        row("2.84", "1278.00", "4.81", "2961.50"),
        row("3.25", "1462.50", "5.59", "3419.00"),
        row("3.72", "1674.00", "6.44", "3928.00"),
        row("4.83", "2173.50", "8.00", "4973.50"),
    ],
};
