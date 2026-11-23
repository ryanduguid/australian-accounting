// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `asic_fees`: ASIC's standard fees for 2026-27, and for a late payment of
//! an annual review fee or a late lodgement the tier its due and received
//! dates fall in. It mirrors the ASIC fees page on ryanduguid.github.io
//! (`assets/asic-fees.mjs`). The amounts are ASIC's fee data for 2026-27,
//! each recomputed from the regulations' CPI indexation; sources read on
//! 24 November 2026 are listed in docs/calculation-evidence.md#asic-fees.

use serde_json::{Value, json};

use super::{Arg, Result, date_arg, int, refuse, result, scope};
use crate::dates;

const KIND: &str = "asic_fees";

/// One fee: its input key, ASIC's code, its 2026-27 amount in dollars (none
/// for a late fee, whose amount turns on the tier), what it is and the item
/// that imposes it.
struct Fee {
    key: &'static str,
    code: &'static str,
    amount: Option<i64>,
    name: &'static str,
    item: &'static str,
}

/// The fees as [`Fee`]'s fields separated by `|`, `-` for a late fee's amount.
const FEES: [&str; 11] = [
    "company_share_capital|I02|636|Register a company with share capital|Corporations (Fees) Regulations 2001 (Compilation No. 41), Schedule 2 item 1(a)(i)",
    "company_no_share_capital|I01|524|Register a company without share capital|Corporations (Fees) Regulations 2001 (Compilation No. 41), Schedule 2 item 1(a)(ii)",
    "name_reservation|R01|65|Reserve a company name, or extend a reservation|Corporations (Fees) Regulations 2001 (Compilation No. 41), Schedule 2 item 8",
    "review_public|A51|1591|Annual review fee, public company (not a special purpose company or small transferring financial institution)|Corporations (Review Fees) Regulations 2003 (Compilation No. 11), Schedule 1 Part 1 item 1",
    "review_proprietary|A52|342|Annual review fee, proprietary company (not a special purpose company)|Corporations (Review Fees) Regulations 2003 (Compilation No. 11), Schedule 1 Part 1 item 2",
    "review_special_public|A53|65|Annual review fee, special purpose company that is not a proprietary company|Corporations (Review Fees) Regulations 2003 (Compilation No. 11), Schedule 1 Part 1 item 4",
    "review_special_proprietary|A58|70|Annual review fee, special purpose company that is a proprietary company|Corporations (Review Fees) Regulations 2003 (Compilation No. 11), Schedule 1 Part 1 item 5",
    "business_name_1|BZA or BZC|47|Register or renew a business name for 1 year|Business Names Registration (Fees) Regulations 2022, section 6 items 1 and 2",
    "business_name_3|BZB or BZD|108|Register or renew a business name for 3 years|Business Names Registration (Fees) Regulations 2022, section 6 items 3 and 4",
    "late_review_payment|LPF1_ETM or LPF2_ETM|-|Late payment of an annual review fee|Corporations (Review Fees) Regulations 2003 (Compilation No. 11), Schedule 1 Part 3 items 1 and 2",
    "late_lodgement|LPF1_ETM or LPF2_ETM|-|Late lodgement of a document|Corporations (Fees) Regulations 2001 (Compilation No. 41), Schedule 2 item 14",
];

fn fee(row: &'static str) -> Fee {
    let mut fields = row.splitn(5, '|');
    let mut next = || fields.next().unwrap_or_default();
    Fee {
        key: next(),
        code: next(),
        amount: next().parse().ok(),
        name: next(),
        item: next(),
    }
}

/// The late fees in 2026-27 (LPF1_ETM within one month, LPF2_ETM beyond), in
/// dollars: Review Fees Regulations Schedule 1 Part 3 items 1 and 2, and
/// Corporations (Fees) Regulations Schedule 2 item 14, as indexed.
const LATE_WITHIN: i64 = 102;
const LATE_BEYOND: i64 = 428;
/// The payment or lodgement dates the 2026-27 amounts apply to.
const YEAR_DATES: (&str, &str) = ("2026-07-01", "2027-06-30");
const MONTH: &str =
    "Acts Interpretation Act 1901 s 2G(1) and s 36(1) item 6; Corporations Act 2001 s 105(1)";

pub struct AsicFeesArgs<'a> {
    pub fee: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub due_date: Option<&'a Arg>,
    pub received_date: Option<&'a Arg>,
}

/// The last day of the month that starts the day after `due` (AIA s 36(1)
/// item 6): under s 2G(1) it ends immediately before the corresponding day
/// of the next calendar month or, if there is none, at that month's end.
fn first_month_ends(due: i64) -> i64 {
    dates::months_end(due + 1, 1)
}

/// The Wednesday after a Saturday or Sunday, or the day itself on a weekday:
/// a thing due on a weekend may be done on the next business day (AIA
/// s 36(2), Corporations Act s 105(3)), and the Wednesday covers the national
/// Christmas and Easter substitutions. Day 0, 1 January 1970, is a Thursday.
fn latest_move(day: i64) -> i64 {
    match day.rem_euclid(7) {
        2 => day + 4,
        3 => day + 3,
        _ => day,
    }
}

