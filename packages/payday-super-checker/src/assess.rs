//! Payday-super assessment orchestration, verdicts and exposure figures.

use crate::calendar::BusinessCalendar;
use crate::csv_io::{cents, dmax, dmin, money, remitted_credit, zero};
use crate::deadlines::{
    ContribLine, Deadline, EventKind, ITEM4_ALIGNED, REGIME_START, SKIP_DB, USUAL_7BD,
    annotate_calendar_risk, annotate_missing_flag, apply_item4, compute_due,
    earliest_prepayment_day, receipt_amount_cap, receipt_amount_evidenced,
};
use crate::decimal::Decimal;
use crate::py::Date;
use crate::rates::GicTable;
use crate::sgc::{Uplift, exposure_range, notional_earnings, uplift_scenarios};
use crate::{Failure, Result, returned};

pub const TRANSITION_END: Date = Date {
    year: 2026,
    month: 7,
    day: 28,
};

/// The last QE day PCG 2026/1 covers.
const FIRST_YEAR_END: Date = Date {
    year: 2027,
    month: 6,
    day: 30,
};

pub const ON_TIME: &str = "ON_TIME";
pub const LATE: &str = "LATE";
pub const AT_RISK: &str = "AT_RISK";
pub const UNPAID: &str = "UNPAID";
pub const UNKNOWN: &str = "UNKNOWN";
pub const SKIPPED: &str = "SKIPPED";
const NOT_YET_DUE: &str = "NOT_YET_DUE";

pub const VERDICTS: [&str; 6] = [ON_TIME, AT_RISK, LATE, UNPAID, UNKNOWN, SKIPPED];
pub const EXPOSED: [&str; 2] = [LATE, UNPAID];

pub const NO_RECEIPT_CAVEAT: &str = "no fund-receipt date supplied: the statutory test is receipt by the \
     fund (SGAA s 18C(1)), so a remittance date alone cannot show the \
     contribution was on time";

/// Australian financial year label for a date, for example 2026-27.
pub fn financial_year(d: Date) -> String {
    let start = if d.month >= 7 { d.year } else { d.year - 1 };
    let next = (start + 1).to_string();
    format!("{start}-{}", &next[2.min(next.len())..])
}

/// One assessed contribution line (Python's `Result`).
#[derive(Clone, Debug)]
pub struct Assessed {
    pub line: ContribLine,
    pub deadline: Deadline,
    pub verdict: String,
    pub days_late: Option<i64>,
    pub lateness_basis: String,
    pub base_shortfall: Option<Decimal>,
    pub final_shortfall: Option<Decimal>,
    pub offset_s18d: bool,
    pub nec: Option<Decimal>,
    pub sgc_low: Option<Decimal>,
    pub sgc_high: Option<Decimal>,
    pub uplift: Option<Uplift>,
    pub notes: Vec<String>,
    pub caveats: Vec<String>,
    pub horizon_verdicts: Option<(String, String)>,
    pub receipt_established: bool,
}

impl Assessed {
    pub fn is_exposed(&self) -> bool {
        EXPOSED.contains(&self.verdict.as_str())
    }

    pub fn warnings(&self) -> Vec<String> {
        self.caveats.iter().chain(&self.notes).cloned().collect()
    }
}

fn date_problem(line: &ContribLine) -> Option<String> {
    match (line.received, line.remitted) {
        (Some(received), Some(remitted)) if received < remitted => Some(format!(
            "row {}: fund receipt date {} is before the \
             remittance date {}, which cannot happen",
            line.row,
            received.isoformat(),
            remitted.isoformat()
        )),
        _ => None,
    }
}

fn received_credit(line: &ContribLine, received_as_at: Option<Date>) -> Result<Decimal> {
    if received_as_at.is_none() {
        return Ok(zero());
    }
    let owed = cents(Some(&line.sg_amount))?;
    dmin(&cents(Some(receipt_amount_cap(line)))?, &owed)
}

fn amount_problem(line: &ContribLine) -> Result<Option<String>> {
    let row = line.row;
    if line.sg_amount.lt(&zero())? {
        return Ok(Some(format!("row {row}: sg_amount cannot be negative")));
    }
    if let Some(r) = &line.remitted_amount {
        if r.lt(&zero())? || r.gt(&line.sg_amount)? {
            return Ok(Some(format!(
                "row {row}: remitted_amount must be between zero and sg_amount"
            )));
        }
        if line.remitted.is_none() {
            return Ok(Some(format!(
                "row {row}: remitted_amount requires remitted_date"
            )));
        }
    }
    if let Some(m) = &line.matched_amount {
        if m.lt(&zero())? || m.gt(&line.sg_amount)? {
            return Ok(Some(format!(
                "row {row}: matched_amount must be between zero and sg_amount"
            )));
        }
        if let Some(r) = &line.remitted_amount
            && r.gt(m)?
        {
            return Ok(Some(format!(
                "row {row}: remitted_amount cannot exceed matched_amount"
            )));
        }
        if m.lt(&line.sg_amount)? && line.remitted.is_some() && line.remitted_amount.is_none() {
            return Ok(Some(format!(
                "row {row}: matched_amount below sg_amount requires \
                 remitted_amount when remitted_date is present"
            )));
        }
    }
    Ok(None)
}

/// The fields two rows must share to be counted as one doubled row.
struct DuplicateKey {
    employee_id: String,
    qe_day: Date,
    sg_amount: Decimal,
    remitted: Option<Date>,
    effective_remitted_amount: Option<Decimal>,
    effective_matched_amount: Decimal,
    received: Option<Date>,
    flags: (bool, bool, Option<Date>, bool),
}

fn same_decimal(a: &Decimal, b: &Decimal) -> Result<bool> {
    Ok(a.equals(b)?)
}

impl DuplicateKey {
    fn same(&self, other: &DuplicateKey) -> Result<bool> {
        let optional = |a: &Option<Decimal>, b: &Option<Decimal>| -> Result<bool> {
            match (a, b) {
                (None, None) => Ok(true),
                (Some(a), Some(b)) => same_decimal(a, b),
                _ => Ok(false),
            }
        };
        Ok(self.employee_id == other.employee_id
            && self.qe_day == other.qe_day
            && same_decimal(&self.sg_amount, &other.sg_amount)?
            && self.remitted == other.remitted
            && optional(
                &self.effective_remitted_amount,
                &other.effective_remitted_amount,
            )?
            && same_decimal(
                &self.effective_matched_amount,
                &other.effective_matched_amount,
            )?
            && self.received == other.received
            && self.flags == other.flags)
    }
}

/// The key fields compared exactly; amounts are compared as values within
/// a bucket, as Python's tuple equality compared Decimals.
type Bucket = (
    String,
    Date,
    Option<Date>,
    Option<Date>,
    (bool, bool, Option<Date>, bool),
);

