// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Not-for-profit employers in `fbt-return`: the rebate of FBTAA s 65J for a
//! rebatable employer, the capping thresholds of s 5B(1D) to (1M) for the
//! employers whose benefits s 57A exempts, and items 20 to 22 from the FBT
//! instalments reported. It cites the FBTAA as compiled on 1 July 2026
//! (compilation 97, C2026C00332) and the ATO's 'Completing your FBT return
//! 2026 – not-for-profit employers', both read on 22 November 2026.
use super::kind::{Entertainment, Kind};
use super::{Valued, calc, cents, gross_up};
use crate::api::Refusal;
use crate::decimal::{Decimal, Rounding};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// FBTAA s 65J(2B) step 2: the rebatable employer's cap per employee.
const REBATE_CAP: i64 = 30_000;

/// FBTAA s 5B(1E) step 4 and s 65J(2B) step 2A: the separate cap for salary
/// packaged meal entertainment and entertainment facility leasing benefits.
const ENTERTAINMENT_CAP: i64 = 5_000;

/// FBTAA s 5B(1L) step 1 and s 65J(2H) step 1: excluded meal entertainment,
/// car parking and entertainment facility leasing benefits are disregarded.
pub(super) fn disregarded(b: &Valued) -> bool {
    b.given.excluded
        && (matches!(
            b.given.command,
            "fbt-car-parking" | "fbt-meal-entertainment"
        ) || b.given.entertainment == Entertainment::NotSalaryPackaged)
}

fn nil() -> Decimal {
    Decimal::from_int(0)
}

fn not_below_nil(value: Decimal) -> Result<Decimal, Refusal> {
    Ok(if value.lt(&nil()).map_err(super::signal)? {
        nil()
    } else {
        value
    })
}

/// Each employee's type 1 and type 2 bases and the salary packaged
/// entertainment within them (s 5B(1H) to (1M); s 65J(2E) to (2J)).
fn bases<'a>(valued: &'a [Valued]) -> Result<BTreeMap<&'a str, [Decimal; 4]>, Refusal> {
    let mut out: BTreeMap<&str, [Decimal; 4]> = BTreeMap::new();
    for b in valued.iter().filter(|b| !disregarded(b)) {
        let entry = out
            .entry(b.given.employee)
            .or_insert_with(|| [nil(), nil(), nil(), nil()]);
        let at = usize::from(!b.given.type_one);
        entry[at] = calc(entry[at].add(&b.taxable))?;
        if b.given.entertainment == Entertainment::SalaryPackaged {
            entry[at + 2] = calc(entry[at + 2].add(&b.taxable))?;
        }
    }
    Ok(out)
}

/// The grossed-up amount of a type 1 and a type 2 base, by `fbt`'s
/// s 5B(1B) and (1C) formulas (s 5B(1F) and (1G); s 65J(2C) and (2D)).
fn grossed(v: &Value, year: i64, one: &Decimal, two: &Decimal) -> Result<Decimal, Refusal> {
    let fbt = gross_up(v, year, one, two)?;
    let [one, two] = ["type_one_grossed_up", "type_two_grossed_up"]
        .map(|key| super::amount(&fbt, key).map(|a| a.expect("fbt's grossed-up amount")));
    calc(one?.add(&two?))
}

/// One employee's amount over the caps (s 5B(1E) steps 1 to 4; s 65J(2B)
/// steps 1 to 2A), with the amounts it comes from.
fn over_caps(
    v: &Value,
    year: i64,
    cap: i64,
    [one, two, packaged_one, packaged_two]: &[Decimal; 4],
) -> Result<[Decimal; 3], Refusal> {
    let individual = grossed(v, year, one, two)?;
    let packaged = grossed(v, year, packaged_one, packaged_two)?;
    let mut over = not_below_nil(calc(individual.sub(&Decimal::from_int(cap)))?)?;
    if over.gt(&nil()).map_err(super::signal)? {
        let limit = Decimal::from_int(ENTERTAINMENT_CAP);
        let less = if packaged.lt(&limit).map_err(super::signal)? {
            packaged.clone()
        } else {
            limit
        };
        over = not_below_nil(calc(over.sub(&less))?)?;
    }
    Ok([individual, packaged, over])
}

/// Every employee's amounts over the caps, and their sum (s 5B(1E) step 5;
/// s 65J(2B) step 3).
fn capping(v: &Value, year: i64, valued: &[Valued], cap: i64) -> Result<(Value, Decimal), Refusal> {
    let mut employees = Map::new();
    let mut total = nil();
    for (employee, base) in bases(valued)? {
        let [individual, packaged, over] = over_caps(v, year, cap, &base)?;
        total = calc(total.add(&over))?;
        employees.insert(
            employee.into(),
            json!({
                "individual_grossed_up_amount": cents(&individual)?,
                "salary_packaged_entertainment_grossed_up": cents(&packaged)?,
                "amount_over_caps": cents(&over)?,
            }),
        );
    }
    let out = json!({
        "cap": cents(&Decimal::from_int(cap))?,
        "entertainment_cap": cents(&Decimal::from_int(ENTERTAINMENT_CAP))?,
        "employees": employees,
    });
    Ok((out, total))
}

