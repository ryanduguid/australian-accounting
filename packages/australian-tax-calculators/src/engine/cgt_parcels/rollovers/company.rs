// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Subdivision 122-A: an individual or trustee disposes of a CGT asset, or
//! all the assets of a business, to a company it wholly owns (ss 122-15 to
//! 122-50 and 122-70), every asset acquired on or after 20 September 1985.

use super::*;

const SHARES_ONE: &str = "ITAA 1997 s 122-40(2): the first element of each share's cost base is the asset's cost base when disposed of, less the liabilities the company undertakes to discharge in respect of it, divided by the number of shares; the reduced cost base is worked out similarly";
const SHARES_ALL: &str = "ITAA 1997 s 122-50(1): the first element of each share's cost base is the market values of the precluded assets and the cost bases of the other assets, less the liabilities the company undertakes to discharge in respect of them, divided by the number of shares; the reduced cost base is worked out similarly";
const COMPANY_SIDE: &str = "ITAA 1997 s 122-70(2): each asset other than a precluded asset has, in the company's hands, the transferor's cost base and reduced cost base when disposed of; for the discount tests the company is taken to have acquired it when the transferor did (s 115-30(1) item 1)";
const SHARES_DISCOUNT: &str = "The shares are acquired at the trigger event; if one has been owned for less than 12 months at a later CGT event, ss 115-25 and 115-40 apply as if it had been acquired at least 12 months before (ITAA 1997 s 115-34(1)(c)(i) and (2)).";
const PRECLUDED: &str = "A precluded asset (a depreciating asset, trading stock, a film copyright interest under s 118-30 or a registered emissions unit, s 122-25(3)) counts at its market value in the shares' cost base, but s 122-70 does not carry its cost to the company: Division 40 and Division 70 decide that, outside this worksheet.";
const PROCEEDS: &str = "The disregarded gains and losses are worked out from each asset's stated capital proceeds: the market value of the shares and the liabilities undertaken that the preparer attributes to it.";

/// One asset disposed of to the company.
struct Asset {
    label: String,
    acquired: i64,
    cost: Decimal,
    reduced: Decimal,
    value: Decimal,
    precluded: bool,
    proceeds: Option<Decimal>,
}

fn asset(n: usize, fields: &Map<String, Value>, day: i64) -> Result<Asset> {
    let mut f = Fields::new(format!("asset {n}"), fields);
    let label = f.text("label")?;
    let acquired = f.date("acquired")?;
    let class = choice(
        &mut f,
        "asset_class",
        &["ordinary", "collectable", "personal_use"],
    )?;
    let a = Asset {
        label,
        acquired,
        cost: f.amount("cost_base", false)?,
        reduced: f.amount("reduced_cost_base", false)?,
        value: f.amount("market_value", false)?,
        precluded: yes(&mut f, "precluded")?,
        proceeds: match f.get("capital_proceeds") {
            None => None,
            Some(_) => Some(f.amount("capital_proceeds", false)?),
        },
    };
    f.none_left()?;
    if acquired > day {
        return conflict(format!(
            "asset {n}: acquired must be on or before event_date."
        ));
    }
    if class != "ordinary" {
        return no_rollover(format!(
            "asset {n}: Subdivision 122-A does not apply to a collectable or a personal use asset (ITAA 1997 s 122-25(2) items 1(a) and 2(a)); this worksheet does not separate one out."
        ));
    }
    if pre_cgt(acquired) {
        return refuse(
            "cgt_rollover.pre_cgt",
            format!(
                "asset {n} was acquired before 20 September 1985: the shares are then taken to be acquired before that day in whole or part (ITAA 1997 ss 122-40(3), 122-55 and 122-60), which this worksheet does not work out."
            ),
        );
    }
    Ok(a)
}

/// The first element of the shares' cost base and reduced cost base in
/// total, after the s 122-35 limit on the liabilities.
fn shares_bases(all: &[Asset], business: bool, liabilities: &Decimal) -> Result<[Decimal; 2]> {
    let mut sums = [int(0), int(0)];
    for a in all {
        let (c, r) = if a.precluded {
            (&a.value, &a.value)
        } else {
            (&a.cost, &a.reduced)
        };
        sums = [sums[0].add(c)?, sums[1].add(r)?];
    }
    if liabilities.gt(&sums[0])? {
        let item = if business {
            "s 122-35(2) item 1: the sum of the precluded assets' market values and the other assets' cost bases"
        } else {
            "s 122-35(1) item 1: the asset's cost base"
        };
        return no_rollover(format!(
            "The liabilities the company undertakes, {liabilities}, exceed what ITAA 1997 {item}, {}, allows, so there is no roll-over.",
            sums[0]
        ));
    }
    Ok([
        sums[0].sub(liabilities)?,
        max_nil(sums[1].sub(liabilities)?)?,
    ])
}

