// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The loss carry back tax offset (Div 160 ITAA 1997): the temporary version for
//! the 2020-21 to 2022-23 claim years, and the permanent version that Treasury Laws
//! Amendment (Tax Reform No. 2) Act 2026 Sch 1 substituted for income years
//! starting on or after 1 July 2026. Eligibility facts are stated, never judged.
use crate::api::{array, decimal, number};
use crate::company_tax::{amount, flag, floor, least, money, refuse};
use crate::decimal::Decimal as D;
use crate::engine::{Bre, Result, cents, d};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq)]
enum Regime {
    Temporary,
    Permanent,
}
const TEMPORARY: [(&str, &str); 6] = [
    ("corporate_tax_entity_throughout", "ss 160-5(b), 160-25"),
    ("returns_lodged", "s 160-5(e)"),
    ("choice_in_approved_form", "ss 160-5(f), 160-15(2)"),
    ("carried_on_business_each_loss_year", "s 160-20"),
    ("no_transferred_losses", "s 160-30"),
    ("no_integrity_scheme", "s 160-35"),
];
const PERMANENT: [(&str, &str); 6] = [
    (
        "corporate_tax_entity_throughout",
        "s 160-5(1)(b) and (2)(c)",
    ),
    ("not_significant_global_entity", "s 160-5(1)(e)"),
    ("returns_lodged", "s 160-5(1)(f)"),
    ("choice_in_approved_form", "ss 160-5(1)(g), 160-15(2)"),
    ("no_transferred_losses", "s 160-25"),
    ("no_integrity_scheme", "s 160-30"),
];
const FIVE_BILLION: &str = "5000000000";

fn regime(fy: i64) -> Result<Regime> {
    match fy {
        2021..=2023 => Ok(Regime::Temporary),
        2027 => Ok(Regime::Permanent),
        _ => Err(refuse(
            format!("there is no loss carry back tax offset for the claim year ending in {fy}"),
            "s 160-5(a) ITAA 1997 to 30 June 2026; Treasury Laws Amendment (Tax Reform No. 2) Act 2026 Sch 1 item 11",
        )),
    }
}
fn conditions(lcb: &Value, regime: Regime) -> Result<()> {
    let given = lcb.get("conditions").unwrap_or(&Value::Null);
    let law = if regime == Regime::Temporary {
        "as in force to 30 June 2026"
    } else {
        "as substituted from 1 July 2026"
    };
    for (key, provision) in if regime == Regime::Temporary {
        TEMPORARY
    } else {
        PERMANENT
    } {
        if !flag(given, key)? {
            return Err(refuse(
                format!("loss carry back condition {key} is not stated as true"),
                &format!("{provision} ITAA 1997 {law}"),
            ));
        }
    }
    Ok(())
}

struct Earlier {
    liability: D,
    nei: D,
    used: D,
}
fn earlier_years(lcb: &Value) -> Result<BTreeMap<i64, Earlier>> {
    let mut years = BTreeMap::new();
    for e in array(lcb, "earlier_years")? {
        let row = Earlier {
            liability: money(e, "income_tax_liability")?,
            nei: money(e, "net_exempt_income")?,
            used: money(e, "liability_previously_used")?,
        };
        if years.insert(number(e, "year")?, row).is_some() {
            return Err("earlier_years lists a year twice".into());
        }
    }
    Ok(years)
}

/// Each loss year's tax loss and corporate tax rate; the claim year's own comes from the statement.
fn loss_years(lcb: &Value, fy: i64, current: (D, D, D)) -> Result<BTreeMap<i64, (D, D, D)>> {
    let mut years = BTreeMap::from([(fy, current)]);
    for e in array(lcb, "loss_years")? {
        let year = number(e, "year")?;
        let bre = Bre {
            fy: year,
            turnover: decimal(e, "aggregated_turnover")?,
            assessable: decimal(e, "assessable_income")?,
            passive: decimal(e, "passive_income")?,
        };
        let rate = bre.corporate_tax()?.rate;
        if year >= fy
            || years
                .insert(year, (money(e, "tax_loss")?, rate, bre.turnover))
                .is_some()
        {
            return Err("loss_years lists the claim year, a later year or a year twice".into());
        }
    }
    Ok(years)
}

