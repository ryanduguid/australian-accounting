// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `vic_property_taxes`: one of 3 Victorian taxes on land for the 2025 or
//! 2026 tax year, on values the operator states. Vacant residential land tax
//! (Land Tax Act 2005 (Vic) Part 2 Division 6 and s 35(3)), the commercial and
//! industrial property tax (Commercial and Industrial Property Tax Reform Act
//! 2024 (Vic)), nil in both years with the land's first tax year, and windfall
//! gains tax (Windfall Gains Tax Act 2021 (Vic)) on a rezoning in either year.
//! Every figure was read on 23 and 24 November 2026;
//! docs/calculation-evidence.md#vic-property-taxes lists the versions.

use serde_json::{Map, Value};

use super::trade_in::when;
use super::{
    Arg, Result, Step, cents, d, derived_result, flag, int, money, precise, rate, refuse, scope,
    step, whole_dollars,
};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "vic_property_taxes";

/// The facts as the command passes them; see the README for each.
pub struct VicPropertyTaxesArgs<'a> {
    pub tax: &'a Arg,
    pub capital_improved_value: Option<&'a Arg>,
    pub residential_land: Option<&'a Arg>,
    pub new_residential_concession: Option<&'a Arg>,
    pub liable_previous_tax_year: Option<&'a Arg>,
    pub liable_tax_year_before: Option<&'a Arg>,
    pub entry_date: Option<&'a Arg>,
    pub parcels: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tax {
    Vacant,
    Commercial,
    Windfall,
}

type Rates = Vec<(String, String)>;

const VRLT_2B: &str = "Land Tax Act 2005 (Vic) ss 19(1A), 34A, 37A and 35(3)(a): 1% of the capital improved value of undeveloped land in a residential zone (s 34B(2B))";
const VRLT_88EB: &str = "Land Tax Act 2005 (Vic) ss 19(1A), 34A, 37A and 88EB: the concessional 1% of the capital improved value of new residential land unused, unoccupied and unsold";
const VRLT_1: &str = "Land Tax Act 2005 (Vic) ss 19(1A), 34A, 37A and 35(3)(b)(i): 1% of the capital improved value, the land not liable in the preceding tax year";
const VRLT_2: &str = "Land Tax Act 2005 (Vic) ss 19(1A), 34A, 37A and 35(3)(b)(ii): 2% of the capital improved value, the land liable in the preceding tax year but not the one before";
const VRLT_3: &str = "Land Tax Act 2005 (Vic) ss 19(1A), 34A, 37A and 35(3)(b)(iii): 3% of the capital improved value, the land liable in the last 2 preceding tax years";
const CIPT: &str = "Commercial and Industrial Property Tax Reform Act 2024 (Vic) ss 3 and 17: no land is CIPT taxable land while within its 10-year transition period, which starts on its entry date (on or after 1 July 2024, s 2)";
const UPLIFT: &str = "Windfall Gains Tax Act 2021 (Vic) s 11: value uplift = CIV2 - CIV1, the title's capital improved value after the rezoning less before it";
const TAXABLE: &str = "Windfall Gains Tax Act 2021 (Vic) ss 10 and 16(1) and (3): the taxable value uplift aggregated over the land the rezoning rezoned, negative uplifts ignored; no deductions are prescribed";
const WGT: &str = "Windfall Gains Tax Act 2021 (Vic) s 9: nil to $100,000; 62.5% of the part over $100,000 below $500,000; 50% of the whole uplift from $500,000";
const TOTAL: &str = "The tax for the year, to the cent";

fn tax(value: &Arg) -> Result<Tax> {
    match value.text() {
        Some("vacant_residential_land") => Ok(Tax::Vacant),
        Some("commercial_industrial") => Ok(Tax::Commercial),
        Some("windfall_gains") => Ok(Tax::Windfall),
        _ => refuse(
            "vic_property_taxes.tax_invalid",
            "tax must be vacant_residential_land, commercial_industrial or windfall_gains.",
        ),
    }
}

