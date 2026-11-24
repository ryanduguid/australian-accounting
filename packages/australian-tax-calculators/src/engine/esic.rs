// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The early stage investor tax offset (ITAA 1997 Subdivision 360-A) for one
//! investor's shares in one company in an income year from 2016-17 to
//! 2026-27: the early stage innovation company tests (s 360-40), the 100-point
//! innovation test (s 360-45) from the items the operator states, the investor
//! tests (ss 360-15(1) and 360-20), the 20% offset (s 360-25(1)) and its
//! $200,000 cap with affiliates and carried forward offsets (s 360-25(2)).
//! The text is the ATO legal database's, read on 24 November 2026 with its
//! history notes: Act No. 8 of 2020 Sch 2 changed ss 360-15(1), 360-25(1) and
//! 360-40(1) for income years from 1 July 2018, and Act No. 92 of 2020 Sch 5
//! item 37 added s 360-40(2)(b) for assessments for income years from 1 July
//! 2021. The principles-based test (s 360-40(1)(e)(i) to (v)) is the
//! operator's statement; nothing is prescribed under ss 360-40(3), 360-40(4)
//! or 360-45(2).

use serde_json::{Value, json};

use super::business_capital::year_first;
use super::{Arg, Result, Step, derived_result, flag, int, money, precise, py_max, py_min, refuse};
use super::{d, scope, step};
use crate::decimal::Decimal;

const KIND: &str = "esic";

/// The first income year Act No. 8 of 2020's text applies to (2018-19).
const AMENDED_2020: i64 = 2018;

/// s 360-45(1): each item's points.
const POINTS: [i64; 8] = [75, 75, 50, 50, 50, 50, 25, 25];

/// The command's inputs, in the order of its catalogue entry; then the
/// optional amount receivable and the other companies' amount and exemption.
pub struct EsicArgs<'a> {
    pub investor: &'a Arg,
    pub affiliate_of_company: &'a Arg,
    pub ess_interests: &'a Arg,
    pub more_than_30_percent: &'a Arg,
    pub amount_paid: &'a Arg,
    pub non_cash_benefit: &'a Arg,
    pub disclosure_exemption: &'a Arg,
    pub incorporated_year: &'a Arg,
    pub abr_registered_year: &'a Arg,
    pub expenses_three_prior_years: &'a Arg,
    pub total_expenses_previous_year: &'a Arg,
    pub assessable_income_previous_year: &'a Arg,
    pub accelerating_commercialisation_grant: &'a Arg,
    pub rd_clawback_income: &'a Arg,
    pub listed: &'a Arg,
    pub foreign_company: &'a Arg,
    pub innovation_items: &'a Arg,
    pub principles_test: &'a Arg,
    pub other_offsets: &'a Arg,
    pub carried_forward_offsets: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub other_companies_paid: Option<&'a Arg>,
    pub amount_receivable: Option<&'a Arg>,
    pub other_companies_exempt: Option<&'a Arg>,
}

/// The tests' rows, in the order they are worked.
struct Tests {
    rows: Vec<Value>,
}

impl Tests {
    /// Records one test; `met` is `None` where the year's text has no such test.
    fn add(&mut self, id: &str, authority: &str, met: Option<bool>, finding: impl Into<String>) {
        self.rows.push(json!({
            "test": id, "authority": authority, "met": met, "finding": finding.into(),
        }));
    }

    fn all_met(&self) -> bool {
        self.rows.iter().all(|r| r["met"] != json!(false))
    }
}

/// The investor's kind under s 360-15(1)(a): `Ok(None)` when it may be
/// entitled, `Ok(Some(why))` when it is not.
fn investor_kind(given: &Arg, first: i64) -> Result<Option<&'static str>> {
    const MEMBERS: &str = "A trust or partnership is not entitled under ITAA 1997 s 360-15(1)(a)(i), but its members and trustee may be (ss 360-15(2), (3), 360-30 and 360-35), which this worksheet does not work out.";
    match (given.text(), first >= AMENDED_2020) {
        (Some("individual" | "company"), _) => Ok(None),
        (Some("widely_held_company"), _) => Ok(Some(
            "a widely held company or a 100% subsidiary of one (s 360-15(1)(a)(ii))",
        )),
        (Some("esvclp"), true) => Ok(Some("an ESVCLP (s 360-15(1)(a)(ia))")),
        (Some("trust" | "partnership"), _) | (Some("esvclp"), false) => {
            refuse("esic.trust_or_partnership_unsupported", MEMBERS)
        }
        _ => refuse(
            "esic.investor_invalid",
            "investor must be individual, company, widely_held_company, esvclp, trust or partnership.",
        ),
    }
}