enum Tier {
    NotLate,
    Within,
    Beyond,
}

fn undecided(why: &str) -> Result<Tier> {
    refuse(
        "asic_fees.weekend_undecided",
        format!(
            "{why}: subsection 105(3) of the Corporations Act or subsection 36(2) of the Acts Interpretation Act may move it, which this worksheet does not decide. Check ASIC's invoice."
        ),
    )
}

/// The tier `received` falls in for a fee due on `due`, refused wherever a
/// move of the due date, or of the first month's end, past a weekend could
/// change it.
fn tier(due: i64, received: i64, ends: i64) -> Result<Tier> {
    if received <= due {
        return Ok(Tier::NotLate);
    }
    let due_moves = latest_move(due);
    if received <= due_moves {
        return undecided("The due date is a Saturday or Sunday");
    }
    if received <= ends {
        return Ok(Tier::Within);
    }
    // The latest the first month could end: from the latest day the due date
    // could move to, then that month's end moved past a weekend.
    let latest = (due..=due_moves)
        .map(|day| latest_move(first_month_ends(day)))
        .fold(ends, i64::max);
    if received <= latest {
        return undecided(if due_moves > due {
            "The due date is a Saturday or Sunday, so the first month after it may end later"
        } else {
            "The first month after the due date ends on a Saturday or Sunday"
        });
    }
    Ok(Tier::Beyond)
}

/// A late fee's amount, code, tier and first month, from its dates.
fn late(fee: &Fee, args: &AsicFeesArgs, out: &mut Value) -> Result<i64> {
    let date = |given: Option<&Arg>, key: &str| match given {
        Some(value) => date_arg(value, key),
        None => refuse(
            "input.missing_field",
            format!("{key} is required with a late fee."),
        ),
    };
    let due = date(args.due_date, "due_date")?;
    let received = date(args.received_date, "received_date")?;
    let (from, to) = (
        dates::parse(YEAR_DATES.0).unwrap_or_default(),
        dates::parse(YEAR_DATES.1).unwrap_or_default(),
    );
    if due.min(received) < from || due.max(received) > to {
        return refuse(
            "asic_fees.dates_outside_year",
            "The 2026-27 amounts apply to a due date and a received date from 2026-07-01 to 2027-06-30; which year's amount applies to dates either side of 1 July is not settled.",
        );
    }
    let ends = first_month_ends(due);
    let (amount, code, tier) = match tier(due, received, ends)? {
        Tier::NotLate => (0, Value::Null, "not late"),
        Tier::Within => (LATE_WITHIN, json!("LPF1_ETM"), "within one month"),
        Tier::Beyond => (LATE_BEYOND, json!("LPF2_ETM"), "more than one month"),
    };
    let (due, received, ends) = (
        dates::format(due),
        dates::format(received),
        dates::format(ends),
    );
    out["code"] = code;
    out["tier"] = json!(tier);
    out["first_month_ends"] = json!(ends);
    out["steps"] = json!([
        {"id": "first_month", "authority": MONTH,
         "working": format!("The first month after {due} runs to {ends}")},
        {"id": "tier", "authority": fee.item,
         "working": format!("Received {received}, due {due}: {tier}, ${amount}")},
    ]);
    Ok(amount)
}

pub fn asic_fees(args: &AsicFeesArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let given = args.fee.text().unwrap_or_default();
    let Some(fee) = FEES.map(fee).into_iter().find(|fee| fee.key == given) else {
        let keys = FEES.map(|row| fee(row).key);
        return refuse(
            "asic_fees.fee_unknown",
            format!("fee must be one of {}.", keys.join(", ")),
        );
    };
    let mut out = json!({"code": fee.code, "tier": null, "first_month_ends": null, "steps": []});
    let amount = match fee.amount {
        Some(amount) => amount,
        None => late(&fee, args, &mut out)?,
    };
    let mut full = result(KIND, &year, vec![("fee", int(amount))], Vec::new())?;
    full["fee"] = json!(fee.key);
    full["description"] = json!(fee.name);
    full["authority"] = json!(fee.item);
    for key in ["code", "tier", "first_month_ends", "steps"] {
        full[key] = out[key].take();
    }
    Ok(full)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekdays_count_from_a_thursday() {
        let day = |text| dates::parse(text).unwrap();
        // Saturday 15 and Sunday 16 August 2026 move as late as Wednesday 19.
        assert_eq!(latest_move(day("2026-08-15")), day("2026-08-19"));
        assert_eq!(latest_move(day("2026-08-16")), day("2026-08-19"));
        assert_eq!(latest_move(day("2026-08-17")), day("2026-08-17"));
        assert_eq!(first_month_ends(day("2026-11-30")), day("2026-12-31"));
        assert_eq!(first_month_ends(day("2027-01-29")), day("2027-02-28"));
    }
}
