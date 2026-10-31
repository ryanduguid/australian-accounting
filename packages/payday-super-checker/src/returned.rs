//! Returned and resubmitted contributions: the dated, amount-specific events on
//! a line, what they leave eligible and what makes them unusable.
//!
//! A returned amount is not a contribution (SIS reg 7.07G(4); LCR 2026/2
//! paragraphs 24 and 25), whatever the as-at date, so a receipt only ever counts
//! for what the fund kept. A return after the fund's 3 business days is not shown
//! to be either, so `assess` leaves its line undecided instead of relying on it.

use crate::Result;
use crate::calendar::BusinessCalendar;
use crate::csv_io::{cents, money, zero};
use crate::deadlines::{ContribLine, EventKind, receipt_amount_cap, receipt_amount_evidenced};
use crate::decimal::Decimal;
use crate::py::Date;

/// The event a file or the API names, with the fields only a return may carry.
pub fn event_kind(
    text: &str,
    receipt: Option<Date>,
    notified: Option<Date>,
) -> std::result::Result<EventKind, String> {
    match text.to_ascii_lowercase().as_str() {
        "received" if receipt.is_some() || notified.is_some() => {
            Err("receipt_date and notified_date belong on a returned event".to_string())
        }
        "received" => Ok(EventKind::Received),
        "returned" => Ok(EventKind::Returned { receipt, notified }),
        _ => Err(format!(
            "event {} is neither received nor returned",
            crate::pytext::repr(text)
        )),
    }
}

/// One day's fund receipt on a line and how much of it came back.
#[derive(Clone, Debug)]
pub struct Receipt {
    pub date: Date,
    pub amount: Decimal,
    pub returned: Decimal,
}

impl Receipt {
    pub fn eligible(&self) -> Result<Decimal> {
        Ok(self.amount.sub(&self.returned)?)
    }
}

pub fn has_return_event(line: &ContribLine) -> bool {
    line.events
        .iter()
        .any(|e| matches!(e.kind, EventKind::Returned { .. }))
}

/// Whether the line must take the event path rather than the single-receipt one.
pub fn carries_return_facts(line: &ContribLine) -> bool {
    line.returned_by_fund || !line.events.is_empty()
}

/// Whether the line says something came back without saying what, or holds a
/// receipt with no amount to reconcile against.
pub fn cannot_reconcile(line: &ContribLine) -> bool {
    (line.returned_by_fund && !has_return_event(line))
        || (line.received.is_some() && !receipt_amount_evidenced(line))
        || line.return_not_shown
}

/// The end of the 3 business days a fund has to allocate or refund a receipt
/// (SIS reg 7.07G(2) and (3)), not counting the day of receipt.
///
/// shortcut: this uses the checker's national calendar. The ATO measures the
/// fund's period by holidays in the fund's location, which the input does not
/// hold. A location's regional holiday is the one case the fund has longer.
pub fn fund_window_end(receipt: Date, cal: &BusinessCalendar) -> Result<Date> {
    cal.add_business_days(receipt, 3)
}

/// The (receipt, return) dates of returns later than the fund's window. Only a
/// refund under reg 7.07G(3) is taken not to have been made, and LCR 2026/2
/// paragraph 24 speaks of a rejected attempt, so these are neither.
pub fn returns_not_shown(line: &ContribLine, cal: &BusinessCalendar) -> Result<Vec<(Date, Date)>> {
    let mut out = Vec::new();
    for event in &line.events {
        if let EventKind::Returned {
            receipt: Some(day), ..
        } = event.kind
            && event.date > fund_window_end(day, cal)?
        {
            out.push((day, event.date));
        }
    }
    Ok(out)
}

/// The line's receipts with an amount, in date order, less what came back.
pub fn receipts(line: &ContribLine) -> Result<Vec<Receipt>> {
    let mut out = Vec::new();
    if let Some(date) = line.received
        && receipt_amount_evidenced(line)
    {
        out.push(Receipt {
            date,
            amount: cents(Some(receipt_amount_cap(line)))?,
            returned: zero(),
        });
    }
    for event in &line.events {
        if matches!(event.kind, EventKind::Received) {
            out.push(Receipt {
                date: event.date,
                amount: cents(Some(&event.amount))?,
                returned: zero(),
            });
        }
    }
    for event in &line.events {
        if let EventKind::Returned {
            receipt: Some(day), ..
        } = event.kind
            && let Some(r) = out.iter_mut().find(|r| r.date == day)
        {
            r.returned = r.returned.add(&cents(Some(&event.amount))?)?;
        }
    }
    out.sort_by_key(|r| r.date);
    Ok(out)
}

