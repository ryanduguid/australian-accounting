// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `chattel_mortgage_schedule`: a loan to buy goods, secured by a chattel
//! mortgage over them, repaid by level instalments and any balloon, with each
//! payment split into interest and principal by the actuarial method at a
//! stated rate or the rate the payments imply, the interest by income year and
//! the purchaser's input tax credit on the price. The buyer owns the goods
//! from the purchase, so the agreement is not a hire purchase agreement (ITAA
//! 1997 s 995-1) and Division 240 does not apply (s 240-10). Sources: ITAA
//! 1997 compilation 268 (1 October 2026), ss 8-1, 240-10 and 995-1; GST Act
//! compilation 96 (1 January 2026), ss 9-90, 29-10, 40-5 and 69-10; the ATO's
//! GST issues register 'Financial services - questions and answers', issues
//! 1.14 and 1.19 to 1.21 (updated to 2 May 2013); TR 93/16, paras 2, 18 and 28
//! to 31; and 'Purchasing a motor vehicle' (last updated 3 June 2026), all
//! read on 19 November 2026.

use serde_json::{Value, json};

use super::regime::{self, Unit};
use super::{
    Agreement, Period, by_income_year, gst, optional_money, rates, start_in_year, statement, timing,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::engine::{
    Arg, Result, Step, derived_result, finite_between, int, money, precise, refuse, scope, step,
};

const KIND: &str = "chattel_mortgage_schedule";

const FINANCED: &str = "As established: the amount lent to buy the goods, which the borrower owns from the purchase; an agreement under which title passes at the purchase is not a hire purchase agreement (ITAA 1997 s 995-1), so Division 240 does not treat it as a notional sale and loan (s 240-10)";
const INSTALMENT: &str = "TR 93/16 para 18 (the actuarial method): the level instalment, shown to 16 places, at which the balance owing, charged annual_rate / 12 (or / 52) a period, is repaid with the balloon at the end of the term, rounded half up to the cent";
const ADJUSTMENT: &str = "The balance, shown to 16 places, that the rounded instalments and the balloon leave at the stated rate at the end of the term, rounded half up to the cent and added to the last instalment, so the schedule ends at nil";
const TOTAL_PAYMENTS: &str =
    "The instalments and the balloon the agreement schedules to the end of the term";
const TOTAL_INTEREST: &str = "TR 93/16 paras 18 and 29 (the actuarial method): each period's interest is the balance owing times the rate for the period, so the interest over the term is the payments less the amount financed. It is deductible under ITAA 1997 s 8-1(1) to the extent the borrowed money is used to gain or produce assessable income, and not to the extent it is capital, private or domestic (s 8-1(2)), which the operator establishes; where the lender's statements give the interest, that figure is used (TR 93/16 para 2)";
const DEPOSIT: &str = "As established: paid by the purchaser at the purchase towards the price";
const TRADE_IN: &str =
    "As established: the trade-in allowance the supplier credits at the purchase towards the price";
const PRICE: &str = "The price of the goods: the amount financed, which is paid to the supplier at the purchase, with the deposit and the trade-in allowance (the consideration for the supply, GST Act s 9-75(1))";

/// The facts as the command passes them; see the README for each.
pub struct ChattelMortgageArgs<'a> {
    pub amount_financed: &'a Arg,
    pub instalment: Option<&'a Arg>,
    pub annual_rate: Option<&'a Arg>,
    pub instalments: &'a Arg,
    pub frequency: Option<&'a Arg>,
    pub balloon: &'a Arg,
    pub timing: &'a Arg,
    pub start_date: &'a Arg,
    pub deposit: Option<&'a Arg>,
    pub trade_in: Option<&'a Arg>,
    pub gst: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The instalment as given, or the stated annual rate it is worked out at.
enum Terms {
    Instalment(Decimal),
    AnnualRate(Decimal),
}

fn terms(args: &ChattelMortgageArgs) -> Result<Terms> {
    match (args.instalment, args.annual_rate) {
        (Some(instalment), None) => Ok(Terms::Instalment(money(instalment)?)),
        (None, Some(rate)) => match finite_between(rate, "0", "1")? {
            Some(rate) => Ok(Terms::AnnualRate(rate)),
            None => refuse(
                "input.fraction_invalid",
                "annual_rate must be a finite decimal fraction from 0 to 1.",
            ),
        },
        _ => refuse(
            "chattel_mortgage.terms_conflict",
            "Give instalment, and the rate is found from the payments, or annual_rate, and the instalment is worked out at it: one of them, not both or neither.",
        ),
    }
}

/// The level agreement of `instalments` instalments of `amount`.
fn level(
    args: &ChattelMortgageArgs,
    financed: &Decimal,
    balloon: &Decimal,
    amount: Decimal,
    in_advance: bool,
) -> Result<Agreement> {
    let plan = regime::level(amount, args.instalments, args.frequency)?;
    Ok(Agreement {
        financed: financed.clone(),
        balloon: balloon.clone(),
        in_advance,
        regime: regime::build(plan, in_advance, false)?,
    })
}

/// The agreement at a stated annual rate: the rate a period, the rounded
/// instalment, the instalment before rounding, the balance left before the
/// last instalment absorbs it and that adjustment.
struct Stated {
    agreement: Agreement,
    rate: Decimal,
    instalment: Decimal,
    exact: Decimal,
    left: Decimal,
    adjustment: Decimal,
}

/// The level instalment at `annual_rate` a year for monthly or weekly
/// instalments in arrears. The balance a schedule leaves falls by the same
/// amount for each dollar of instalment, so it is the balance with no
/// instalment over the fall per dollar.
fn stated(
    args: &ChattelMortgageArgs,
    financed: &Decimal,
    balloon: &Decimal,
    annual_rate: &Decimal,
    in_advance: bool,
) -> Result<Stated> {
    let nil = level(args, financed, balloon, int(0), in_advance)?;
    if in_advance || nil.regime.periods[0].1 != 1 {
        return refuse(
            "chattel_mortgage.stated_rate_unsupported",
            "annual_rate is taken for monthly or weekly instalments in arrears, charged annual_rate / 12 or / 52 a period; for another frequency or instalments in advance, give the instalment and the rate is found from the payments.",
        );
    }
    let per_year = match nil.regime.unit {
        Unit::Month => 12,
        Unit::Week => 52,
    };
    let rate = annual_rate.div(&int(per_year))?;
    let unpaid = nil.residual(&rate)?;
    let per_dollar =
        unpaid.sub(&level(args, financed, balloon, int(1), false)?.residual(&rate)?)?;
    // To 16 places, as the step shows it, then to the cent.
    let exact = unpaid.div(&per_dollar)?.quantize(-16, Rounding::HalfUp)?;
    let instalment = exact.quantize(-2, Rounding::HalfUp)?;
    if !int(0).lt(&instalment)? {
        return refuse(
            "chattel_mortgage.balloon_too_large",
            "At annual_rate the balloon alone repays the amount financed or more, leaving no instalment to work out.",
        );
    }
    let mut agreement = level(args, financed, balloon, instalment.clone(), false)?;
    let (left, adjustment) = absorb(&mut agreement, &rate)?;
    Ok(Stated {
        agreement,
        rate,
        instalment,
        exact,
        left,
        adjustment,
    })
}

/// Adds to the last instalment the balance left before it, at `rate`: that
/// balance to 16 places and the adjustment, its cents. An adjustment of more
/// than a whole instalment either way, which a cent's rounding compounds to
/// only at a high rate over a long term, is refused.
fn absorb(agreement: &mut Agreement, rate: &Decimal) -> Result<(Decimal, Decimal)> {
    let left = agreement.residual(rate)?.quantize(-16, Rounding::HalfUp)?;
    let adjustment = left.quantize(-2, Rounding::HalfUp)?;
    let last = agreement.term() as usize;
    let level = &agreement.regime.due[last];
    let final_instalment = level.add(&adjustment)?;
    if final_instalment.lt(&int(0))? || adjustment.gt(level)? {
        return refuse(
            "chattel_mortgage.final_instalment_unsupported",
            "Rounding the instalment to the cent moves the last instalment by more than a whole instalment at this rate and term, below nil or above twice the instalment; give the instalment instead.",
        );
    }
    agreement.regime.due[last] = final_instalment;
    agreement.regime.total = agreement.regime.total.add(&adjustment)?;
    Ok((left, adjustment))
}

/// The steps that set a stated-rate agreement's payments.
fn stated_steps(s: &Stated, balloon: &Decimal) -> Result<[Step; 3]> {
    let total = s.agreement.regime.total.add(balloon)?;
    let count = int(s.agreement.regime.periods.len() as i64);
    Ok([
        step(
            "instalment",
            INSTALMENT,
            &format!("cents({})", s.exact.fixed(16, false)),
            &[],
            s.instalment.clone(),
        ),
        step(
            "final_adjustment",
            ADJUSTMENT,
            &format!("cents({})", s.left.fixed(16, false)),
            &[],
            s.adjustment.clone(),
        ),
        step(
            "total_payments",
            TOTAL_PAYMENTS,
            "instalment x instalments + final_adjustment + balloon",
            &[&s.instalment, &count, &s.adjustment, balloon],
            total,
        ),
    ])
}

/// The agreement, the rate a period it is scheduled at, and the steps that
/// set its payments.
fn agreement(args: &ChattelMortgageArgs, terms: &Terms) -> Result<(Agreement, Decimal, Vec<Step>)> {
    let financed = money(args.amount_financed)?;
    let balloon = money(args.balloon)?;
    let in_advance = timing(args.timing)?;
    let mut steps = vec![step(
        "amount_financed",
        FINANCED,
        "amount_financed",
        &[&financed],
        financed.clone(),
    )];
    match terms {
        Terms::Instalment(amount) => {
            if financed.is_zero() || amount.is_zero() {
                return refuse(
                    "hire_purchase.amount_nil",
                    "amount_financed and instalment must be more than nil.",
                );
            }
            let agreement = level(args, &financed, &balloon, amount.clone(), in_advance)?;
            let total = agreement.regime.total.add(&balloon)?;
            steps.push(step(
                "total_payments",
                TOTAL_PAYMENTS,
                "instalment x instalments + balloon",
                &[
                    amount,
                    &int(agreement.regime.periods.len() as i64),
                    &balloon,
                ],
                total,
            ));
            let rate = agreement.implicit_rate()?;
            Ok((agreement, rate, steps))
        }
        Terms::AnnualRate(annual_rate) => {
            if financed.is_zero() {
                return refuse(
                    "hire_purchase.amount_nil",
                    "amount_financed must be more than nil.",
                );
            }
            let s = stated(args, &financed, &balloon, annual_rate, in_advance)?;
            steps.extend(stated_steps(&s, &balloon)?);
            Ok((s.agreement, s.rate, steps))
        }
    }
}

/// The deposit, trade-in and price steps.
fn price_steps(financed: &Decimal, deposit: &Decimal, trade_in: &Decimal) -> Result<[Step; 3]> {
    let price = financed.add(deposit)?.add(trade_in)?;
    Ok([
        step("deposit", DEPOSIT, "deposit", &[deposit], deposit.clone()),
        step(
            "trade_in",
            TRADE_IN,
            "trade_in",
            &[trade_in],
            trade_in.clone(),
        ),
        step(
            "price",
            PRICE,
            "amount_financed + deposit + trade_in",
            &[financed, deposit, trade_in],
            price,
        ),
    ])
}

/// The schedule as a printable Markdown table.
fn printable(rows: &[Value]) -> String {
    let columns = [
        ("instalment", "Instalment"),
        ("period_end", "Period end"),
        ("payment_date", "Paid"),
        ("opening", "Opening"),
        ("interest", "Interest"),
        ("payment", "Instalment paid"),
        ("balloon", "Balloon"),
        ("principal", "Principal"),
        ("closing", "Closing"),
    ];
    let cells = rows
        .iter()
        .map(|row| {
            columns
                .iter()
                .map(|(key, _)| match &row[key] {
                    Value::String(s) => s.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                })
                .collect()
        })
        .collect::<Vec<Vec<String>>>();
    let headers: Vec<&str> = columns.iter().map(|c| c.1).collect();
    let right = [true, false, false, true, true, true, true, true, true];
    statement::markdown(&headers, &right, &cells)
}

/// The rate a period it is scheduled at, the schedule rows and periods, and
/// the steps.
type Worked = (
    Decimal,
    Vec<Value>,
    Vec<Period>,
    Vec<Step>,
    Option<gst::Worked>,
);

fn worked(
    args: &ChattelMortgageArgs,
    terms: &Terms,
    start: i64,
    upfront: [&Decimal; 2],
    gst_facts: &gst::Facts,
) -> Result<(Agreement, Worked)> {
    let (agreement, rate, mut steps) = agreement(args, terms)?;
    let financed = &agreement.financed;
    let charges = agreement
        .regime
        .total
        .add(&agreement.balloon)?
        .sub(financed)?;
    steps.push(step(
        "total_interest",
        TOTAL_INTEREST,
        "total_payments - amount_financed",
        &[&agreement.regime.total.add(&agreement.balloon)?, financed],
        charges.clone(),
    ));
    let (rows, periods) = agreement.schedule(&rate, start, &charges, None)?;
    let [deposit, trade_in] = upfront;
    let mut credit = None;
    if args.deposit.is_some() || args.trade_in.is_some() || args.gst.is_some() {
        let price = price_steps(financed, deposit, trade_in)?;
        let price_value = price[2].value.clone();
        steps.extend(price);
        credit = gst::purchase_worked(gst_facts, &price_value)?;
        if let Some(worked) = &mut credit {
            steps.append(&mut worked.steps);
        }
    }
    Ok((agreement, (rate, rows, periods, steps, credit)))
}

pub fn chattel_mortgage_schedule(args: &ChattelMortgageArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let terms = terms(args)?;
    let start = start_in_year(args.start_date, &year)?;
    let deposit = optional_money(args.deposit)?;
    let trade_in = optional_money(args.trade_in)?;
    let gst_facts = gst::read(args.gst, false)?;
    let (agreement, (rate, rows, periods, steps, credit)) =
        precise(|| worked(args, &terms, start, [&deposit, &trade_in], &gst_facts))?;
    let unit = agreement.regime.unit;
    let car_limit = credit.as_ref().and_then(|w| w.car_limit.as_ref());
    let implicit = matches!(terms, Terms::Instalment(_));
    let mut out = derived_result(KIND, &year, steps, rates(&rate, unit, implicit, car_limit)?)?;
    let (years, across) = precise(|| by_income_year(&periods))?;
    out["balloon_date"] = json!(dates::format(unit.date(start, agreement.term())));
    out["statement"] = json!(printable(&rows));
    out["schedule"] = Value::Array(rows);
    out["interest_by_income_year"] = years;
    out["interest_across_30_june"] = across;
    out["gst"] = gst::purchase_summary(&gst_facts, &dates::format(start));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_printable_table_leaves_a_null_cell_blank_and_writes_a_number_as_text() {
        let given = printable(&[json!({"instalment": 1, "balloon": null})]);
        assert_eq!(
            given,
            printable(&[json!({"instalment": "1", "balloon": ""})])
        );
    }
}
