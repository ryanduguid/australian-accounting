// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The unused leave worksheet's steps: each part's assessable share and offset
//! (ITAA 1997 ss 83-10 to 83-90), the Schedule 7 withholding and the income
//! statement amounts.

use super::super::{
    AdditionalPay, Result, Step, d, int, lookup, precise, py_min, spread_withholding, step,
    steps_from,
};
use super::{Leave, Part};
use crate::decimal::{Decimal, Rounding};
use crate::metadata as meta;

/// The amounts each later step uses.
struct Shares {
    annual_marginal: Decimal,
    pre_1978: Decimal,
    pre_1978_assessable: Decimal,
    long_service_marginal: Decimal,
    capped: Decimal,
}

const ANNUAL_CAPPED: &str = "ITAA 1997 ss 83-10(2) and (3) and 83-15(a): all of it, paid in connection with a genuine redundancy payment, an early retirement scheme payment or an invalidity segment";
const ANNUAL_CAPPED_NORMAL: &str = "ITAA 1997 ss 83-10(2) and (3) and 83-15(b): the part in respect of employment before 18 August 1993";
const LSL_CAPPED: &str = "ITAA 1997 s 83-85(2)(a) and (b): the pre-18/8/93 and post-17/8/93 periods, paid in connection with a genuine redundancy payment, an early retirement scheme payment or an invalidity segment";
const LSL_CAPPED_NORMAL: &str =
    "ITAA 1997 s 83-85(2)(a): the pre-18/8/93 period, 16 August 1978 to 17 August 1993";
const UNDER_300: &str = "Schedule 7, 'When a TFN is provided': a post-17 August 1993 payment from normal termination under $300 has the lesser of the marginal rate amount and 32% withheld; 'Rounding of withholding amounts'";
const MARGINAL_STEPS: &str = "Schedule 7, 'Marginal rate calculation' steps 6 and 7";

const ANNUAL_MARGINAL: [(&str, &str, &str); 1] = [(
    "annual_leave_marginal",
    "ITAA 1997 s 83-10(2) with no s 83-15 offset; Schedule 7, annual leave and loading, normal termination, post-17 August 1993: marginal rates",
    "annual_leave + leave_loading - annual_leave_capped",
)];

const PRE_1978: [(&str, &str, &str); 2] = [
    (
        "long_service_leave_pre_1978_assessable",
        "ITAA 1997 s 83-80(1) item 1: 5%",
        "long_service_leave_pre_1978 x pre_1978_assessable_rate",
    ),
    (
        "long_service_leave_pre_1978_not_assessable",
        "ITAA 1997 s 83-80(2): not assessable income and not exempt income",
        "long_service_leave_pre_1978 - long_service_leave_pre_1978_assessable",
    ),
];

const TOTALS: [(&str, &str, &str); 3] = [
    (
        "long_service_leave_marginal",
        "ITAA 1997 s 83-80(1) item 3 with no s 83-85 offset: the post-17/8/93 period on a normal termination; Schedule 7: marginal rates",
        "long_service_leave - long_service_leave_pre_1978 - long_service_leave_capped",
    ),
    (
        "assessable_income",
        "ITAA 1997 ss 83-10(2) and 83-80(1)",
        "annual_leave + leave_loading + long_service_leave - long_service_leave_pre_1978_not_assessable",
    ),
    (
        "offset_capped_income",
        "ITAA 1997 ss 83-15 and 83-85: a tax offset ensures the rate of tax on it does not exceed 30%",
        "annual_leave_capped + long_service_leave_capped",
    ),
];

/// A part's step: its formula is the input's name, or 0 where it is not given.
fn given_part(id: &'static str, authority: &'static str, part: &Part) -> Step {
    let (name, value) = part.term();
    let operands: Vec<&Decimal> = part.value.iter().collect();
    step(id, authority, name, &operands, value)
}