/// Every reason the line's events cannot be relied on, each naming its row.
pub fn event_problems(line: &ContribLine) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    let row = line.row;
    if !carries_return_facts(line) {
        return Ok(problems);
    }
    if line.db_interest || line.sg_amount.le(&zero())? {
        problems.push(format!(
            "row {row}: a return or an event needs a row carrying an SG amount that is \
             tested for lateness (returned_by_fund=yes or --events)"
        ));
        return Ok(problems);
    }
    let mut received_days: Vec<Date> = line.received.into_iter().collect();
    let mut returns: Vec<(Option<Date>, Date)> = Vec::new();
    for event in &line.events {
        let at = format!("row {row}, event row {}", event.row);
        if !cents(Some(&event.amount))?.gt(&zero())? {
            problems.push(format!("{at}: an event amount must be more than nil"));
        }
        match event.kind {
            EventKind::Received => {
                if received_days.contains(&event.date) {
                    problems.push(format!(
                        "{at}: a receipt dated {} is already recorded for this payday. \
                         Counting both would count the money twice; put a day's receipts \
                         in one event, and leave out a receipt fund_received_date holds",
                        event.date.isoformat()
                    ));
                }
                received_days.push(event.date);
            }
            EventKind::Returned { receipt, .. } => {
                if returns.contains(&(receipt, event.date)) {
                    problems.push(format!(
                        "{at}: this receipt already has a return dated {}. Put a day's \
                         return of one receipt in one event",
                        event.date.isoformat()
                    ));
                }
                returns.push((receipt, event.date));
            }
        }
    }
    let known = receipts(line)?;
    for event in &line.events {
        let EventKind::Returned { receipt, notified } = event.kind else {
            continue;
        };
        let at = format!("row {row}, event row {}", event.row);
        match receipt {
            Some(day) => {
                if !received_days.contains(&day) {
                    problems.push(format!(
                        "{at}: the return names a receipt dated {} that this payday does \
                         not record",
                        day.isoformat()
                    ));
                }
                if event.date < day {
                    problems.push(format!(
                        "{at}: the return dated {} is before the receipt it names ({})",
                        event.date.isoformat(),
                        day.isoformat()
                    ));
                }
                if notified.is_some_and(|n| n < day) {
                    problems.push(format!(
                        "{at}: the employer cannot be told of a return before the receipt \
                         it names ({})",
                        day.isoformat()
                    ));
                }
            }
            None => {
                if received_days.iter().any(|d| *d <= event.date) {
                    problems.push(format!(
                        "{at}: the return does not say which receipt came back, and this \
                         payday records a receipt on or before it. Fill receipt_date"
                    ));
                }
                if cents(Some(&event.amount))?.gt(&cents(Some(&line.sg_amount))?)? {
                    problems.push(format!(
                        "{at}: a return naming no receipt cannot be more than sg_amount"
                    ));
                }
            }
        }
    }
    let mut kept = zero();
    for r in &known {
        if r.returned.gt(&r.amount)? {
            problems.push(format!(
                "row {row}: the returns of the receipt dated {} total ${}, more than the \
                 ${} received",
                r.date.isoformat(),
                money(Some(&r.returned))?,
                money(Some(&r.amount))?
            ));
        }
        kept = kept.add(&r.eligible()?)?;
    }
    if kept.gt(&cents(Some(&line.sg_amount))?)? {
        problems.push(format!(
            "row {row}: the receipts less the returns leave ${}, more than the ${} sg_amount. \
             A resubmission without the return of what it replaces would count the money \
             twice, so record that return. An amount that truly exceeds this payday belongs \
             on the payday it is applied to",
            money(Some(&kept))?,
            money(Some(&line.sg_amount))?
        ));
    }
    Ok(problems)
}

/// The line's receipts and returns in date order, for the reader and the pack.
pub fn trail(line: &ContribLine) -> Result<String> {
    let mut entries: Vec<(Date, u8, String)> = Vec::new();
    if let Some(date) = line.received {
        let what = if receipt_amount_evidenced(line) {
            format!("received ${}", money(Some(receipt_amount_cap(line)))?)
        } else {
            "received, no amount stated".to_string()
        };
        entries.push((date, 0, what));
    }
    for event in &line.events {
        let amount = money(Some(&event.amount))?;
        match event.kind {
            EventKind::Received => entries.push((event.date, 0, format!("received ${amount}"))),
            EventKind::Returned { receipt, notified } => {
                let of = receipt.map_or_else(
                    || "with no receipt recorded".to_string(),
                    |d| format!("of the {} receipt", d.isoformat()),
                );
                let told = notified.map_or_else(String::new, |n| {
                    format!(", employer told {}", n.isoformat())
                });
                entries.push((event.date, 1, format!("returned ${amount} {of}{told}")));
            }
        }
    }
    entries.sort_by_key(|e| (e.0, e.1));
    let listed: Vec<String> = entries
        .iter()
        .map(|(d, _, what)| format!("{} {what}", d.isoformat()))
        .collect();
    Ok(format!("contribution events: {}", listed.join("; ")))
}
