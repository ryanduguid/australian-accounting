// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `franking-account`'s result: the summary, the ledger and the rows beside them.

use super::*;

/// An amount to the cent, half up, as text.
pub(super) fn shown(value: &Decimal) -> Result<String> {
    Ok(precise(|| Ok(value.quantize(-2, Rounding::HalfUp)?))?.to_string())
}

pub(super) fn two_places(value: &Decimal) -> Result<String> {
    Ok(precise(|| Ok(value.quantize(-2, Rounding::HalfUp)?))?.to_string())
}

/// What the summary and the refund rows are worked from.
pub(super) struct Parts<'a> {
    pub(super) opening: &'a Decimal,
    pub(super) facts: &'a Facts,
    pub(super) t: &'a Totals,
    pub(super) offset: &'a Offset,
    pub(super) oft: &'a Decimal,
    pub(super) varies: bool,
    pub(super) before_refunds: &'a Decimal,
    pub(super) tax: Option<&'a IncomeTax>,
}

/// The last day of the month after the year's end: the franking return's
/// and franking tax's due date (s 214-150(1); the ATO's 'How to lodge and
/// pay').
pub(super) fn due_date(facts: &Facts) -> i64 {
    dates::add_months(facts.year_end + 1, 1) - 1
}

pub(super) fn summary(year: &str, p: &Parts, account: &Account) -> Result<Value> {
    let t = p.t;
    let from_refunds = precise(|| Ok(t.fdt.sub(p.before_refunds)?))?;
    let payable = precise(|| Ok(t.fdt.add(p.oft)?))?;
    let mut amounts = vec![
        ("opening_balance", p.opening.clone()),
        ("credits", t.credits.clone()),
        ("debits", t.debits.clone()),
        ("closing_balance", t.closing.clone()),
        ("franking_deficit_tax", t.fdt.clone()),
        ("fdt_from_refunds_after_year_end", from_refunds),
        ("fdt_offset_reduction", p.offset.reduction.clone()),
        ("offsetable_fdt", p.offset.amount.clone()),
        ("debits_not_subject_to_reduction", t.label_k.clone()),
        ("over_franking_tax", p.oft.clone()),
        ("franking_tax_payable", payable),
        ("rd_deferred_debit_carried_forward", account.rd_pool.clone()),
    ];
    if let Some(tax) = p.tax {
        amounts.extend(offset_use(p.offset, tax)?);
    }
    let required = !t.fdt.is_zero() || !p.oft.is_zero() || p.varies;
    let rates = vec![
        rate("corporate_tax_rate", &p.facts.rate),
        rate("gross_up_rate", two_places_more(&p.facts.gross_up)?),
        rate(
            "balance_code",
            if t.closing.lt(&int(0))? { "D" } else { "S" },
        ),
        rate("offset_code", p.offset.code.unwrap_or("none")),
        rate("offset_authority", p.offset.authority),
        rate("significant_variation", p.varies),
        rate("return_required", required),
        rate(
            "return_due",
            if required {
                dates::format(due_date(p.facts))
            } else {
                "none".into()
            },
        ),
    ];
    let mut out = result(KIND, year, amounts, rates)?;
    out["return_labels"] = return_labels(&out["amounts"]);
    Ok(out)
}

/// s 205-70(1)(c) and (2): the offset with any brought forward, the part the
/// year's income tax uses and the rest carried forward.
fn offset_use(offset: &Offset, tax: &IncomeTax) -> Result<[(&'static str, Decimal); 3]> {
    let total = precise(|| Ok(offset.amount.add(&tax.brought_forward)?))?;
    let used = if total.lt(&tax.before_offset)? {
        total.clone()
    } else {
        tax.before_offset.clone()
    };
    let carried = precise(|| Ok(total.sub(&used)?))?;
    Ok([
        ("fdt_offset_total", total),
        ("fdt_offset_used", used),
        ("fdt_offset_carried_forward", carried),
    ])
}

/// The franking return's labels, from the amounts.
fn return_labels(amounts: &Value) -> Value {
    json!({
        "balance": amounts["closing_balance"],
        "A": amounts["credits"],
        "B": amounts["franking_deficit_tax"],
        "C": amounts["offsetable_fdt"],
        "D": amounts["over_franking_tax"],
        "E": amounts["franking_tax_payable"],
        "F": amounts["fdt_from_refunds_after_year_end"],
        "K": amounts["debits_not_subject_to_reduction"],
    })
}

/// The gross-up rate to 4 places, as the ATO's example 5 shows it; the
/// arithmetic keeps it exact.
pub(super) fn two_places_more(value: &Decimal) -> Result<String> {
    Ok(precise(|| Ok(value.quantize(-4, Rounding::HalfUp)?))?.to_string())
}

pub(super) fn ledger(opening: &Decimal, lines: &[Line]) -> Result<Value> {
    let mut running = opening.clone();
    let mut rows = Vec::new();
    for line in lines {
        let credit = line.class == Class::Credit;
        running = precise(|| {
            Ok(if credit {
                running.add(&line.amount)?
            } else {
                running.sub(&line.amount)?
            })
        })?;
        rows.push(json!({
            "date": dates::format(line.date),
            "item": line.item,
            "credit": if credit { json!(shown(&line.amount)?) } else { Value::Null },
            "debit": if credit { Value::Null } else { json!(shown(&line.amount)?) },
            "balance": shown(&running)?,
        }));
    }
    Ok(Value::Array(rows))
}

pub(super) fn distributions_out(
    dists: &[Distribution],
    worked: &[Worked],
    periods: &[Period],
) -> Result<Value> {
    let mut rows = Vec::new();
    for (dist, w) in dists.iter().zip(worked) {
        rows.push(json!({
            "date": dates::format(dist.date),
            "amount": shown(&dist.amount)?,
            "franking_credit_allocated": shown(&dist.allocated)?,
            "maximum_franking_credit": shown(&w.maximum)?,
            "franking_credit": shown(&w.credit)?,
            "franking_percentage": two_places(&w.percentage)?,
            "franking_period": dates::format(periods[w.period].start),
            "compared_with": match &w.reference { Some(r) => json!(two_places(r)?), None => Value::Null },
            "over_franking_tax": shown(&w.over_franking_tax)?,
            "under_franking_debit": shown(&w.under_franking_debit)?,
        }));
    }
    Ok(Value::Array(rows))
}

/// Each refund paid after the year's end: those s 205-50(2) takes into the
/// year, with the franking deficit tax each adds and when it is due
/// (s 214-150(4)): with a franking return still outstanding for the year,
/// that return's due date if the refund is accounted for in it, otherwise
/// 14 days after the refund was received; and those left for the next year.
pub(super) fn deferred_out(account: &Account, p: &Parts) -> Result<Value> {
    let outstanding = !p.before_refunds.is_zero() || !p.oft.is_zero() || p.varies;
    let due = due_date(p.facts);
    let mut rows = Vec::new();
    for (date, added) in &account.deferred {
        let with_return = (outstanding && *date <= due).then(|| dates::format(due));
        rows.push(json!({
            "date": dates::format(*date),
            "taken_into_year": true,
            "fdt_added": shown(added)?,
            "due_with_outstanding_return": with_return,
            "due_otherwise": dates::format(date + 14),
        }));
    }
    rows.extend(account.not_posted.iter().cloned());
    Ok(Value::Array(rows))
}