fn flag_duplicates(lines: &mut [ContribLine]) -> Result<()> {
    let mut groups: Vec<(DuplicateKey, Vec<usize>)> = Vec::new();
    let mut buckets: std::collections::HashMap<Bucket, Vec<usize>> =
        std::collections::HashMap::new();
    for (i, line) in lines.iter().enumerate() {
        let key = DuplicateKey {
            employee_id: line.employee_id.clone(),
            qe_day: line.qe_day,
            sg_amount: line.sg_amount.clone(),
            remitted: line.remitted,
            effective_remitted_amount: match line.remitted {
                Some(r) => Some(remitted_credit(line, Some(r))?),
                None => None,
            },
            effective_matched_amount: cents(Some(receipt_amount_cap(line)))?,
            received: line.received,
            flags: (
                line.first_to_fund,
                line.out_of_cycle,
                line.next_standard_qe_day,
                line.db_interest,
            ),
        };
        let bucket = buckets
            .entry((
                key.employee_id.clone(),
                key.qe_day,
                key.remitted,
                key.received,
                key.flags,
            ))
            .or_default();
        let mut found = None;
        for &g in bucket.iter() {
            if groups[g].0.same(&key)? {
                found = Some(g);
                break;
            }
        }
        match found {
            Some(g) => groups[g].1.push(i),
            None => {
                bucket.push(groups.len());
                groups.push((key, vec![i]));
            }
        }
    }
    for (_, members) in groups {
        if members.len() > 1 {
            let rows: Vec<String> = members.iter().map(|&i| lines[i].row.to_string()).collect();
            let note = format!(
                "rows {} are identical, so this payday is counted \
                 {} times. Check the export was not doubled",
                rows.join(", "),
                members.len()
            );
            for &i in &members {
                lines[i].duplicate_note = note.clone();
            }
        }
    }
    Ok(())
}

/// The options an assessment runs under.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Options {
    pub assessment_date: Option<Date>,
    pub transition_allocation_confirmed: bool,
    pub allow_stale_gic: bool,
}

fn rows_listed(lines: &[&ContribLine]) -> String {
    let rows: Vec<String> = lines.iter().take(10).map(|l| l.row.to_string()).collect();
    let more = if lines.len() > 10 {
        format!(" and {} more", lines.len() - 10)
    } else {
        String::new()
    };
    format!("{}{more}", rows.join(", "))
}

/// Every amount and date problem in the lines, refused together.
pub fn check_inputs(lines: &[ContribLine]) -> Result<()> {
    let mut problems = Vec::new();
    for line in lines {
        if let Some(p) = amount_problem(line)? {
            problems.push(p);
        }
        if let Some(p) = date_problem(line) {
            problems.push(p);
        }
        problems.extend(returned::event_problems(line)?);
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(Failure::Value(problems.join("; ")))
    }
}

/// Assess each contribution line.
pub fn assess(
    mut lines: Vec<ContribLine>,
    cal: &BusinessCalendar,
    gic: &GicTable,
    as_at: Date,
    options: Options,
) -> Result<Vec<Assessed>> {
    check_inputs(&lines)?;
    flag_duplicates(&mut lines)?;

    let pre_regime: Vec<&ContribLine> = lines.iter().filter(|l| l.qe_day < REGIME_START).collect();
    if !pre_regime.is_empty() {
        let earliest = pre_regime
            .iter()
            .map(|l| l.qe_day)
            .min()
            .expect("not empty");
        return Err(Failure::Value(format!(
            "{} row(s) have a QE day before 1 Jul 2026 (rows {}; earliest {}): the old \
             quarterly SG law applies to them and this tool covers payday super only. \
             Remove them and run again.",
            pre_regime.len(),
            rows_listed(&pre_regime),
            earliest.isoformat()
        )));
    }

    let mut transition = vec![false; lines.len()];
    let mut transition_rows: Vec<&ContribLine> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let contribution_date = line.received.or(line.remitted);
        let event_in_transition = line.events.iter().any(|e| e.date <= TRANSITION_END);
        if !line.db_interest
            && line.sg_amount.gt(&zero())?
            && (event_in_transition
                || (receipt_amount_cap(line).gt(&zero())?
                    && contribution_date.is_some_and(|d| d <= TRANSITION_END)))
        {
            transition[i] = true;
            transition_rows.push(line);
        }
    }
    if !transition_rows.is_empty() && !options.transition_allocation_confirmed {
        return Err(Failure::Value(format!(
            "{} row(s) use a contribution dated no later than \
             28 Jul 2026 (rows {}). LCR 2026/1 requires pre-1 July \
             amounts to be unused excess and 1-28 July amounts to reduce any \
             employee June-quarter shortfall first. This file cannot calculate \
             those old-regime balances. Reconcile them for every affected employee, \
             then rerun with --confirm-transition-allocation; no payroll payment, \
             lodgment or accounting decision is made by this tool",
            transition_rows.len(),
            rows_listed(&transition_rows)
        )));
    }

    let mut pairs = Vec::with_capacity(lines.len());
    for line in lines {
        let dl = compute_due(&line, cal)?;
        pairs.push((line, dl));
    }
    apply_item4(&mut pairs, Some(as_at))?;
    annotate_missing_flag(&mut pairs);
    annotate_calendar_risk(&mut pairs, cal);

    let mut results = Vec::with_capacity(pairs.len());
    for (i, (line, dl)) in pairs.into_iter().enumerate() {
        results.push(assess_line(
            line,
            dl,
            cal,
            gic,
            as_at,
            options,
            transition[i],
        )?);
    }
    Ok(results)
}

struct Facts {
    settled: Option<Date>,
    remitted: Option<Date>,
    credit: Decimal,
    operational_unremitted: Decimal,
    fully_remitted: bool,
    receipt_credit: Decimal,
    receipt_covers_all: bool,
    past_horizon: bool,
    horizon_unknown: String,
    horizon_figures: String,
    possible_item4_due: Option<Date>,
    item4_uncertain: bool,
    item4_unknown: String,
    item4_partial_unknown: String,
    horizon_partial_unknown: String,
    receipt_amount_unevidenced: bool,
    /// The late credit an event trail counted, which replaces the single
    /// receipt's s 18D test.
    event_late_credit: Option<Decimal>,
    /// The trail keeps more than one receipt, so item 1 would extend only the first.
    several_receipts: bool,
}

fn horizon_unknown_text() -> String {
    "the date recorded here is after that deadline, and a holiday the \
     calendar does not hold could move the deadline past it, so the line is \
     left unassessed rather than called late. Supply the missing holidays \
     with --holidays-override and set its \"verified_until\" to the last date \
     you entered them for, or extend data/business_days.json, \
     to assess it"
        .to_string()
}

fn horizon_figures_text(due: Date, cal: &BusinessCalendar) -> String {
    format!(
        "the deadline {} runs past the calendar's coverage \
         ({}) and can only move later, so days late \
         is left blank and the notional earnings and SG charge figures on this \
         line are a maximum, not a settled amount",
        due.isoformat(),
        cal.coverage_until.isoformat()
    )
}

fn item4_unknown_text(possible: Date) -> String {
    format!(
        "an earlier positive row could extend this deadline to \
         {} under s 18C(2) item 4, but the \
         file does not evidence an eligible contribution received by the fund, \
         applied to that earlier QE day and on time. The deadline shown is the \
         latest one proved by the supplied facts; reconcile the fund receipt and \
         statutory allocation before deciding between the candidate verdicts",
        possible.isoformat()
    )
}