/// The assets as the company holds them, and what the roll-over disregards.
fn company_side(all: &[Asset]) -> Result<(Value, Option<[Decimal; 2]>)> {
    let mut rows = Vec::with_capacity(all.len());
    let mut totals: Option<[Decimal; 2]> = None;
    for a in all {
        let mut row = json!({"label": a.label, "precluded": a.precluded});
        if !a.precluded {
            row["cost_base"] = json!(cents(a.cost.clone())?.to_string());
            row["reduced_cost_base"] = json!(cents(a.reduced.clone())?.to_string());
            row["acquired_for_discount"] = json!(dates::format(a.acquired));
            if let Some(p) = &a.proceeds {
                let [gain, loss] = disregarded(p, [&a.cost, &a.reduced])?;
                row["disregarded_capital_gain"] = json!(cents(gain.clone())?.to_string());
                row["disregarded_capital_loss"] = json!(cents(loss.clone())?.to_string());
                let [g, l] = totals.unwrap_or([int(0), int(0)]);
                totals = Some([g.add(&gain)?, l.add(&loss)?]);
            }
        }
        rows.push(row);
    }
    Ok((Value::Array(rows), totals))
}

fn share_count(f: &mut Fields) -> Result<u64> {
    match f.required("shares")?.as_u64() {
        Some(n) if (1..=1_000_000_000).contains(&n) => Ok(n),
        _ => f.invalid("shares", "a JSON integer from 1 to 1,000,000,000"),
    }
}

/// The stated assets, checked against the case and s 122-25(2).
fn assets_of(f: &mut Fields, business: bool, day: i64) -> Result<Vec<Asset>> {
    let Some(list) = optional_objects(f, "assets")? else {
        return f.invalid("assets", &format!("a list of 1 to {MOST} objects"));
    };
    let all = list
        .iter()
        .enumerate()
        .map(|(i, o)| asset(i + 1, o, day))
        .collect::<Result<Vec<_>>>()?;
    if !business && all.len() != 1 {
        return f.invalid("assets", "a list of one asset when case is asset");
    }
    if !business && all[0].precluded {
        return no_rollover(
            "Subdivision 122-A does not apply to the disposal of a precluded asset on its own (ITAA 1997 s 122-25(2) item 1(c)).",
        );
    }
    Ok(all)
}

/// Subdivision 122-A on the stated facts.
pub(super) fn work(f: &mut Fields, year: &str) -> Result<Outcome> {
    let transferor = choice(f, "transferor", &["individual", "trustee"])?;
    let business = choice(f, "case", &["asset", "business"])? == "business";
    if !yes(f, "conditions_met")? {
        return no_rollover(
            "The roll-over needs the conditions of ITAA 1997 ss 122-20 and 122-25: only non-redeemable shares (and, for a disposal, liabilities undertaken) as consideration, of substantially the same market value, all the shares owned just after, a company that is not an exempt entity, and the residency requirements.",
        );
    }
    let day = in_year(f, "event_date", year)?;
    let shares = share_count(f)?;
    let liabilities = f.amount("liabilities", true)?;
    let all = assets_of(f, business, day)?;
    let [cost, reduced] = shares_bases(&all, business, &liabilities)?;
    let count = Decimal::from_int(i64::try_from(shares).expect("at most 1e9"));
    let mut amounts = vec![
        ("shares_cost_base", cost.clone()),
        ("shares_reduced_cost_base", reduced.clone()),
        ("cost_base_per_share", cost.div(&count)?),
        ("reduced_cost_base_per_share", reduced.div(&count)?),
    ];
    let (rows, totals) = company_side(&all)?;
    if let Some([gain, loss]) = totals {
        amounts.push(("disregarded_capital_gain", gain));
        amounts.push(("disregarded_capital_loss", loss));
    }
    let mut extra = Map::new();
    extra.insert("transferor".into(), json!(transferor));
    extra.insert(
        "case".into(),
        json!(if business { "business" } else { "asset" }),
    );
    extra.insert(
        "shares_authority".into(),
        json!(if business { SHARES_ALL } else { SHARES_ONE }),
    );
    extra.insert("shares_acquired".into(), json!(dates::format(day)));
    extra.insert("company_authority".into(), json!(COMPANY_SIDE));
    extra.insert("assets".into(), rows);
    extra.insert("warnings".into(), warnings(&all));
    Ok((amounts, extra))
}

/// The warnings the stated assets call for.
fn warnings(all: &[Asset]) -> Value {
    let indexed = dates::parse(INDEXATION_LAST_DAY).expect("a date");
    let mut warnings = vec![json!(SHARES_DISCOUNT)];
    if all.iter().any(|a| a.precluded) {
        warnings.push(json!(PRECLUDED));
    }
    if all.iter().any(|a| a.proceeds.is_some()) {
        warnings.push(json!(PROCEEDS));
    }
    if all.iter().any(|a| a.acquired <= indexed) {
        warnings.push(json!(INDEXED));
    }
    Value::Array(warnings)
}
