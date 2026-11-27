// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Reading the stated trial balances and acquisitions.

use super::*;

pub(super) fn object<'v>(
    v: &'v Value,
    what: &str,
    keys: &[&str],
) -> Result<&'v Map<String, Value>> {
    match v.as_object() {
        Some(o)
            if o.keys().all(|k| keys.contains(&k.as_str()))
                && keys.iter().all(|k| o.contains_key(*k)) =>
        {
            Ok(o)
        }
        _ => bad(
            "consolidation_eliminations.item_invalid",
            format!(
                "{what}: must be an object with exactly {}.",
                keys.join(", ")
            ),
        ),
    }
}

pub(super) fn label(v: &Value, what: &str) -> Result<String> {
    match v.as_str() {
        Some(t) if !t.trim().is_empty() && t.chars().count() <= LONGEST_LABEL => Ok(t.to_string()),
        _ => bad(
            "consolidation_eliminations.item_invalid",
            format!("{what}: must be text of 1 to {LONGEST_LABEL} characters."),
        ),
    }
}

pub(super) fn amount(v: &Value, what: &str, signed: bool) -> Result<Decimal> {
    let parsed = v
        .as_str()
        .and_then(|t| Decimal::parse_input(t).ok())
        .map(Arg::Decimal);
    let checked = parsed.map(|a| if signed { signed_money(&a) } else { money(&a) });
    match checked {
        Some(Ok(v)) => Ok(v),
        _ => bad(
            "consolidation_eliminations.item_invalid",
            format!(
                "{what}: must be a decimal string, at most 2 dp and 1e12{}.",
                if signed {
                    " either side of nil"
                } else {
                    ", from nil"
                }
            ),
        ),
    }
}

pub(super) fn list<'a>(
    given: &'a Arg,
    what: &str,
    least: usize,
    most: usize,
) -> Result<&'a Vec<Value>> {
    match given {
        Arg::Json(Value::Array(items)) if (least..=most).contains(&items.len()) => Ok(items),
        _ => bad(
            "consolidation_eliminations.item_invalid",
            format!("{what} must be a list of {least} to {most} entries."),
        ),
    }
}

pub(super) fn class(v: &Value, what: &str) -> Result<&'static str> {
    match CLASSES.iter().find(|c| Some(**c) == v.as_str()) {
        Some(c) => Ok(c),
        None => bad(
            "consolidation_eliminations.item_invalid",
            format!("{what}: class must be one of {}.", CLASSES.join(", ")),
        ),
    }
}

pub(super) fn line(v: &Value, what: &str) -> Result<Line> {
    let o = object(v, what, &["account", "class", "amount"])?;
    Ok(Line {
        account: label(&o["account"], what)?,
        class: class(&o["class"], what)?,
        amount: amount(&o["amount"], what, true)?,
    })
}

/// Debits (assets and expenses) less credits on natural sides: nil when the
/// trial balance balances.
pub(super) fn imbalance(lines: &[Line]) -> Calc<Decimal> {
    let mut net = d("0");
    for l in lines {
        net = if matches!(l.class, "asset" | "expense") {
            net.add(&l.amount)?
        } else {
            net.sub(&l.amount)?
        };
    }
    Ok(net)
}

pub(super) fn lines(v: &Value, name: &str) -> Result<Vec<Line>> {
    let items = match v.as_array() {
        Some(items) if (1..=MOST_LINES).contains(&items.len()) => items,
        _ => {
            return bad(
                "consolidation_eliminations.item_invalid",
                format!("{name}: lines must be a list of 1 to {MOST_LINES} lines."),
            );
        }
    };
    let read: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, l)| line(l, &format!("{name} line {}", i + 1)))
        .collect::<Result<_>>()?;
    for (i, l) in read.iter().enumerate() {
        if read[..i].iter().any(|o| o.account == l.account)
            || [NCI, GOODWILL].contains(&l.account.as_str())
        {
            return bad(
                "consolidation_eliminations.account_invalid",
                format!(
                    "{name}: {} appears twice or is a name the worksheet keeps for itself ({NCI}, {GOODWILL}).",
                    l.account
                ),
            );
        }
    }
    if !imbalance(&read)?.is_zero() {
        return bad(
            "consolidation_eliminations.unbalanced",
            format!("{name}: assets and expenses must equal liabilities, equity and income."),
        );
    }
    Ok(read)
}

pub(super) fn parent(given: &Arg) -> Result<Entity> {
    let Arg::Json(v) = given else {
        return bad(
            "consolidation_eliminations.item_invalid",
            "parent must be an object {name, lines}.".into(),
        );
    };
    let o = object(v, "parent", &["name", "lines"])?;
    let name = label(&o["name"], "parent name")?;
    Ok(Entity {
        lines: lines(&o["lines"], &name)?,
        name,
        acquisition: None,
    })
}

pub(super) fn ownership(v: &Value, what: &str) -> Result<Decimal> {
    let parsed = v.as_str().and_then(|t| Decimal::parse_input(t).ok());
    match parsed {
        Some(r) if r.is_finite() && r.exponent() >= -6 && r.gt(&d("0"))? && !r.gt(&d("1"))? => {
            Ok(r)
        }
        _ => bad(
            "consolidation_eliminations.ownership_invalid",
            format!(
                "{what}: ownership must be a decimal fraction above 0 and at most 1, at most 6 places."
            ),
        ),
    }
}

pub(super) fn acquisition(o: &Map<String, Value>, name: &str) -> Result<Acquisition> {
    let own = ownership(&o["ownership"], name)?;
    let nci = amount(
        &o["nci_at_acquisition"],
        &format!("{name} nci_at_acquisition"),
        false,
    )?;
    if own.equals(&d("1"))? != nci.is_zero() {
        return bad(
            "consolidation_eliminations.nci_invalid",
            format!(
                "{name}: nci_at_acquisition must be nil for a wholly owned subsidiary and above nil otherwise (AASB 3 para 19)."
            ),
        );
    }
    let equity = match o["equity_at_acquisition"].as_array() {
        Some(items) if (1..=MOST_LINES).contains(&items.len()) => items,
        _ => {
            return bad(
                "consolidation_eliminations.item_invalid",
                format!(
                    "{name}: equity_at_acquisition must be a list of 1 to {MOST_LINES} entries."
                ),
            );
        }
    };
    let equity = equity
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let what = format!("{name} equity_at_acquisition {}", i + 1);
            let e = object(e, &what, &["account", "amount"])?;
            Ok((
                label(&e["account"], &what)?,
                amount(&e["amount"], &what, true)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let investment_account = label(
        &o["investment_account"],
        &format!("{name} investment_account"),
    );
    Ok(Acquisition {
        ownership: own,
        investment_account: investment_account?,
        consideration: amount(&o["consideration"], &format!("{name} consideration"), false)?,
        nci,
        equity,
    })
}

pub(super) fn subsidiaries(given: &Arg) -> Result<Vec<Entity>> {
    const KEYS: [&str; 7] = [
        "name",
        "lines",
        "ownership",
        "investment_account",
        "consideration",
        "nci_at_acquisition",
        "equity_at_acquisition",
    ];
    let items = list(given, "subsidiaries", 1, MOST_SUBSIDIARIES)?;
    items
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let o = object(v, &format!("subsidiary {}", i + 1), &KEYS)?;
            let name = label(&o["name"], &format!("subsidiary {} name", i + 1))?;
            Ok(Entity {
                lines: lines(&o["lines"], &name)?,
                acquisition: Some(acquisition(o, &name)?),
                name,
            })
        })
        .collect()
}
