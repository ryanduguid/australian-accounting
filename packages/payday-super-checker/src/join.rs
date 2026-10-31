//! Join a payroll export to a super-payments export.

use std::cmp::Ordering;

use crate::csv_io::{cents, dmin, zero};
use crate::decimal::Decimal;
use crate::profiles::normalise_name;
use crate::py::{Date, MAX_DATE};
use crate::{Failure, Result};

#[derive(Clone, Debug)]
pub struct PayrollRow {
    pub employee_id: Option<String>,
    pub employee_name: Option<String>,
    pub payday: Date,
    pub period_end: Option<Date>,
    pub sg_amount: Decimal,
    pub row: i64,
}

impl PayrollRow {
    pub fn effective_period_end(&self) -> Date {
        self.period_end.unwrap_or(self.payday)
    }
}

#[derive(Clone, Debug)]
pub struct SuperRow {
    pub employee_id: Option<String>,
    pub employee_name: Option<String>,
    pub period_start: Option<Date>,
    pub period_end: Option<Date>,
    pub paid_date: Option<Date>,
    pub amount: Decimal,
    pub row: i64,
    pub unpaid_status: Option<String>,
    /// The vendor status says a fund returned part of this payment's batch.
    pub refund_unreconciled: bool,
}

#[derive(Clone, Debug)]
pub struct MatchOutcome {
    /// Index into the payroll rows.
    pub payroll: usize,
    pub remitted: Option<Date>,
    pub flag: String,
    pub last_known_paid_date: Option<Date>,
    pub remitted_amount: Option<Decimal>,
    pub matched_amount: Option<Decimal>,
    pub refund_unreconciled: bool,
}

pub const ORPHAN_NO_PAYDAY: &str = "no payday matched";
pub const ORPHAN_PAYDAYS_SETTLED: &str = "paydays already settled";
pub const ORPHAN_NOTHING_OWED: &str = "paydays owe nothing";
pub const ORPHAN_NO_AMOUNT: &str = "payment has nothing to allocate";