/// An income year given as text, or `none`: its first calendar year.
fn year_or_none(given: &Arg, key: &str, current: i64) -> Result<Option<i64>> {
    let first = match given.text() {
        Some("none") => return Ok(None),
        Some(text) => year_first(text),
        None => None,
    };
    match first {
        Some(first) if first > current => refuse(
            "esic.incorporation_after_year",
            format!("{key} is after the income year of the test."),
        ),
        Some(first) => Ok(Some(first)),
        None => refuse(
            "esic.year_invalid",
            format!("{key} must be an income year such as 2024-25, or none."),
        ),
    }
}

/// s 360-40(1)(a): incorporated or registered recently enough.
fn age_test(args: &EsicArgs, current: i64, tests: &mut Tests) -> Result<()> {
    let incorporated = year_or_none(args.incorporated_year, "incorporated_year", current)?;
    let registered = year_or_none(args.abr_registered_year, "abr_registered_year", current)?;
    let within = |year: Option<i64>, span: i64| year.is_some_and(|y| current - y < span);
    let (authority, met, finding) = if within(incorporated, 3) {
        (
            "ITAA 1997 s 360-40(1)(a)(i)",
            true,
            "incorporated in Australia within the last 3 income years",
        )
    } else if within(registered, 3) {
        (
            "ITAA 1997 s 360-40(1)(a)(iii)",
            true,
            "registered in the Australian Business Register within the last 3 income years",
        )
    } else if within(incorporated, 6) {
        if current < AMENDED_2020 {
            return refuse(
                "esic.six_year_route_before_2018_19",
                "Before 2018-19, s 360-40(1)(a)(ii) counted expenses across the last 3 of the 6 income years including the current year (Act No. 8 of 2020 Sch 2 item 12 added 'before the current year'); this worksheet takes the 3 years before it only.",
            );
        }
        let spent = money(args.expenses_three_prior_years)?;
        let met = spent.le(&int(1_000_000))?;
        (
            "ITAA 1997 s 360-40(1)(a)(ii)",
            met,
            if met {
                "incorporated in Australia within the last 6 income years, with total expenses of $1 million or less across the 3 income years before the current year"
            } else {
                "incorporated in Australia within the last 6 income years, but with total expenses over $1 million across the 3 income years before the current year"
            },
        )
    } else {
        (
            "ITAA 1997 s 360-40(1)(a)",
            false,
            "neither incorporated in Australia within the last 6 income years nor registered in the Australian Business Register within the last 3",
        )
    };
    tests.add("incorporation", authority, Some(met), finding);
    Ok(())
}

/// s 360-40(1)(c) and (2): the previous year's assessable income counted.
fn counted_income(args: &EsicArgs, current: i64) -> Result<Step> {
    let income = money(args.assessable_income_previous_year)?;
    let grant = money(args.accelerating_commercialisation_grant)?;
    let clawback = money(args.rd_clawback_income)?;
    if grant.add(&clawback)?.gt(&income)? {
        return refuse(
            "esic.amount_above_income",
            "accelerating_commercialisation_grant and rd_clawback_income are parts of assessable_income_previous_year and together cannot exceed it.",
        );
    }
    if current == 2021 && !clawback.is_zero() {
        return refuse(
            "esic.clawback_year_unsettled",
            "s 360-40(2)(b) applies to assessments for income years commencing on or after 1 July 2021 (Act No. 92 of 2020 Sch 5 item 37) and refers to s 355-450(1) as that Act amended it; a 2020-21 amount tested in 2021-22 is not decided here.",
        );
    }
    Ok(if current >= 2022 {
        step(
            "counted_assessable_income",
            "ITAA 1997 s 360-40(1)(c) and (2): the previous year's assessable income, disregarding an Accelerating Commercialisation Grant and a s 355-450(1) amount",
            "assessable_income_previous_year - accelerating_commercialisation_grant - rd_clawback_income",
            &[&income, &grant, &clawback],
            income.sub(&grant)?.sub(&clawback)?,
        )
    } else {
        step(
            "counted_assessable_income",
            "ITAA 1997 s 360-40(1)(c) and (2) as in force before Act No. 92 of 2020 Sch 5 item 37: disregarding an Accelerating Commercialisation Grant only",
            "assessable_income_previous_year - accelerating_commercialisation_grant",
            &[&income, &grant],
            income.sub(&grant)?,
        )
    })
}