/// What a rebatable or s 57A employer changes in the composed return.
pub(super) struct Composed {
    labels: Vec<(&'static str, Value)>,
    amounts: Vec<(&'static str, Value, &'static str)>,
    capping: Value,
    exempt: bool,
}

/// Items 14C, 15, 16 and 19 for a s 57A employer (s 5B(1D) and (1E)).
fn exempt_return(capping: Value, total: &Decimal, rate: &Decimal) -> Result<Composed, Refusal> {
    let item_14c = calc(total.quantize(0, Rounding::HalfUp))?;
    let tax = calc(item_14c.mul(rate))?;
    Ok(Composed {
        labels: vec![
            ("14C", cents(&item_14c)?),
            ("15", cents(&item_14c)?),
            ("16", cents(&tax)?),
            ("19", cents(&tax)?),
        ],
        amounts: vec![
            (
                "aggregate_non_exempt_amount",
                cents(total)?,
                "sum of each employee's amount_over_caps (FBTAA s 5B(1E) step 5)",
            ),
            (
                "fringe_benefits_taxable_amount",
                cents(&item_14c)?,
                "aggregate_non_exempt_amount in whole dollars, 0.5 up as fbt rounds items 14A and 14B (no provision states a rounding): every benefit is exempt under s 57A, so the s 5B(1A) amount is nil and s 5B(1D) adds the aggregate non-exempt amount; return item 14C and 15",
            ),
            (
                "fbt_payable",
                cents(&tax)?,
                "fringe_benefits_taxable_amount x fbt_rate (FBTAA s 66); return items 16 and 19",
            ),
        ],
        capping,
        exempt: true,
    })
}

/// Items 17, 18 and 19 for a rebatable employer (s 65J(2A) and (2B)).
fn rebatable_return(
    capping: Value,
    total: &Decimal,
    rate: &Decimal,
    gross_tax: &Decimal,
    [rebatable, employer]: [i64; 2],
) -> Result<Composed, Refusal> {
    let item_17 = calc(calc(total.mul(rate))?.quantize(-2, Rounding::HalfUp))?;
    let base = calc(calc(gross_tax.sub(&item_17))?.mul(rate))?;
    let share = calc(base.mul(&Decimal::from_int(rebatable)))?;
    let rebate = calc(share.div(&Decimal::from_int(employer)))?;
    let item_18 = not_below_nil(calc(rebate.quantize(-2, Rounding::HalfUp))?)?;
    let item_19 = calc(gross_tax.sub(&item_18))?;
    Ok(Composed {
        labels: vec![
            ("17", cents(&item_17)?),
            ("18", cents(&item_18)?),
            ("19", cents(&item_19)?),
        ],
        amounts: vec![
            (
                "aggregate_non_rebatable_amount",
                cents(&item_17)?,
                "sum of each employee's amount_over_caps x fbt_rate, in cents 0.5 up (FBTAA s 65J(2B) steps 3 and 4); return item 17",
            ),
            (
                "rebate",
                cents(&item_18)?,
                "fbt_rate x (fbt_payable - aggregate_non_rebatable_amount) x rebatable_days / employer_days, in cents 0.5 up, not below nil (FBTAA s 65J(2A)); return item 18",
            ),
            (
                "fbt_payable_after_rebate",
                cents(&item_19)?,
                "fbt_payable - rebate; return item 19",
            ),
        ],
        capping,
        exempt: false,
    })
}

/// The rebate or the capped amount, for any kind but a taxable employer.
pub(super) fn apply(
    kind: Kind,
    v: &Value,
    year: i64,
    valued: &[Valued],
    fbt: &Value,
    days: Option<[i64; 2]>,
) -> Result<Option<Composed>, Refusal> {
    let rate = calc(Decimal::parse(
        fbt["rates"]["fbt_rate"].as_str().expect("fbt's FBT rate"),
    ));
    let rate = rate?;
    match kind {
        Kind::Taxable => Ok(None),
        Kind::Exempt(cap) => {
            let (capping, total) = capping(v, year, valued, cap)?;
            exempt_return(capping, &total, &rate).map(Some)
        }
        Kind::Rebatable => {
            let (capping, total) = capping(v, year, valued, REBATE_CAP)?;
            let gross_tax = super::amount(fbt, "return_item_16")?.expect("fbt's item 16");
            let days = days.expect("check_days requires a rebatable employer's days");
            rebatable_return(capping, &total, &rate, &gross_tax, days).map(Some)
        }
    }
}

const NFP_SCOPE: &str = "As for a taxable employer, with the employer's kind (employer_kind) established by the operator, for the whole FBT year: a rebatable employer (FBTAA s 65J(1)), a registered public benevolent institution endorsed under s 123C (s 57A(1)), a registered health promotion charity endorsed under s 123D(1) (s 57A(5)), a public hospital (s 57A(3)(a)), a not-for-profit hospital (s 57A(4)) or a provider of public ambulance services every one of whose employees in the return is predominantly involved in those services (s 57A(3)(b)). Each employee's individual grossed-up amount is the type 1 base grossed up by fbt's s 5B(1B) formula plus the type 2 base x 1 / (1 - the FBT rate), each to the cent (s 5B(1F) and (1G); s 65J(2C) and (2D)); the bases are the taxable values of the employee's benefits by GST type, excluded benefits included except meal entertainment, car parking and benefits marked entertainment not_salary_packaged (s 5B(1H) to (1L); s 65J(2E) to (2H)). That amount less the cap ($30,000 for a rebatable employer, a public benevolent institution or a health promotion charity; $17,000 for a hospital or ambulance service), not below nil, is reduced where positive by the lesser of $5,000 and the grossed-up value of the employee's benefits marked entertainment salary_packaged (s 5B(1E) steps 2 to 4 and (1M); s 65J(2B) steps 2 and 2A and (2J)). For a s 57A employer the sum is the aggregate non-exempt amount, given at items 14C and 15 in whole dollars, and items 14A and 14B are blank; item 23 leaves out the benefits the caps disregard. For a rebatable employer items 14A to 16 are as for a taxable employer, item 17 is the sum x the FBT rate and item 18 the rebate of s 65J(2A), with the rebatable days and the days it was an employer given; an exempt electric car adds nothing to its caps, as s 65J(2G) reads s 5E. Not for: a government body under s 57A(2); an organisation partly a public benevolent institution and partly rebatable; s 58 live-in residential care workers; one employee's benefits from more than one employer (each return caps its own employer's benefits); an employee label that stands for more than one person, as the caps apply to each individual; an exempt electric car for a s 57A employer; the operator's choice of whether a benefit is salary packaged, meal entertainment or attributable to entertainment facility leasing expenses. Reportable fringe benefits amounts are as for a taxable employer, as s 135Q(2) to (4) gives them for a s 57A employer on the same benefits.";

const NFP_SOURCES: [&str; 3] = [
    "https://www.legislation.gov.au/C2004A03280/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_1/document_1.html",
    "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/completing-your-fbt-return-2026-not-for-profit-employers",
    "https://www.ato.gov.au/forms-and-instructions/fringe-benefits-tax-return-2026-instructions/worked-examples-for-nfp-employers-completing-fbt-return-2026",
];

/// Put a rebatable or s 57A employer's amounts into the composed result.
pub(super) fn merge(out: &mut Value, name: &str, composed: Composed) {
    out["employer_kind"] = json!(name);
    if composed.exempt {
        for key in ["type_one_grossed_up", "type_two_grossed_up"] {
            remove(out, "amounts", key);
            remove(out, "working", key);
        }
        for label in ["14A", "14B"] {
            remove(out, "return_labels", label);
        }
        remove(out, "components", "fbt");
    }
    for (key, value, working) in composed.amounts {
        out["amounts"][key] = value;
        out["working"][key] = json!(working);
    }
    for (label, value) in composed.labels {
        out["return_labels"][label] = value;
    }
    remove(out, "left_to_preparer", "14C, 17 and 18");
    out["capping"] = composed.capping;
    out["scope"] = json!(NFP_SCOPE);
    let sources = out["sources"].as_array_mut().expect("the sources");
    sources.extend(NFP_SOURCES.map(|s| json!(s)));
    out["source_checked"] = json!("2026-11-22");
}

fn remove(out: &mut Value, section: &str, key: &str) {
    if let Some(map) = out[section].as_object_mut() {
        map.remove(key);
    }
}

/// Items 20 to 22 from the instalments reported (FBT return 2026
/// instructions, items 20 to 22; s 105).
pub(super) fn instalment_labels(out: &mut Value, reported: Option<Decimal>) -> Result<(), Refusal> {
    let Some(reported) = reported else {
        return Ok(());
    };
    let item_19 = super::amount(&json!({"amounts": out["return_labels"]}), "19")?.expect("item 19");
    out["return_labels"]["20"] = cents(&reported)?;
    let due = calc(item_19.sub(&reported))?;
    if due.gt(&nil()).map_err(super::signal)? {
        out["return_labels"]["21"] = cents(&due)?;
    } else if due.lt(&nil()).map_err(super::signal)? {
        out["return_labels"]["22"] = cents(&calc(due.neg())?)?;
    }
    out["working"]["instalments_reported"] = json!(
        "the FBT instalments for the year reported on the activity statements, less credits claimed at 6B (FBTAA s 105(2)), at item 20; item 19 less item 20 at item 21 where positive, item 20 less item 19 at item 22 where positive"
    );
    remove(out, "left_to_preparer", "20 to 22");
    Ok(())
}
