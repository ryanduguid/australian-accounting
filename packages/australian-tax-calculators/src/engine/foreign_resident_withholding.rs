// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `foreign_resident_withholding`: the foreign resident capital gains
//! withholding amount a purchaser pays the Commissioner under TAA 1953
//! Schedule 1 s 14-200 on acquiring taxable Australian real property or a
//! company title interest on a day from 1 July 2017 to 30 June 2027. The
//! citations are to TAA compilation 227 (1 October 2026), compared with
//! compilations 148 (1 July 2017) and 215 (11 December 2024) and the
//! compilation of 1 January 2025, Acts No. 57 of 2017 and No. 135 of 2024,
//! the multiple vendor class variations F2016L01123 and LI 2026/38
//! (F2026L01231), and the ATO's foreign resident capital gains withholding
//! pages, all read on 22 November 2026. Clearance certificates, notices of
//! variation, class variations and exclusions are the operator's facts.

use serde_json::{Map, Value, json};

use super::business_capital::year_first;
use super::{Arg, Result, cents, date_arg, flag, int, money, precise, rate, refuse, result, scope};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "foreign_resident_withholding";
const MAX_VENDORS: usize = 50;
/// Act No. 135 of 2024 Sch 1 item 7: 15% and no threshold for acquisitions
/// on or after 1 January 2025.
const FIFTEEN_FROM: &str = "2025-01-01";
/// Act No. 57 of 2017 Sch 1 items 1, 3 and 4.
const EARLIER: (&str, &str) = ("0.125", "750000");
const LATER_RATE: &str = "0.15";

/// The s 14-215 paragraphs the operator can state, with the paragraph.
const EXCLUSIONS: [(&str, &str); 6] = [
    ("approved_stock_exchange", "s 14-215(b)"),
    ("crossing_system", "s 14-215(c)"),
    ("already_withheld", "s 14-215(d)"),
    ("securities_lending", "s 14-215(e)"),
    ("insolvent_company", "s 14-215(f)"),
    ("bankruptcy", "s 14-215(g)"),
];

const ASSETS: [&str; 2] = ["taxable_australian_real_property", "company_title_interest"];

/// The facts as the command passes them; see the README for each.
pub struct ForeignResidentWithholdingArgs<'a> {
    pub payment: &'a Arg,
    pub acquisition_date: &'a Arg,
    pub asset: &'a Arg,
    pub market_value: &'a Arg,
    pub cost_base_first_element: &'a Arg,
    pub option_payments: &'a Arg,
    pub vendors: &'a Arg,
    pub exclusion: &'a Arg,
    pub class_variation_to_nil: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One vendor's document: none, a clearance certificate (s 14-210(2)) or a
/// notice of variation (s 14-235(2)) with its rate and maximum price.
enum Document {
    None,
    Clearance,
    Variation { rate: Decimal, maximum: Decimal },
}

struct Vendor {
    interest: Decimal,
    document: Document,
}

fn vendors_invalid<T>(message: impl Into<String>) -> Result<T> {
    refuse("foreign_resident_withholding.vendors_invalid", message)
}

/// A decimal string for a varied rate, from 0 to 1.
fn variation_rate(fields: &Map<String, Value>, name: &str) -> Result<Decimal> {
    let parsed = match fields.get("rate") {
        Some(Value::String(text)) if text.len() <= 32 => Decimal::parse_input(text).ok(),
        _ => None,
    };
    match parsed {
        Some(r) if r.is_finite() && !r.lt(&int(0))? && !r.gt(&int(1))? => Ok(r),
        _ => vendors_invalid(format!(
            "{name}: rate must be a decimal string from 0 to 1, the varied rate on the notice of variation."
        )),
    }
}

fn document(fields: &Map<String, Value>, name: &str) -> Result<Document> {
    let kind = fields.get("document").and_then(Value::as_str);
    let keys: &[&str] = match kind {
        Some("variation") => &["interest", "document", "rate", "maximum_price"],
        _ => &["interest", "document"],
    };
    super::stated::known_keys(fields, name, "a vendor", keys)?;
    match kind {
        Some("none") => Ok(Document::None),
        Some("clearance_certificate") => Ok(Document::Clearance),
        Some("variation") => Ok(Document::Variation {
            rate: variation_rate(fields, name)?,
            maximum: super::stated::amount(fields, name, "maximum_price")?,
        }),
        _ => vendors_invalid(format!(
            "{name}: document must be none, clearance_certificate or variation."
        )),
    }
}

fn vendor(item: &Value, name: &str) -> Result<Vendor> {
    let Value::Object(fields) = item else {
        return vendors_invalid(format!(
            "{name} must be an object with interest and document."
        ));
    };
    let interest = super::stated::amount(fields, name, "interest")?;
    if interest.is_zero() {
        return vendors_invalid(format!("{name}: interest must be above nil."));
    }
    let document = document(fields, name)?;
    Ok(Vendor { interest, document })
}

