// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Each jurisdiction's land tax scales as its Act or revenue office states
//! them, read on 23 November 2026 (docs/calculation-evidence.md#land-tax).

/// Values from `from` up to the next band's `from` pay `base` plus `rate` of
/// the value over `over`: per dollar, or for a per-$100 scale for every $100
/// or part of $100.
pub(super) struct Band {
    pub from: i64,
    pub base: &'static str,
    pub rate: &'static str,
    pub over: i64,
}

pub(super) struct Scale {
    pub authority: &'static str,
    pub per_hundred: bool,
    pub bands: &'static [Band],
}

/// How a foreign or absentee owner's surcharge is worked.
pub(super) enum Surcharge {
    /// `rate` of a value the operator states apart from the taxable value.
    Separate(&'static str, &'static str),
    /// A scale on the taxable value.
    OnValue(Scale),
    /// Victoria: the absentee owner rates (general, then trust) less the
    /// rates without the surcharge, at the same value.
    Difference(Scale, Scale),
}

/// Rounding an Act sets for the amount payable, beyond cents.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Payable {
    Cents,
    /// Land Tax Rating Act 2000 (Tas) s 7.
    TenCents,
    /// Land Tax Act 1936 (SA) s 6.
    NilBelowTwenty,
}

pub(super) struct Place {
    pub code: &'static str,
    pub year: &'static str,
    pub fixed: Option<&'static str>,
    pub general: Scale,
    pub trust: Option<Scale>,
    /// The scale a foreign or absentee owner pays instead of `general`.
    pub absentee: Option<Scale>,
    pub surcharge: Option<Surcharge>,
    pub payable: Payable,
}

const fn band(from: i64, base: &'static str, rate: &'static str, over: i64) -> Band {
    Band {
        from,
        base,
        rate,
        over,
    }
}

const NIL: Band = band(0, "0", "0", 0);

const fn scale(authority: &'static str, bands: &'static [Band]) -> Scale {
    Scale {
        authority,
        per_hundred: false,
        bands,
    }
}

const NSW_GENERAL: &str = "Revenue NSW, 'Land tax thresholds and rates' (last updated 16 March 2026): for the 2024 land tax year onwards, more than $1,075,000 but less than $6,571,000, $100 + 1.6% of the land value above $1,075,000; more than $6,571,000, $88,036 + 2% above $6,571,000, read on 23 November 2026";
const NSW_TRUST: &str = "Revenue NSW, 'How trusts are assessed': a special trust does not receive the threshold and is taxed at a flat 1.6% up to the premium threshold, then 2% on the value above $6,571,000, read on 23 November 2026";
const NSW_SURCHARGE: &str = "Revenue NSW, 'What is surcharge land tax' (last updated 8 April 2026): 5% of the land value of the residential land a foreign person owns, from the 2025 land tax year, with no threshold, read on 23 November 2026";

const NSW: Place = Place {
    code: "nsw",
    year: "2026",
    fixed: None,
    general: scale(
        NSW_GENERAL,
        &[
            NIL,
            band(1_075_001, "100", "0.016", 1_075_000),
            band(6_571_001, "88036", "0.02", 6_571_000),
        ],
    ),
    trust: Some(scale(
        NSW_TRUST,
        &[
            band(0, "0", "0.016", 0),
            band(6_571_001, "105136", "0.02", 6_571_000),
        ],
    )),
    absentee: None,
    surcharge: Some(Surcharge::Separate("0.05", NSW_SURCHARGE)),
    payable: Payable::Cents,
};

