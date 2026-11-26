// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The combined lines and the consolidated totals.

use super::*;

/// One account's combined, eliminated and consolidated amounts.
pub(super) struct Row {
    account: String,
    class: &'static str,
    combined: Decimal,
    eliminations: Decimal,
}

pub(super) fn row<'r>(
    rows: &'r mut Vec<Row>,
    account: &str,
    class: &'static str,
) -> Result<&'r mut Row> {
    let i = match rows.iter().position(|r| r.account == account) {
        Some(i) if rows[i].class == class => i,
        Some(_) => {
            return bad(
                "consolidation_eliminations.class_conflict",
                format!(
                    "{account} is given with more than one class; like items combine only with like (AASB 10 para B86(a))."
                ),
            );
        }
        None => {
            rows.push(Row {
                account: account.to_string(),
                class,
                combined: d("0"),
                eliminations: d("0"),
            });
            rows.len() - 1
        }
    };
    Ok(&mut rows[i])
}

/// Para B86(a): like lines combined, then the journals posted.
pub(super) fn rows(g: &Group) -> Result<Vec<Row>> {
    let mut out = Vec::new();
    for l in g.entities.iter().flat_map(|e| &e.lines) {
        let r = row(&mut out, &l.account, l.class)?;
        r.combined = r.combined.add(&l.amount)?;
    }
    for p in g.journals.iter().flat_map(|j| &j.postings) {
        let r = row(&mut out, &p.account, p.class)?;
        r.eliminations = r.eliminations.add(&p.change)?;
    }
    out.sort_by_key(|r| CLASSES.iter().position(|c| *c == r.class));
    Ok(out)
}

pub(super) fn profit(lines: impl Iterator<Item = (&'static str, Decimal)>) -> Calc<Decimal> {
    let mut net = d("0");
    for (class, v) in lines {
        net = match class {
            "income" => net.add(&v)?,
            "expense" => net.sub(&v)?,
            _ => net,
        };
    }
    Ok(net)
}

/// Para B94: each partly owned subsidiary's profit, less its own after-tax
/// unrealised profit, times the non-controlling share.
pub(super) fn nci_profit(g: &Group, after_tax: &[Decimal]) -> Calc<Decimal> {
    let mut total = d("0");
    for (e, unrealised) in g.entities.iter().zip(after_tax) {
        let Some(acq) = &e.acquisition else { continue };
        let own = profit(e.lines.iter().map(|l| (l.class, l.amount.clone())))?;
        let share = own.sub(unrealised)?.mul(&d("1").sub(&acq.ownership)?)?;
        total = total.add(&share.quantize(-2, Rounding::HalfUp)?)?;
    }
    Ok(total)
}

pub(super) fn shown_rows(rows: &[Row]) -> Calc<Value> {
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(json!({
            "account": r.account,
            "class": r.class,
            "combined": r.combined.to_string(),
            "eliminations": r.eliminations.to_string(),
            "consolidated": r.combined.add(&r.eliminations)?.to_string(),
        }));
    }
    Ok(Value::Array(out))
}

/// One posting as a debit or a credit: a rise in an asset or expense, or a
/// fall in anything else, is a debit.
pub(super) fn shown_posting(p: &Posting) -> Calc<Value> {
    let up = p.change.gt(&d("0"))?;
    let size = if up {
        p.change.clone()
    } else {
        p.change.neg()?
    };
    let debit = matches!(p.class, "asset" | "expense") == up;
    let (dr, cr) = if debit {
        (size.to_string(), "0.00".to_string())
    } else {
        ("0.00".to_string(), size.to_string())
    };
    Ok(json!({"account": p.account, "class": p.class, "debit": dr, "credit": cr}))
}

pub(super) fn shown_journals(journals: &[Journal]) -> Calc<Value> {
    let mut out = Vec::with_capacity(journals.len());
    for j in journals {
        let lines = j
            .postings
            .iter()
            .map(shown_posting)
            .collect::<Calc<Vec<_>>>()?;
        out.push(json!({"journal": j.id, "authority": j.authority, "lines": lines}));
    }
    Ok(Value::Array(out))
}

pub(super) fn sum(rows: &[Row], class: &str, skip: &str) -> Calc<Decimal> {
    let mut total = d("0");
    for r in rows
        .iter()
        .filter(|r| r.class == class && r.account != skip)
    {
        total = total.add(&r.combined)?.add(&r.eliminations)?;
    }
    Ok(total)
}

/// Goodwill, profit and its attribution (para B94), the non-controlling
/// interests, the owners' equity and the totals.
pub(super) fn totals(
    g: &Group,
    rows: &[Row],
    after_tax: &[Decimal],
    goodwill: Decimal,
) -> Calc<Vec<(&'static str, Decimal)>> {
    let consolidated: Vec<(&'static str, Decimal)> = rows
        .iter()
        .map(|r| Ok((r.class, r.combined.add(&r.eliminations)?)))
        .collect::<Calc<_>>()?;
    let consolidated = profit(consolidated.into_iter())?;
    let nci_share = nci_profit(g, after_tax)?;
    let owners = consolidated.sub(&nci_share)?;
    let owners_equity = sum(rows, "equity", NCI)?;
    let nci = sum(rows, "equity", "")?
        .sub(&owners_equity)?
        .add(&nci_share)?;
    Ok(vec![
        ("goodwill", goodwill),
        ("consolidated_profit", consolidated),
        ("profit_attributable_to_owners", owners.clone()),
        (
            "profit_attributable_to_non_controlling_interests",
            nci_share,
        ),
        ("non_controlling_interests", nci),
        ("equity_attributable_to_owners", owners_equity.add(&owners)?),
        ("total_assets", sum(rows, "asset", "")?),
        ("total_liabilities", sum(rows, "liability", "")?),
    ])
}
