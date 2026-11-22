// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The R&D tax incentive offset (ITAA 1997 s 355-100) for an income year
//! starting on or after 1 July 2021, and the amount s 355-450 includes in
//! assessable income for a recoupment or feedstock clawback amount (ss 355-440
//! and 355-445) attributable to that year. The rates are those Act No. 92 of
//! 2020 set from that date (Sch 4 item 14, Sch 5 item 56(3)); the corporate tax
//! rate is Income Tax Rates Act 1986 s 23(2). Sections read from the ATO's
//! legal database, and the Act from its published text, on 22 November 2026.

use serde_json::Value;

use super::{
    Arg, Error, Result, Step, d, derived_result, flag, int, money, precise, py_max, py_min, rate,
    refuse, scope, step,
};
use crate::decimal::Decimal;

const KIND: &str = "rd_tax_offset";

/// The income years Division 355 ran before Act No. 92 of 2020's rates.
const EARLIER_YEARS: [&str; 10] = [
    "2011-12", "2012-13", "2013-14", "2014-15", "2015-16", "2016-17", "2017-18", "2018-19",
    "2019-20", "2020-21",
];

const EARLIER_RATES: &str = "Income years starting before 1 July 2021 used the earlier R&D offset rates, cap and clawback rules, replaced by Act No. 92 of 2020 (Sch 4 item 14, Sch 5 item 56(3)); this worksheet does not hold them.";

/// The command's inputs, in the order of its catalogue entry.
pub struct RdTaxOffsetArgs<'a> {
    pub notional_deductions: &'a Arg,
    pub rsp_crc_expenditure: &'a Arg,
    pub aggregated_turnover: &'a Arg,
    pub exempt_entity_controlled: &'a Arg,
    pub base_rate_entity: &'a Arg,
    pub total_expenses: &'a Arg,
    pub recoupment_clawback: &'a Arg,
    pub feedstock_revenue: &'a Arg,
    pub feedstock_expenditure: &'a Arg,
    pub present_year_base_rate_entity: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The facts every offset in the worksheet shares.
struct Entity {
    notional: Decimal,
    rsp_crc: Decimal,
    total_expenses: Decimal,
    refundable: bool,
    rate: Decimal,
}