/// s 360-45(1): the points of the items stated to apply.
fn innovation_points(given: &Arg) -> Result<i64> {
    let invalid = || {
        refuse(
            "esic.innovation_items_invalid",
            "innovation_items must be a list of distinct whole numbers from 1 to 8, the s 360-45(1) table items that apply.",
        )
    };
    let Arg::Json(Value::Array(items)) = given else {
        return invalid();
    };
    let mut seen = [false; 8];
    for item in items {
        match item.as_u64().filter(|n| (1..=8).contains(n)) {
            Some(n) if !seen[n as usize - 1] => seen[n as usize - 1] = true,
            _ => return invalid(),
        }
    }
    if seen[0] && seen[2] {
        return refuse(
            "esic.innovation_items_conflict",
            "Items 1 (at least 50% of total expenses) and 3 (at least 15% but less than 50%) cannot both apply.",
        );
    }
    // Item 7 applies only 'unless item 6 applies'.
    if seen[5] {
        seen[6] = false;
    }
    Ok(seen
        .iter()
        .zip(POINTS)
        .filter(|(s, _)| **s)
        .map(|(_, p)| p)
        .sum())
}

/// s 360-40(1)(b) and (c): the previous year's expenses and income.
fn limit_tests(args: &EsicArgs, income: &Decimal, tests: &mut Tests) -> Result<()> {
    let expenses = money(args.total_expenses_previous_year)?;
    let rows = [
        (
            "total_expenses",
            "(b)",
            "total expenses",
            &expenses,
            1_000_000,
            "$1,000,000",
        ),
        (
            "assessable_income",
            "(c)",
            "assessable income counted",
            income,
            200_000,
            "$200,000",
        ),
    ];
    for (id, paragraph, what, value, limit, shown) in rows {
        tests.add(
            id,
            &format!("ITAA 1997 s 360-40(1){paragraph}"),
            Some(value.le(&int(limit))?),
            format!("{what} of ${value} in the previous income year, limit {shown}"),
        );
    }
    Ok(())
}

/// s 360-40(1)(d) to (f): listing, innovation and foreign company.
fn status_tests(args: &EsicArgs, current: i64, tests: &mut Tests) -> Result<i64> {
    let listed = flag(args.listed, "listed")?;
    tests.add(
        "not_listed",
        "ITAA 1997 s 360-40(1)(d)",
        Some(!listed),
        "equity interests listed for quotation on a stock exchange: stated",
    );
    let points = innovation_points(args.innovation_items)?;
    let principles = flag(args.principles_test, "principles_test")?;
    let finding = format!(
        "{points} points under s 360-45(1); principles-based test stated {}",
        if principles { "met" } else { "not met" }
    );
    tests.add(
        "innovation",
        "ITAA 1997 s 360-40(1)(e)",
        Some(points >= 100 || principles),
        finding,
    );
    let foreign = flag(args.foreign_company, "foreign_company")?;
    let (authority, met, finding) = if current >= AMENDED_2020 {
        (
            "ITAA 1997 s 360-40(1)(f)",
            Some(!foreign),
            "a foreign company within the meaning of the Corporations Act 2001: stated",
        )
    } else {
        (
            "ITAA 1997 s 360-40(1)(f), inserted by Act No. 8 of 2020 for income years from 2018-19",
            None,
            "not a test before 2018-19",
        )
    };
    tests.add("not_foreign_company", authority, met, finding);
    Ok(points)
}

/// s 360-15(1)(d) to (f): the stated facts, each of which must be false.
fn investor_tests(args: &EsicArgs, current: i64, tests: &mut Tests) -> Result<()> {
    let thirty = if current >= AMENDED_2020 {
        (
            "ITAA 1997 s 360-15(1)(f)",
            "more than 30% of income, capital or voting rights in the company or a connected entity",
        )
    } else {
        (
            "ITAA 1997 s 360-15(1)(f) before Act No. 8 of 2020",
            "more than 30% of the equity interests in the company or a connected entity",
        )
    };
    let rows = [
        (
            "not_affiliate",
            (
                "ITAA 1997 s 360-15(1)(d)",
                "the investor and the company affiliates of each other",
            ),
            args.affiliate_of_company,
            "affiliate_of_company",
        ),
        (
            "not_ess_interests",
            (
                "ITAA 1997 s 360-15(1)(e)",
                "an acquisition of ESS interests under an employee share scheme",
            ),
            args.ess_interests,
            "ess_interests",
        ),
        (
            "not_more_than_30_percent",
            thirty,
            args.more_than_30_percent,
            "more_than_30_percent",
        ),
    ];
    for (id, (authority, fact), given, key) in rows {
        let stated = flag(given, key)?;
        tests.add(
            id,
            authority,
            Some(!stated),
            format!("{fact}: stated {}", if stated { "yes" } else { "no" }),
        );
    }
    Ok(())
}