fn assessment_facts(result: &mut Assessed, cal: &BusinessCalendar, as_at: Date) -> Result<Facts> {
    let line = result.line.clone();
    let due = result.deadline.due.expect("assessed lines have a deadline");
    let settled = line.received.filter(|r| *r <= as_at);
    let remitted = line.remitted.filter(|r| *r <= as_at);
    result.receipt_established = settled.is_some();
    if let Some(received) = line.received
        && received > as_at
    {
        result.caveats.push(format!(
            "fund receipt date {} is after the as-at date \
             {}: it is ignored for this as-at report",
            received.isoformat(),
            as_at.isoformat()
        ));
    }
    if let Some(r) = line.remitted
        && r > as_at
    {
        result.caveats.push(format!(
            "remittance date {} is after the as-at date \
             {}: it is ignored for this as-at report",
            r.isoformat(),
            as_at.isoformat()
        ));
    }
    if settled.is_none() && remitted.is_some() {
        if let Some(received) = line.received {
            result.caveats.push(format!(
                "the only fund-receipt date on record \
                 ({}) is after the as-at date, so as at \
                 {} the statutory test of receipt by the fund \
                 (SGAA s 18C(1)) is not met and a remittance date alone cannot \
                 show the contribution was on time",
                received.isoformat(),
                as_at.isoformat()
            ));
        } else {
            result.caveats.push(NO_RECEIPT_CAVEAT.into());
        }
    }
    let credit = remitted_credit(&line, remitted)?;
    let mut operational_unremitted = cents(Some(&line.sg_amount))?.sub(&credit)?;
    if operational_unremitted.lt(&zero())? {
        operational_unremitted = zero();
    }
    let fully_remitted = operational_unremitted.is_zero();
    let receipt_credit = received_credit(&line, settled)?;
    let receipt_amount_unevidenced =
        settled.is_some() && !receipt_amount_evidenced(&line) && line.sg_amount.gt(&zero())?;
    let receipt_covers_all =
        !receipt_credit.lt(&cents(Some(&line.sg_amount))?)? && !receipt_amount_unevidenced;
    if receipt_amount_unevidenced {
        result.caveats.push(format!(
            "the fund receipt dated {} carries no amount: neither \
             matched_amount nor remitted_amount is on this row, so how much the fund \
             received against the ${} SG amount is unknown. A \
             receipt date alone is not evidence of a full receipt, so the line is not \
             read as ON_TIME. Supply matched_amount (the amount the fund received for \
             this payday) from the fund or clearing-house confirmation and rerun",
            settled
                .expect("unevidenced receipts are settled")
                .isoformat(),
            money(Some(&line.sg_amount))?
        ));
    }
    if credit.gt(&zero())? && operational_unremitted.gt(&zero())? {
        result.notes.push(format!(
            "part-paid: ${} of ${} is evidenced \
             as remitted, leaving ${} unremitted. \
             Remittance does not establish statutory credit until the fund receipt \
             and its timing are evidenced",
            money(Some(&credit))?,
            money(Some(&line.sg_amount))?,
            money(Some(&operational_unremitted))?
        ));
    }
    let past_horizon = due > cal.coverage_until;
    let horizon_unknown = horizon_unknown_text();
    let horizon_figures = horizon_figures_text(due, cal);
    let possible_item4_due = result.deadline.possible_item4_due;
    let item4_uncertain = possible_item4_due.is_some_and(|p| p > due);
    let item4_unknown = match possible_item4_due {
        Some(p) if item4_uncertain => item4_unknown_text(p),
        _ => String::new(),
    };
    Ok(Facts {
        settled,
        remitted,
        credit,
        operational_unremitted,
        fully_remitted,
        receipt_credit,
        receipt_covers_all,
        past_horizon,
        horizon_unknown,
        horizon_figures,
        possible_item4_due,
        item4_uncertain,
        item4_unknown,
        item4_partial_unknown:
            "because the evidenced fund receipt covers only part of the SG amount, \
            the unresolved item 4 deadline also leaves UNPAID possible: that is \
            the outcome if the actual deadline falls on or after the receipt date \
            but on or before the as-at date"
                .into(),
        horizon_partial_unknown:
            "because the evidenced fund receipt covers only part of the SG amount, \
            a missing holiday also leaves UNPAID possible: that is the outcome if \
            the actual deadline falls on or after the receipt date but on or before \
            the as-at date"
                .into(),
        receipt_amount_unevidenced,
        event_late_credit: None,
        several_receipts: false,
    })
}

fn receipt_could_be_timely(
    line: &ContribLine,
    due: Date,
    settled: Date,
    facts: &Facts,
) -> Result<bool> {
    if settled < line.qe_day {
        return Ok(settled >= earliest_prepayment_day(line.qe_day)?);
    }
    if settled <= due || facts.past_horizon {
        return Ok(true);
    }
    Ok(facts.item4_uncertain && facts.possible_item4_due.is_some_and(|p| settled <= p))
}

fn verdicts(worse: &str, better: &str) -> Option<(String, String)> {
    Some((worse.to_string(), better.to_string()))
}

/// Whether a possible item 4 deadline has not yet passed.
fn item4_open(facts: &Facts, as_at: Date) -> bool {
    facts.item4_uncertain && facts.possible_item4_due.is_some_and(|p| as_at <= p)
}