struct Pair {
    loss: i64,
    earlier: i64,
    amount: D,
}
fn pair(e: &Value, fy: i64, regime: Regime) -> Result<Pair> {
    let p = Pair {
        loss: number(e, "loss_year")?,
        earlier: number(e, "earlier_year")?,
        amount: money(e, "amount")?,
    };
    let ok = match regime {
        Regime::Temporary => (2020..=fy).contains(&p.loss) && (2019..p.loss).contains(&p.earlier),
        Regime::Permanent => p.loss == fy && (fy - 2..fy).contains(&p.earlier),
    };
    if !ok {
        let (law, rule) = match regime {
            Regime::Temporary => (
                "ss 160-5(c) and (d), 160-15(1) ITAA 1997 as in force to 30 June 2026",
                "a loss year from 2019-20 to the claim year and an earlier year from 2018-19",
            ),
            Regime::Permanent => (
                "ss 160-5(2)(a), 160-15(1) ITAA 1997 as substituted from 1 July 2026",
                "the claim year's loss and the 2 income years before it",
            ),
        };
        return Err(refuse(
            format!(
                "carrying the loss year ending in {} back to the year ending in {} is outside {rule}",
                p.loss, p.earlier
            ),
            law,
        ));
    }
    Ok(p)
}
fn check_losses(pairs: &[Pair], losses: &BTreeMap<i64, (D, D, D)>, regime: Regime) -> Result<()> {
    if let Some(p) = pairs.iter().find(|p| !losses.contains_key(&p.loss)) {
        return Err(format!(
            "loss_years has no entry for the loss year ending in {}",
            p.loss
        ));
    }
    for (year, (tax_loss, _, turnover)) in losses {
        let carried = pairs
            .iter()
            .filter(|p| p.loss == *year)
            .fold(d("0"), |t, p| t.add(&p.amount));
        if carried > *tax_loss {
            return Err(refuse(
                format!(
                    "{} carried back from the year ending in {year} exceeds its tax loss of {}",
                    carried.fixed(2, false),
                    tax_loss.fixed(2, false)
                ),
                "s 160-10(2) method statement step 1 ITAA 1997",
            ));
        }
        if regime == Regime::Temporary && carried > d("0") && *turnover >= d(FIVE_BILLION) {
            return Err(refuse(
                format!(
                    "aggregated turnover for the loss year ending in {year} is not under $5 billion"
                ),
                "s 160-20 ITAA 1997 as in force to 30 June 2026",
            ));
        }
    }
    Ok(())
}

/// One earlier year's component: method steps 1 to 3 for each loss carried to it,
/// oldest first against its unused net exempt income, capped at its unused liability.
fn component(
    year: i64,
    e: &Earlier,
    pairs: &[&Pair],
    losses: &BTreeMap<i64, (D, D, D)>,
    regime: Regime,
) -> Result<(D, Value)> {
    if regime == Regime::Permanent && e.liability <= d("0") {
        return Err(refuse(
            format!(
                "the year ending in {year} has no income tax liability, so it is not an eligible carry back year"
            ),
            "s 160-5(2)(b) ITAA 1997",
        ));
    }
    let (mut nei, mut sum, mut steps) = (e.nei.clone(), d("0"), Vec::new());
    for p in pairs {
        let rate = &losses[&p.loss].1;
        let reduced = floor(p.amount.sub(&nei));
        nei = floor(nei.sub(&p.amount));
        let product = cents(&reduced.mul(rate))?;
        sum = sum.add(&product);
        steps.push(json!({"loss_year": p.loss, "carried_back": amount(&p.amount),
            "after_net_exempt_income": amount(&reduced), "loss_year_rate": rate.to_string(), "step_3": amount(&product)}));
    }
    let cap = floor(e.liability.sub(&e.used));
    let comp = least(sum.clone(), cap.clone());
    let row = json!({"earlier_year": year, "losses": steps, "sum_of_step_3": amount(&sum),
        "unused_income_tax_liability": amount(&cap), "component": amount(&comp)});
    Ok((comp, row))
}

