// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `land_tax`: one owner's land tax for a full land tax year in one of the 7
//! jurisdictions that levy it, on a total taxable value the operator states,
//! at the general or trust rates, with a foreign or absentee owner's surcharge
//! and Western Australia's metropolitan region improvement tax. It mirrors the
//! land tax calculator on ryanduguid.github.io and follows the Act where they
//! differ: Queensland's surcharge from $349,999, South Australia's $20 minimum
//! and Tasmania's rounding to 10 cents. Every figure was read on 23 November
//! 2026; docs/calculation-evidence.md#land-tax lists the versions read.

use serde_json::Value;

use super::trade_in::when;
use super::{
    Arg, Result, Step, cents, d, derived_result, flag, int, money, precise, rate, refuse, scope,
    step, whole_dollars,
};
use crate::decimal::{Decimal, Rounding};

mod scales;
use scales::{Band, MRIT, PLACES, Payable, Place, Scale, Surcharge};

const KIND: &str = "land_tax";

/// The facts as the command passes them; see the README for each.
pub struct LandTaxArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub taxable_value: &'a Arg,
    pub trust_rates: &'a Arg,
    pub foreign_owner: &'a Arg,
    pub surcharge_value: Option<&'a Arg>,
    pub metropolitan_region: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The owner's choices once each is checked against the jurisdiction.
struct Choices {
    trust: bool,
    foreign: bool,
    /// The value a separate surcharge applies to; nil where none does.
    separate: Decimal,
    metro: bool,
}

/// A whole-dollar value the scales apply to.
fn dollars(value: &Arg, what: &str) -> Result<Decimal> {
    let amount = money(value)?;
    whole_dollars(&amount, what)?;
    Ok(amount)
}