/// Annual leave and loading: the part the s 83-15 offset caps and the rest.
fn annual_steps(leave: &Leave, concessional: bool) -> Result<(Vec<Step>, Decimal, Decimal)> {
    let all = leave.annual.add(&leave.loading)?;
    let [(a93, a93v), (l93, l93v)] = [&leave.parts[0], &leave.parts[1]].map(Part::term);
    let (capped, capped_step) = if concessional {
        let formula = "annual_leave + leave_loading";
        let operands = [&leave.annual, &leave.loading];
        (
            all.clone(),
            step(
                "annual_leave_capped",
                ANNUAL_CAPPED,
                formula,
                &operands,
                all.clone(),
            ),
        )
    } else {
        let value = a93v.add(&l93v)?;
        let given: Vec<&Decimal> = leave.parts[..2]
            .iter()
            .filter_map(|p| p.value.as_ref())
            .collect();
        let formula = format!("{a93} + {l93}");
        let id = "annual_leave_capped";
        (
            value.clone(),
            step(id, ANNUAL_CAPPED_NORMAL, &formula, &given, value),
        )
    };
    let marginal = all.sub(&capped)?;
    let mut steps = vec![capped_step];
    steps.extend(steps_from(
        &ANNUAL_MARGINAL,
        [(
            &[&leave.annual, &leave.loading, &capped][..],
            marginal.clone(),
        )],
    ));
    Ok((steps, capped, marginal))
}

/// The pre-16 August 1978 long service leave, the 5% of it that is assessable
/// and the rest, which is not: its steps, then the part and the 5%.
fn pre_1978_steps(leave: &Leave) -> Result<(Vec<Step>, [Decimal; 3])> {
    let five = d(lookup(
        &meta::UNUSED_LEAVE_PAYMENT,
        "pre_1978_assessable_rate",
    ));
    let period = "ITAA 1997 ss 83-80(1) item 1 and 83-90(1): the pre-16/8/78 period";
    let pre_1978 = leave.parts[2].term().1;
    let mut steps = vec![given_part(
        "long_service_leave_pre_1978",
        period,
        &leave.parts[2],
    )];
    let assessable_5 = pre_1978.mul(&five)?;
    let exempt = pre_1978.sub(&assessable_5)?;
    steps.extend(steps_from(
        &PRE_1978,
        [
            (&[&pre_1978, &five][..], assessable_5.clone()),
            (&[&pre_1978, &assessable_5][..], exempt.clone()),
        ],
    ));
    Ok((steps, [pre_1978, assessable_5, exempt]))
}

/// Long service leave: the pre-16 August 1978 steps, the part the s 83-85
/// offset caps and the rest; then the assessable total and the capped total
/// with annual leave.
fn long_service_steps(
    leave: &Leave,
    concessional: bool,
    (annual_capped, annual_marginal): (Decimal, Decimal),
) -> Result<(Vec<Step>, Shares)> {
    let (mut steps, [pre_1978, assessable_5, exempt]) = pre_1978_steps(leave)?;
    let after_1978 = leave.long_service.sub(&pre_1978)?;
    let id = "long_service_leave_capped";
    let lsl_capped = if concessional {
        let formula = "long_service_leave - long_service_leave_pre_1978";
        let operands = [&leave.long_service, &pre_1978];
        step(id, LSL_CAPPED, formula, &operands, after_1978.clone())
    } else {
        given_part(id, LSL_CAPPED_NORMAL, &leave.parts[3])
    };
    let capped_lsl = lsl_capped.value.clone();
    steps.push(lsl_capped);
    let marginal = after_1978.sub(&capped_lsl)?;
    let all_annual = leave.annual.add(&leave.loading)?;
    let assessable = all_annual.add(&leave.long_service)?.sub(&exempt)?;
    let capped = annual_capped.add(&capped_lsl)?;
    steps.extend(steps_from(
        &TOTALS,
        [
            (
                &[&leave.long_service, &pre_1978, &capped_lsl][..],
                marginal.clone(),
            ),
            (
                &[&leave.annual, &leave.loading, &leave.long_service, &exempt][..],
                assessable,
            ),
            (&[&annual_capped, &capped_lsl][..], capped.clone()),
        ],
    ));
    let shares = Shares {
        annual_marginal,
        pre_1978,
        pre_1978_assessable: assessable_5,
        long_service_marginal: marginal,
        capped,
    };
    Ok((steps, shares))
}