/// The amount paid to other companies and whether an offer of theirs had a
/// disclosure exemption: given exactly when this company's offers had none.
fn paid_elsewhere(args: &EsicArgs) -> Result<Option<(Decimal, bool)>> {
    let exempt = flag(args.disclosure_exemption, "disclosure_exemption")?;
    match (
        exempt,
        args.other_companies_paid,
        args.other_companies_exempt,
    ) {
        (true, None, None) => Ok(None),
        (true, ..) => refuse(
            "input.field_unknown",
            "other_companies_paid and other_companies_exempt are inputs only when disclosure_exemption is false",
        ),
        (false, Some(paid), Some(other)) => {
            Ok(Some((money(paid)?, flag(other, "other_companies_exempt")?)))
        }
        (false, ..) => refuse(
            "input.missing_field",
            "other_companies_paid and other_companies_exempt are required when disclosure_exemption is false.",
        ),
    }
}

/// s 360-20: the $50,000 limit where no offer had a disclosure exemption.
fn retail_limit(
    args: &EsicArgs,
    money_paid: &Decimal,
    base: &Decimal,
    tests: &mut Tests,
) -> Result<()> {
    let Some((elsewhere, other_exempt)) = paid_elsewhere(args)? else {
        tests.add("retail_limit", "ITAA 1997 s 360-20(1)(a)", None, "s 708(8), (10) or (11) of the Corporations Act 2001 removed the need for a disclosure document for an offer: no $50,000 limit");
        return Ok(());
    };
    let limit = int(50_000);
    // s 360-20(1)(b) on money paid; on money receivable and a non-cash
    // benefit too; and the ATO's count across all ESICs, which an exempt offer
    // by another ESIC lifts.
    let readings = [
        money_paid.gt(&limit)?,
        base.gt(&limit)?,
        !other_exempt && base.add(&elsewhere)?.gt(&limit)?,
    ];
    let unsettled = readings.iter().any(|r| *r != readings[0]);
    if unsettled && !tests.all_met() {
        tests.add(
            "retail_limit",
            "ITAA 1997 s 360-20",
            None,
            "not decided: the readings differ on these amounts, and another test is not met",
        );
        return Ok(());
    }
    if unsettled {
        return refuse(
            "esic.retail_limit_unsettled",
            "s 360-20(1)(b) counts more than $50,000 paid for this company's shares where none of its offers had a disclosure exemption; the ATO counts investments in all ESICs and lifts the limit for an exempt offer by any of them, and money still receivable or a non-cash benefit may or may not be 'paid'. On these amounts the readings differ.",
        );
    }
    let over = readings[0];
    tests.add(
        "retail_limit",
        "ITAA 1997 s 360-20",
        Some(!over),
        if over {
            "more than $50,000 paid without a disclosure exemption"
        } else {
            "$50,000 or less paid without a disclosure exemption"
        },
    );
    Ok(())
}

/// s 360-25(1): 20% of the base where entitled.
fn gross_step(base: &Decimal, entitled: bool) -> Result<Step> {
    Ok(if entitled {
        step(
            "gross_offset",
            "ITAA 1997 s 360-25(1): 20% of the amount for the issue of the shares",
            "offset_base x 0.20",
            &[base],
            base.mul(&d("0.20"))?,
        )
    } else {
        step(
            "gross_offset",
            "ITAA 1997 s 360-15(1): not entitled, a test is not met",
            "0",
            &[],
            int(0),
        )
    })
}