const VIC_GENERAL: &str = "Land Tax Act 2005 (Vic) Sch 1 cl 1.6 and Table 1.6 (2024 to 2033), authorised version 086 (1 May 2026), and the State Revenue Office's 'Land tax (current rates)', read on 23 November 2026";
const VIC_TRUST: &str = "Land Tax Act 2005 (Vic) Sch 1 cl 3.6 and Table 3.6, trust surcharge rates for 2024 to 2033, authorised version 086, read on 23 November 2026";
const VIC_ABSENTEE: &str = "Land Tax Act 2005 (Vic) Sch 1 cl 4.5 and Table 4.5, absentee owner rates for 2024 to 2033, authorised version 086, read on 23 November 2026";
const VIC_ABSENTEE_TRUST: &str = "Land Tax Act 2005 (Vic) Sch 1 cl 5.5 and Table 5.5, absentee trust rates for 2024 to 2033, authorised version 086, read on 23 November 2026";

const VIC: Place = Place {
    code: "vic",
    year: "2026",
    fixed: None,
    general: scale(
        VIC_GENERAL,
        &[
            NIL,
            band(50_000, "500", "0", 50_000),
            band(100_000, "975", "0", 100_000),
            band(300_000, "1350", "0.003", 300_000),
            band(600_000, "2250", "0.006", 600_000),
            band(1_000_000, "4650", "0.009", 1_000_000),
            band(1_800_000, "11850", "0.0165", 1_800_000),
            band(3_000_000, "31650", "0.0265", 3_000_000),
        ],
    ),
    trust: Some(scale(
        VIC_TRUST,
        &[
            NIL,
            band(25_000, "82", "0.00375", 25_000),
            band(50_000, "676", "0.00375", 50_000),
            band(100_000, "1338", "0.00375", 100_000),
            band(250_000, "1901", "0.00675", 250_000),
            band(600_000, "4263", "0.00975", 600_000),
            band(1_000_000, "8163", "0.01275", 1_000_000),
            band(1_800_000, "18363", "0.011072", 1_800_000),
            band(3_000_000, "31650", "0.0265", 3_000_000),
        ],
    )),
    absentee: None,
    surcharge: Some(Surcharge::Difference(
        scale(
            VIC_ABSENTEE,
            &[
                NIL,
                band(50_000, "2500", "0.04", 50_000),
                band(100_000, "4975", "0.04", 100_000),
                band(300_000, "13350", "0.043", 300_000),
                band(600_000, "26250", "0.046", 600_000),
                band(1_000_000, "44650", "0.049", 1_000_000),
                band(1_800_000, "83850", "0.0565", 1_800_000),
                band(3_000_000, "151650", "0.0665", 3_000_000),
            ],
        ),
        scale(
            VIC_ABSENTEE_TRUST,
            &[
                NIL,
                band(25_000, "1082", "0.04375", 25_000),
                band(50_000, "2676", "0.04375", 50_000),
                band(100_000, "5338", "0.04375", 100_000),
                band(250_000, "11901", "0.04675", 250_000),
                band(600_000, "28263", "0.04975", 600_000),
                band(1_000_000, "48163", "0.05275", 1_000_000),
                band(1_800_000, "90363", "0.051072", 1_800_000),
                band(3_000_000, "151650", "0.0665", 3_000_000),
            ],
        ),
    )),
    payable: Payable::Cents,
};

const QLD_GENERAL: &str = "Land Tax Act 2010 (Qld) s 32(1)(a) and Sch 1, rates for individuals other than absentees and trustees (current reprint), with the Queensland Revenue Office's 'Land tax threshold and rates for individuals', read on 23 November 2026";
const QLD_TRUST: &str = "Land Tax Act 2010 (Qld) s 32(1)(b)(i) and Sch 2 Pt 1, the general rate for companies and trustees (current reprint), read on 23 November 2026";
const QLD_ABSENTEE: &str = "Land Tax Act 2010 (Qld) s 32(1)(c)(i) and Sch 3 Pt 1, the general rate for absentees (current reprint), read on 23 November 2026";
const QLD_SURCHARGE: &str = "Land Tax Act 2010 (Qld) s 32(1)(b)(ii) and (c)(ii) and Sch 2 Pt 2 and Sch 3 Pt 2: nil below $350,000, then 3.0c for each $1 more than $349,999 (current reprint, read on 23 November 2026). The Queensland Revenue Office's pages give (taxable value - $350,000) x 3%, which is 3 cents less; the Act is applied";