const FLAT: [(&str, &str, &str); 2] = [
    (
        "flat_rate_withholding",
        "Schedule 7, 'Withholding amounts for long service leave, annual leave and annual leave loading': 32%; 'Rounding of withholding amounts': to the nearest dollar, 50 cents up",
        "dollars(offset_capped_income x flat_rate)",
    ),
    (
        "marginal_rate_amount",
        "Schedule 7, 'Withholding amounts for long service leave, annual leave and annual leave loading': post-17 August 1993 leave on normal termination and 5% of pre-16 August 1978 long service leave at marginal rates, added together as its example does",
        "annual_leave_marginal + long_service_leave_marginal + long_service_leave_pre_1978_assessable",
    ),
];

const PER_PERIOD: [(&str, &str, &str); 1] = [(
    "marginal_amount_per_period",
    "Schedule 7, 'Marginal rate calculation' steps 2 and 3",
    "floor(marginal_rate_amount / pay_periods_in_year)",
)];

const BEFORE_LIMIT: [(&str, &str, &str); 1] = [(
    "marginal_withholding_before_limit",
    MARGINAL_STEPS,
    "(withholding_on_earnings_and_share - withholding_on_earnings) x pay_periods_in_year",
)];

/// Schedule 7's marginal rate calculation, steps 1 to 7, on `marginal`.
fn marginal_steps(pay: &AdditionalPay, marginal: &Decimal) -> Result<(Vec<Step>, Decimal)> {
    let mut steps = Vec::new();
    let on_earnings = pay.table(&pay.earnings)?;
    let pushed = pay.table_step(
        "withholding_on_earnings",
        "Schedule 7, 'Marginal rate calculation' step 1 and 'Normal gross earnings'; Schedule 1, 'Using a formula', without the study and training support loan component Schedule 7 excludes",
        (&pay.earnings, "earnings", &[&pay.earnings]),
        on_earnings.clone(),
    );
    steps.push(pushed?);
    let (share, with_share, before_limit) =
        spread_withholding(pay, &pay.earnings, &on_earnings, marginal, pay.per_year)?;
    let per_year = int(pay.per_year);
    steps.extend(steps_from(
        &PER_PERIOD,
        [(&[marginal, &per_year][..], share.clone())],
    ));
    let pushed = pay.table_step(
        "withholding_on_earnings_and_share",
        "Schedule 7, 'Marginal rate calculation' steps 4 and 5; Schedule 1, 'Using a formula'",
        (
            &pay.earnings.add(&share)?,
            "earnings + marginal_amount_per_period",
            &[&pay.earnings, &share],
        ),
        with_share.clone(),
    );
    steps.push(pushed?);
    steps.extend(steps_from(
        &BEFORE_LIMIT,
        [(
            &[&with_share, &on_earnings, &per_year][..],
            before_limit.clone(),
        )],
    ));
    Ok((steps, before_limit))
}

/// The marginal rate calculation's result, limited under $300 to 32% of the
/// amount it was worked on.
fn limit_step(
    under_300: bool,
    before_limit: Decimal,
    marginal: &Decimal,
    flat: &Decimal,
) -> Result<Step> {
    if !under_300 {
        let formula = "marginal_withholding_before_limit";
        let operands = [&before_limit];
        return Ok(step(
            "marginal_withholding",
            MARGINAL_STEPS,
            formula,
            &operands,
            before_limit.clone(),
        ));
    }
    let lesser = py_min(before_limit.clone(), marginal.mul(flat)?)?;
    Ok(step(
        "marginal_withholding",
        UNDER_300,
        "dollars(min(marginal_withholding_before_limit, marginal_rate_amount x flat_rate))",
        &[&before_limit, marginal, flat],
        lesser.quantize(0, Rounding::HalfUp)?,
    ))
}