/// s 360-25(1): the amount for the issue of the shares; `receivable` where
/// the input was given.
fn base_step(
    [paid, non_cash]: [&Decimal; 2],
    receivable: Option<&Decimal>,
    current: i64,
) -> Result<Step> {
    if current < AMENDED_2020 {
        return Ok(step(
            "offset_base",
            "ITAA 1997 s 360-25(1) before Act No. 8 of 2020: the total amount paid for the shares",
            "amount_paid",
            &[paid],
            paid.clone(),
        ));
    }
    let authority = "ITAA 1997 s 360-25(1)(a) and (b): money received or entitled to be received for the issue, and the market value of any non-cash benefit";
    let Some(receivable) = receivable else {
        let base = precise(|| Ok(paid.add(non_cash)?))?;
        let formula = "amount_paid + non_cash_benefit";
        return Ok(step(
            "offset_base",
            authority,
            formula,
            &[paid, non_cash],
            base,
        ));
    };
    Ok(step(
        "offset_base",
        authority,
        "amount_paid + amount_receivable + non_cash_benefit",
        &[paid, receivable, non_cash],
        precise(|| Ok(paid.add(receivable)?.add(non_cash)?))?,
    ))
}

/// s 360-25: the offset's base, the 20%, the cap room and the offset.
fn offset_steps(args: &EsicArgs, first: Step, entitled: bool) -> Result<Vec<Step>> {
    let other = money(args.other_offsets)?;
    let carried = money(args.carried_forward_offsets)?;
    precise(|| {
        let gross = gross_step(&first.value, entitled)?;
        let room = py_max(int(200_000).sub(&other)?.sub(&carried)?, int(0))?;
        let offset = py_min(gross.value.clone(), room.clone())?;
        let cap = step(
            "cap_room",
            "ITAA 1997 s 360-25(2): $200,000 less the offsets for the year of the investor and its affiliates and those they carry forward to it",
            "max(200000 - other_offsets - carried_forward_offsets, 0)",
            &[&other, &carried],
            room.clone(),
        );
        let last = step(
            "tax_offset",
            "ITAA 1997 s 360-25(2): the offset, reduced so the sum does not exceed $200,000",
            "min(gross_offset, cap_room)",
            &[&gross.value, &room],
            offset,
        );
        Ok(vec![first, gross, cap, last])
    })
}

/// The amounts for the issue, refusing a non-cash benefit before 2018-19.
fn amounts(args: &EsicArgs, current: i64) -> Result<[Decimal; 3]> {
    let paid = money(args.amount_paid)?;
    let receivable = args.amount_receivable.map_or(Ok(int(0)), money)?;
    let non_cash = money(args.non_cash_benefit)?;
    if current < AMENDED_2020 && !non_cash.is_zero() {
        return refuse(
            "esic.non_cash_before_2018_19",
            "Before 2018-19, s 360-25(1) took 20% of 'the total amount paid for the shares'; Act No. 8 of 2020 Sch 2 item 8 added the market value of non-cash benefits. This worksheet does not decide what the earlier text counted.",
        );
    }
    Ok([paid, receivable, non_cash])
}

/// ITAA 1997 Subdivision 360-A: one investor's early stage investor tax offset
/// for shares one company issued in an income year.
pub fn esic(args: &EsicArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let current = year_first(&year).expect("a supported year");
    let not_entitled = investor_kind(args.investor, current)?;
    let [paid, receivable, non_cash] = amounts(args, current)?;
    let mut tests = Tests { rows: Vec::new() };
    age_test(args, current, &mut tests)?;
    let income = counted_income(args, current)?;
    limit_tests(args, &income.value, &mut tests)?;
    let points = status_tests(args, current, &mut tests)?;
    let is_esic = tests.all_met();
    tests.add(
        "investor",
        "ITAA 1997 s 360-15(1)(a)",
        Some(not_entitled.is_none()),
        not_entitled.unwrap_or("an individual or a company that is not widely held"),
    );
    investor_tests(args, current, &mut tests)?;
    let whole = paid.add(&receivable)?.add(&non_cash)?;
    retail_limit(args, &paid, &whole, &mut tests)?;
    let entitled = tests.all_met();
    let given = args.amount_receivable.map(|_| &receivable);
    let first = base_step([&paid, &non_cash], given, current)?;
    let mut steps = vec![income];
    steps.extend(offset_steps(args, first, entitled)?);
    let mut out = derived_result(KIND, &year, steps, Vec::new())?;
    out["esic"] = json!(is_esic);
    out["innovation_points"] = json!(points);
    out["entitled"] = json!(entitled);
    out["modified_cgt_treatment"] = json!(entitled);
    out["tests"] = json!(tests.rows);
    Ok(out)
}