#[derive(Clone, Debug)]
pub struct OrphanReason {
    /// Index into the super rows.
    pub super_row: usize,
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct JoinResult {
    pub payroll_rows: Vec<PayrollRow>,
    pub super_rows: Vec<SuperRow>,
    pub outcomes: Vec<MatchOutcome>,
    /// Indices into the super rows.
    pub orphans: Vec<usize>,
    pub key_mode: String,
    pub warnings: Vec<String>,
    pub orphan_reasons: Vec<OrphanReason>,
}

/// The identity the two files are matched on.
pub fn key(employee_id: Option<&str>, employee_name: Option<&str>, mode: &str) -> String {
    if mode == "id" {
        return employee_id.unwrap_or("").to_string();
    }
    normalise_name(employee_name.unwrap_or(""))
}

pub fn payroll_key(row: &PayrollRow, mode: &str) -> String {
    key(
        row.employee_id.as_deref(),
        row.employee_name.as_deref(),
        mode,
    )
}

pub fn super_key(row: &SuperRow, mode: &str) -> String {
    key(
        row.employee_id.as_deref(),
        row.employee_name.as_deref(),
        mode,
    )
}

/// Whether a super row's period includes a date, both ends inclusive.
pub fn covers(s: &SuperRow, target: Date) -> bool {
    match (s.period_start, s.period_end) {
        (None, end) => end == Some(target),
        (Some(start), None) => start == target,
        (Some(start), Some(end)) => start <= target && target <= end,
    }
}

fn check_reversed_periods(super_rows: &[SuperRow]) -> Result<()> {
    for s in super_rows {
        if let (Some(start), Some(end)) = (s.period_start, s.period_end)
            && start > end
        {
            return Err(Failure::Value(format!(
                "row {}: pay period start {} is after period end {}. That is not a valid pay \
                 period, so this payment cannot be matched to a payday without guessing which \
                 one was meant.",
                s.row,
                start.isoformat(),
                end.isoformat()
            )));
        }
    }
    Ok(())
}

fn coverage(s: &SuperRow, candidates: &[usize], payroll: &[PayrollRow]) -> Vec<usize> {
    if s.period_start.is_none() && s.period_end.is_none() {
        return candidates.to_vec();
    }
    candidates
        .iter()
        .copied()
        .filter(|&r| covers(s, payroll[r].effective_period_end()))
        .collect()
}

fn unmet(row: usize, payroll: &[PayrollRow], allocated: &[Decimal]) -> Result<Decimal> {
    let left = payroll[row].sg_amount.sub(&allocated[row])?;
    // `max(Decimal("0"), left)`: the first argument unless the second is larger.
    Ok(if left.gt(&zero())? { left } else { zero() })
}

fn check_defensible(
    s: &SuperRow,
    competing: &[usize],
    payroll: &[PayrollRow],
    allocated: &[Decimal],
) -> Result<()> {
    let mut need = zero();
    for &r in competing {
        need = need.add(&unmet(r, payroll, allocated)?)?;
    }
    if !s.amount.lt(&need)? {
        return Ok(());
    }
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for &r in competing {
        let row = &payroll[r];
        let mut found = None;
        for (g, members) in groups.iter().enumerate() {
            let first = &payroll[members[0]];
            if first.payday == row.payday
                && first.effective_period_end() == row.effective_period_end()
                && first.sg_amount.equals(&row.sg_amount)?
            {
                found = Some(g);
                break;
            }
        }
        match found {
            Some(g) => groups[g].push(r),
            None => groups.push(vec![r]),
        }
    }
    for members in groups {
        if members.len() <= 1 {
            continue;
        }
        let mut numbers: Vec<i64> = members.iter().map(|&r| payroll[r].row).collect();
        numbers.sort();
        let numbers: Vec<String> = numbers.iter().map(i64::to_string).collect();
        let numbers = numbers.join(", ");
        if s.period_start.is_none() && s.period_end.is_none() {
            return Err(Failure::Value(format!(
                "super row {} has no pay period on record, so it is treated as \
                 covering every payday for this employee, and rows {numbers} are \
                 identical in payday, pay period and amount as well, so there is no \
                 defensible way to decide which of them the payment settled. The \
                 super file is missing its pay period column(s) -- add them so the \
                 payment can be matched to a specific payday, or give the payroll \
                 rows distinct pay periods or amounts.",
                s.row
            )));
        }
        return Err(Failure::Value(format!(
            "super row {} covers rows {numbers}, which are identical in payday, \
             pay period and amount, so there is no defensible way to decide which of \
             them the payment settled. Give them distinct pay periods or amounts, or \
             merge the duplicate.",
            s.row
        )));
    }
    Ok(())
}

fn order_key(row: &PayrollRow) -> (Date, Date, i64) {
    (row.payday, row.effective_period_end(), row.row)
}

fn allocate(
    s: &SuperRow,
    covered: &[usize],
    payroll: &[PayrollRow],
    allocated: &[Decimal],
) -> Result<Vec<(usize, Decimal)>> {
    let mut ordered = covered.to_vec();
    ordered.sort_by_key(|&r| order_key(&payroll[r]));
    let mut remaining = s.amount.clone();
    let mut allocations: Vec<(usize, Decimal)> = Vec::new();
    for r in ordered {
        if remaining.le(&zero())? {
            break;
        }
        let share = dmin(&remaining, &unmet(r, payroll, allocated)?)?;
        if share.gt(&zero())? {
            allocations.push((r, share.clone()));
        }
        remaining = remaining.sub(&share)?;
    }
    if remaining.gt(&zero())? && !allocations.is_empty() {
        let mut newest = 0;
        for i in 1..allocations.len() {
            if order_key(&payroll[allocations[i].0]) > order_key(&payroll[allocations[newest].0]) {
                newest = i;
            }
        }
        allocations[newest].1 = allocations[newest].1.add(&remaining)?;
    }
    Ok(allocations)
}

fn opt_date(d: Option<Date>, fallback: Date) -> Date {
    d.unwrap_or(fallback)
}

const MIN_DATE: Date = Date {
    year: 1,
    month: 1,
    day: 1,
};

/// `_super_order`: a total order over super rows.
fn super_order(a: &SuperRow, b: &SuperRow) -> Result<Ordering> {
    let text = |s: &Option<String>| s.clone().unwrap_or_default();
    let ordering = a
        .row
        .cmp(&b.row)
        .then_with(|| {
            crate::pytext::code_points(&text(&a.employee_id))
                .cmp(crate::pytext::code_points(&text(&b.employee_id)))
        })
        .then_with(|| {
            crate::pytext::code_points(&text(&a.employee_name))
                .cmp(crate::pytext::code_points(&text(&b.employee_name)))
        })
        .then_with(|| opt_date(a.period_start, MIN_DATE).cmp(&opt_date(b.period_start, MIN_DATE)))
        .then_with(|| opt_date(a.period_end, MIN_DATE).cmp(&opt_date(b.period_end, MIN_DATE)))
        .then_with(|| opt_date(a.paid_date, MIN_DATE).cmp(&opt_date(b.paid_date, MIN_DATE)));
    if ordering != Ordering::Equal {
        return Ok(ordering);
    }
    Ok(a.amount.compare(&b.amount)?)
}

/// Sort indices stably with a fallible comparison.
fn sort_indices(
    indices: &mut [usize],
    mut cmp: impl FnMut(usize, usize) -> Result<Ordering>,
) -> Result<()> {
    let mut failure = None;
    indices.sort_by(|&a, &b| match cmp(a, b) {
        Ok(o) => o,
        Err(e) => {
            failure.get_or_insert(e);
            Ordering::Equal
        }
    });
    failure.map_or(Ok(()), Err)
}

fn why_orphaned(
    covered: &[usize],
    payroll: &[PayrollRow],
    allocated: &[Decimal],
) -> Result<(String, String)> {
    if covered.is_empty() {
        return Ok((ORPHAN_NO_PAYDAY.into(), "matched no payday".into()));
    }
    // `all(r.sg_amount == 0 ...)`: stops at the first non-zero, and a
    // signalling NaN raises InvalidOperation.
    let mut all_zero = true;
    for &r in covered {
        if !payroll[r].sg_amount.equals(&zero())? {
            all_zero = false;
            break;
        }
    }
    if all_zero {
        return Ok((
            ORPHAN_NOTHING_OWED.into(),
            "matched only paydays that owe no super guarantee".into(),
        ));
    }
    let mut all_settled = true;
    for &r in covered {
        if !unmet(r, payroll, allocated)?.equals(&zero())? {
            all_settled = false;
            break;
        }
    }
    if all_settled {
        return Ok((
            ORPHAN_PAYDAYS_SETTLED.into(),
            "matched only paydays that were already settled by other payments, so \
             this payment is on top of what was owed"
                .into(),
        ));
    }
    Ok((
        ORPHAN_NO_AMOUNT.into(),
        "matched paydays but carries no amount to allocate".into(),
    ))
}

/// Whether each file structurally carries a period column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Columns {
    pub payroll_has_period_end: bool,
    pub super_has_period_start: bool,
    pub super_has_period_end: bool,
}