/// The ids of an offset's 7 steps: the year's offset, then the adjusted one.
type Ids = [&'static str; 7];

const STARTING: Ids = [
    "offset_base",
    "capped_base",
    "base_offset",
    "premium_tier_1",
    "premium_tier_2",
    "excess_offset",
    "tax_offset",
];

const ADJUSTED: Ids = [
    "adjusted_offset_base",
    "adjusted_capped_base",
    "adjusted_base_offset",
    "adjusted_premium_tier_1",
    "adjusted_premium_tier_2",
    "adjusted_excess_offset",
    "adjusted_offset",
];

const BELOW_THRESHOLD: &str = "ITAA 1997 s 355-100(2): the total is less than $20,000, so the offset is worked out on the expenditure to research service providers that are not associates and CRC contributions only";
const AT_THRESHOLD: &str = "ITAA 1997 s 355-100(1): the total of the notional deductions under ss 355-205, 355-305, 355-480, 355-520 and 355-580, $20,000 or more";
const ADJUSTED_BASE: &str = "ITAA 1997 s 355-450(1) 'adjusted offset': the s 355-100(1) total reduced by the clawback amount attributable to the offset year";
const CAPPED: &str = "ITAA 1997 s 355-100(3)(a): the part of the total up to $150 million";
const REFUNDABLE_RATE: &str = "ITAA 1997 s 355-100(1) item 1: aggregated turnover under $20 million and no exempt entity control, the corporate tax rate plus 18.5 percentage points";
const COMPANY_RATE: &str = "ITAA 1997 s 355-100(1) items 2 and 3: the corporate tax rate";
const NO_PREMIUM: &str = "ITAA 1997 s 355-100(1A): no premium where item 1 applies";
const TIER_1: &str = "ITAA 1997 s 355-100(1A) item 1: 8.5% of the part not exceeding 2% of total expenses (s 355-115)";
const TIER_2: &str =
    "ITAA 1997 s 355-100(1A) item 2: 16.5% of the part exceeding 2% of total expenses (s 355-115)";
const EXCESS: &str =
    "ITAA 1997 s 355-100(3)(b): the excess over $150 million at the corporate tax rate";
const TOTAL: &str = "ITAA 1997 s 355-100: the tax offset, the sum of its parts";

/// `refusal` with `EARLIER_RATES` added when it refuses one of Division 355's
/// earlier years.
fn earlier_years(refusal: Error, year: &Arg) -> Error {
    match (refusal, year) {
        (
            Error::Value {
                code: "period.unsupported",
                message,
            },
            Arg::Text(y),
        ) if EARLIER_YEARS.contains(&y.as_str()) => Error::Value {
            code: "period.unsupported",
            message: format!("{message} {EARLIER_RATES}"),
        },
        (refusal, _) => refusal,
    }
}

/// The corporate tax rate: Income Tax Rates Act 1986 s 23(2)(a) or (b).
fn corporate_rate(base_rate_entity: bool) -> Decimal {
    d(if base_rate_entity { "0.25" } else { "0.30" })
}

/// The base offset and the 2 premium tiers on `capped` (s 355-100(1), (1A)).
fn rated(capped: &Decimal, e: &Entity) -> Result<[Decimal; 3]> {
    if e.refundable {
        return Ok([capped.mul(&e.rate.add(&d("0.185"))?)?, int(0), int(0)]);
    }
    let two_percent = e.total_expenses.mul(&d("0.02"))?;
    let first = py_min(capped.clone(), two_percent.clone())?;
    let rest = py_max(capped.sub(&two_percent)?, int(0))?;
    Ok([
        capped.mul(&e.rate)?,
        first.mul(&d("0.085"))?,
        rest.mul(&d("0.165"))?,
    ])
}

/// s 355-100 on `total` (the s 355-100(1) total), as the values of its 7 steps.
fn offset_values(total: &Decimal, e: &Entity) -> Result<[Decimal; 7]> {
    let base = if total.lt(&int(20000))? {
        e.rsp_crc.clone()
    } else {
        total.clone()
    };
    let capped = py_min(base.clone(), int(150_000_000))?;
    let [base_offset, tier_1, tier_2] = rated(&capped, e)?;
    let excess = py_max(base.sub(&int(150_000_000))?, int(0))?.mul(&e.rate)?;
    let sum = base_offset.add(&tier_1)?.add(&tier_2)?.add(&excess)?;
    Ok([base, capped, base_offset, tier_1, tier_2, excess, sum])
}

/// The formulas and authorities of the base offset and the 2 premium tiers on
/// the step named `capped`.
fn rated_rules(capped: &str, refundable: bool) -> [(String, &'static str); 3] {
    if refundable {
        return [
            (
                format!("{capped} x (corporate_tax_rate + 0.185)"),
                REFUNDABLE_RATE,
            ),
            ("0".to_string(), NO_PREMIUM),
            ("0".to_string(), NO_PREMIUM),
        ];
    }
    [
        (format!("{capped} x corporate_tax_rate"), COMPANY_RATE),
        (
            format!("min({capped}, 0.02 x total_expenses) x 0.085"),
            TIER_1,
        ),
        (
            format!("max({capped} - 0.02 x total_expenses, 0) x 0.165"),
            TIER_2,
        ),
    ]
}

/// The formula and authority of each of an offset's steps named `ids`.
fn offset_rules(ids: &Ids, e: &Entity, total: &Decimal) -> Result<[(String, &'static str); 7]> {
    let [base, capped, base_offset, tier_1, tier_2, excess, _] = ids;
    let opening = match (total.lt(&int(20000))?, ids == &ADJUSTED) {
        (true, _) => ("rsp_crc_expenditure".to_string(), BELOW_THRESHOLD),
        (false, false) => ("notional_deductions".to_string(), AT_THRESHOLD),
        (false, true) => (
            "notional_deductions - clawback_amount".to_string(),
            ADJUSTED_BASE,
        ),
    };
    let [p0, p1, p2] = rated_rules(capped, e.refundable);
    Ok([
        opening,
        (format!("min({base}, 150000000)"), CAPPED),
        p0,
        p1,
        p2,
        (
            format!("max({base} - 150000000, 0) x corporate_tax_rate"),
            EXCESS,
        ),
        (
            format!("{base_offset} + {tier_1} + {tier_2} + {excess}"),
            TOTAL,
        ),
    ])
}

/// An offset's 7 steps on `total`, named `ids`; `clawback` is the clawback
/// amount the adjusted offset's total is reduced by.
fn offset_steps(ids: &Ids, e: &Entity, total: &Decimal, clawback: &Decimal) -> Result<Vec<Step>> {
    let v = offset_values(total, e)?;
    let rules = offset_rules(ids, e, total)?;
    let base_operands: Vec<&Decimal> = match (total.lt(&int(20000))?, ids == &ADJUSTED) {
        (true, _) => vec![&e.rsp_crc],
        (false, false) => vec![&e.notional],
        (false, true) => vec![&e.notional, clawback],
    };
    let premium_operands: [Vec<&Decimal>; 2] = if e.refundable {
        [vec![], vec![]]
    } else {
        [
            vec![&v[1], &e.total_expenses],
            vec![&v[1], &e.total_expenses],
        ]
    };
    let [first, second] = premium_operands;
    let operands = [
        base_operands,
        vec![&v[0]],
        vec![&v[1], &e.rate],
        first,
        second,
        vec![&v[0], &e.rate],
        vec![&v[2], &v[3], &v[4], &v[5]],
    ];
    Ok(ids
        .iter()
        .zip(rules)
        .zip(operands)
        .zip(v.iter())
        .map(|(((id, (formula, authority)), ops), value)| {
            step(*id, authority, &formula, &ops, value.clone())
        })
        .collect())
}

/// The clawback amount (ss 355-440 and 355-445) and the s 355-450 steps after
/// the adjusted offset.
struct Clawback {
    recoupment: Decimal,
    revenue: Decimal,
    expenditure: Decimal,
    amount: Decimal,
}

impl Clawback {
    fn new(recoupment: Decimal, revenue: Decimal, expenditure: Decimal) -> Result<Clawback> {
        let amount =
            precise(|| Ok(recoupment.add(&py_min(revenue.clone(), expenditure.clone())?)?))?;
        Ok(Clawback {
            recoupment,
            revenue,
            expenditure,
            amount,
        })
    }

    fn step(&self) -> Step {
        step(
            "clawback_amount",
            "ITAA 1997 ss 355-435(a), 355-440(2) and 355-445(2): the recoupment amount plus the lesser of the feedstock revenue and the feedstock expenditure",
            "recoupment_clawback + min(feedstock_revenue, feedstock_expenditure)",
            &[&self.recoupment, &self.revenue, &self.expenditure],
            self.amount.clone(),
        )
    }
}

/// The deduction amount and the amount included in assessable income.
fn income_steps(
    starting: &Decimal,
    adjusted: &Decimal,
    e: &Entity,
    claw: &Clawback,
    present_rate: &Decimal,
) -> Result<Vec<Step>> {
    let deduction = claw.amount.mul(&e.rate)?;
    let income = starting.sub(adjusted)?.sub(&deduction)?.div(present_rate)?;
    Ok(vec![
        step(
            "deduction_amount",
            "ITAA 1997 s 355-450(1) 'deduction amount': the clawback amount at the corporate tax rate for the offset year",
            "clawback_amount x corporate_tax_rate",
            &[&claw.amount, &e.rate],
            deduction.clone(),
        ),
        step(
            "clawback_income",
            "ITAA 1997 s 355-450(1): included in assessable income for the present year, divided by the corporate tax rate for the present year",
            "(tax_offset - adjusted_offset - deduction_amount) / present_year_corporate_tax_rate",
            &[starting, adjusted, &deduction, present_rate],
            income,
        ),
    ])
}

/// The refundable and non-refundable parts of the offset (s 67-30(1)).
fn split_steps(offset: &Decimal, refundable: bool) -> [Step; 2] {
    const REFUNDABLE: &str = "ITAA 1997 s 67-30(1): refundable where item 1 of the s 355-100(1) table applies, disregarding s 355-100(3)";
    const NON_REFUNDABLE: &str = "ITAA 1997 s 63-10(1) item 35: a Division 355 offset not covered by s 67-30 is non-refundable and may be carried forward (Division 65)";
    let part = |id: &'static str, authority, taken: bool| {
        if taken {
            step(id, authority, "tax_offset", &[offset], offset.clone())
        } else {
            step(id, authority, "0", &[], int(0))
        }
    };
    [
        part("refundable_offset", REFUNDABLE, refundable),
        part("non_refundable_offset", NON_REFUNDABLE, !refundable),
    ]
}

/// The amounts' bounds the law sets between them.
fn check(e: &Entity, claw: &Clawback) -> Result<()> {
    if e.rsp_crc.gt(&e.notional)? {
        return refuse(
            "rd_tax_offset.threshold_expenditure_above_total",
            "rsp_crc_expenditure is part of notional_deductions and cannot exceed it (ITAA 1997 s 355-100(2)).",
        );
    }
    if claw.expenditure.gt(&e.notional)? {
        return refuse(
            "rd_tax_offset.feedstock_expenditure_above_total",
            "feedstock_expenditure is deductions for this offset year and cannot exceed notional_deductions (ITAA 1997 s 355-445(1)(b) and (2)(b)).",
        );
    }
    if claw.amount.gt(&e.notional)? {
        return refuse(
            "rd_tax_offset.clawback_above_total",
            "The clawback amount exceeds notional_deductions, the total the adjusted offset reduces (ITAA 1997 s 355-450(1)).",
        );
    }
    let reduced = e.notional.sub(&claw.amount)?;
    if !claw.amount.is_zero() && reduced.lt(&int(20000))? {
        return refuse(
            "rd_tax_offset.clawback_below_threshold",
            "With a clawback amount, the s 355-100(1) total or the adjusted total is under $20,000: s 355-100(2) then limits the adjusted offset to research service provider and CRC expenditure, but neither s 355-450 nor the ATO says how the clawback reduces that expenditure, and the ATO's feedstock example applies the full rate below $20,000; this worksheet does not choose between them.",
        );
    }
    Ok(())
}

/// The entity's facts from the inputs, and whether item 1 applies.
fn entity(args: &RdTaxOffsetArgs) -> Result<Entity> {
    let notional = money(args.notional_deductions)?;
    let rsp_crc = money(args.rsp_crc_expenditure)?;
    let turnover = money(args.aggregated_turnover)?;
    let exempt = flag(args.exempt_entity_controlled, "exempt_entity_controlled")?;
    let bre = flag(args.base_rate_entity, "base_rate_entity")?;
    Ok(Entity {
        notional,
        rsp_crc,
        total_expenses: money(args.total_expenses)?,
        refundable: turnover.lt(&int(20_000_000))? && !exempt,
        rate: corporate_rate(bre),
    })
}

/// Every step: the offset and its parts, the clawback amount, the adjusted
/// offset and the s 355-450 amounts.
fn all_steps(e: &Entity, claw: &Clawback, present_rate: &Decimal) -> Result<Vec<Step>> {
    precise(|| {
        let reduced = e.notional.sub(&claw.amount)?;
        let mut steps = offset_steps(&STARTING, e, &e.notional, &claw.amount)?;
        let starting = steps[6].value.clone();
        steps.extend(split_steps(&starting, e.refundable));
        steps.push(claw.step());
        let adjusted = offset_steps(&ADJUSTED, e, &reduced, &claw.amount)?;
        let adjusted_offset = adjusted[6].value.clone();
        steps.extend(adjusted);
        let income = income_steps(&starting, &adjusted_offset, e, claw, present_rate)?;
        steps.extend(income);
        Ok(steps)
    })
}

/// ITAA 1997 s 355-100: one R&D entity's tax offset for an income year from
/// 2021-22, and s 355-450's amount for a clawback amount attributable to it.
pub fn rd_tax_offset(args: &RdTaxOffsetArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)
        .map_err(|refusal| earlier_years(refusal, args.year))?;
    let e = entity(args)?;
    let claw = Clawback::new(
        money(args.recoupment_clawback)?,
        money(args.feedstock_revenue)?,
        money(args.feedstock_expenditure)?,
    )?;
    let present_year = args.present_year_base_rate_entity;
    let present_rate = corporate_rate(flag(present_year, "present_year_base_rate_entity")?);
    check(&e, &claw)?;
    let built = all_steps(&e, &claw, &present_rate)?;
    let rates = vec![
        rate("corporate_tax_rate", &e.rate),
        rate("present_year_corporate_tax_rate", &present_rate),
    ];
    derived_result(KIND, &year, built, rates)
}
