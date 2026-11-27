// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The downsizer contribution test of ITAA 1997 s 292-102 for
//! `contribution_caps`: how much of one contribution is covered, from the
//! age at the contribution (55 for contributions from 1 January 2023, Act
//! No. 84 of 2022 Sch 5 item 1), the contract date (from 1 July 2018, Act No.
//! 132 of 2017 Sch 2 item 4), the 90 days after the change of ownership and
//! the cap of s 292-102(3). The other criteria are the operator's statement.
//! Read from the ATO's legal database, with the history notes, on 24 November
//! 2026.

use serde_json::{Value, json};

use super::{Arg, Result, Step, date_arg, flag, int, money, py_max, py_min, refuse, step};
use crate::dates;
use crate::decimal::Decimal;
use crate::texts;

/// The optional inputs, all given or none.
pub const DOWNSIZER_INPUTS: [&str; 10] = [
    "downsizer_contribution",
    "downsizer_contribution_date",
    "downsizer_age",
    "downsizer_contract_date",
    "downsizer_settlement_date",
    "downsizer_longer_period_allowed",
    "downsizer_capital_proceeds",
    "downsizer_earlier_own",
    "downsizer_earlier_from_proceeds",
    "downsizer_conditions_met",
];

/// The inputs, in the order of [`DOWNSIZER_INPUTS`].
pub struct DownsizerArgs<'a> {
    pub given: [&'a Arg; 10],
}

/// The steps, whether the contribution is covered at all, and the tests.
pub(super) struct Downsized {
    pub steps: Vec<Step>,
    pub eligible: bool,
    pub tests: Value,
}

fn row(id: &str, authority: &str, met: bool, finding: String) -> Value {
    json!({"test": id, "authority": authority, "met": met, "finding": finding})
}

/// The days from the change of ownership to the contribution, checked.
fn dates_in_order(args: &DownsizerArgs, year: &str) -> Result<(i64, i64)> {
    let [_, made, _, contract, settled, ..] = args.given;
    let made = date_arg(made, "downsizer_contribution_date")?;
    let contract = date_arg(contract, "downsizer_contract_date")?;
    let settled = date_arg(settled, "downsizer_settlement_date")?;
    let first: i64 = year[..4].parse().expect("supported period");
    let start = dates::parse(&format!("{first}-07-01")).expect("1 July");
    let end = dates::parse(&format!("{}-06-30", first + 1)).expect("30 June");
    if !(start..=end).contains(&made) {
        return refuse(
            "downsizer.date_outside_year",
            format!("downsizer_contribution_date is not in the {year} income year."),
        );
    }
    if settled < contract {
        return refuse(
            "downsizer.settlement_before_contract",
            "downsizer_settlement_date is before downsizer_contract_date.",
        );
    }
    if made < settled {
        return refuse(
            "downsizer.contribution_before_settlement",
            "s 292-102(1)(g) allows a contribution within 90 days after the change of ownership; one made before it is not decided here.",
        );
    }
    Ok((made - settled, contract))
}

/// s 292-102(1)(a), (g), the contract date and the stated conditions.
fn tests(args: &DownsizerArgs, year: &str) -> Result<Vec<Value>> {
    let Some(Some(age)) = args.given[2]
        .int()
        .filter(|n| n.is_some_and(|n| (0..=150).contains(&n)))
    else {
        return refuse(
            "input.age_invalid",
            "downsizer_age must be an integer number of years from 0 to 150.",
        );
    };
    let (days, contract) = dates_in_order(args, year)?;
    let longer = flag(args.given[5], "downsizer_longer_period_allowed")?;
    let stated = flag(args.given[9], "downsizer_conditions_met")?;
    let from = dates::parse("2018-07-01").expect("a date");
    Ok(vec![
        row(
            "age",
            "ITAA 1997 s 292-102(1)(a), 55 for contributions from 1 January 2023 (Act No. 84 of 2022 Sch 5 item 1)",
            age >= 55,
            format!("aged {age} at the contribution, at least 55 required"),
        ),
        row(
            "contract_date",
            "Act No. 132 of 2017 Sch 2 item 4: a disposal under a contract entered into on or after 1 July 2018",
            contract >= from,
            format!("contract entered into on {}", dates::format(contract)),
        ),
        row(
            "within_90_days",
            "ITAA 1997 s 292-102(1)(g)",
            days <= 90 || longer,
            format!("{days} days after the change of ownership; longer period allowed: {longer}"),
        ),
        row(
            "stated_conditions",
            "ITAA 1997 s 292-102(1)(b) to (f), (h), (i) and (2)",
            stated,
            "stated by the operator".to_string(),
        ),
    ])
}