/// (on-time receipt credit, stale pre-payment, finished).
fn assess_received(
    result: &mut Assessed,
    as_at: Date,
    facts: &Facts,
) -> Result<(Decimal, bool, bool)> {
    let line = result.line.clone();
    let due = result.deadline.due.expect("assessed lines have a deadline");
    let settled = facts.settled.expect("a settled receipt");
    let mut stale_prepayment = false;
    let mut on_time = zero();
    if facts.receipt_amount_unevidenced && receipt_could_be_timely(&line, due, settled, facts)? {
        result.verdict = UNKNOWN.into();
        let worse = if settled > due {
            LATE
        } else if due < as_at {
            UNPAID
        } else {
            NOT_YET_DUE
        };
        result.horizon_verdicts = verdicts(worse, ON_TIME);
        if settled > due {
            if facts.item4_uncertain {
                result.caveats.push(facts.item4_unknown.clone());
            }
            if facts.past_horizon {
                result.caveats.push(facts.horizon_unknown.clone());
            }
            result.caveats.push(
                "because the receipt amount is not evidenced, UNPAID is also \
                 possible: that is the outcome if the receipt covered only part \
                 of the SG amount and the actual deadline falls on or after the \
                 receipt date but on or before the as-at date"
                    .into(),
            );
        }
        return Ok((on_time, stale_prepayment, true));
    }
    if settled < line.qe_day {
        let earliest = earliest_prepayment_day(line.qe_day)?;
        if settled >= earliest {
            on_time = facts.receipt_credit.clone();
            result.notes.push(format!(
                "${} received before the QE day: counted \
                 as an on-time pre-payment under s 18C(1)(c)(ii)",
                money(Some(&facts.receipt_credit))?
            ));
            if facts.receipt_covers_all {
                result.verdict = ON_TIME.into();
            } else if due >= as_at {
                result.caveats.push(
                    "the eligible pre-payment covers only part of the SG amount, \
                     but the deadline has not passed, so there is nothing to \
                     assess yet"
                        .into(),
                );
                return Ok((on_time, stale_prepayment, true));
            } else if facts.past_horizon || item4_open(facts, as_at) {
                result.verdict = UNKNOWN.into();
                result.horizon_verdicts = verdicts(UNPAID, NOT_YET_DUE);
                if facts.past_horizon {
                    result.caveats.push(
                        "the eligible pre-payment covers only part of the SG \
                         amount, but the deadline runs past the calendar's \
                         coverage and may not have passed. No exposure is \
                         calculated until the missing holiday facts are supplied"
                            .into(),
                    );
                }
                if facts.item4_uncertain {
                    result.caveats.push(facts.item4_unknown.clone());
                }
                return Ok((on_time, stale_prepayment, true));
            } else {
                result.verdict = UNPAID.into();
            }
        } else {
            stale_prepayment = true;
            result.caveats.push(format!(
                "received {}, before the 12-month pre-payment \
                 window in s 18C(1)(c)(ii), so it cannot be applied to this payday. \
                 The payday is treated as unfunded",
                settled.isoformat()
            ));
            if due >= as_at {
                result.caveats.push(
                    "the deadline has not passed, so there is nothing to assess \
                     on this payday yet"
                        .into(),
                );
                return Ok((on_time, stale_prepayment, true));
            }
            if facts.past_horizon || item4_open(facts, as_at) {
                result.verdict = UNKNOWN.into();
                result.horizon_verdicts = verdicts(LATE, NOT_YET_DUE);
                if facts.past_horizon {
                    result.caveats.push(
                        "the payday is unfunded, but the deadline runs past the \
                         calendar's coverage and may not have passed. No exposure \
                         is calculated until the missing whole-of-jurisdiction \
                         holiday facts are supplied"
                            .into(),
                    );
                }
                if facts.item4_uncertain {
                    result.caveats.push(facts.item4_unknown.clone());
                }
                return Ok((on_time, stale_prepayment, true));
            }
            result.verdict = LATE.into();
        }
    } else if facts.item4_uncertain
        && let Some(possible) = facts.possible_item4_due
        && due < settled
        && settled <= possible
    {
        result.verdict = UNKNOWN.into();
        let partial_better = if as_at <= possible {
            NOT_YET_DUE
        } else {
            UNPAID
        };
        result.horizon_verdicts = verdicts(
            LATE,
            if facts.receipt_covers_all {
                ON_TIME
            } else {
                partial_better
            },
        );
        result.caveats.push(facts.item4_unknown.clone());
        if !facts.receipt_covers_all {
            if partial_better == NOT_YET_DUE {
                result.caveats.push(facts.item4_partial_unknown.clone());
            } else {
                result.caveats.push(
                    "the possible item 4 deadline has now passed. Because \
                     the evidenced fund receipt covers only part of the SG \
                     amount, the later-deadline outcome is UNPAID rather than \
                     NOT_YET_DUE"
                        .into(),
                );
            }
        }
        return Ok((on_time, stale_prepayment, true));
    } else if facts.past_horizon && settled > due {
        result.verdict = UNKNOWN.into();
        result.horizon_verdicts = verdicts(
            LATE,
            if facts.receipt_covers_all {
                ON_TIME
            } else {
                NOT_YET_DUE
            },
        );
        result.caveats.push(facts.horizon_unknown.clone());
        if !facts.receipt_covers_all {
            result.caveats.push(facts.horizon_partial_unknown.clone());
        }
        return Ok((on_time, stale_prepayment, true));
    } else if settled <= due {
        on_time = facts.receipt_credit.clone();
        if facts.receipt_covers_all {
            result.verdict = ON_TIME.into();
        } else if facts.past_horizon || item4_open(facts, as_at) {
            result.verdict = UNKNOWN.into();
            result.horizon_verdicts = verdicts(UNPAID, NOT_YET_DUE);
            if facts.past_horizon {
                result.caveats.push(
                    "the fund receipt covers only part of the SG amount, but \
                     the deadline runs past the calendar's coverage and may \
                     not have passed. No exposure is calculated until the \
                     missing holiday facts are supplied"
                        .into(),
                );
            }
            if facts.item4_uncertain {
                result.caveats.push(facts.item4_unknown.clone());
            }
            return Ok((on_time, stale_prepayment, true));
        } else if due < as_at {
            result.verdict = UNPAID.into();
        } else {
            result.caveats.push(
                "the fund receipt covers only part of the SG amount, but the \
                 deadline has not passed, so there is nothing to assess yet"
                    .into(),
            );
            return Ok((on_time, stale_prepayment, true));
        }
    } else {
        result.verdict = LATE.into();
    }
    Ok((on_time, stale_prepayment, false))
}

/// The remittance or receipt dates the as-at filter discarded, in date order.
fn ignored_dates(line: &ContribLine, as_at: Date) -> Vec<Date> {
    let mut dates: Vec<Date> = [line.remitted, line.received]
        .into_iter()
        .flatten()
        .filter(|d| *d > as_at)
        .collect();
    dates.sort();
    dates
}

fn joined_dates(dates: &[Date]) -> String {
    dates
        .iter()
        .map(|d| d.isoformat())
        .collect::<Vec<_>>()
        .join(", ")
}

fn assess_without_receipt(result: &mut Assessed, as_at: Date, facts: &Facts) -> Result<bool> {
    let line = result.line.clone();
    let due = result.deadline.due.expect("assessed lines have a deadline");
    if let (Some(remitted), true) = (facts.remitted, facts.fully_remitted) {
        if facts.item4_uncertain
            && facts
                .possible_item4_due
                .is_some_and(|p| due < remitted && remitted <= p)
        {
            result.verdict = UNKNOWN.into();
            result.horizon_verdicts = verdicts(LATE, AT_RISK);
            result.caveats.push(facts.item4_unknown.clone());
            return Ok(true);
        }
        if facts.past_horizon && remitted > due {
            result.verdict = UNKNOWN.into();
            result.horizon_verdicts = verdicts(LATE, AT_RISK);
            result.caveats.push(facts.horizon_unknown.clone());
            return Ok(true);
        }
        result.verdict = if remitted <= due { AT_RISK } else { LATE }.into();
    } else if due < as_at {
        if facts.past_horizon || item4_open(facts, as_at) {
            result.verdict = UNKNOWN.into();
            result.horizon_verdicts = verdicts(UNPAID, NOT_YET_DUE);
            if facts.past_horizon {
                result.caveats.push(format!(
                    "no fund receipt is established for this payday, but the \
                     deadline shown ({}) runs past the calendar's \
                     coverage and may not have passed. The row is left unassessed \
                     with no exposure until the missing whole-of-jurisdiction \
                     holiday facts are supplied",
                    due.isoformat()
                ));
            }
            if facts.item4_uncertain {
                result.caveats.push(facts.item4_unknown.clone());
            }
            return Ok(true);
        }
        result.verdict = UNPAID.into();
        let ignored = ignored_dates(&line, as_at);
        let none_recorded = if facts.credit.gt(&zero())? {
            format!(
                "${} is evidenced as remitted, leaving \
                 ${} unremitted, and no \
                 fund-receipt date is recorded \
                 for the remainder",
                money(Some(&facts.credit))?,
                money(Some(&facts.operational_unremitted))?
            )
        } else if ignored.is_empty() {
            "no remittance or fund-receipt date is recorded".into()
        } else {
            format!(
                "the only remittance or fund-receipt date on record ({}) is after the as-at \
                 date and is ignored here",
                joined_dates(&ignored)
            )
        };
        result.caveats.push(format!(
            "the deadline passed on {} and {none_recorded}. \
             Figures assume the contribution is \
             still unpaid; if your export has no date columns, supply them \
             before relying on this",
            due.isoformat()
        ));
    } else {
        let ignored = ignored_dates(&line, as_at);
        let head = if ignored.is_empty() {
            "no remittance or fund-receipt date supplied".to_string()
        } else {
            format!(
                "the only remittance or fund-receipt date on record ({}) is after the as-at \
                 date and is ignored here",
                joined_dates(&ignored)
            )
        };
        result.caveats.push(format!(
            "{head}, and the deadline has not passed: nothing to assess yet"
        ));
        return Ok(true);
    }
    Ok(false)
}