/// A whole-dollar value.
fn dollars(value: &Arg, what: &str) -> Result<Decimal> {
    let amount = money(value)?;
    whole_dollars(&amount, what)?;
    Ok(amount)
}

/// A tax's own inputs, each with its name.
type Inputs<'a> = &'a [(Option<&'a Arg>, &'a str)];

/// The inputs of the taxes not chosen are refused.
fn refuse_others(chosen: Tax, args: &VicPropertyTaxesArgs) -> Result<()> {
    let groups: [(Tax, &str, Inputs); 3] = [
        (
            Tax::Vacant,
            "for vacant_residential_land",
            &[
                (args.capital_improved_value, "capital_improved_value"),
                (args.residential_land, "residential_land"),
                (
                    args.new_residential_concession,
                    "new_residential_concession",
                ),
                (args.liable_previous_tax_year, "liable_previous_tax_year"),
                (args.liable_tax_year_before, "liable_tax_year_before"),
            ],
        ),
        (
            Tax::Commercial,
            "for commercial_industrial",
            &[(args.entry_date, "entry_date")],
        ),
        (
            Tax::Windfall,
            "for windfall_gains",
            &[(args.parcels, "parcels")],
        ),
    ];
    for (tax, why, inputs) in groups {
        if tax != chosen {
            for (given, key) in inputs {
                when(false, *given, key, why)?;
            }
        }
    }
    Ok(())
}

/// Whether the land is undeveloped (s 34B(2B)) and whether it is a vacant
/// home (s 34B(1)), refusing undeveloped land before 2026.
fn residential_land(args: &VicPropertyTaxesArgs, year: &str) -> Result<(bool, bool)> {
    let why = "for vacant_residential_land";
    let land = when(true, args.residential_land, "residential_land", why)?.and_then(Arg::text);
    match land {
        Some("vacant_home") => Ok((false, true)),
        Some("construction" | "uninhabitable") => Ok((false, false)),
        Some("undeveloped") if year == "2025" => refuse(
            "vic_property_taxes.undeveloped_before_2026",
            "Vacant residential land tax applies to undeveloped land (s 34B(2B)) from the 2026 tax year.",
        ),
        Some("undeveloped") => Ok((true, false)),
        _ => refuse(
            "vic_property_taxes.residential_land_invalid",
            "residential_land must be vacant_home, construction, uninhabitable or undeveloped.",
        ),
    }
}

/// Whether the s 88EB concessional rate applies: asked except for
/// undeveloped land, and only a vacant home may have it.
fn concession(args: &VicPropertyTaxesArgs, undeveloped: bool, home: bool) -> Result<bool> {
    let why = "for vacant_residential_land other than undeveloped land";
    let key = "new_residential_concession";
    let given = when(!undeveloped, args.new_residential_concession, key, why)?;
    let concession = given.map_or(Ok(false), |v| flag(v, key))?;
    if concession && !home {
        return refuse(
            "vic_property_taxes.concession_unavailable",
            "new_residential_concession applies only to a vacant_home: s 88EB is for land that became residential land within s 34B(1).",
        );
    }
    Ok(concession)
}

/// The VRLT rate and its authority: 1% for s 34B(2B) land and under s 88EB;
/// otherwise 1%, 2% or 3% by the land's liability in the 2 preceding tax
/// years (s 35(3)(b)), which is asked only then.
fn vrlt_rate(args: &VicPropertyTaxesArgs, year: &str) -> Result<(Decimal, &'static str)> {
    let (undeveloped, home) = residential_land(args, year)?;
    let concession = concession(args, undeveloped, home)?;
    let why = "for vacant_residential_land at the s 35(3)(b) rates, which depend on it";
    let history = !undeveloped && !concession;
    let previous = when(
        history,
        args.liable_previous_tax_year,
        "liable_previous_tax_year",
        why,
    )?;
    let before = when(
        history,
        args.liable_tax_year_before,
        "liable_tax_year_before",
        why,
    )?;
    let (Some(previous), Some(before)) = (previous, before) else {
        let authority = if undeveloped { VRLT_2B } else { VRLT_88EB };
        return Ok((d("0.01"), authority));
    };
    let previous = flag(previous, "liable_previous_tax_year")?;
    let before = flag(before, "liable_tax_year_before")?;
    Ok(match (previous, before) {
        (false, _) => (d("0.01"), VRLT_1),
        (true, false) => (d("0.02"), VRLT_2),
        (true, true) => (d("0.03"), VRLT_3),
    })
}

