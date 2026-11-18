// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The tax free component of a life benefit employment termination payment:
//! its invalidity segment (ITAA 1997 s 82-150) and pre-July 83 segment
//! (s 82-155), and the taxable component left (ss 82-140 and 82-145). Read on
//! the ATO's legal database on 19 November 2026, 9.25 am to 9.40 am AEDT, with
//! the 'last retirement day' definition in s 995-1(1) (the current text, which
//! already omits the 'laundry expense' definition Act No. 49 of 2026 repealed),
//! the ATO's 'Tax-free component of ETPs' page (last updated 24 July 2025:
//! Jackson's example) and its worksheet NAT 71949 (June 2014), whose steps 1
//! and 2 count the employment days with both the first day and the
//! termination day.

use serde_json::Value;

use super::{
    Arg, Result, cents, date_arg, derived_result, flag, int, money, precise, rate, refuse,
};
use super::{scope, step};
use crate::dates;

const KIND: &str = "etp_tax_free_component";

/// `employment_start` to `termination_date`, both counted (NAT 71949 steps 1
/// to 3), and the part of them before 1 July 1983.
fn employment_days(start: i64, ended: i64) -> Result<(i64, i64)> {
    if start > ended {
        return refuse(
            "etp_tax_free_component.dates_invalid",
            "employment_start must be on or before termination_date: the employment days are those from the day employment started to the day it was terminated (ITAA 1997 ss 82-150(2) and 82-155(2)).",
        );
    }
    let before = dates::parse("1983-06-30").expect("30 June 1983");
    Ok((ended - start + 1, (before.min(ended) - start + 1).max(0)))
}

/// The invalidity segment's authority when it is nil, or `None` when
/// s 82-150(1) is met: `ill_health` covers paragraphs (a) and (b),
/// `certified` paragraph (d), and the days to retirement paragraph (c).
fn no_invalidity(
    ill_health: bool,
    certified: bool,
    days_to_retirement: i64,
) -> Option<&'static str> {
    if !ill_health {
        Some(
            "ITAA 1997 s 82-150(1)(a) and (b): the payment was not made because the person stopped being gainfully employed through ill-health, so it has no invalidity segment",
        )
    } else if !certified {
        Some(
            "ITAA 1997 s 82-150(1)(d): 2 legally qualified medical practitioners have not certified that, because of the ill-health, the person is unlikely ever to be gainfully employed in a capacity for which they are reasonably qualified, so the payment has no invalidity segment",
        )
    } else if days_to_retirement <= 0 {
        Some(
            "ITAA 1997 s 82-150(1)(c): the gainful employment did not stop before the person's last retirement day, so the payment has no invalidity segment",
        )
    } else {
        None
    }
}

/// ITAA 1997 ss 82-140 to 82-155: the invalidity segment, the pre-July 83
/// segment of the rest, their total as the tax free component and the
/// taxable component left, each segment to the cent, half up.
#[allow(clippy::too_many_arguments)]
pub fn etp_tax_free_component(
    payment: &Arg,
    employment_start: &Arg,
    termination_date: &Arg,
    last_retirement_day: &Arg,
    ill_health: &Arg,
    medical_certification: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
) -> Result<Value> {
    let year = scope(scope_confirmed, year, KIND)?;
    let payment = money(payment)?;
    let start = date_arg(employment_start, "employment_start")?;
    let ended = date_arg(termination_date, "termination_date")?;
    let retirement = date_arg(last_retirement_day, "last_retirement_day")?;
    let ill_health = flag(ill_health, "ill_health")?;
    let certified = flag(medical_certification, "medical_certification")?;
    let (employment, before_july_1983) = employment_days(start, ended)?;
    // s 82-150(2): the days from the termination day to the last retirement
    // day, the first not counted, as the Library's example counts 349 days
    // from 31 July 2026 to 15 July 2027.
    let to_retirement = retirement - ended;
    let nil = no_invalidity(ill_health, certified, to_retirement);
    let days_to_retirement = if nil.is_some() { 0 } else { to_retirement };
    let [employment_n, retirement_n, before_n] =
        [employment, days_to_retirement, before_july_1983].map(int);
    let steps = precise(|| {
        let invalidity = match nil {
            Some(authority) => step("invalidity_segment", authority, "0", &[], int(0)),
            None => {
                let value = cents(
                    payment
                        .mul(&retirement_n)?
                        .div(&employment_n.add(&retirement_n)?)?,
                )?;
                step(
                    "invalidity_segment",
                    "ITAA 1997 s 82-150(2), to the cent",
                    "cents(payment x days_to_retirement / (employment_days + days_to_retirement))",
                    &[&payment, &retirement_n, &employment_n, &retirement_n],
                    value,
                )
            }
        };
        let rest = payment.sub(&invalidity.value)?;
        let pre_july_1983 = if before_july_1983 == 0 {
            step(
                "pre_july_83_segment",
                "ITAA 1997 s 82-155(1): none of the employment to which the payment relates occurred before 1 July 1983",
                "0",
                &[],
                int(0),
            )
        } else {
            step(
                "pre_july_83_segment",
                "ITAA 1997 s 82-155(2) step 2, to the cent",
                "cents(payment_less_invalidity_segment x pre_july_1983_days / employment_days)",
                &[&rest, &before_n, &employment_n],
                cents(rest.mul(&before_n)?.div(&employment_n)?)?,
            )
        };
        let tax_free = invalidity.value.add(&pre_july_1983.value)?;
        let taxable = payment.sub(&tax_free)?;
        let rest_step = step(
            "payment_less_invalidity_segment",
            "ITAA 1997 s 82-155(2) step 1",
            "payment - invalidity_segment",
            &[&payment, &invalidity.value],
            rest,
        );
        let tax_free_step = step(
            "tax_free_component",
            "ITAA 1997 s 82-140",
            "invalidity_segment + pre_july_83_segment",
            &[&invalidity.value, &pre_july_1983.value],
            tax_free.clone(),
        );
        let taxable_step = step(
            "taxable_component",
            "ITAA 1997 s 82-145",
            "payment - tax_free_component",
            &[&payment, &tax_free],
            taxable,
        );
        Ok(vec![
            invalidity,
            rest_step,
            pre_july_1983,
            tax_free_step,
            taxable_step,
        ])
    })?;
    // s 82-10(6)(c): a payment that includes an invalidity segment, or would
    // but for s 82-150(1)(c), is an excluded payment.
    let excluded = ill_health && certified;
    let rates = vec![
        rate("employment_days", employment),
        rate("days_to_retirement", days_to_retirement),
        rate("pre_july_1983_days", before_july_1983),
        rate("invalidity_excluded_payment", excluded),
    ];
    derived_result(KIND, &year, steps, rates)
}