#[allow(clippy::too_many_arguments)]
fn apply_exposure(
    result: &mut Assessed,
    cal: &BusinessCalendar,
    gic: &GicTable,
    as_at: Date,
    options: Options,
    facts: &Facts,
    on_time: &Decimal,
    stale_prepayment: bool,
) -> Result<()> {
    let line = result.line.clone();
    let due = result.deadline.due.expect("assessed lines have a deadline");
    let settled = facts.settled;
    let owed = cents(Some(&line.sg_amount))?;
    let base_shortfall = dmax(&owed.sub(on_time)?, &zero())?;
    let landed = settled.or(facts.remitted);

    if on_time.gt(&zero())? && base_shortfall.gt(&zero())? {
        result.notes.push(format!(
            "an eligible on-time fund receipt of \
             ${} reduces the base shortfall to \
             ${}; the remaining amount is unfunded",
            money(Some(on_time))?,
            money(Some(&base_shortfall))?
        ));
    }

    let mut offset_credit = zero();
    if let Some(counted) = &facts.event_late_credit {
        offset_credit = dmin(counted, &base_shortfall)?;
    } else if let Some(s) = settled
        && !stale_prepayment
        && s > due
        && options.assessment_date.is_none_or(|a| s < a)
    {
        if facts.receipt_amount_unevidenced {
            result.caveats.push(format!(
                "the late fund receipt dated {} carries no \
                 amount, so the s 18D reduction of the final shortfall is not \
                 applied: the final shortfall shown is the whole base shortfall \
                 and is a maximum until matched_amount is supplied",
                s.isoformat()
            ));
        } else {
            offset_credit = dmin(&facts.receipt_credit, &base_shortfall)?;
        }
    }
    let final_shortfall = dmax(&base_shortfall.sub(&offset_credit)?, &zero())?;
    let offset = offset_credit.gt(&zero())?;

    if offset {
        let s = settled.expect("an offset needs a receipt");
        let assumption = match options.assessment_date {
            Some(a) => format!("before the assessment on {}", a.isoformat()),
            None => "and no SG charge assessment had issued for this payday by then".into(),
        };
        if final_shortfall.is_zero() {
            result.notes.push(format!(
                "contribution received {} {assumption}: \
                 the final shortfall is nil under s 18D, leaving notional \
                 earnings and uplift",
                s.isoformat()
            ));
        } else {
            result.notes.push(format!(
                "contribution received {} {assumption}: \
                 s 18D reduces the final shortfall by \
                 ${} to ${}; it \
                 does not clear the remaining shortfall",
                s.isoformat(),
                money(Some(&offset_credit))?,
                money(Some(&final_shortfall))?
            ));
        }
    }

    let mut nec_end;
    let outstanding_to;
    match settled {
        Some(s) if !stale_prepayment && facts.receipt_covers_all => {
            outstanding_to = s;
            result.lateness_basis = "fund receipt".into();
            nec_end = if final_shortfall.is_zero() { s } else { as_at };
        }
        Some(_) if !stale_prepayment && facts.receipt_amount_unevidenced => {
            nec_end = as_at;
            outstanding_to = as_at;
            result.lateness_basis = "as-at date (fund receipt amount not evidenced)".into();
            result.notes.push(
                "the fund receipt carries no amount: notional earnings are estimated on \
                 the whole base shortfall to the as-at date and are a maximum. They \
                 would end at the receipt date if matched_amount showed it covered the \
                 whole liability"
                    .into(),
            );
        }
        Some(_) if !stale_prepayment => {
            nec_end = as_at;
            outstanding_to = as_at;
            result.lateness_basis = "as-at date (shortfall remains after part receipt)".into();
            result.notes.push(
                "the fund received only part of the liability: notional earnings \
                 continue on the base shortfall while the final shortfall remains \
                 greater than nil"
                    .into(),
            );
        }
        _ if landed.is_some() && !stale_prepayment => {
            nec_end = as_at;
            outstanding_to = as_at;
            result.lateness_basis = "as-at date (no fund receipt recorded)".into();
            result.notes.push(
                "treated as still unpaid at the as-at date: notional earnings keep \
                 accruing until the fund receives the contribution"
                    .into(),
            );
        }
        _ => {
            nec_end = as_at;
            outstanding_to = as_at;
            result.lateness_basis = "as-at date (nothing applied to this payday)".into();
        }
    }

    if let Some(a) = options.assessment_date
        && nec_end >= a
    {
        nec_end = a.add_days(-1)?;
        result.caveats.push(format!(
            "notional earnings stop {}, the day before the \
             assessment. Interest on the unpaid charge after that is general \
             interest charge, which this tool does not estimate",
            nec_end.isoformat()
        ));
    }

    result.days_late = if facts.past_horizon {
        None
    } else {
        Some(outstanding_to.days_since(due).max(0))
    };
    if facts.past_horizon {
        result.caveats.push(facts.horizon_figures.clone());
    }

    result.offset_s18d = offset;
    result.base_shortfall = Some(base_shortfall.clone());
    result.final_shortfall = Some(final_shortfall.clone());
    let nec = if nec_end > due {
        notional_earnings(&base_shortfall, due, nec_end, gic, options.allow_stale_gic)
    } else {
        Ok(zero())
    };
    match nec {
        Err(Failure::StaleGic(message)) => {
            result.caveats.push(format!(
                "notional earnings and the SG charge estimate are not assessed for this \
                 row: {message}"
            ));
        }
        Err(other) => return Err(other),
        Ok(nec) => {
            result.uplift = Some(uplift_scenarios(&final_shortfall, &nec)?);
            let (low, high) = exposure_range(&final_shortfall, &nec)?;
            result.sgc_low = Some(low);
            result.sgc_high = Some(high);
            result.nec = Some(nec);
        }
    }

    if result.deadline.pathway == USUAL_7BD
        && !facts.several_receipts
        && let Some(landed) = landed
        && !stale_prepayment
        && landed >= line.qe_day
    {
        let extended = cal.add_business_days(line.qe_day, 20)?;
        if landed <= extended {
            let unfunded = cents(Some(&line.sg_amount))?.sub(&facts.receipt_credit)?;
            let becomes = if settled.is_some() && facts.receipt_covers_all {
                "the line becomes on time".to_string()
            } else if settled.is_some() && facts.receipt_amount_unevidenced {
                "the line can become on time only once matched_amount \
                 evidences that the fund received the whole amount"
                    .to_string()
            } else if settled.is_none() && facts.fully_remitted {
                "the line stops being late, though it stays at risk until \
                 you supply a fund-receipt date"
                    .to_string()
            } else {
                format!(
                    "${} of ${} still has \
                     no evidenced fund receipt, so only the deadline moves",
                    money(Some(&unfunded))?,
                    money(Some(&line.sg_amount))?
                )
            };
            result.caveats.push(format!(
                "this assumes the contribution is not the first to this fund. If \
                 it is a new starter or a fund switch, set \
                 first_contribution_to_fund=yes and {becomes} \
                 (due {})",
                extended.isoformat()
            ));
        }
    }

    if result.nec.is_some()
        && let Some(stale) = gic.staleness(nec_end)
    {
        result.caveats.push(stale);
    }
    Ok(())
}