impl Default for Columns {
    fn default() -> Columns {
        Columns {
            payroll_has_period_end: true,
            super_has_period_start: true,
            super_has_period_end: true,
        }
    }
}

pub fn both_have_ids(payroll: &[PayrollRow], supers: &[SuperRow]) -> bool {
    !payroll.is_empty()
        && !supers.is_empty()
        && payroll
            .iter()
            .all(|r| r.employee_id.as_deref().is_some_and(|s| !s.is_empty()))
        && supers
            .iter()
            .all(|r| r.employee_id.as_deref().is_some_and(|s| !s.is_empty()))
}

/// Match payroll rows to the super payments that settled them.
pub fn join(
    payroll_rows: Vec<PayrollRow>,
    super_rows: Vec<SuperRow>,
    columns: Columns,
) -> Result<JoinResult> {
    check_reversed_periods(&super_rows)?;
    let mut warnings = Vec::new();
    let key_mode = if both_have_ids(&payroll_rows, &super_rows) {
        "id"
    } else {
        "name"
    };
    if key_mode == "name" {
        warnings.push(
            "matched on employee name because one of the files has no id column. \
             Two employees sharing a name would be merged."
                .to_string(),
        );
    }
    if !columns.payroll_has_period_end {
        warnings.push(
            "the payroll file has no pay period end column, so matching falls back \
             to the payday. A super payment recorded against the pay period rather \
             than the payday it was paid on could be missed."
                .into(),
        );
    }
    if !(columns.super_has_period_start && columns.super_has_period_end) {
        if columns.super_has_period_start || columns.super_has_period_end {
            warnings.push(
                "the super file has only one of the pay period start/end columns, so \
                 a payment's coverage collapses to a single day and could miss the \
                 payday it actually settled."
                    .into(),
            );
        } else {
            warnings.push(
                "the super file has no pay period columns at all, so a payment \
                 cannot be matched to a specific payday by its period and is treated \
                 as covering every payday for that employee, which can trigger the \
                 same-payment ambiguity check for an employee with more than one \
                 payday."
                    .into(),
            );
        }
    }

    let mut grouped: crate::Groups<usize> = crate::Groups::default();
    for (i, row) in payroll_rows.iter().enumerate() {
        let k = payroll_key(row, key_mode);
        if k.is_empty() {
            return Err(Failure::Value(format!(
                "row {}: the employee column is empty",
                row.row
            )));
        }
        grouped.push(k, i);
    }
    let coverage_of: Vec<Vec<usize>> = super_rows
        .iter()
        .map(|s| {
            let k = super_key(s, key_mode);
            let candidates = grouped.get(&k).map_or(&[][..], Vec::as_slice);
            coverage(s, candidates, &payroll_rows)
        })
        .collect();

    let mut contributions: Vec<Vec<(usize, Decimal, String)>> =
        vec![Vec::new(); payroll_rows.len()];
    let mut allocated: Vec<Decimal> = vec![zero(); payroll_rows.len()];
    let mut used = vec![false; super_rows.len()];

    let mut order: Vec<usize> = (0..super_rows.len()).collect();
    sort_indices(&mut order, |a, b| {
        let (sa, sb) = (&super_rows[a], &super_rows[b]);
        match opt_date(sa.paid_date, MAX_DATE).cmp(&opt_date(sb.paid_date, MAX_DATE)) {
            Ordering::Equal => super_order(sa, sb),
            other => Ok(other),
        }
    })?;
    for si in order {
        let s = &super_rows[si];
        let covered = &coverage_of[si];
        let mut competing = Vec::new();
        for &r in covered {
            if unmet(r, &payroll_rows, &allocated)?.gt(&zero())? {
                competing.push(r);
            }
        }
        check_defensible(s, &competing, &payroll_rows, &allocated)?;
        let allocations = allocate(s, covered, &payroll_rows, &allocated)?;
        let shared = competing.len() > 1;
        let paid = s
            .paid_date
            .map_or_else(|| "no date on record".to_string(), |d| d.isoformat());
        for (r, share) in &allocations {
            let note = if shared {
                format!(
                    "{} of {} allocated from super row {} (paid {paid}), one of {} paydays \
                     that payment covered",
                    share.to_sci_string(),
                    s.amount.to_sci_string(),
                    s.row,
                    covered.len()
                )
            } else {
                String::new()
            };
            contributions[*r].push((si, share.clone(), note));
            allocated[*r] = allocated[*r].add(share)?;
        }
        if !allocations.is_empty() {
            used[si] = true;
        }
    }

    let mut outcomes = Vec::new();
    for (i, row) in payroll_rows.iter().enumerate() {
        if row.sg_amount.equals(&zero())? {
            outcomes.push(MatchOutcome {
                payroll: i,
                remitted: None,
                flag: "no super guarantee owed for this payday".into(),
                last_known_paid_date: None,
                remitted_amount: None,
                matched_amount: Some(zero()),
                refund_unreconciled: false,
            });
            continue;
        }
        let entries = &contributions[i];
        if entries.is_empty() {
            outcomes.push(MatchOutcome {
                payroll: i,
                remitted: None,
                flag: "no super payment found".into(),
                last_known_paid_date: None,
                remitted_amount: None,
                matched_amount: Some(zero()),
                refund_unreconciled: false,
            });
            continue;
        }
        let mut total = zero();
        for (_, amount, _) in entries {
            total = total.add(amount)?;
        }
        let mut flag_parts: Vec<String> = Vec::new();
        let paid = cents(Some(&total))?;
        let owed = cents(Some(&row.sg_amount))?;
        let matched_amount = dmin(&paid, &owed)?;
        if paid.lt(&owed)? {
            flag_parts.push(format!(
                "partial: {} of {} matched",
                total.to_sci_string(),
                row.sg_amount.to_sci_string()
            ));
        } else if paid.gt(&owed)? {
            flag_parts.push(format!(
                "over: {} against {}, check for salary sacrifice in the contribution types",
                total.to_sci_string(),
                row.sg_amount.to_sci_string()
            ));
        }
        flag_parts.extend(
            entries
                .iter()
                .filter(|(_, _, n)| !n.is_empty())
                .map(|(_, _, n)| n.clone()),
        );
        let paid_date = |si: usize| super_rows[si].paid_date;
        let last_known = entries.iter().filter_map(|(si, _, _)| paid_date(*si)).max();
        let mut dated_total = zero();
        let mut undated_total = zero();
        let mut undated = 0;
        for (si, amount, _) in entries {
            if paid_date(*si).is_some() {
                dated_total = dated_total.add(amount)?;
            } else {
                undated += 1;
                undated_total = undated_total.add(amount)?;
            }
        }
        let remitted_amount = (!dated_total.is_zero()).then_some(dated_total);
        if undated > 0 {
            if undated == entries.len() {
                flag_parts.push("matched super rows carry no payment date".into());
            } else {
                flag_parts.push(format!(
                    "{} of {} matched has no payment date on record; latest known payment date {}",
                    undated_total.to_sci_string(),
                    total.to_sci_string(),
                    last_known.expect("a dated entry").isoformat()
                ));
            }
        }
        outcomes.push(MatchOutcome {
            payroll: i,
            remitted: last_known,
            flag: flag_parts.join("; "),
            last_known_paid_date: last_known,
            remitted_amount,
            matched_amount: Some(matched_amount),
            refund_unreconciled: entries
                .iter()
                .any(|(si, _, _)| super_rows[*si].refund_unreconciled),
        });
    }

    let mut orphans: Vec<usize> = (0..super_rows.len()).filter(|&i| !used[i]).collect();
    sort_indices(&mut orphans, |a, b| {
        super_order(&super_rows[a], &super_rows[b])
    })?;
    let mut orphan_reasons = Vec::new();
    for &si in &orphans {
        let (code, message) = why_orphaned(&coverage_of[si], &payroll_rows, &allocated)?;
        orphan_reasons.push(OrphanReason {
            super_row: si,
            code,
            message,
        });
    }
    Ok(JoinResult {
        payroll_rows,
        super_rows,
        outcomes,
        orphans,
        key_mode: key_mode.into(),
        warnings,
        orphan_reasons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i64, m: i64, d: i64) -> Date {
        Date::new(y, m, d).unwrap()
    }

    fn row(start: Option<Date>, end: Option<Date>) -> SuperRow {
        SuperRow {
            employee_id: None,
            employee_name: Some("A".into()),
            period_start: start,
            period_end: end,
            paid_date: Some(day(2026, 7, 14)),
            amount: Decimal::parse("612.00").unwrap(),
            row: 2,
            unpaid_status: None,
            refund_unreconciled: false,
        }
    }

    #[test]
    fn a_row_with_only_a_period_start_covers_that_day_alone() {
        let start_only = row(Some(day(2026, 7, 1)), None);
        assert!(covers(&start_only, day(2026, 7, 1)));
        assert!(!covers(&start_only, day(2026, 7, 9)));
    }

    #[test]
    fn a_period_less_row_covers_nothing() {
        assert!(!covers(&row(None, None), day(2026, 7, 9)));
    }
}
