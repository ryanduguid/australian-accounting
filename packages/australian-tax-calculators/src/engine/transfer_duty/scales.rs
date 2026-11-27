// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Each jurisdiction's general transfer duty scale and first home buyer
//! concession for 2026-27, as its Act, determination or revenue office states
//! them, read on 24 November 2026 (docs/calculation-evidence.md#transfer-duty).

/// How a band charges the value over its threshold.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Charge {
    /// `rate` for every $100, or part of $100, over `over`.
    PerHundred,
    /// `rate` (a fraction) of the value over `over`.
    Excess,
    /// `rate` (a fraction) of the whole value.
    WholeValue,
    /// Stamp Duty Act 1978 (NT) Sch 1 cl 1(2)(a): 0.06571441 V^2 + 15 V,
    /// V being the value divided by 1,000.
    Territory,
}

/// Values from `from` up to the next band's `from` pay `base` plus `rate`
/// charged as `charge` on the value over `over`.
pub(super) struct Band {
    pub from: i64,
    pub base: &'static str,
    pub rate: &'static str,
    pub over: i64,
    pub charge: Charge,
}

pub(super) struct Scale {
    pub authority: &'static str,
    pub bands: &'static [Band],
}

/// A first home buyer concession on a home, as its Act sets it.
pub(super) enum FirstHome {
    /// No concession: the general rate applies.
    None(&'static str),
    /// No duty, whatever the value.
    Exempt(&'static str),
    /// Duties Act 1997 (NSW) s 78A: nil to `exempt_to`; below `limit`, the
    /// general duty less (limit - value) / (limit - exempt_to) of the duty on
    /// `exempt_to`, the bracket rounded to the cent.
    PhaseOut {
        authority: &'static str,
        exempt_to: i64,
        limit: i64,
    },
    /// Duties Act 2000 (Vic) s 57JA: nil to `exempt_to`; to `limit`, the
    /// general duty times (value - exempt_to) / (limit - exempt_to).
    Share {
        authority: &'static str,
        exempt_to: i64,
        limit: i64,
    },
    /// Duties Act 2008 (WA) Sch 2 Div 2: nil to `exempt_to`; to `limit`,
    /// `rate` for every $100, or part of $100, over `exempt_to`.
    PerHundred {
        authority: &'static str,
        exempt_to: i64,
        limit: i64,
        rate: &'static str,
    },
    /// Duties Act 2001 (Qld) ss 91 and 92: the home concession duty less the
    /// Sch 4A amount.
    Queensland,
}

/// The rounding an Act sets for its general duty, beyond the cent.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Rounding {
    Cents,
    /// Duties Act 2000 (Vic) s 28(1): to the nearest whole dollar, 50 cents
    /// down.
    DollarHalfDown,
}

pub(super) struct Place {
    pub code: &'static str,
    pub general: Scale,
    /// Duties Act 1997 (NSW) s 32A: the premium band for residential land
    /// and its authority.
    pub premium: Option<(&'static Band, &'static str)>,
    /// Duties Act 1997 (NSW) s 273: the minimum amount of duty.
    pub minimum: Option<&'static str>,
    pub rounding: Rounding,
    pub established: FirstHome,
    pub new: FirstHome,
}

const fn band(
    from: i64,
    base: &'static str,
    rate: &'static str,
    over: i64,
    charge: Charge,
) -> Band {
    Band {
        from,
        base,
        rate,
        over,
        charge,
    }
}

const fn hundred(from: i64, base: &'static str, rate: &'static str, over: i64) -> Band {
    band(from, base, rate, over, Charge::PerHundred)
}

const NSW_GENERAL: &str = "Duties Act 1997 (NSW) ss 32 and 33AC to 33AF (AustLII, as at 29 June 2026), with Revenue NSW's 2026-27 thresholds and rates ('How to calculate transfer duty', last updated 6 October 2026): $1.25 for every $100 to $18,000, $225 plus $1.50 over $18,000, $525 plus $1.75 over $38,000, $1,662 plus $3.50 over $103,000, $11,602 plus $4.50 over $387,000, $52,237 plus $5.50 over $1,290,000, each for every $100 or part, with s 273's $20 minimum, read on 24 November 2026";
const NSW_PREMIUM: &str = "Duties Act 1997 (NSW) s 32A (AustLII, as at 29 June 2026), with Revenue NSW's 2026-27 premium threshold: residential land over $3,870,000, $194,137 plus $7.00 for every $100, or part, over $3,870,000, read on 24 November 2026";
const NSW_FIRST_HOME: &str = "Duties Act 1997 (NSW) ss 74(3)(a) and 78A (AustLII, as at 29 June 2026), and Revenue NSW's 'First Home Buyers Assistance Scheme' (contracts from 1 July 2023): no duty to $800,000; above $800,000 and below $1,000,000, N - ((1,000,000 - V) / 200,000 x D), D the s 32 duty on $800,000, the bracket rounded to the cent as Revenue NSW's calculator does, with s 273's $20 minimum, read on 24 November 2026";

const NSW_PREMIUM_BAND: Band = hundred(3_870_001, "194137", "7.00", 3_870_000);

const NSW: Place = Place {
    code: "nsw",
    general: Scale {
        authority: NSW_GENERAL,
        bands: &[
            hundred(0, "0", "1.25", 0),
            hundred(18_001, "225", "1.50", 18_000),
            hundred(38_001, "525", "1.75", 38_000),
            hundred(103_001, "1662", "3.50", 103_000),
            hundred(387_001, "11602", "4.50", 387_000),
            hundred(1_290_001, "52237", "5.50", 1_290_000),
        ],
    },
    premium: Some((&NSW_PREMIUM_BAND, NSW_PREMIUM)),
    minimum: Some("20"),
    rounding: Rounding::Cents,
    established: FirstHome::PhaseOut {
        authority: NSW_FIRST_HOME,
        exempt_to: 800_000,
        limit: 1_000_000,
    },
    new: FirstHome::PhaseOut {
        authority: NSW_FIRST_HOME,
        exempt_to: 800_000,
        limit: 1_000_000,
    },
};

const VIC_GENERAL: &str = "Duties Act 2000 (Vic) s 28(1), authorised version 141 (in force from 24 June 2026): 1.4% to $25,000, $350 plus 2.4% over $25,000, $2,870 plus 6% over $130,000, 5.5% of the whole value over $960,000 to $2,000,000, $110,000 plus 6.5% over $2,000,000, chargeable to the nearest whole dollar (50 cents down), and the State Revenue Office's non-principal place of residence rates (contracts from 1 July 2021), read on 24 November 2026";
const VIC_FIRST_HOME: &str = "Duties Act 2000 (Vic) s 57JA, authorised version 141: an exemption on a PPR transfer to $600,000, and above $600,000 to $750,000 the concessional amount (A - $600,000) / $150,000 x B, B the duty payable but for the section, read on 24 November 2026";

const VIC: Place = Place {
    code: "vic",
    general: Scale {
        authority: VIC_GENERAL,
        bands: &[
            band(0, "0", "0.014", 0, Charge::Excess),
            band(25_001, "350", "0.024", 25_000, Charge::Excess),
            band(130_001, "2870", "0.06", 130_000, Charge::Excess),
            band(960_001, "0", "0.055", 0, Charge::WholeValue),
            band(2_000_001, "110000", "0.065", 2_000_000, Charge::Excess),
        ],
    },
    premium: None,
    minimum: None,
    rounding: Rounding::DollarHalfDown,
    established: FirstHome::Share {
        authority: VIC_FIRST_HOME,
        exempt_to: 600_000,
        limit: 750_000,
    },
    new: FirstHome::Share {
        authority: VIC_FIRST_HOME,
        exempt_to: 600_000,
        limit: 750_000,
    },
};

const QLD_GENERAL: &str = "Duties Act 2001 (Qld) s 24(2) and Sch 3 (current as at 22 October 2026): nil to $5,000, $1.50 for each $100 or part over $5,000, $1,050 plus $3.50 over $75,000, $17,325 plus $4.50 over $540,000, $38,025 plus $5.75 over $1m, and the Queensland Revenue Office's 'Transfer duty rates', read on 24 November 2026";
pub(super) const QLD_HOME: &str = "Duties Act 2001 (Qld) s 91(3) and (5): to $350,000, $1 for each $100 or part; above $350,000, $3,500 plus the Sch 3 duty on the value less the Sch 3 duty on $350,000, read on 24 November 2026";
pub(super) const QLD_FIRST_HOME: &str = "Duties Act 2001 (Qld) s 92(2) and Sch 4A (liability from 9 June 2024): the s 91 duty less the concession amount, $17,350 (or the s 91 duty if less) to $709,999.99, then $15,615 falling by $1,735 for each $10,000 to nil from $800,000, and the Queensland Revenue Office's 'Home concession rates', read on 24 November 2026";
const QLD_NEW: &str = "Duties Act 2001 (Qld) s 92A(3) (liability from 1 May 2025): the Sch 3 duty less the Sch 3 rate applied to the residential land, nil where the dutiable property is the residential land alone, with no cap on its value, read on 24 November 2026";

const QLD_SCHEDULE_3: &[Band] = &[
    hundred(0, "0", "0", 0),
    hundred(5_001, "0", "1.50", 5_000),
    hundred(75_001, "1050", "3.50", 75_000),
    hundred(540_001, "17325", "4.50", 540_000),
    hundred(1_000_001, "38025", "5.75", 1_000_000),
];

/// Sch 4A: (lowest whole-dollar value, concession amount); nil from $800,000.
pub(super) const QLD_SCHEDULE_4A: [(i64, &str); 11] = [
    (0, "17350"),
    (710_000, "15615"),
    (720_000, "13880"),
    (730_000, "12145"),
    (740_000, "10410"),
    (750_000, "8675"),
    (760_000, "6940"),
    (770_000, "5205"),
    (780_000, "3470"),
    (790_000, "1735"),
    (800_000, "0"),
];

const QLD: Place = Place {
    code: "qld",
    general: Scale {
        authority: QLD_GENERAL,
        bands: QLD_SCHEDULE_3,
    },
    premium: None,
    minimum: None,
    rounding: Rounding::Cents,
    established: FirstHome::Queensland,
    new: FirstHome::Exempt(QLD_NEW),
};

const WA_GENERAL: &str = "Duties Act 2008 (WA) Sch 2 Div 1 (AustLII, with No. 18 of 2026): $1.90 for every $100 or part to $120,000, $2,280 plus $2.85 over $120,000, $3,135 plus $3.80 over $150,000, $11,115 plus $4.75 over $360,000, $28,453 plus $5.15 over $725,000, and the Department of Treasury and Finance's 'Transfer duty assessment', read on 24 November 2026";
const WA_FIRST_HOME: &str = "Duties Act 2008 (WA) s 143 and Sch 2 Div 2, a property that includes a home (agreements from 7 May 2026): nil to $600,000, $16.15 for every $100 or part over $600,000 to $800,000, and the 'Duties Fact Sheet - First Home Owner Rate' (last updated 27 August 2026), read on 24 November 2026";

const WA: Place = Place {
    code: "wa",
    general: Scale {
        authority: WA_GENERAL,
        bands: &[
            hundred(0, "0", "1.90", 0),
            hundred(120_001, "2280", "2.85", 120_000),
            hundred(150_001, "3135", "3.80", 150_000),
            hundred(360_001, "11115", "4.75", 360_000),
            hundred(725_001, "28453", "5.15", 725_000),
        ],
    },
    premium: None,
    minimum: None,
    rounding: Rounding::Cents,
    established: FirstHome::PerHundred {
        authority: WA_FIRST_HOME,
        exempt_to: 600_000,
        limit: 800_000,
        rate: "16.15",
    },
    new: FirstHome::PerHundred {
        authority: WA_FIRST_HOME,
        exempt_to: 600_000,
        limit: 800_000,
        rate: "16.15",
    },
};

const SA_GENERAL: &str = "Stamp Duties Act 1923 (SA) Sch 2 item 3(b) (version 15.9.2026): $1 for every $100 or part to $12,000, then $120, $480, $1,080, $2,830, $6,830, $8,955, $11,330 and $21,330 plus $2, $3, $3.50, $4, $4.25, $4.75, $5 and $5.50 for every $100 or part over $12,000, $30,000, $50,000, $100,000, $200,000, $250,000, $300,000 and $500,000, and RevenueSA's 'Rate of stamp duty', read on 24 November 2026";
const SA_NEW: &str = "Stamp Duties Act 1923 (SA) s 71DD(2)(b) (version 15.9.2026): no duty on the conveyance of a new home under a contract from 6 June 2024, whatever its value, and RevenueSA's 'Relief available', read on 24 November 2026";
const SA_ESTABLISHED: &str = "Stamp Duties Act 1923 (SA) s 71DD: relief for a new home or vacant land only; an established home pays the general rate, read on 24 November 2026";

const SA: Place = Place {
    code: "sa",
    general: Scale {
        authority: SA_GENERAL,
        bands: &[
            hundred(0, "0", "1.00", 0),
            hundred(12_001, "120", "2.00", 12_000),
            hundred(30_001, "480", "3.00", 30_000),
            hundred(50_001, "1080", "3.50", 50_000),
            hundred(100_001, "2830", "4.00", 100_000),
            hundred(200_001, "6830", "4.25", 200_000),
            hundred(250_001, "8955", "4.75", 250_000),
            hundred(300_001, "11330", "5.00", 300_000),
            hundred(500_001, "21330", "5.50", 500_000),
        ],
    },
    premium: None,
    minimum: None,
    rounding: Rounding::Cents,
    established: FirstHome::None(SA_ESTABLISHED),
    new: FirstHome::Exempt(SA_NEW),
};

const TAS_GENERAL: &str = "Duties Act 2001 (Tas) s 29(1) (current, legislation.tas.gov.au): $50 to $3,000, then $50, $435, $1,560, $5,935, $12,935 and $27,810 plus $1.75, $2.25, $3.50, $4, $4.25 and $4.50 for every $100 or part over $3,000, $25,000, $75,000, $200,000, $375,000 and $725,000, read on 24 November 2026";
const TAS_NONE: &str = "Duties Act 2001 (Tas) ss 57C and 57G: the first home buyer exemption for an established home has an eligible period ending 30 June 2026, and the State Revenue Office says it is not available for transactions settling after 30 June 2026; no first home concession applies in 2026-27, read on 24 November 2026";

const TAS: Place = Place {
    code: "tas",
    general: Scale {
        authority: TAS_GENERAL,
        bands: &[
            hundred(0, "50", "0", 0),
            hundred(3_001, "50", "1.75", 3_000),
            hundred(25_001, "435", "2.25", 25_000),
            hundred(75_001, "1560", "3.50", 75_000),
            hundred(200_001, "5935", "4.00", 200_000),
            hundred(375_001, "12935", "4.25", 375_000),
            hundred(725_001, "27810", "4.50", 725_000),
        ],
    },
    premium: None,
    minimum: None,
    rounding: Rounding::Cents,
    established: FirstHome::None(TAS_NONE),
    new: FirstHome::None(TAS_NONE),
};

const ACT_GENERAL: &str = "Taxation Administration (Amounts Payable—Duty) Determination 2026 (DI2026-155, commenced 1 July 2026) s 5(1)(b) and Table 2, for property used wholly for purposes other than a commercial purpose and a transaction that is not an eligible owner occupier transaction: $1.20 for every $100 or part to $200,000, then $2,400, $4,600, $11,400, $22,200 and $36,950 plus $2.20, $3.40, $4.32, $5.90 and $6.40 for every $100 or part over $200,000, $300,000, $500,000, $750,000 and $1m, and over $1,455,000 $4.54 per $100 of the whole value, read on 24 November 2026";
const ACT_HOME: &str = "Taxation Administration (Amounts Payable—Home Buyer Concession Scheme) Determination 2026 (DI2026-157, commenced 1 July 2026) s 7: a nil rate of duty on an eligible transaction (a home or vacant land, transaction date from 1 July 2026), with no income test or value limit, read on 24 November 2026";

const ACT: Place = Place {
    code: "act",
    general: Scale {
        authority: ACT_GENERAL,
        bands: &[
            hundred(0, "0", "1.20", 0),
            hundred(200_001, "2400", "2.20", 200_000),
            hundred(300_001, "4600", "3.40", 300_000),
            hundred(500_001, "11400", "4.32", 500_000),
            hundred(750_001, "22200", "5.90", 750_000),
            hundred(1_000_001, "36950", "6.40", 1_000_000),
            band(1_455_001, "0", "0.0454", 0, Charge::WholeValue),
        ],
    },
    premium: None,
    minimum: None,
    rounding: Rounding::Cents,
    established: FirstHome::Exempt(ACT_HOME),
    new: FirstHome::Exempt(ACT_HOME),
};

const NT_GENERAL: &str = "Stamp Duty Act 1978 (NT) Sch 1 cl 1(2) (reprint REPS013, as in force at 1 July 2025): to $525,000, D = (0.06571441 x V^2) + 15V, V being 1/1,000 of the dutiable value; above $525,000 and below $3,000,000, 4.95%; $3,000,000 or more and below $5,000,000, 5.75%; $5,000,000 or more, 5.95% of the dutiable value, read on 24 November 2026";
const NT_NONE: &str = "Stamp Duty Act 1978 (NT) s 88A: the home incentive schemes in Part 5 Division 2 apply to conveyances first executed on or before 30 June 2021; no first home concession applies in 2026-27 (the house and land package exemption is not a first home concession and is not applied), read on 24 November 2026";

const NT: Place = Place {
    code: "nt",
    general: Scale {
        authority: NT_GENERAL,
        bands: &[
            band(0, "0", "0", 0, Charge::Territory),
            band(525_001, "0", "0.0495", 0, Charge::WholeValue),
            band(3_000_000, "0", "0.0575", 0, Charge::WholeValue),
            band(5_000_000, "0", "0.0595", 0, Charge::WholeValue),
        ],
    },
    premium: None,
    minimum: None,
    rounding: Rounding::Cents,
    established: FirstHome::None(NT_NONE),
    new: FirstHome::None(NT_NONE),
};

pub(super) const PLACES: [Place; 8] = [NSW, VIC, QLD, WA, SA, TAS, ACT, NT];

#[cfg(test)]
mod tests {
    use super::{Charge, band, hundred};

    #[test]
    fn band_helpers_keep_each_field() {
        let built = band(18_000, "225", "0.015", 18_000, Charge::Excess);
        assert_eq!(
            (built.from, built.base, built.rate, built.over),
            (18_000, "225", "0.015", 18_000)
        );
        assert!(built.charge == Charge::Excess);
        let per_hundred = hundred(38_000, "525", "1.75", 38_000);
        assert_eq!((per_hundred.from, per_hundred.base), (38_000, "525"));
        assert!(per_hundred.charge == Charge::PerHundred);
    }
}
