// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Reading and checking `franking-account`'s inputs.

use super::*;

/// A list input of at most 10,000 objects whose keys are all in `fields`.
pub(super) fn objects<'a>(
    value: &'a Arg,
    name: &str,
    fields: &[&str],
    code: &'static str,
) -> Result<Vec<&'a Map<String, Value>>> {
    let Arg::Json(Value::Array(items)) = value else {
        return refuse(
            code,
            format!("{name} must be a list of at most 10000 objects."),
        );
    };
    if items.len() > 10_000 {
        return refuse(
            code,
            format!("{name} must be a list of at most 10000 objects."),
        );
    }
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let Some(o) = item.as_object() else {
                return bad(code, name, i + 1, "each must be an object.");
            };
            if let Some(key) = o.keys().find(|k| !fields.contains(&k.as_str())) {
                return refuse(
                    "input.field_unknown",
                    format!("{name} {}: {key} is not an input of {COMMAND}", i + 1),
                );
            }
            Ok(o)
        })
        .collect()
}

/// `field` of `o` as an amount of AUD, or `None` when it is not one.
pub(super) fn aud(o: &Map<String, Value>, field: &str) -> Option<Decimal> {
    let parsed = o
        .get(field)?
        .as_str()
        .and_then(|s| Decimal::parse_input(s).ok())?;
    money(&Arg::Decimal(parsed)).ok()
}

/// `field` of `o` as a day number, or `None` when it is not a date.
pub(super) fn day(o: &Map<String, Value>, field: &str) -> Option<i64> {
    o.get(field)?.as_str().and_then(dates::parse)
}

/// `value` as a percentage from 0 to 100, or `None`.
pub(super) fn percentage(value: &Value) -> Option<Decimal> {
    let parsed = value.as_str().and_then(|s| Decimal::parse_input(s).ok())?;
    let ok = parsed.is_finite()
        && !parsed.lt(&int(0)).ok()?
        && parsed.le(&int(100)).ok()?
        && parsed.exponent() >= -10;
    ok.then_some(parsed)
}

pub(super) fn event(n: usize, o: &Map<String, Value>, facts: &Facts) -> Result<Event> {
    let fail = |why: String| bad("franking_account.event_invalid", "event", n, why);
    let name = o.get("type").and_then(Value::as_str).unwrap_or_default();
    let Some(kind) = EVENT_TYPES.iter().find(|t| t.name == name) else {
        let names: Vec<&str> = EVENT_TYPES.iter().map(|t| t.name).collect();
        return fail(format!("type must be one of {}.", names.join(", ")));
    };
    let refund = kind.name == "income_tax_refund";
    // s 205-50(2): a refund paid within 3 months after the year's end can be
    // taken to have been received just before it.
    let (last, after) = if refund {
        (
            dates::add_months(facts.year_end, 3),
            " or the 3 months after it",
        )
    } else {
        (facts.year_end, "")
    };
    let Some(date) = day(o, "date").filter(|d| (facts.year_start..=last).contains(d)) else {
        return fail(format!(
            "date must be a YYYY-MM-DD date in the income year{after}."
        ));
    };
    let Some(amount) = aud(o, "amount").filter(|a| !a.is_zero()) else {
        return fail("amount must be a decimal string of AUD above nil, at most 2 decimal places and 1000000000000.00.".into());
    };
    let rd_part = match (refund, o.get("rd_offset_part")) {
        (false, None) => int(0),
        (false, Some(_)) => return fail("rd_offset_part is for an income_tax_refund only.".into()),
        (true, _) => match aud(o, "rd_offset_part") {
            Some(part) if part.le(&amount)? => part,
            _ => return fail("an income_tax_refund needs rd_offset_part, the part of the refund attributable to a refundable R&D tax offset (s 205-30(2)(b)), from 0.00 to the amount.".into()),
        },
    };
    Ok(Event {
        date,
        kind,
        amount,
        rd_part,
    })
}

pub(super) fn distribution(
    n: usize,
    o: &Map<String, Value>,
    facts: &Facts,
) -> Result<Distribution> {
    let fail = |why: &str| {
        bad(
            "franking_account.distribution_invalid",
            "distribution",
            n,
            why,
        )
    };
    let Some(date) = day(o, "date").filter(|d| (facts.year_start..=facts.year_end).contains(d))
    else {
        return fail("date must be a YYYY-MM-DD date in the income year.");
    };
    let Some(amount) = aud(o, "amount").filter(|a| !a.is_zero()) else {
        return fail(
            "amount must be the frankable distribution in AUD, above nil, at most 2 decimal places.",
        );
    };
    let Some(allocated) = aud(o, "franking_credit") else {
        return fail(
            "franking_credit must be the franking credit allocated to it in AUD, at most 2 decimal places.",
        );
    };
    let permitted = match o.get("permitted_percentage").map(percentage) {
        None => None,
        Some(Some(p)) => Some(p),
        Some(None) => return fail("permitted_percentage must be a decimal string from 0 to 100."),
    };
    let late_statement = match o.get("statement_after_distribution") {
        None => false,
        Some(Value::Bool(late)) if !*late || facts.private => *late,
        Some(Value::Bool(_)) => {
            return fail(
                "only a private company may give a distribution statement after the distribution (s 202-75(2) and (3)).",
            );
        }
        Some(_) => return fail("statement_after_distribution must be a boolean."),
    };
    Ok(Distribution {
        date,
        amount,
        allocated,
        permitted,
        late_statement,
    })
}