fn vendors(given: &Arg) -> Result<Vec<Vendor>> {
    let Arg::Json(Value::Array(items)) = given else {
        return vendors_invalid("vendors must be a list of objects with interest and document.");
    };
    if items.is_empty() || items.len() > MAX_VENDORS {
        return vendors_invalid(format!("vendors must hold 1 to {MAX_VENDORS} entries."));
    }
    items
        .iter()
        .enumerate()
        .map(|(i, item)| vendor(item, &format!("vendor {}", i + 1)))
        .collect()
}

/// The acquisition date, inside the income year `year`.
fn acquisition(args: &ForeignResidentWithholdingArgs, year: &str) -> Result<i64> {
    let first = year_first(year).expect("a supported period is an income year");
    let start = dates::parse(&format!("{first}-07-01")).expect("a date");
    let end = dates::parse(&format!("{}-06-30", first + 1)).expect("a date");
    let day = date_arg(args.acquisition_date, "acquisition_date")?;
    if !(start..=end).contains(&day) {
        return refuse(
            "foreign_resident_withholding.date_outside_year",
            format!(
                "acquisition_date must be in the {year} income year, from {} to {}.",
                dates::format(start),
                dates::format(end)
            ),
        );
    }
    Ok(day)
}

/// The payment and asset the command covers, refused otherwise.
fn covered(args: &ForeignResidentWithholdingArgs) -> Result<()> {
    if args.payment.text() != Some("capital_gains") {
        return refuse(
            "foreign_resident_withholding.payment_unsupported",
            "Only payment capital_gains (TAA Sch 1 s 14-200) is built. Interest, dividend and royalty withholding (Subdivision 12-F), managed investment trust fund payments (Subdivision 12-H) and s 12-315 payments are not.",
        );
    }
    match args.asset.text() {
        Some(asset) if ASSETS.contains(&asset) => Ok(()),
        _ => refuse(
            "foreign_resident_withholding.asset_unsupported",
            "asset must be taxable_australian_real_property or company_title_interest. For another indirect Australian real property interest or an option or right, s 14-210(1)(a) to (d) turn on what the purchaser knows or believes and s 14-225 declarations can apply; a look-through earnout (s 14-205) is not built either.",
        ),
    }
}

/// The s 14-215 paragraph the operator states, if any.
fn exclusion(given: &Arg) -> Result<Option<&'static str>> {
    if matches!(given, Arg::Other(none) if none == "None") {
        return Ok(None);
    }
    match EXCLUSIONS
        .iter()
        .find(|(key, _)| given.text() == Some(*key))
    {
        Some((_, paragraph)) => Ok(Some(paragraph)),
        None => refuse(
            "foreign_resident_withholding.exclusion_invalid",
            "exclusion must be null or one of approved_stock_exchange, crossing_system, already_withheld, securities_lending, insolvent_company or bankruptcy (s 14-215).",
        ),
    }
}

/// The rate and, before 1 January 2025, the s 14-215(1)(a) threshold.
fn rate_for(day: i64) -> (&'static str, Option<&'static str>) {
    if day >= dates::parse(FIFTEEN_FROM).expect("a date") {
        (LATER_RATE, None)
    } else {
        (EARLIER.0, Some(EARLIER.1))
    }
}

struct Facts {
    day: i64,
    market_value: Decimal,
    /// The first element of the cost base: the price a notice's maximum caps.
    price: Decimal,
    base: Decimal,
    vendors: Vec<Vendor>,
    excluded: Option<&'static str>,
    nil_class: bool,
}

fn facts(args: &ForeignResidentWithholdingArgs, year: &str) -> Result<Facts> {
    covered(args)?;
    let day = acquisition(args, year)?;
    let market_value = money(args.market_value)?;
    let first = money(args.cost_base_first_element)?;
    let option = money(args.option_payments)?;
    if option.gt(&first)? {
        return refuse(
            "foreign_resident_withholding.option_exceeds_price",
            "option_payments cannot exceed cost_base_first_element.",
        );
    }
    let vendors = vendors(args.vendors)?;
    let excluded = exclusion(args.exclusion)?;
    let nil_class = flag(args.class_variation_to_nil, "class_variation_to_nil")?;
    Ok(Facts {
        day,
        market_value,
        base: first.sub(&option)?,
        price: first,
        vendors,
        excluded,
        nil_class,
    })
}

/// Why nothing is payable before the vendors are looked at, if so.
fn whole_nil(facts: &Facts, threshold: Option<&str>) -> Result<Option<String>> {
    if let Some(paragraph) = facts.excluded {
        return Ok(Some(format!("excluded transaction ({paragraph})")));
    }
    if let Some(limit) = threshold
        && facts.market_value.lt(&Decimal::parse(limit)?)?
    {
        return Ok(Some(
            "excluded transaction: market value under $750,000 (s 14-215(1)(a), acquisitions before 1 January 2025)".into(),
        ));
    }
    if facts.nil_class {
        return Ok(Some("class variation to nil (s 14-235(5) and (6))".into()));
    }
    if facts
        .vendors
        .iter()
        .all(|v| matches!(v.document, Document::Clearance))
    {
        return Ok(Some(
            "every vendor gave a clearance certificate: s 14-210(1) applies to none (s 14-210(2))"
                .into(),
        ));
    }
    Ok(None)
}