pub struct CarryBack {
    pub offset: D,
    pub from_current_year: D,
    pub json: Value,
}
fn components(
    lcb: &Value,
    pairs: &[Pair],
    losses: &BTreeMap<i64, (D, D, D)>,
    regime: Regime,
) -> Result<(D, Vec<Value>)> {
    let earlier = earlier_years(lcb)?;
    let mut targets: Vec<i64> = pairs.iter().map(|p| p.earlier).collect();
    targets.sort_unstable();
    targets.dedup();
    let (mut total, mut rows) = (d("0"), Vec::new());
    for year in targets {
        let e = earlier
            .get(&year)
            .ok_or_else(|| format!("earlier_years has no entry for the year ending in {year}"))?;
        let mut to: Vec<&Pair> = pairs.iter().filter(|p| p.earlier == year).collect();
        to.sort_by_key(|p| p.loss);
        let (comp, row) = component(year, e, &to, losses, regime)?;
        total = total.add(&comp);
        rows.push(row);
    }
    Ok((total, rows))
}
/// The claim year's regime and franking account balance, once its stated conditions hold.
fn setup(lcb: &Value, fy: i64) -> Result<(Regime, D)> {
    if !lcb.is_object() {
        return Err("loss_carry_back must be an object".into());
    }
    let regime = regime(fy)?;
    conditions(lcb, regime)?;
    if lcb
        .get("franking_account_balance")
        .is_none_or(Value::is_null)
    {
        return Err("loss_carry_back.franking_account_balance is required".into());
    }
    Ok((regime, money(lcb, "franking_account_balance")?))
}
pub fn evaluate(v: &Value, fy: i64, rate: &D, tax_loss: &D) -> Result<Option<CarryBack>> {
    let Some(lcb) = v.get("loss_carry_back").filter(|x| !x.is_null()) else {
        return Ok(None);
    };
    let (regime, balance) = setup(lcb, fy)?;
    let turnover = decimal(v, "aggregated_turnover")?;
    let losses = loss_years(lcb, fy, (tax_loss.clone(), rate.clone(), turnover))?;
    let pairs = array(lcb, "carry_backs")?
        .iter()
        .map(|e| pair(e, fy, regime))
        .collect::<Result<Vec<_>>>()?;
    if regime == Regime::Permanent && *tax_loss <= d("0") {
        return Err(refuse(
            "the claim year is not a loss year",
            "s 160-5(1)(c) ITAA 1997",
        ));
    }
    check_losses(&pairs, &losses, regime)?;
    let (total, rows) = components(lcb, &pairs, &losses, regime)?;
    let offset = least(total.clone(), balance.clone());
    let from_current_year = pairs
        .iter()
        .filter(|p| p.loss == fy)
        .fold(d("0"), |t, p| t.add(&p.amount));
    let law = match regime {
        Regime::Temporary => {
            "Div 160 ITAA 1997 as in force to 30 June 2026 (claim years 2020-21 to 2022-23)"
        }
        Regime::Permanent => {
            "Div 160 ITAA 1997 as substituted by Treasury Laws Amendment (Tax Reform No. 2) Act 2026 Sch 1 (income years from 1 July 2026)"
        }
    };
    let json = json!({"law": law, "components": rows, "sum_of_components": amount(&total),
        "franking_account_balance": amount(&balance), "offset": amount(&offset),
        "label": "E (refundable tax offset, s 67-23 ITAA 1997)"});
    Ok(Some(CarryBack {
        offset,
        from_current_year,
        json,
    }))
}