/// The corporate tax rate for imputation purposes and its gross-up rate,
/// (100% less the rate) over the rate (s 995-1(1), 'corporate tax gross-up
/// rate'): 25% or 30% for 2025-26 (the ATO's section B, 'Over-franking tax').
pub(super) fn rates(value: &Arg) -> Result<(Decimal, Decimal)> {
    let rate = match value {
        Arg::Decimal(r) if r.equals(&d("0.25"))? || r.equals(&d("0.30"))? => r.clone(),
        _ => {
            return refuse(
                "franking_account.rate_invalid",
                "corporate_tax_rate must be \"0.25\" or \"0.30\": the entity's corporate tax rate for imputation purposes for the income year.",
            );
        }
    };
    let gross_up = precise(|| Ok(int(1).sub(&rate)?.div(&rate)?))?;
    Ok((rate, gross_up))
}

pub(super) fn income_tax(value: Option<&Arg>, private: bool) -> Result<Option<IncomeTax>> {
    let Some(value) = value else { return Ok(None) };
    let Arg::Json(Value::Object(o)) = value else {
        return refuse(
            "franking_account.income_tax_invalid",
            "income_tax must be an object.",
        );
    };
    if let Some(key) = o.keys().find(|k| !INCOME_TAX_FIELDS.contains(&k.as_str())) {
        return refuse(
            "input.field_unknown",
            format!("income_tax: {key} is not an input of {COMMAND}"),
        );
    }
    let (Some(before_offset), Some(brought_forward)) = (
        aud(o, "before_fdt_offset"),
        aud(o, "offset_brought_forward"),
    ) else {
        return refuse(
            "franking_account.income_tax_invalid",
            "income_tax needs before_fdt_offset, the income tax for the year after every tax offset s 63-10 applies before this one, and offset_brought_forward, the offset left from the last year (s 205-70(1)(c)), each in AUD.",
        );
    };
    let earlier_liability = match (o.get("earlier_year_liability"), private) {
        (Some(Value::Bool(b)), true) => Some(*b),
        (None, false) => None,
        (_, true) => {
            return refuse(
                "franking_account.income_tax_invalid",
                "income_tax needs earlier_year_liability for a private company: true when it had an income tax liability for any earlier income year (s 205-70(5)(c)).",
            );
        }
        (Some(_), false) => {
            return refuse(
                "franking_account.income_tax_invalid",
                "earlier_year_liability is for a private company only (s 205-70(5)(a)).",
            );
        }
    };
    Ok(Some(IncomeTax {
        before_offset,
        brought_forward,
        earlier_liability,
    }))
}

/// The benchmark of the last franking period before the year with a
/// frankable distribution, and the franking periods from the one after it to
/// the year's first, counted (s 204-70(1)).
pub(super) fn last_benchmark(value: &Arg) -> Result<Option<(Decimal, i64)>> {
    let bad = || {
        refuse(
            "franking_account.last_benchmark_invalid",
            "last_benchmark must be null, when no frankable distribution was made in an earlier franking period, or an object with percentage (a decimal string from 0 to 100, the benchmark franking percentage of the last earlier franking period in which one was made) and periods_since (a whole number from 1: the franking periods from the one after it to the income year's first, counted).",
        )
    };
    match value {
        Arg::Json(Value::Null) => Ok(None),
        Arg::Json(Value::Object(o)) if o.len() == 2 => {
            let pct = o.get("percentage").and_then(percentage);
            let since = o.get("periods_since").and_then(Value::as_i64);
            match (pct, since) {
                (Some(p), Some(n)) if (1..=1000).contains(&n) => Ok(Some((p, n))),
                _ => bad(),
            }
        }
        _ => bad(),
    }
}

pub(super) fn read_facts(a: &FrankingArgs, year: &str) -> Result<Facts> {
    let private = flag(a.private_company, "private_company")?;
    let exempt = flag(a.benchmark_rule_exempt, "benchmark_rule_exempt")?;
    if private && exempt {
        return refuse(
            "franking_account.facts_conflict",
            "A private company is not a listed public company or its 100% subsidiary, so s 203-20 cannot exempt it from the benchmark rule.",
        );
    }
    let (rate, gross_up) = rates(a.corporate_tax_rate)?;
    let end_year: i64 = year[..4].parse::<i64>().expect("income year") + 1;
    Ok(Facts {
        year_start: dates::parse(&format!("{}-07-01", end_year - 1)).expect("1 July"),
        year_end: dates::parse(&format!("{end_year}-06-30")).expect("30 June"),
        private,
        exempt,
        rate,
        gross_up,
    })
}