const QLD: Place = Place {
    code: "qld",
    year: "2026-27",
    fixed: None,
    general: scale(
        QLD_GENERAL,
        &[
            NIL,
            band(600_000, "500", "0.01", 600_000),
            band(1_000_000, "4500", "0.0165", 1_000_000),
            band(3_000_000, "37500", "0.0125", 3_000_000),
            band(5_000_000, "62500", "0.0175", 5_000_000),
            band(10_000_000, "150000", "0.0225", 10_000_000),
        ],
    ),
    trust: Some(scale(
        QLD_TRUST,
        &[
            NIL,
            band(350_000, "1450", "0.017", 350_000),
            band(2_250_000, "33750", "0.015", 2_250_000),
            band(5_000_000, "75000", "0.0225", 5_000_000),
            band(10_000_000, "187500", "0.0275", 10_000_000),
        ],
    )),
    absentee: Some(scale(
        QLD_ABSENTEE,
        &[
            NIL,
            band(350_000, "1450", "0.017", 350_000),
            band(2_250_000, "33750", "0.015", 2_250_000),
            band(5_000_000, "75000", "0.02", 5_000_000),
            band(10_000_000, "175000", "0.025", 10_000_000),
        ],
    )),
    surcharge: Some(Surcharge::OnValue(scale(
        QLD_SURCHARGE,
        &[NIL, band(350_000, "0", "0.03", 349_999)],
    ))),
    payable: Payable::Cents,
};

const WA_GENERAL: &str = "Land Tax Act 2002 (WA) s 5 Table 11, land tax rates for 2015/16 and subsequent financial years (compilation 02-d0-04), with the Department of Treasury and Finance's 'Land tax assessment' (last updated 20 May 2026), read on 23 November 2026";

const WA: Place = Place {
    code: "wa",
    year: "2026-27",
    fixed: None,
    general: scale(
        WA_GENERAL,
        &[
            NIL,
            band(300_001, "300", "0", 300_000),
            band(420_001, "300", "0.0025", 420_000),
            band(1_000_001, "1750", "0.009", 1_000_000),
            band(1_800_001, "8950", "0.018", 1_800_000),
            band(5_000_001, "66550", "0.02", 5_000_000),
            band(11_000_001, "186550", "0.0267", 11_000_000),
        ],
    ),
    trust: None,
    absentee: None,
    surcharge: None,
    payable: Payable::Cents,
};

/// Metropolitan region improvement tax.
pub(super) static MRIT: Scale = scale(
    "Metropolitan Region Improvement Tax Act 1959 (WA) s 10 Table 3, for 2009/10 and subsequent years of assessment: nil to $300,000, then 0.14 cent for each $1 in excess of $300,000 (compilation 04-a0-08), read on 23 November 2026",
    &[NIL, band(300_001, "0", "0.0014", 300_000)],
);

const SA_GENERAL: &str = "Land Tax Act 1936 (SA) s 8A(1) and Sch 1 Pt 2 cl 3 (version 4.12.2025), with thresholds A to D for 2026-27 of $936,000, $1,504,000, $2,188,000 and $3,504,000 (RevenueSA's 'Rates and thresholds', 2026-27 general rates), each charged for every $100 or fractional part of $100, read on 23 November 2026; s 6: where the total would be less than $20, no land tax is payable";
const SA_TRUST: &str = "Land Tax Act 1936 (SA) s 8A(1a) and Sch 1 Pt 3 cl 5 (version 4.12.2025), with the 2026-27 thresholds (RevenueSA's 'Rates and thresholds', 2026-27 trust rates), each charged for every $100 or fractional part of $100, read on 23 November 2026; s 6: where the total would be less than $20, no land tax is payable";