/// The band of `scale` that holds `value`: the last whose `from` it reaches.
fn band(scale: &'static Scale, value: &Decimal) -> Result<&'static Band> {
    let mut found = &scale.bands[0];
    for band in scale.bands {
        if !value.lt(&int(band.from))? {
            found = band;
        }
    }
    Ok(found)
}

/// Tasmania's s 7: cents not divisible by 10 round up to the next 10 cents if
/// more than 5 cents remain, otherwise down.
fn tens(amount: &Decimal) -> Result<Decimal> {
    let cents = amount.mul(&int(100))?;
    let rest = cents.rem(&int(10))?;
    let mut kept = cents.sub(&rest)?;
    if rest.gt(&int(5))? {
        kept = kept.add(&int(10))?;
    }
    Ok(kept.div(&int(100))?)
}

/// An Act's rounding of the amount payable, beyond cents, on `amount` and its
/// `formula`.
fn payable(rule: Payable, amount: Decimal, formula: String) -> Result<(Decimal, String)> {
    Ok(match rule {
        Payable::Cents => (amount, formula),
        Payable::TenCents => (tens(&amount)?, format!("tens({formula})")),
        Payable::NilBelowTwenty => {
            let kept = if amount.lt(&int(20))? { int(0) } else { amount };
            (kept, format!("nil_below_20({formula})"))
        }
    })
}

/// Rates as a step names them: `prefix` before each of the band's figures.
type Rates = Vec<(String, String)>;

/// The band's charge on `excess`, the value over its threshold, and its
/// formula: per dollar, or for every $100 or part of $100.
fn charge(
    scale: &Scale,
    excess: &Decimal,
    applied: &Decimal,
    prefix: &str,
) -> Result<(Decimal, String)> {
    if !scale.per_hundred {
        let formula = format!("{prefix}base + (taxable_value - {prefix}threshold) x {prefix}rate");
        return Ok((excess.mul(applied)?, formula));
    }
    let lots = excess.div(&int(100))?;
    let mut whole = lots.quantize(0, Rounding::Down)?;
    if whole.lt(&lots)? {
        whole = whole.add(&int(1))?;
    }
    let formula = format!(
        "{prefix}base + dollars_up((taxable_value - {prefix}threshold) / 100) x {prefix}rate_per_100"
    );
    Ok((whole.mul(applied)?, formula))
}

/// `id`, worked on `value` from the band of `scale` that holds it, with any
/// fixed charge and the jurisdiction's rounding of the amount payable, and the
/// band's figures as rates named with `prefix`.
fn scale_step(
    (id, prefix): (&'static str, &str),
    scale: &'static Scale,
    value: &Decimal,
    (fixed, rule): (Option<&str>, Payable),
) -> Result<(Step, Rates)> {
    let band = band(scale, value)?;
    let [base, applied, over] = [d(band.base), d(band.rate), int(band.over)];
    let (charged, mut formula) = charge(scale, &value.sub(&over)?, &applied, prefix)?;
    let per = if scale.per_hundred {
        "rate_per_100"
    } else {
        "rate"
    };
    let mut amount = base.add(&charged)?;
    let mut rates = vec![
        rate(&format!("{prefix}base"), band.base),
        rate(&format!("{prefix}{per}"), band.rate),
        rate(&format!("{prefix}threshold"), band.over),
    ];
    let fixed = fixed.map(d);
    let mut operands = vec![&base, value, &over, &applied];
    if let Some(charge) = &fixed {
        amount = amount.add(charge)?;
        formula = format!("fixed_charge + {formula}");
        operands.insert(0, charge);
        rates.push(rate("fixed_charge", charge));
    }
    let (amount, formula) = payable(rule, amount, formula)?;
    let step = step(id, scale.authority, &formula, &operands, amount);
    Ok((step, rates))
}

/// The jurisdiction for `jurisdiction`; the Northern Territory has none.
fn place(jurisdiction: &Arg) -> Result<&'static Place> {
    match PLACES.iter().find(|p| Some(p.code) == jurisdiction.text()) {
        Some(place) => Ok(place),
        None => refuse(
            "land_tax.jurisdiction_invalid",
            "jurisdiction must be nsw, vic, qld, wa, sa, tas or act; the Northern Territory levies no land tax (NT Government, 'Property taxes').",
        ),
    }
}

/// Whether all the land is in the Perth metropolitan region: asked in Western
/// Australia only, where land partly in it is not worked.
fn metro(place: &Place, given: Option<&Arg>) -> Result<bool> {
    let why = "in Western Australia (Metropolitan Region Improvement Tax Act 1959 (WA))";
    match when(place.code == "wa", given, "metropolitan_region", why)?.map(Arg::text) {
        None | Some(Some("none")) => Ok(false),
        Some(Some("all")) => Ok(true),
        Some(Some("part")) => refuse(
            "land_tax.metropolitan_region_part",
            "Land partly in the Perth metropolitan region is not worked: the metropolitan region improvement tax applies to the value of the land in the region only.",
        ),
        Some(_) => refuse(
            "land_tax.metropolitan_region_invalid",
            "metropolitan_region must be all, none or part.",
        ),
    }
}

/// The owner's choices, refused where the jurisdiction has no rates for them.
fn choices(place: &Place, args: &LandTaxArgs) -> Result<Choices> {
    let trust = flag(args.trust_rates, "trust_rates")?;
    let foreign = flag(args.foreign_owner, "foreign_owner")?;
    if trust && place.trust.is_none() {
        return refuse(
            "land_tax.trust_rates_unavailable",
            "trust_rates is true, but this worksheet has trust rates only for nsw (a special trust), vic (trust surcharge rates), qld (companies and trustees) and sa (land held on trust); Western Australia taxes a trustee's land at the general rates.",
        );
    }
    if foreign && trust && place.code == "vic" {
        return refuse(
            "land_tax.absentee_trust_unsupported",
            "A Victorian absentee trust's surcharge turns on its absentee beneficiaries' interests (Land Tax Act 2005 (Vic) s 46IA), which this worksheet does not take.",
        );
    }
    if foreign && place.surcharge.is_none() {
        return refuse(
            "land_tax.surcharge_unavailable",
            "foreign_owner is true, but this worksheet has a foreign or absentee owner surcharge only for nsw, vic, qld, tas and act.",
        );
    }
    let separate = foreign && matches!(place.surcharge, Some(Surcharge::Separate(..)));
    let why = "for a foreign owner in nsw or tas, whose surcharge applies to a value apart from the taxable value";
    let separate = when(separate, args.surcharge_value, "surcharge_value", why)?
        .map_or(Ok(int(0)), |value| dollars(value, "surcharge values"))?;
    let metro = metro(place, args.metropolitan_region)?;
    Ok(Choices {
        trust,
        foreign,
        separate,
        metro,
    })
}

/// The scale the land tax itself is worked on.
fn main_scale(place: &'static Place, c: &Choices) -> &'static Scale {
    match (&place.trust, &place.absentee) {
        (Some(trust), _) if c.trust => trust,
        (_, Some(absentee)) if c.foreign => absentee,
        _ => &place.general,
    }
}

/// The surcharge's steps after `land`, its rate added to `rates`.
fn surcharge(
    place: &'static Place,
    value: &Decimal,
    c: &Choices,
    land: &Step,
    rates: &mut Rates,
) -> Result<Vec<Step>> {
    let Some(kind) = &place.surcharge else {
        return Ok(Vec::new());
    };
    Ok(match kind {
        Surcharge::Separate(applied, authority) => {
            let applied = d(applied);
            let formula = "surcharge_value x surcharge_rate".to_string();
            let amount = c.separate.mul(&applied)?;
            let (amount, formula) = payable(place.payable, amount, formula)?;
            rates.push(rate("surcharge_rate", &applied));
            let operands = [&c.separate, &applied];
            vec![step("surcharge", authority, &formula, &operands, amount)]
        }
        Surcharge::OnValue(scale) => {
            let named = ("surcharge", "surcharge_");
            let (step, applied) = scale_step(named, scale, value, (None, Payable::Cents))?;
            rates.extend(applied);
            vec![step]
        }
        Surcharge::Difference(general, trust) => {
            let with = if c.trust { trust } else { general };
            let named = ("absentee_rate_tax", "absentee_");
            let (absentee, applied) = scale_step(named, with, value, (None, place.payable))?;
            rates.extend(applied);
            let amount = absentee.value.sub(&land.value)?;
            let operands = [&absentee.value, &land.value];
            let formula = "absentee_rate_tax - land_tax";
            let surcharge = step("surcharge", with.authority, formula, &operands, amount);
            vec![absentee, surcharge]
        }
    })
}

/// Tasmania's FILTS land is residential general land, so for one owner its
/// value is part of the taxable value. Section 7 rounds 'the amount of land
/// tax payable', and FILTS is a component of land tax assessed folio by folio
/// (Land Tax Act 2000 (Tas) ss 16A and 16B(1)): whether s 7 rounds each amount
/// or their sum is unsettled, so a case where the 2 readings differ is refused.
fn tasmanian_surcharge(place: &'static Place, value: &Decimal, c: &Choices) -> Result<()> {
    let Some(Surcharge::Separate(applied, _)) = &place.surcharge else {
        return Ok(());
    };
    if place.payable != Payable::TenCents || !c.foreign {
        return Ok(());
    }
    if c.separate.gt(value)? {
        return refuse(
            "land_tax.surcharge_value_above_taxable_value",
            "surcharge_value is more than taxable_value: in Tasmania the foreign investor land tax applies to residential general land, part of the owner's taxable land.",
        );
    }
    let (land, _) = scale_step(
        ("land_tax", ""),
        main_scale(place, c),
        value,
        (None, Payable::Cents),
    )?;
    let surcharge = c.separate.mul(&d(applied))?;
    let apart = tens(&land.value)?.add(&tens(&surcharge)?)?;
    let together = tens(&land.value.add(&surcharge)?)?;
    if apart.equals(&together)? {
        return Ok(());
    }
    refuse(
        "land_tax.rounding_unsettled",
        format!(
            "Rounding the land tax and the foreign investor land tax to 10 cents each gives {apart}, rounding their sum gives {together}; which Land Tax Rating Act 2000 (Tas) s 7 requires is unsettled."
        ),
    )
}

const TOTAL: &str = "The land tax and any surcharge and metropolitan region improvement tax, each to the cent, for the full land tax year";

/// Every step, with `total` last, and the rates applied.
fn worked(place: &'static Place, value: &Decimal, c: &Choices) -> Result<(Vec<Step>, Rates)> {
    let main = main_scale(place, c);
    let named = ("land_tax", "");
    let (land, mut rates) = scale_step(named, main, value, (place.fixed, place.payable))?;
    let mut steps = vec![];
    if c.foreign {
        steps = surcharge(place, value, c, &land, &mut rates)?;
    }
    steps.insert(0, land);
    if c.metro {
        let (mrit, applied) = scale_step(("mrit", "mrit_"), &MRIT, value, (None, Payable::Cents))?;
        rates.extend(applied);
        steps.push(mrit);
    }
    let charged: Vec<&Step> = steps
        .iter()
        .filter(|s| s.static_id() != "absentee_rate_tax")
        .collect();
    let names: Vec<String> = charged
        .iter()
        .map(|s| format!("cents({})", s.static_id()))
        .collect();
    let operands: Vec<&Decimal> = charged.iter().map(|s| &s.value).collect();
    let mut sum = int(0);
    for s in &charged {
        sum = sum.add(&cents(s.value.clone())?)?;
    }
    let total = step("total", TOTAL, &names.join(" + "), &operands, sum);
    steps.push(total);
    Ok((steps, rates))
}

pub fn land_tax(args: &LandTaxArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let place = place(args.jurisdiction)?;
    if year != place.year {
        let kind = if place.year == "2026" {
            "a calendar year"
        } else {
            "a financial year"
        };
        return refuse(
            "period.unsupported",
            format!(
                "Unsupported period '{year}' for {}; supported: {}, its land tax year ({kind}).",
                place.code, place.year
            ),
        );
    }
    let c = choices(place, args)?;
    let value = dollars(args.taxable_value, "taxable values")?;
    let (steps, rates) = precise(|| {
        tasmanian_surcharge(place, &value, &c)?;
        worked(place, &value, &c)
    })?;
    derived_result(KIND, &year, steps, rates)
}
