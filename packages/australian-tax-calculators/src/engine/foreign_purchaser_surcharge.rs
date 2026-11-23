// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `foreign_purchaser_surcharge`: the foreign purchaser duty surcharge on one
//! buyer's share of one property in 2026-27, and the foreign or absentee owner
//! land tax surcharge for a full land tax year, in one of the 8 states and
//! territories. It mirrors the foreign surcharge calculator on
//! ryanduguid.github.io for those 2 amounts, takes whether the buyer or owner
//! is foreign as established, and follows the Act where they differ:
//! Queensland's surcharge from $349,999 and Tasmania's rounding to 10 cents, as
//! `land-tax` does. Every figure was read on 24 November 2026;
//! docs/calculation-evidence.md#foreign-purchaser-surcharge lists the versions.

use serde_json::Value;

use super::{
    Arg, Result, Step, d, derived_result, flag, int, money, precise, rate, refuse, scope, step,
    whole_dollars,
};
use crate::decimal::Decimal;

const KIND: &str = "foreign_purchaser_surcharge";

/// The facts as the command passes them; see the README for each.
pub struct ForeignPurchaserSurchargeArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub property_use: &'a Arg,
    pub dutiable_value: &'a Arg,
    pub foreign_purchaser: &'a Arg,
    pub land_value: &'a Arg,
    pub foreign_owner: &'a Arg,
    pub owner: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// How a jurisdiction charges its land tax surcharge.