fn vacant(args: &VicPropertyTaxesArgs, year: &str) -> Result<(Vec<Step>, Rates)> {
    let why = "for vacant_residential_land";
    let civ = when(
        true,
        args.capital_improved_value,
        "capital_improved_value",
        why,
    )?
    .expect("required above");
    let civ = dollars(civ, "capital improved values")?;
    let (applied, authority) = vrlt_rate(args, year)?;
    let tax = precise(|| Ok(civ.mul(&applied)?))?;
    let operands = [&civ, &applied];
    let vrlt = step(
        "vrlt",
        authority,
        "capital_improved_value x vrlt_rate",
        &operands,
        tax,
    );
    let rates = vec![rate("vrlt_rate", &applied)];
    Ok((vec![vrlt], rates))
}

/// The first tax year after the transition period of 10 years from `entry`:
/// the year after the year it ends in. Liability is fixed at midnight on the
/// 31 December before each tax year (s 3, 'CIPT taxable land').
fn commercial(args: &VicPropertyTaxesArgs) -> Result<(Vec<Step>, Rates)> {
    let why = "for commercial_industrial";
    let given = when(true, args.entry_date, "entry_date", why)?.expect("required above");
    let Some(entry) = given.text().and_then(dates::parse) else {
        return refuse(
            "input.date_invalid",
            "entry_date must be a YYYY-MM-DD date from 1900 to 2100.",
        );
    };
    if entry < dates::parse("2024-07-01").expect("a date") {
        return refuse(
            "vic_property_taxes.entry_date_before_reform",
            "entry_date is before 1 July 2024, when the Commercial and Industrial Property Tax Reform Act 2024 (Vic) commenced (s 2); no land entered the scheme before then.",
        );
    }
    let ends = dates::months_end(entry, 120);
    let first = dates::to_civil(ends).0 + 1;
    let nil = int(0);
    let cipt = step("cipt", CIPT, "0", &[], nil);
    let rates = vec![
        rate("entry_date", dates::format(entry)),
        rate("transition_period_ends", dates::format(ends)),
        rate("first_tax_year", first),
        rate("cipt_rate", "0.01"),
        rate("btr_cipt_rate", "0.005"),
    ];
    Ok((vec![cipt], rates))
}

/// The value before and after the rezoning of the title `n`.
fn parcel(n: usize, fields: &Map<String, Value>) -> Result<(Decimal, Decimal)> {
    if let Some(key) = fields
        .keys()
        .find(|k| !matches!(k.as_str(), "civ_before" | "civ_after"))
    {
        return refuse(
            "input.field_unknown",
            format!("parcel {n}: {key} is not a fact it takes beside civ_before and civ_after."),
        );
    }
    let value = |key: &str| -> Result<Decimal> {
        let text = fields.get(key).and_then(Value::as_str);
        let parsed = text.and_then(|t| Decimal::parse_input(t).ok());
        match parsed.map(|v| dollars(&Arg::Decimal(v), "capital improved values")) {
            Some(Ok(v)) => Ok(v),
            _ => refuse(
                "vic_property_taxes.parcels_invalid",
                format!(
                    "parcel {n}: {key} must be a whole-dollar amount of 0 or more, as a string."
                ),
            ),
        }
    };
    Ok((value("civ_before")?, value("civ_after")?))
}

const MOST: usize = 100;

