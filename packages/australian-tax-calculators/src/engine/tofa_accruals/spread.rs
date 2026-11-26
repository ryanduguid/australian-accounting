// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The rate of return, the accruals and their allocation to income years.

use super::*;

pub(super) fn sign(v: &Decimal) -> Calc<i8> {
    Ok(if v.is_zero() {
        0
    } else if v.gt(&d("0"))? {
        1
    } else {
        -1
    })
}

/// The net benefits times (1 + rate) to the power of the intervals left: zero
/// at the rate of return.
pub(super) fn horner(net: &[Decimal], x: &Decimal) -> Calc<Decimal> {
    let mut acc = d("0");
    for c in net {
        acc = acc.mul(x)?.add(c)?;
    }
    Ok(acc)
}

/// The rate of return per interval: with one change of sign among the net
/// benefits there is exactly one, found by bisection on 1 + rate.
pub(super) fn rate_of_return(net: &[Decimal]) -> Result<Decimal> {
    let signs: Vec<i8> = net.iter().map(sign).collect::<Calc<_>>()?;
    let nonzero: Vec<i8> = signs.into_iter().filter(|s| *s != 0).collect();
    let changes = nonzero.windows(2).filter(|w| w[0] != w[1]).count();
    if changes != 1 {
        return refuse(
            "tofa_accruals.rate_not_unique",
            "The financial benefits must change sign exactly once (amounts provided, then received, or the reverse), so one rate of return spreads the gain or loss; other patterns are not built.",
        );
    }
    let (lead, tail) = (nonzero[0], nonzero[nonzero.len() - 1]);
    let (mut lo, mut hi) = (d("0.25"), d("4"));
    if sign(&horner(net, &lo)?)? != tail || sign(&horner(net, &hi)?)? != lead {
        return refuse(
            "tofa_accruals.rate_out_of_range",
            "The rate of return per interval falls outside -75% to 300%, the bounds of this worksheet.",
        );
    }
    for _ in 0..BISECTIONS {
        let mid = lo.add(&hi)?.div(&d("2"))?;
        if sign(&horner(net, &mid)?)? == lead {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(lo.add(&hi)?.div(&d("2"))?.sub(&d("1"))?)
}

/// Each interval's exact accrual: the opening balance times the rate, the last
/// taking what remains of the gain or loss (s 230-135(5) and (7)).
pub(super) fn accruals(
    net: &[Decimal],
    rate: &Decimal,
    gain: &Decimal,
) -> Calc<Vec<(Decimal, Decimal)>> {
    let mut balance = net[0].neg()?;
    let mut so_far = d("0");
    let mut out = Vec::with_capacity(net.len() - 1);
    for (k, c) in net.iter().enumerate().skip(1) {
        let accrual = if k == net.len() - 1 {
            gain.sub(&so_far)?
        } else {
            balance.mul(rate)?
        };
        out.push((balance.clone(), accrual.clone()));
        so_far = so_far.add(&accrual)?;
        balance = balance.add(&accrual)?.sub(c)?;
    }
    Ok(out)
}

/// Cents by running total, so the rounded parts sum to the rounded whole.
pub(super) fn rounded(parts: &[Decimal]) -> Calc<Vec<Decimal>> {
    let (mut sum, mut before) = (d("0"), d("0.00"));
    let mut out = Vec::with_capacity(parts.len());
    for p in parts {
        sum = sum.add(p)?;
        let now = sum.quantize(-2, Rounding::HalfUp)?;
        out.push(now.sub(&before)?);
        before = now;
    }
    Ok(out)
}

/// The income year (1 July to 30 June) holding `day`, as its label and index.
pub(super) fn income_year(day: i64) -> i64 {
    let (y, m, _) = dates::to_civil(day);
    if m >= 7 { y } else { y - 1 }
}

pub(super) fn label(first: i64) -> String {
    format!("{first}-{:02}", (first + 1) % 100)
}

/// Days of `from..=to` in each income year, from the first.
pub(super) fn days_by_year(from: i64, to: i64) -> Vec<(i64, i64)> {
    let mut out: Vec<(i64, i64)> = Vec::new();
    let mut day = from;
    while day <= to {
        let y = income_year(day);
        let next = dates::parse(&format!("{}-07-01", y + 1)).expect("a date");
        let last = to.min(next - 1);
        out.push((y, last - day + 1));
        day = last + 1;
    }
    out
}

/// Each interval's accrual allocated to income years by days: the operator's
/// stated reasonable basis for an interval straddling 2 (s 230-170(2)).
pub(super) fn by_year(t: &Term, parts: &[Decimal]) -> Calc<Vec<(i64, Decimal)>> {
    let mut years: Vec<(i64, Decimal)> = Vec::new();
    for (k, part) in (1..=t.count).zip(parts) {
        let (from, to) = (t.first_day(k), t.last_day(k));
        let whole = Decimal::from_int(to - from + 1);
        for (y, days) in days_by_year(from, to) {
            let share = part.mul(&Decimal::from_int(days))?.div(&whole)?;
            add_to(&mut years, y, &share)?;
        }
    }
    Ok(years)
}

pub(super) fn add_to(years: &mut Vec<(i64, Decimal)>, y: i64, v: &Decimal) -> Calc<()> {
    match years.iter_mut().find(|(k, _)| *k == y) {
        Some((_, sum)) => *sum = sum.add(v)?,
        None => years.push((y, v.clone())),
    }
    Ok(())
}

/// The gain or loss by straight line over the days of the life
/// (s 230-135(2)(b)), allocated to income years.
pub(super) fn straight_line(t: &Term, gain: &Decimal) -> Calc<Vec<(i64, Decimal)>> {
    let end = t.last_day(t.count);
    let whole = Decimal::from_int(end - t.start + 1);
    days_by_year(t.start, end)
        .into_iter()
        .map(|(y, days)| Ok((y, gain.mul(&Decimal::from_int(days))?.div(&whole)?)))
        .collect()
}

pub(super) fn method(a: &TofaAccrualsArgs) -> Result<bool> {
    match a.method.text() {
        Some("compounding") => Ok(false),
        Some("straight_line") if flag(a.approximation_confirmed, "approximation_confirmed")? => {
            Ok(true)
        }
        Some("straight_line") => refuse(
            "tofa_accruals.approximation_not_confirmed",
            "A straight line is allowed only where its results approximate compounding accruals, having regard to the length of the period (s 230-135(2)(b)): establish that and set approximation_confirmed.",
        ),
        Some("effective_interest") => refuse(
            "tofa_accruals.method_not_built",
            "The effective interest method qualifies only under s 230-140(3)'s conditions, including an audited financial report, which this worksheet does not apply.",
        ),
        _ => refuse(
            "tofa_accruals.method_invalid",
            "method must be compounding or straight_line.",
        ),
    }
}

pub(super) fn interval_rows(t: &Term, net: &[Decimal], shown: &[Decimal]) -> Calc<Vec<Value>> {
    let mut rows = Vec::with_capacity(shown.len());
    let mut opening = net[0].neg()?;
    for (k, accrual) in (1..=t.count).zip(shown) {
        let benefit = &net[usize::try_from(k).expect("an index")];
        let closing = opening.add(accrual)?.sub(benefit)?;
        rows.push(json!({
            "interval": k,
            "start": dates::format(t.first_day(k)),
            "end": dates::format(t.last_day(k)),
            "opening_balance": opening.quantize(-2, Rounding::HalfUp)?.to_string(),
            "accrual": accrual.to_string(),
            "financial_benefits": benefit.quantize(-2, Rounding::HalfUp)?.to_string(),
            "closing_balance": closing.quantize(-2, Rounding::HalfUp)?.to_string(),
        }));
        opening = closing;
    }
    Ok(rows)
}

pub(super) fn year_rows(
    compounding: &[(i64, Decimal)],
    straight: Option<&[(i64, Decimal)]>,
) -> Calc<Vec<Value>> {
    let c = rounded(
        &compounding
            .iter()
            .map(|(_, v)| v.clone())
            .collect::<Vec<_>>(),
    )?;
    let s = match straight {
        Some(s) => Some(rounded(
            &s.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>(),
        )?),
        None => None,
    };
    Ok(compounding
        .iter()
        .enumerate()
        .map(|(i, (y, _))| {
            let mut row = json!({"income_year": label(*y), "compounding": c[i].to_string()});
            if let Some(s) = &s {
                row["straight_line"] = json!(s[i].to_string());
            }
            row
        })
        .collect())
}

pub(super) struct Spread {
    pub(super) rate: Decimal,
    pub(super) gain: Decimal,
    pub(super) intervals: Vec<Value>,
    pub(super) years: Vec<Value>,
    pub(super) first_year: Decimal,
}

pub(super) fn spread(t: &Term, net: &[Decimal], straight: bool) -> Result<Spread> {
    let mut gain = d("0");
    for c in net {
        gain = gain.add(c)?;
    }
    if gain.is_zero() {
        return refuse(
            "tofa_accruals.no_gain_or_loss",
            "The financial benefits net to nil, so there is no overall gain or loss for the accruals method to spread (s 230-100(2)(a)).",
        );
    }
    let rate = rate_of_return(net)?;
    let exact = accruals(net, &rate, &gain)?;
    let parts: Vec<Decimal> = exact.iter().map(|(_, a)| a.clone()).collect();
    let shown = rounded(&parts)?;
    let compounding = by_year(t, &parts)?;
    let line = if straight {
        Some(straight_line(t, &gain)?)
    } else {
        None
    };
    let years = year_rows(&compounding, line.as_deref())?;
    let key = if straight {
        "straight_line"
    } else {
        "compounding"
    };
    Ok(Spread {
        first_year: d(years[0][key].as_str().expect("an amount")),
        intervals: interval_rows(t, net, &shown)?,
        years,
        rate,
        gain,
    })
}