enum Land {
    /// No foreign owner land tax surcharge.
    None(&'static str),
    /// The rate on the whole value; `tens` rounds to 10 cents (Tasmania's s 7).
    Flat {
        rate: &'static str,
        authority: &'static str,
        tens: bool,
    },
    /// The rate on the whole value once it reaches `from` (Victoria).
    From {
        from: i64,
        rate: &'static str,
        authority: &'static str,
    },
    /// The rate on each dollar over `over`, from `from` (Queensland).
    Over {
        from: i64,
        over: i64,
        rate: &'static str,
        authority: &'static str,
    },
}

struct Place {
    code: &'static str,
    /// The duty surcharge rate and authority on residential property.
    residential: Option<(&'static str, &'static str)>,
    /// The duty surcharge rate and authority on primary production property.
    primary: Option<(&'static str, &'static str)>,
    /// Why no duty surcharge is charged when none applies to the use.
    no_duty: &'static str,
    land: Land,
    /// Whether the land tax surcharge is charged on residential land only.
    land_residential_only: bool,
}

const NSW_DUTY: &str = "Duties Act 1997 (NSW) s 104U(1) and (2): surcharge purchaser duty of 9% of the dutiable value of the residential-related property, or of the share transferred to foreign persons (AustLII, as at 29 June 2026), with Revenue NSW's 'What is surcharge purchaser duty', read on 24 November 2026";
const NSW_LAND: &str = "Revenue NSW, 'What is surcharge land tax' (last updated 8 April 2026): 5% of the land value of the residential land a foreign person owns, from the 2025 land tax year, with no threshold, read on 23 and 24 November 2026; the Land Tax Act 1956 (NSW) could not be read";
const VIC_DUTY: &str = "Duties Act 2000 (Vic) s 28A(2): additional duty of 8% of the dutiable value of the land-related interest in residential property transferred to a foreign purchaser (authorised version 141), with SRO Victoria's 'Calculating foreign purchaser additional duty', read on 24 November 2026";
const VIC_LAND: &str = "Land Tax Act 2005 (Vic) Sch 1 Tables 4.5 and 1.6 for 2024 to 2033 (authorised version 086): the absentee owner rates less the general rates are 4% of the whole taxable value at every band from $50,000, and nil below it; SRO Victoria's 'Absentee individuals' (4% from the 2024 land tax year), read on 23 and 24 November 2026";
const QLD_DUTY: &str = "Duties Act 2001 (Qld) s 244(2): additional foreign acquirer duty of 8% of the dutiable value to the extent of the acquirer's interest in AFAD residential land (current as at 22 October 2026), with the Queensland Revenue Office's AFAD page, read on 24 November 2026";
const QLD_LAND: &str = "Land Tax Act 2010 (Qld) Sch 2 Pt 2 and Sch 3 Pt 2: nil below $350,000, then 3.0c for each $1 more than $349,999 (current reprint, read on 23 November 2026). The Queensland Revenue Office's pages give (taxable value - $350,000) x 3%, which is 3 cents less; the Act is applied";
const WA_DUTY: &str = "Duties Act 2008 (WA) s 205O: foreign transfer duty of 7% of the dutiable value of the foreign dutiable transaction (AustLII), with the Department of Treasury and Finance's foreign transfer duty fact sheet (22 October 2026), read on 24 November 2026";
const WA_LAND: &str = "Western Australia charges no foreign owner land tax surcharge (the Department of Treasury and Finance's 'About land tax', read on 24 November 2026)";
const SA_DUTY: &str = "Stamp Duties Act 1923 (SA) s 72(3): a foreign ownership surcharge of 7% of the value of the interest acquired in residential land (version 15.9.2026), with RevenueSA's foreign ownership surcharge page, read on 24 November 2026";
const SA_LAND: &str = "South Australia charges no foreign owner land tax surcharge (RevenueSA's land tax rates and thresholds, read on 24 November 2026)";
const TAS_DUTY: &str = "Duties Act 2001 (Tas) s 30C(2): duty of 8% of the dutiable value of residential property acquired by a foreign person, for agreements from 1 April 2020, with the State Revenue Office's 'Rates of surcharge', read on 24 November 2026";
const TAS_PRIMARY: &str = "Duties Act 2001 (Tas) s 30E(2): duty of 1.5% of the dutiable value of primary production property acquired by a foreign person, for agreements from 1 April 2020, with the State Revenue Office's 'Rates of surcharge', read on 24 November 2026";
const TAS_LAND: &str = "Land Tax Rating Act 2000 (Tas) s 6A(2): foreign investor land tax of 2% of the assessed land value of the residential land it applies to, and s 7 rounding to 10 cents, read on 23 November 2026";
const ACT_DUTY: &str = "The Taxation Administration (Amounts Payable—Duty) Determination 2026 (DI2026-155) sets no foreign purchaser rate: the ACT charges no foreign purchaser duty surcharge, read on 24 November 2026";
const ACT_LAND: &str = "Land Tax Act 2004 (ACT) s 17E(3)(b), AUV x P, with DI2026-152 s 7: P is 0.75% of the base value, read on 23 November 2026, and the ACT Revenue Office's foreign ownership surcharge page (0.75% of the AUV a year), read on 24 November 2026";
const NT_DUTY: &str = "The Stamp Duty Act 1978 (NT) charges no foreign purchaser duty surcharge (reprint REPS013), read on 24 November 2026";
const NT_LAND: &str =
    "The Northern Territory has no land tax (NT.GOV.AU, 'Land taxes'), read on 24 November 2026";
const RESIDENTIAL_ONLY: &str =
    "No duty surcharge: this jurisdiction charges it on residential property only";

const PLACES: [Place; 8] = [
    Place {
        code: "nsw",
        residential: Some(("0.09", NSW_DUTY)),
        primary: None,
        no_duty: RESIDENTIAL_ONLY,
        land: Land::Flat {
            rate: "0.05",
            authority: NSW_LAND,
            tens: false,
        },
        land_residential_only: true,
    },
    Place {
        code: "vic",
        residential: Some(("0.08", VIC_DUTY)),
        primary: None,
        no_duty: RESIDENTIAL_ONLY,
        land: Land::From {
            from: 50_000,
            rate: "0.04",
            authority: VIC_LAND,
        },
        land_residential_only: false,
    },
    Place {
        code: "qld",
        residential: Some(("0.08", QLD_DUTY)),
        primary: None,
        no_duty: RESIDENTIAL_ONLY,
        land: Land::Over {
            from: 350_000,
            over: 349_999,
            rate: "0.03",
            authority: QLD_LAND,
        },
        land_residential_only: false,
    },
    Place {
        code: "wa",
        residential: Some(("0.07", WA_DUTY)),
        primary: None,
        no_duty: RESIDENTIAL_ONLY,
        land: Land::None(WA_LAND),
        land_residential_only: false,
    },
    Place {
        code: "sa",
        residential: Some(("0.07", SA_DUTY)),
        primary: None,
        no_duty: RESIDENTIAL_ONLY,
        land: Land::None(SA_LAND),
        land_residential_only: false,
    },
    Place {
        code: "tas",
        residential: Some(("0.08", TAS_DUTY)),
        primary: Some(("0.015", TAS_PRIMARY)),
        no_duty: "No duty surcharge: Tasmania charges it on residential and primary production property only",
        land: Land::Flat {
            rate: "0.02",
            authority: TAS_LAND,
            tens: true,
        },
        land_residential_only: true,
    },
    Place {
        code: "act",
        residential: None,
        primary: None,
        no_duty: ACT_DUTY,
        land: Land::Flat {
            rate: "0.0075",
            authority: ACT_LAND,
            tens: false,
        },
        land_residential_only: true,
    },
    Place {
        code: "nt",
        residential: None,
        primary: None,
        no_duty: NT_DUTY,
        land: Land::None(NT_LAND),
        land_residential_only: false,
    },
];

const NOT_FOREIGN_PURCHASER: &str =
    "No duty surcharge: the buyer is not a foreign purchaser under the jurisdiction's test";
const NOT_FOREIGN_OWNER: &str = "No land tax surcharge: the owner is not a foreign person or absentee under the jurisdiction's test";
const LAND_RESIDENTIAL_ONLY: &str =
    "No land tax surcharge: this jurisdiction charges it on residential land only";
const BELOW_THRESHOLD: &str = "No land tax surcharge: the value is below the threshold";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Use {
    Residential,
    Primary,
    Other,
}

/// A nil step, its authority saying why.
fn nil(id: &'static str, authority: &'static str) -> Step {
    step(id, authority, "0", &[], d("0"))
}

/// The duty surcharge step, its rate added to `rates`.
fn duty(
    place: &Place,
    (property, foreign): (Use, bool),
    value: &Decimal,
    rates: &mut Rates,
) -> Result<Step> {
    let charged = match property {
        Use::Residential => place.residential,
        Use::Primary => place.primary,
        Use::Other => None,
    };
    let Some((applied, authority)) = charged else {
        return Ok(nil("duty_surcharge", place.no_duty));
    };
    if !foreign {
        return Ok(nil("duty_surcharge", NOT_FOREIGN_PURCHASER));
    }
    let applied = d(applied);
    rates.push(rate("duty_surcharge_rate", &applied));
    let formula = "dutiable_value x duty_surcharge_rate";
    let amount = value.mul(&applied)?;
    Ok(step(
        "duty_surcharge",
        authority,
        formula,
        &[value, &applied],
        amount,
    ))
}

/// Rates as a step names them.
type Rates = Vec<(String, String)>;

const LAND_ID: &str = "land_tax_surcharge";

/// The land tax surcharge on `value` under `land`, its rates added to `rates`.
fn charged(land: &Land, value: &Decimal, rates: &mut Rates) -> Result<Step> {
    match *land {
        Land::None(authority) => Ok(nil(LAND_ID, authority)),
        Land::Flat {
            rate: applied,
            authority,
            tens,
        } => flat((applied, authority), tens, value, rates),
        Land::From {
            from,
            rate: applied,
            authority,
        } => {
            if value.lt(&int(from))? {
                return Ok(nil(LAND_ID, BELOW_THRESHOLD));
            }
            flat((applied, authority), false, value, rates)
        }
        Land::Over {
            from,
            over,
            rate: applied,
            authority,
        } => {
            if value.lt(&int(from))? {
                return Ok(nil(LAND_ID, BELOW_THRESHOLD));
            }
            let [applied, over] = [d(applied), int(over)];
            rates.push(rate("land_tax_surcharge_rate", &applied));
            rates.push(rate("land_tax_surcharge_threshold", &over));
            let formula = "(land_value - land_tax_surcharge_threshold) x land_tax_surcharge_rate";
            let amount = value.sub(&over)?.mul(&applied)?;
            Ok(step(
                LAND_ID,
                authority,
                formula,
                &[value, &over, &applied],
                amount,
            ))
        }
    }
}

/// The rate on the whole value; `tens` rounds to 10 cents as Tasmania's s 7
/// does: up to the next 10 cents when more than 5 cents remain, otherwise
/// down.
fn flat(
    (applied, authority): (&str, &'static str),
    tens: bool,
    value: &Decimal,
    rates: &mut Rates,
) -> Result<Step> {
    let applied = d(applied);
    rates.push(rate("land_tax_surcharge_rate", &applied));
    let mut amount = value.mul(&applied)?;
    let mut formula = "land_value x land_tax_surcharge_rate";
    if tens {
        let tenths = amount.mul(&int(10))?.sub(&d("0.5"))?;
        amount = ceiling(&tenths)?.div(&int(10))?;
        formula = "dollars_up(land_value x land_tax_surcharge_rate x 10 - 0.5) / 10";
        if amount.is_zero() {
            // Nil land: nil, not the negative zero the ceiling gives.
            amount = d("0");
        }
    }
    Ok(step(
        LAND_ID,
        authority,
        formula,
        &[value, &applied],
        amount,
    ))
}

/// `amount` rounded up to the next whole number.
fn ceiling(amount: &Decimal) -> Result<Decimal> {
    let down = amount.quantize(0, crate::decimal::Rounding::Down)?;
    Ok(if down.lt(amount)? {
        down.add(&int(1))?
    } else {
        down
    })
}

/// The land tax surcharge step.
fn land(
    place: &Place,
    (property, foreign): (Use, bool),
    value: &Decimal,
    rates: &mut Rates,
) -> Result<Step> {
    if matches!(place.land, Land::None(_)) {
        return charged(&place.land, value, rates);
    }
    if !foreign {
        return Ok(nil(LAND_ID, NOT_FOREIGN_OWNER));
    }
    if place.land_residential_only && property != Use::Residential {
        return Ok(nil(LAND_ID, LAND_RESIDENTIAL_ONLY));
    }
    charged(&place.land, value, rates)
}

/// The jurisdiction for `jurisdiction`.
fn place(jurisdiction: &Arg) -> Result<&'static Place> {
    match PLACES.iter().find(|p| Some(p.code) == jurisdiction.text()) {
        Some(place) => Ok(place),
        None => refuse(
            "foreign_purchaser_surcharge.jurisdiction_invalid",
            "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.",
        ),
    }
}

fn property_use(given: &Arg) -> Result<Use> {
    match given.text() {
        Some("residential") => Ok(Use::Residential),
        Some("primary_production") => Ok(Use::Primary),
        Some("other") => Ok(Use::Other),
        _ => refuse(
            "foreign_purchaser_surcharge.use_invalid",
            "property_use must be residential, primary_production or other.",
        ),
    }
}

/// Whether the owner is a trust, refused for a Victorian absentee owner.
fn owner(place: &Place, given: &Arg, foreign: bool) -> Result<()> {
    let trust = match given.text() {
        Some("individual" | "company") => false,
        Some("trust") => true,
        _ => {
            return refuse(
                "foreign_purchaser_surcharge.owner_invalid",
                "owner must be individual, company or trust.",
            );
        }
    };
    if trust && foreign && place.code == "vic" {
        return refuse(
            "foreign_purchaser_surcharge.absentee_trust_unsupported",
            "A Victorian absentee trust's surcharge turns on its absentee beneficiaries' interests (Land Tax Act 2005 (Vic) s 46IA), which this worksheet does not take.",
        );
    }
    Ok(())
}

/// A whole-dollar amount of nil or more.
fn dollars(given: &Arg, what: &str) -> Result<Decimal> {
    let value = money(given)?;
    whole_dollars(&value, what)?;
    Ok(value)
}

pub fn foreign_purchaser_surcharge(args: &ForeignPurchaserSurchargeArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let place = place(args.jurisdiction)?;
    let property = property_use(args.property_use)?;
    let purchaser = flag(args.foreign_purchaser, "foreign_purchaser")?;
    let foreign_owner = flag(args.foreign_owner, "foreign_owner")?;
    owner(place, args.owner, foreign_owner)?;
    let value = dollars(args.dutiable_value, "dutiable values")?;
    if !value.gt(&int(0))? {
        return refuse(
            "foreign_purchaser_surcharge.dutiable_value_nil",
            "dutiable_value must be more than nil.",
        );
    }
    let land_value = dollars(args.land_value, "land values")?;
    let (steps, rates) = precise(|| {
        let mut rates = Vec::new();
        let duty = duty(place, (property, purchaser), &value, &mut rates)?;
        let land = land(place, (property, foreign_owner), &land_value, &mut rates)?;
        Ok((vec![duty, land], rates))
    })?;
    derived_result(KIND, &year, steps, rates)
}