/// The latest day a contribution other than the first eligible one can be on
/// time. Item 1 extends only the first (LCR 2026/2 paragraph 54); items 2 and 4
/// are not limited to it.
fn later_contribution_due(
    line: &ContribLine,
    deadline: &Deadline,
    cal: &BusinessCalendar,
) -> Result<Date> {
    let due = deadline.due.expect("assessed lines have a deadline");
    if !line.first_to_fund {
        return Ok(due);
    }
    let paid = match (line.out_of_cycle, line.next_standard_qe_day) {
        (true, Some(next)) => next,
        _ => line.qe_day,
    };
    let own = cal.add_business_days(paid, 7)?;
    Ok(if deadline.pathway == ITEM4_ALIGNED {
        own.max(due)
    } else {
        own
    })
}

/// Assess a line that a fund returned money on, or that carries events.
///
/// The returned amount is not a contribution (SIS reg 7.07G(4); LCR 2026/2
/// paragraphs 24 and 25) and the deadline does not move for a rejection (ATO,
/// 'Getting it right'). The line is decided only when the kept receipts cover
/// the SG amount; otherwise it is UNKNOWN, carries no exposure and is not LATE.
fn assess_events(
    result: &mut Assessed,
    cal: &BusinessCalendar,
    gic: &GicTable,
    as_at: Date,
    options: Options,
) -> Result<()> {
    let line = result.line.clone();
    let due = result.deadline.due.expect("assessed lines have a deadline");
    let later_due = later_contribution_due(&line, &result.deadline, cal)?;
    let earliest = earliest_prepayment_day(line.qe_day)?;
    let past_horizon = due > cal.coverage_until;
    let possible = result.deadline.possible_item4_due.filter(|p| *p > due);
    let doubtful = returned::cannot_reconcile(&line);
    let owed = cents(Some(&line.sg_amount))?;

    if !line.events.is_empty() {
        result.caveats.push(returned::trail(&line)?);
    }
    for event in &line.events {
        let EventKind::Returned { notified, .. } = event.kind else {
            continue;
        };
        if event.date > as_at {
            result.caveats.push(format!(
                "the return dated {} is after the as-at date {}, but a returned amount \
                 is not a contribution once it is rejected, so it is not counted here",
                event.date.isoformat(),
                as_at.isoformat()
            ));
        }
        if let Some(told) = notified.filter(|n| *n > due) {
            let posture = if line.qe_day <= FIRST_YEAR_END {
                " For this first-year QE day, correcting the error as soon as reasonably \
                 practicable can lower the ATO's compliance risk rating (PCG 2026/1 \
                 paragraphs 17, 26 and 27); that does not change the shortfall"
            } else {
                ""
            };
            result.caveats.push(format!(
                "the employer was told of a return on {}, after the deadline {}. Late \
                 notice does not move the deadline: s 18C has no notice exception and the \
                 ATO says the timeframe does not change.{posture}",
                told.isoformat(),
                due.isoformat()
            ));
        }
    }

    let mut remaining = owed.clone();
    let mut on_time = zero();
    let mut late_counted = zero();
    let mut any_late = false;
    let mut undecided = false;
    let mut first = true;
    let mut last_credit = None;
    let mut last_counted = None;
    let mut kept_receipts = 0;
    for r in returned::receipts(&line)? {
        let eligible = r.eligible()?;
        if !eligible.gt(&zero())? {
            continue;
        }
        if r.date > as_at {
            result.caveats.push(format!(
                "the receipt dated {} is after the as-at date {}: it is ignored for this \
                 as-at report",
                r.date.isoformat(),
                as_at.isoformat()
            ));
            continue;
        }
        if r.date < earliest {
            result.caveats.push(format!(
                "received {}, before the 12-month pre-payment window in \
                 s 18C(1)(c)(ii), so it cannot be applied to this payday",
                r.date.isoformat()
            ));
            continue;
        }
        let window = if first { due } else { later_due };
        first = false;
        kept_receipts += 1;
        let credit = dmin(&eligible, &remaining)?;
        remaining = remaining.sub(&credit)?;
        last_credit = Some(r.date);
        if r.date <= window {
            on_time = on_time.add(&credit)?;
        } else if r.date <= due {
            undecided = true;
            result.caveats.push(format!(
                "the receipt dated {} is not the first eligible contribution, because the \
                 fund kept an earlier amount. LCR 2026/2 paragraph 54 gives only the first \
                 the 20 business days, so this one was on time only to {}. The s 6(1) late \
                 period starts only after {}, so whether it counts as a late contribution \
                 is not established here. No exposure is calculated",
                r.date.isoformat(),
                later_due.isoformat(),
                due.isoformat()
            ));
        } else if past_horizon || possible.is_some_and(|p| r.date <= p) {
            undecided = true;
            if past_horizon {
                result.caveats.push(horizon_unknown_text());
            }
            if let Some(p) = possible {
                result.caveats.push(item4_unknown_text(p));
            }
        } else {
            any_late = true;
            if options.assessment_date.is_none_or(|a| r.date < a) {
                late_counted = late_counted.add(&credit)?;
                last_counted = Some(r.date);
            }
        }
    }

    if doubtful || remaining.gt(&zero())? {
        let row_receipt_late = line
            .received
            .filter(|r| *r <= as_at)
            .is_some_and(|r| r > possible.unwrap_or(due) && !past_horizon);
        let settled_any = last_credit.is_some() || line.received.is_some_and(|r| r <= as_at);
        let window_left = if first {
            possible.unwrap_or(due)
        } else {
            possible.unwrap_or(later_due)
        };
        let window_open = past_horizon || as_at <= window_left;
        let on_time_possible =
            !any_late && !row_receipt_late && (window_open || (doubtful && settled_any));
        result.verdict = UNKNOWN.into();
        result.horizon_verdicts = verdicts(UNPAID, if on_time_possible { ON_TIME } else { LATE });
        if line.returned_by_fund && !returned::has_return_event(&line) {
            result.caveats.push(format!(
                "returned_by_fund=yes: a fund returned some or all of this contribution, and \
                 a returned amount is not a contribution (SIS reg 7.07G(4); LCR 2026/2 \
                 paragraphs 24 and 25). Nothing says which amount came back, so no receipt on \
                 this row is read as eligible. The deadline {} does not restart. No exposure \
                 is calculated; reconcile the contribution with the fund or clearing house \
                 and supply the dated return with --events",
                due.isoformat()
            ));
        } else if doubtful {
            result.caveats.push(
                "the fund receipt on this row states no amount, so what the fund kept cannot \
                 be reconciled against the events. Supply matched_amount and rerun"
                    .into(),
            );
        }
        if remaining.gt(&zero())? && !line.events.is_empty() {
            result.caveats.push(format!(
                "the events leave ${} of the ${} SG amount with no eligible fund receipt on \
                 or before the as-at date. The deadline {} does not restart on a return. The \
                 line stays unresolved, not LATE, and no exposure is calculated, until a fund \
                 receipt for that amount is evidenced",
                money(Some(&remaining))?,
                money(Some(&owed))?,
                due.isoformat()
            ));
        }
        return Ok(());
    }

    result.receipt_established = true;
    if undecided {
        result.verdict = UNKNOWN.into();
        result.horizon_verdicts = verdicts(LATE, ON_TIME);
        return Ok(());
    }
    if !any_late {
        result.verdict = ON_TIME.into();
        return Ok(());
    }
    result.verdict = LATE.into();
    let facts = Facts {
        settled: last_counted.or(last_credit),
        remitted: None,
        credit: zero(),
        operational_unremitted: zero(),
        fully_remitted: false,
        receipt_credit: late_counted.clone(),
        receipt_covers_all: true,
        past_horizon,
        horizon_unknown: horizon_unknown_text(),
        horizon_figures: horizon_figures_text(due, cal),
        possible_item4_due: possible,
        item4_uncertain: possible.is_some(),
        item4_unknown: possible.map_or_else(String::new, item4_unknown_text),
        item4_partial_unknown: String::new(),
        horizon_partial_unknown: String::new(),
        receipt_amount_unevidenced: false,
        event_late_credit: Some(late_counted),
        several_receipts: kept_receipts > 1,
    };
    apply_exposure(result, cal, gic, as_at, options, &facts, &on_time, false)
}