/// Each vendor's share of the base and amount (s 14-200(3); the multiple
/// vendor class variations; the ATO's variation notice rule).
fn vendor_rows(facts: &Facts, full: &Decimal) -> Result<Vec<(Decimal, Decimal, &'static str)>> {
    let total = facts
        .vendors
        .iter()
        .try_fold(int(0), |sum, v| sum.add(&v.interest))?;
    precise(|| {
        facts
            .vendors
            .iter()
            .map(|v| {
                let share = facts.base.mul(&v.interest)?.div(&total)?;
                let (applied, basis) = match &v.document {
                    Document::None => (full.clone(), "no document: full rate"),
                    Document::Clearance => (int(0), "clearance certificate: nil"),
                    Document::Variation { maximum, .. } if facts.price.gt(maximum)? => (
                        full.clone(),
                        "variation does not apply above its maximum price: full rate",
                    ),
                    Document::Variation { rate, .. } => {
                        (rate.clone(), "notice of variation: varied rate")
                    }
                };
                Ok((share.clone(), cents(applied.mul(&share)?)?, basis))
            })
            .collect()
    })
}

const STEPS: [(&str, &str); 4] = [
    (
        "withholding_base",
        "TAA Sch 1 s 14-200(3)(a): the first element of the asset's cost base just after the acquisition less any payment or property given for an option you exercised to acquire it",
    ),
    (
        "rate",
        "TAA Sch 1 s 14-200(3)(a): 12.5% for acquisitions from 1 July 2017 (Act No. 57 of 2017 Sch 1 items 1 and 4), 15% on or after 1 January 2025 (Act No. 135 of 2024 Sch 1 items 1 and 7); the acquisition is when the contract is entered into (ITAA 1997 s 109-5(2) item A1)",
    ),
    (
        "vendors",
        "TAA Sch 1 s 14-210(1)(e) and (2): every vendor without a clearance certificate; each vendor's interest share of the base at the full rate, its notice's varied rate (s 14-235(2)), or nil for a clearance certificate (PAYG withholding variation F2016L01123; LI 2026/38 s 6)",
    ),
    (
        "amount_payable",
        "TAA Sch 1 s 14-200(3) and (4): the sum of the vendors' amounts, each rounded half up to the cent; nil if excluded (s 14-215) or varied to nil (s 14-235(5), (6))",
    ),
];

pub fn foreign_resident_withholding(args: &ForeignResidentWithholdingArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let facts = facts(args, &year)?;
    let (full_rate, threshold) = rate_for(facts.day);
    let full = Decimal::parse(full_rate)?;
    for v in &facts.vendors {
        if let Document::Variation { rate, .. } = &v.document
            && rate.gt(&full)?
        {
            return refuse(
                "foreign_resident_withholding.variation_invalid",
                format!(
                    "A varied rate cannot exceed the full rate of {full_rate} for the acquisition date."
                ),
            );
        }
    }
    let nil = whole_nil(&facts, threshold)?;
    let rows = vendor_rows(&facts, &full)?;
    let payable = match nil {
        Some(_) => int(0),
        None => rows.iter().try_fold(int(0), |sum, row| sum.add(&row.1))?,
    };
    let full_amount = precise(|| Ok(full.mul(&facts.base)?))?;
    let mut rates = vec![rate("rate", full_rate)];
    rates.push(rate("threshold", threshold.unwrap_or("none")));
    let mut out = result(
        KIND,
        &year,
        vec![
            ("withholding_base", facts.base.clone()),
            ("amount_at_full_rate", full_amount),
            ("amount_payable", payable),
        ],
        rates,
    )?;
    explain(&mut out, &facts, &rows, nil);
    Ok(out)
}

fn explain(out: &mut Value, facts: &Facts, rows: &[(Decimal, Decimal, &str)], nil: Option<String>) {
    let applies = nil.is_none();
    out["acquisition_date"] = json!(dates::format(facts.day));
    out["nil_reason"] = json!(nil);
    out["vendors"] = json!(
        rows.iter()
            .map(|(share, amount, basis)| {
                let amount = if applies { amount.to_string() } else { "0.00".into() };
                json!({"share_of_base": cents(share.clone()).map(|s| s.to_string()).unwrap_or_default(), "amount": amount, "basis": basis})
            })
            .collect::<Vec<_>>()
    );
    out["authorities"] = Value::Object(
        STEPS
            .iter()
            .map(|(step, text)| ((*step).to_owned(), json!(text)))
            .collect::<Map<_, _>>(),
    );
}