/// s 292-102(3): the contribution, to the extent of the lesser limit.
fn covered_step([made, own_room, proceeds_room]: [&Decimal; 3], eligible: bool) -> Result<Step> {
    if !eligible {
        let why = "ITAA 1997 s 292-102(1): a criterion is not met";
        return Ok(step("downsizer_covered", why, "0", &[], int(0)));
    }
    let covered = py_min(
        py_min(made.clone(), own_room.clone())?,
        proceeds_room.clone(),
    );
    Ok(step(
        "downsizer_covered",
        "ITAA 1997 s 292-102(3): the contribution, to the extent it does not exceed the lesser of the 2 limits",
        "min(min(downsizer_contribution, downsizer_cap_remaining), downsizer_proceeds_remaining)",
        &[made, own_room, proceeds_room],
        covered?,
    ))
}

/// s 292-102(3): the part of the contribution covered, and the rest.
fn cap_steps(args: &DownsizerArgs, eligible: bool) -> Result<Vec<Step>> {
    let [made, _, _, _, _, _, proceeds, own, earlier, _] = args.given;
    let (made, proceeds) = (money(made)?, money(proceeds)?);
    let (own, earlier) = (money(own)?, money(earlier)?);
    let own_room = py_max(int(300_000).sub(&own)?, int(0))?;
    let proceeds_room = py_max(proceeds.sub(&earlier)?, int(0))?;
    let covered = covered_step([&made, &own_room, &proceeds_room], eligible)?;
    let rest = made.sub(&covered.value)?;
    let not_covered = step(
        "downsizer_not_covered",
        "ITAA 1997 s 292-102(1) and (3): the part that is not a downsizer contribution",
        "downsizer_contribution - downsizer_covered",
        &[&made, &covered.value],
        rest,
    );
    Ok(vec![
        step(
            "downsizer_cap_remaining",
            "ITAA 1997 s 292-102(3)(a): $300,000 less other contributions already covered for this person",
            "max(300000 - downsizer_earlier_own, 0)",
            &[&own],
            own_room.clone(),
        ),
        step(
            "downsizer_proceeds_remaining",
            "ITAA 1997 s 292-102(3)(b): the capital proceeds less other contributions already covered from these disposals for this person or their spouse",
            "max(downsizer_capital_proceeds - downsizer_earlier_from_proceeds, 0)",
            &[&proceeds, &earlier],
            proceeds_room.clone(),
        ),
        covered,
        not_covered,
    ])
}

/// The downsizer steps for one contribution in `year`.
pub(super) fn downsizer(args: &DownsizerArgs, year: &str) -> Result<Downsized> {
    let tests = tests(args, year)?;
    let eligible = tests.iter().all(|t| t["met"] == json!(true));
    let steps = cap_steps(args, eligible)?;
    Ok(Downsized {
        steps,
        eligible,
        tests: json!(tests),
    })
}

/// Appends the downsizer steps for `args` to `steps`; the tests to record.
pub(super) fn extend(
    args: Option<&DownsizerArgs>,
    year: &str,
    steps: &mut Vec<Step>,
) -> Result<Option<(bool, Value)>> {
    let Some(args) = args else {
        return Ok(None);
    };
    let done = downsizer(args, year)?;
    steps.extend(done.steps);
    Ok(Some((done.eligible, done.tests)))
}

/// Records the tests, the source and the warning in a `contribution_caps`
/// result.
pub(super) fn record(out: &mut Value, tests: Option<(bool, Value)>) {
    if let Some((eligible, tests)) = tests {
        out["downsizer_eligible"] = json!(eligible);
        out["downsizer_tests"] = tests;
        texts::add_note(
            out,
            (texts::DOWNSIZER_SOURCE, texts::DOWNSIZER_WARNING),
            texts::DOWNSIZER_SCOPE,
        );
    }
}