fn assess_line(
    line: ContribLine,
    dl: Deadline,
    cal: &BusinessCalendar,
    gic: &GicTable,
    as_at: Date,
    options: Options,
    transition_row: bool,
) -> Result<Assessed> {
    let mut result = Assessed {
        notes: dl.notes.clone(),
        caveats: dl.caveats.clone(),
        line,
        deadline: dl,
        verdict: UNKNOWN.into(),
        days_late: None,
        lateness_basis: String::new(),
        base_shortfall: None,
        final_shortfall: None,
        offset_s18d: false,
        nec: None,
        sgc_low: None,
        sgc_high: None,
        uplift: None,
        horizon_verdicts: None,
        receipt_established: false,
    };
    if transition_row {
        result.notes.push(
            "operator confirmed the LCR 2026/1 transition allocation: any \
             pre-1 July amount is unused excess and any 1-28 July amount remains \
             after the employee's June-quarter shortfall"
                .into(),
        );
    }
    if !result.line.duplicate_note.is_empty() {
        result.caveats.push(result.line.duplicate_note.clone());
    }
    let join_caveats = result.line.join_caveats.clone();
    result.caveats.extend(join_caveats);
    let Some(due) = result
        .deadline
        .due
        .filter(|_| result.deadline.pathway != SKIP_DB)
    else {
        result.verdict = SKIPPED.into();
        return Ok(result);
    };
    if result.line.sg_amount.le(&zero())? {
        if due < as_at {
            result.caveats.push(format!(
                "the deadline passed on {}, but this row records \
                 no SG amount, so there is nothing to assess. Check the amount \
                 column if this payday should have carried super",
                due.isoformat()
            ));
        } else {
            result.caveats.push(
                "this row records no SG amount, so there is nothing to assess. \
                 Check the amount column if this payday should have carried super"
                    .into(),
            );
        }
        return Ok(result);
    }
    if returned::carries_return_facts(&result.line) {
        assess_events(&mut result, cal, gic, as_at, options)?;
        return Ok(result);
    }
    let facts = assessment_facts(&mut result, cal, as_at)?;
    let (on_time, stale_prepayment, finished) = if facts.settled.is_some() {
        assess_received(&mut result, as_at, &facts)?
    } else {
        (
            zero(),
            false,
            assess_without_receipt(&mut result, as_at, &facts)?,
        )
    };
    if finished {
        return Ok(result);
    }
    if result.is_exposed() {
        apply_exposure(
            &mut result,
            cal,
            gic,
            as_at,
            options,
            &facts,
            &on_time,
            stale_prepayment,
        )?;
    }
    Ok(result)
}

#[cfg(test)]
pub(crate) mod tests {
    //! The Python decision-phase tests, which call each phase with chosen facts.

    use super::*;
    use crate::calendar::load_calendar;
    use crate::rates::load_gic;

    pub(crate) fn day(y: i64, m: i64, d: i64) -> Date {
        Date::new(y, m, d).unwrap()
    }

    pub(crate) fn dec(text: &str) -> Decimal {
        Decimal::parse(text).unwrap()
    }

    pub(crate) fn line() -> ContribLine {
        ContribLine {
            employee_id: "E1".into(),
            qe_day: day(2027, 7, 9),
            sg_amount: dec("300"),
            remitted: None,
            remitted_amount: None,
            received: None,
            first_to_fund: false,
            out_of_cycle: false,
            next_standard_qe_day: None,
            db_interest: false,
            row: 2,
            duplicate_note: String::new(),
            matched_amount: None,
            join_caveats: Vec::new(),
            returned_by_fund: false,
            events: Vec::new(),
        }
    }

    pub(crate) fn result(due: Date, verdict: &str) -> Assessed {
        Assessed {
            line: line(),
            deadline: Deadline {
                due: Some(due),
                pathway: USUAL_7BD.into(),
                notes: Vec::new(),
                caveats: Vec::new(),
                own_due: None,
                item2_due: None,
                possible_item4_due: None,
            },
            verdict: verdict.into(),
            days_late: None,
            lateness_basis: String::new(),
            base_shortfall: None,
            final_shortfall: None,
            offset_s18d: false,
            nec: None,
            sgc_low: None,
            sgc_high: None,
            uplift: None,
            notes: Vec::new(),
            caveats: Vec::new(),
            horizon_verdicts: None,
            receipt_established: false,
        }
    }

