// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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

/// Each franking period with its benchmark, shown to 2 places, 5 up (the
/// ATO's section C), and whether it differs from the last benchmark by more
/// than 20 percentage points for each franking period after that one, up to
/// and including it (s 204-70(1)). The disclosure does not apply where the
/// benchmark rule does not (s 204-70(2)).
pub(super) fn variations(
    periods: &[Period],
    last: Option<(Decimal, i64)>,
    exempt: bool,
) -> Result<(Vec<Value>, bool)> {
    let mut out = Vec::new();
    let mut any = false;
    // The last benchmark and its period's index, counting this year's first
    // franking period as 0.
    let mut previous: Option<(Decimal, i64)> = last.map(|(p, since)| (p, -since));
    for (k, period) in (0_i64..).zip(periods) {
        let mut row = json!({
            "start": dates::format(period.start),
            "end": dates::format(period.end),
            "benchmark_percentage": Value::Null,
            "significant_variation": false,
        });
        if let Some(b) = &period.benchmark {
            row["benchmark_percentage"] = json!(two_places(b)?);
            if let Some((reference, at)) = &previous {
                let count = k - at;
                let gap = precise(|| Ok(b.sub(reference)?))?;
                let gap = if gap.lt(&int(0))? { gap.neg()? } else { gap };
                let varies = !exempt && gap.gt(&int(20 * count))?;
                row["compared_with"] = json!(two_places(reference)?);
                row["threshold_points"] = json!(20 * count);
                row["significant_variation"] = json!(varies);
                any |= varies;
            }
            previous = Some((b.clone(), k));
        }
        out.push(row);
    }
    Ok((out, any))
}

/// s 202-75(4): a private company's late statement in a deficit year is
/// flagged, not decided.
pub(super) fn late_statement_warning(out: &mut Value) {
    out["warnings"]
        .as_array_mut()
        .expect("warnings")
        .push(json!("ITAA 1997 s 202-75(4): a private company may not give a distribution statement after the distribution if the franking credit it allocates, alone or with the year's others, creates or increases franking deficit tax. A distribution here has a statement given late and the year ends in deficit: check that its allocation was allowed; this command does not decide it."));
}

pub(super) fn sum_of_worked(worked: &[Worked]) -> Result<Decimal> {
    precise(|| {
        worked
            .iter()
            .try_fold(int(0), |t, w| Ok(t.add(&w.over_franking_tax)?))
    })
}