fn parcels(args: &VicPropertyTaxesArgs) -> Result<Vec<(Decimal, Decimal)>> {
    let why = "for windfall_gains";
    let given = when(true, args.parcels, "parcels", why)?.expect("required above");
    let list = match given {
        Arg::Json(Value::Array(list)) if !list.is_empty() && list.len() <= MOST => list,
        _ => {
            return refuse(
                "vic_property_taxes.parcels_invalid",
                format!(
                    "parcels must be a list of 1 to {MOST} objects, one for each title the rezoning rezoned."
                ),
            );
        }
    };
    list.iter()
        .enumerate()
        .map(|(i, item)| match item.as_object() {
            Some(fields) => parcel(i + 1, fields),
            None => refuse(
                "vic_property_taxes.parcels_invalid",
                format!(
                    "parcel {}: must be an object with civ_before and civ_after.",
                    i + 1
                ),
            ),
        })
        .collect()
}

/// WGT on the taxable value uplift `taxable` (s 9), with its formula.
fn wgt_on(taxable: &Decimal) -> Result<(Decimal, &'static str, Vec<Decimal>)> {
    if !taxable.gt(&int(100_000))? {
        return Ok((int(0), "0 x taxable_value_uplift", vec![taxable.clone()]));
    }
    if taxable.lt(&int(500_000))? {
        let tax = taxable.sub(&int(100_000))?.mul(&d("0.625"))?;
        let formula = "(taxable_value_uplift - wgt_threshold) x wgt_lower_rate";
        return Ok((
            tax,
            formula,
            vec![taxable.clone(), int(100_000), d("0.625")],
        ));
    }
    let tax = taxable.mul(&d("0.5"))?;
    Ok((
        tax,
        "taxable_value_uplift x wgt_upper_rate",
        vec![taxable.clone(), d("0.5")],
    ))
}

fn windfall(args: &VicPropertyTaxesArgs) -> Result<(Vec<Step>, Rates)> {
    let parcels = parcels(args)?;
    let mut steps = Vec::new();
    let mut rates = Vec::new();
    let mut names = Vec::new();
    for (i, (before, after)) in parcels.iter().enumerate() {
        let n = i + 1;
        rates.push(rate(&format!("civ_before_{n}"), before));
        rates.push(rate(&format!("civ_after_{n}"), after));
        let formula = format!("civ_after_{n} - civ_before_{n}");
        let uplift = after.sub(before)?;
        steps.push(step(
            format!("value_uplift_{n}"),
            UPLIFT,
            &formula,
            &[after, before],
            uplift,
        ));
        names.push(format!("max(value_uplift_{n}, 0)"));
    }
    let mut taxable = int(0);
    for s in &steps {
        if s.value.gt(&int(0))? {
            taxable = taxable.add(&s.value)?;
        }
    }
    let uplifts: Vec<&Decimal> = steps.iter().map(|s| &s.value).collect();
    let sum = step(
        "taxable_value_uplift",
        TAXABLE,
        &names.join(" + "),
        &uplifts,
        taxable.clone(),
    );
    let (tax, formula, operands) = wgt_on(&taxable)?;
    let operands: Vec<&Decimal> = operands.iter().collect();
    let wgt = step("wgt", WGT, formula, &operands, tax);
    steps.push(sum);
    steps.push(wgt);
    rates.extend([
        rate("wgt_threshold", 100_000),
        rate("wgt_lower_rate", "0.625"),
        rate("wgt_upper_band", 500_000),
        rate("wgt_upper_rate", "0.5"),
    ]);
    Ok((steps, rates))
}

pub fn vic_property_taxes(args: &VicPropertyTaxesArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let chosen = tax(args.tax)?;
    refuse_others(chosen, args)?;
    let (mut steps, rates) = precise(|| match chosen {
        Tax::Vacant => vacant(args, &year),
        Tax::Commercial => commercial(args),
        Tax::Windfall => windfall(args),
    })?;
    let last = steps.last().expect("a tax step");
    let id = last.static_id().to_string();
    let amount = cents(last.value.clone())?;
    let operands = [&last.value];
    let total = step("total", TOTAL, &format!("cents({id})"), &operands, amount);
    steps.push(total);
    derived_result(KIND, &year, steps, rates)
}