    fn facts() -> Facts {
        Facts {
            settled: Some(day(2026, 7, 10)),
            remitted: None,
            credit: dec("0"),
            operational_unremitted: dec("300"),
            fully_remitted: false,
            receipt_credit: dec("100"),
            receipt_covers_all: false,
            past_horizon: false,
            horizon_unknown: "calendar unknown".into(),
            horizon_figures: "calendar figures".into(),
            possible_item4_due: None,
            item4_uncertain: false,
            item4_unknown: "item 4 unknown".into(),
            item4_partial_unknown: "item 4 partial".into(),
            horizon_partial_unknown: "calendar partial".into(),
            receipt_amount_unevidenced: false,
            event_late_credit: None,
            several_receipts: false,
        }
    }

    fn item4(facts: Facts) -> Facts {
        Facts {
            possible_item4_due: Some(day(2027, 7, 20)),
            item4_uncertain: true,
            ..facts
        }
    }

    fn received(r: &mut Assessed, as_at: Date, f: &Facts) -> (String, bool, bool) {
        let (credit, stale, finished) = assess_received(r, as_at, f).unwrap();
        (credit.to_sci_string(), stale, finished)
    }

    fn pair(a: &str, b: &str) -> Option<(String, String)> {
        Some((a.into(), b.into()))
    }

    fn last(r: &Assessed, back: usize) -> &str {
        &r.caveats[r.caveats.len() - back]
    }

    #[test]
    fn the_received_phase_keeps_partial_prepayment_boundaries() {
        let mut r = result(day(2027, 7, 20), UNKNOWN);
        assert_eq!(
            received(&mut r, day(2027, 7, 12), &facts()),
            ("100".into(), false, true)
        );
        assert!(last(&r, 1).contains("deadline has not passed"));

        let mut r = result(day(2027, 7, 10), UNKNOWN);
        let f = item4(Facts {
            past_horizon: true,
            ..facts()
        });
        assert_eq!(
            received(&mut r, day(2027, 7, 12), &f),
            ("100".into(), false, true)
        );
        assert!(last(&r, 2).contains("calendar's coverage"));
        assert_eq!(last(&r, 1), "item 4 unknown");

        let mut r = result(day(2027, 7, 10), UNKNOWN);
        let f = item4(Facts {
            settled: Some(day(2026, 7, 1)),
            ..facts()
        });
        received(&mut r, day(2027, 7, 12), &f);
        assert_eq!(last(&r, 1), "item 4 unknown");
    }

    #[test]
    fn the_received_phase_keeps_stale_and_partial_receipt_outcomes() {
        let mut r = result(day(2027, 7, 10), UNKNOWN);
        let f = item4(Facts {
            settled: Some(day(2026, 7, 1)),
            past_horizon: true,
            ..facts()
        });
        assert_eq!(
            received(&mut r, day(2027, 7, 12), &f),
            ("0".into(), true, true)
        );
        assert!(last(&r, 2).contains("calendar's coverage"));
        assert_eq!(last(&r, 1), "item 4 unknown");

        let partial = Facts {
            settled: Some(day(2027, 7, 15)),
            ..facts()
        };
        let mut overdue = result(day(2027, 7, 20), UNKNOWN);
        assert!(!received(&mut overdue, day(2027, 8, 1), &partial).2);
        assert_eq!(overdue.verdict, UNPAID);

        let mut pending = result(day(2027, 7, 20), UNKNOWN);
        assert!(received(&mut pending, day(2027, 7, 18), &partial).2);
        assert!(last(&pending, 1).contains("deadline has not passed"));
    }

    #[test]
    fn the_no_receipt_phase_keeps_item4_uncertainty() {
        let mut r = result(day(2027, 7, 10), UNKNOWN);
        let f = item4(Facts {
            settled: None,
            remitted: Some(day(2027, 7, 15)),
            fully_remitted: true,
            ..facts()
        });
        assert!(assess_without_receipt(&mut r, day(2027, 7, 16), &f).unwrap());
        assert_eq!(r.horizon_verdicts, pair(LATE, AT_RISK));
        assert_eq!(r.caveats, ["item 4 unknown"]);
    }

    #[test]
    fn the_exposure_phase_keeps_horizon_and_new_starter_boundaries() {
        let (cal, gic) = (load_calendar(None).unwrap(), load_gic().unwrap());
        // Both as-at dates are past the last published GIC quarter, so
        // neither row carries a charge estimate. The boundaries under test
        // are the horizon and new-starter ones, which do not read a rate.
        let mut r = result(day(2027, 7, 20), LATE);
        let f = Facts {
            settled: None,
            receipt_credit: dec("0"),
            past_horizon: true,
            ..facts()
        };
        apply_exposure(
            &mut r,
            &cal,
            &gic,
            day(2027, 8, 1),
            Options::default(),
            &f,
            &zero(),
            false,
        )
        .unwrap();
        assert_eq!(r.days_late, None);
        assert!(r.caveats.iter().any(|c| c == "calendar figures"));

        let mut r = result(day(2027, 7, 20), LATE);
        let f = Facts {
            settled: Some(day(2027, 8, 20)),
            receipt_credit: dec("300"),
            receipt_covers_all: true,
            ..facts()
        };
        apply_exposure(
            &mut r,
            &cal,
            &gic,
            day(2027, 9, 1),
            Options::default(),
            &f,
            &zero(),
            false,
        )
        .unwrap();
        assert!(
            !r.caveats
                .iter()
                .any(|c| c.contains("first_contribution_to_fund"))
        );
    }

    #[test]
    fn a_partial_prepayment_past_its_deadline_is_unpaid_when_nothing_is_uncertain() {
        let mut r = result(day(2027, 7, 10), UNKNOWN);
        assert_eq!(
            received(&mut r, day(2027, 7, 12), &facts()),
            ("100".into(), false, false)
        );
        assert_eq!(r.verdict, UNPAID);
    }

    #[test]
    fn a_partial_prepayment_names_only_the_uncertainty_that_applies() {
        let mut item4_only = result(day(2027, 7, 10), UNKNOWN);
        received(&mut item4_only, day(2027, 7, 12), &item4(facts()));
        assert_eq!(item4_only.horizon_verdicts, pair(UNPAID, NOT_YET_DUE));
        assert_eq!(item4_only.caveats, ["item 4 unknown"]);

        let mut horizon_only = result(day(2027, 7, 10), UNKNOWN);
        received(
            &mut horizon_only,
            day(2027, 7, 12),
            &Facts {
                past_horizon: true,
                ..facts()
            },
        );
        assert_eq!(horizon_only.caveats.len(), 1);
        assert!(horizon_only.caveats[0].contains("calendar's coverage"));
    }

    #[test]
    fn no_receipt_past_a_deadline_item4_may_extend_is_left_unknown() {
        let mut r = result(day(2027, 7, 10), UNKNOWN);
        let f = item4(Facts {
            settled: None,
            ..facts()
        });
        assert!(assess_without_receipt(&mut r, day(2027, 7, 12), &f).unwrap());
        assert_eq!(r.verdict, UNKNOWN);
        assert_eq!(r.horizon_verdicts, pair(UNPAID, NOT_YET_DUE));
        assert_eq!(r.caveats, ["item 4 unknown"]);
    }
}