/// The 32% part, then Schedule 7's marginal rate calculation on the rest with
/// the $300 limit, and the total withheld.
fn withholding_steps(pay: &AdditionalPay, shares: &Shares) -> Result<(Vec<Step>, Decimal)> {
    let [flat, small] =
        ["flat_rate", "small_payment"].map(|key| d(lookup(&meta::UNUSED_LEAVE_PAYMENT, key)));
    let flat_withheld = shares.capped.mul(&flat)?.quantize(0, Rounding::HalfUp)?;
    let post_1993 = shares.annual_marginal.add(&shares.long_service_marginal)?;
    let marginal = post_1993.add(&shares.pre_1978_assessable)?;
    // Under $300 the leave has no pre-16 August 1978 part (the input rules).
    let under_300 = !post_1993.is_zero() && post_1993.lt(&small)?;
    let [a, l, p] = [
        &shares.annual_marginal,
        &shares.long_service_marginal,
        &shares.pre_1978_assessable,
    ];
    let mut steps = steps_from(
        &FLAT,
        [
            (&[&shares.capped, &flat][..], flat_withheld.clone()),
            (&[a, l, p][..], marginal.clone()),
        ],
    );
    let (marginal_steps, before_limit) = marginal_steps(pay, &marginal)?;
    steps.extend(marginal_steps);
    let limited = limit_step(under_300, before_limit, &marginal, &flat)?;
    let marginal_withheld = limited.value.clone();
    steps.push(limited);
    let total = flat_withheld.add(&marginal_withheld)?;
    steps.push(step(
        "withholding",
        "Schedule 7, example: the 32% part and the marginal rate part",
        "flat_rate_withholding + marginal_withholding",
        &[&flat_withheld, &marginal_withheld],
        total,
    ));
    Ok((steps, post_1993))
}

const LABELS: [(&str, &str, &str); 4] = [
    (
        "lump_sum_a",
        "Schedule 7, label A; STP Phase 2 employer reporting guidelines, 'When an employee transfers or leaves': lump sum A",
        "offset_capped_income",
    ),
    (
        "lump_sum_b",
        "Schedule 7, label B; STP Phase 2 employer reporting guidelines, 'When an employee transfers or leaves': lump sum B, the whole amount though only 5% is taxed",
        "long_service_leave_pre_1978",
    ),
    (
        "unused_leave_on_termination",
        "Schedule 7, 'Include in salary/wages'; STP Phase 2 employer reporting guidelines, 'When an employee transfers or leaves': unused leave on termination (paid leave type U)",
        "annual_leave_marginal + long_service_leave_marginal",
    ),
    (
        "qualifying_earnings",
        "SGAA 1992 s 6(1) 'ordinary time earnings': not an unused annual leave payment or unused long service leave payment within the meaning of ITAA 1997, and s 10A(1); ATO, 'What payments are qualifying earnings', table 13: unused leave on termination, including annual leave, leave loading and long service leave, is neither ordinary time earnings nor qualifying earnings, whatever the reason for termination",
        "0",
    ),
];

/// Without a TFN: `no_tfn_rate` of the whole payment, cents ignored.
fn no_tfn_step(leave: &Leave, no_tfn_rate: &str) -> Result<Step> {
    let rate = d(no_tfn_rate);
    let payment = leave.annual.add(&leave.loading)?.add(&leave.long_service)?;
    let withheld = payment.mul(&rate)?.quantize(0, Rounding::Down)?;
    Ok(step(
        "withholding",
        "Schedule 7, 'When a TFN has not been provided': 47% from the payment, 45% for a foreign resident; 'Rounding of withholding amounts': cents ignored",
        "floor((annual_leave + leave_loading + long_service_leave) x no_tfn_rate)",
        &[&leave.annual, &leave.loading, &leave.long_service, &rate],
        withheld,
    ))
}

/// Each part's assessable share and offset, the Schedule 7 withholding and the
/// income statement amounts, in that order.
pub(super) fn leave_steps(
    pay: &AdditionalPay,
    leave: &Leave,
    concessional: bool,
    no_tfn_rate: Option<&str>,
) -> Result<Vec<Step>> {
    precise(|| {
        let (mut steps, annual_capped, annual_marginal) = annual_steps(leave, concessional)?;
        let (lsl, shares) =
            long_service_steps(leave, concessional, (annual_capped, annual_marginal))?;
        steps.extend(lsl);
        let post_1993 = match no_tfn_rate {
            Some(no_tfn) => {
                steps.push(no_tfn_step(leave, no_tfn)?);
                shares.annual_marginal.add(&shares.long_service_marginal)?
            }
            None => {
                let (withholding, post_1993) = withholding_steps(pay, &shares)?;
                steps.extend(withholding);
                post_1993
            }
        };
        let s = &shares;
        steps.extend(steps_from(
            &LABELS,
            [
                (&[&s.capped][..], s.capped.clone()),
                (&[&s.pre_1978][..], s.pre_1978.clone()),
                (
                    &[&s.annual_marginal, &s.long_service_marginal][..],
                    post_1993,
                ),
                (&[][..], int(0)),
            ],
        ));
        Ok(steps)
    })
}