const SA: Place = Place {
    code: "sa",
    year: "2026-27",
    fixed: None,
    general: Scale {
        authority: SA_GENERAL,
        per_hundred: true,
        bands: &[
            NIL,
            band(936_001, "0", "0.50", 936_000),
            band(1_504_001, "2840", "1.00", 1_504_000),
            band(2_188_001, "9680", "2.00", 2_188_000),
            band(3_504_001, "36000", "2.40", 3_504_000),
        ],
    },
    trust: Some(Scale {
        authority: SA_TRUST,
        per_hundred: true,
        bands: &[
            NIL,
            band(25_001, "125", "0.50", 25_000),
            band(936_001, "4680", "1.00", 936_000),
            band(1_504_001, "10360", "1.50", 1_504_000),
            band(2_188_001, "20620", "2.40", 2_188_000),
            band(3_504_001, "52204", "2.40", 3_504_000),
        ],
    }),
    absentee: None,
    surcharge: None,
    payable: Payable::NilBelowTwenty,
};

const TAS_GENERAL: &str = "Land Tax Rating Act 2000 (Tas) s 6 and Sch 1 (current, authorised 12 July 2024), with the State Revenue Office's 'Rates of land tax' from 1 July 2025, read on 23 November 2026; s 7: an amount in cents not divisible by 10 rounds up to the next 10 cents if more than 5 cents remain, otherwise down";
const TAS_SURCHARGE: &str = "Land Tax Rating Act 2000 (Tas) s 6A(2): foreign investor land tax of 2% of the assessed land value of the land it applies to, and s 7 rounding to 10 cents, with the State Revenue Office's 'Rate of surcharge' and foreign investor land tax surcharge guideline (December 2025), read on 23 November 2026";

const TAS: Place = Place {
    code: "tas",
    year: "2026-27",
    fixed: None,
    general: scale(
        TAS_GENERAL,
        &[
            NIL,
            band(125_000, "50", "0.0045", 125_000),
            band(500_000, "1737.50", "0.015", 500_000),
        ],
    ),
    trust: None,
    absentee: None,
    surcharge: Some(Surcharge::Separate("0.02", TAS_SURCHARGE)),
    payable: Payable::TenCents,
};

const ACT_GENERAL: &str = "Land Tax Act 2004 (ACT) s 9(3)(b), FC + (AUV x P) (republication 34, 26 December 2025), with the Taxation Administration (Amounts Payable—Land Tax) Determination 2026 (DI2026-152) s 6: FC $1,778 and Table 1, from 1 July 2026, read on 23 November 2026";
const ACT_SURCHARGE: &str = "Land Tax Act 2004 (ACT) s 17E(3)(b), AUV x P, with DI2026-152 s 7: P is 0.75% of the base value, read on 23 November 2026";

const ACT: Place = Place {
    code: "act",
    year: "2026-27",
    fixed: Some("1778"),
    general: scale(
        ACT_GENERAL,
        &[
            band(0, "0", "0.0054", 0),
            band(150_001, "810", "0.0064", 150_000),
            band(275_001, "1610", "0.0124", 275_000),
            band(1_000_001, "10600", "0.0125", 1_000_000),
            band(2_000_001, "23100", "0.0126", 2_000_000),
        ],
    ),
    trust: None,
    absentee: None,
    surcharge: Some(Surcharge::OnValue(scale(
        ACT_SURCHARGE,
        &[band(0, "0", "0.0075", 0)],
    ))),
    payable: Payable::Cents,
};

pub(super) static PLACES: [Place; 7] = [NSW, VIC, QLD, WA, SA, TAS, ACT];

#[cfg(test)]
mod tests {
    use super::{NIL, band, scale};

    #[test]
    fn scale_helpers_keep_each_field_and_charge_per_dollar() {
        let built = band(1_075_000, "100", "0.016", 1_075_000);
        assert_eq!(
            (built.from, built.base, built.rate, built.over),
            (1_075_000, "100", "0.016", 1_075_000)
        );
        let general = scale("authority", &[NIL]);
        assert_eq!(general.authority, "authority");
        assert!(!general.per_hundred);
        assert_eq!(general.bands.len(), 1);
    }
}
