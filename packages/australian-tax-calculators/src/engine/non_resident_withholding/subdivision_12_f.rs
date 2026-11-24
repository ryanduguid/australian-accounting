// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use serde_json::Value;

use super::{
    Facts, Worked, check_already_withheld, non_negative, parse_optional_rate, step, whole_dollars,
};
use crate::decimal::{Decimal, Rounding};
use crate::engine::{Result, d, flag, int, money, rate, refuse};

const DIVIDEND_RATE: &str = "0.30";
const INTEREST_RATE: &str = "0.10";
const ROYALTY_RATE: &str = "0.30";

/// Floor to whole cents so a statutory upper limit (TAA Sch 1 s 12-300(b)) is
/// never exceeded by half-cent rounding.
fn floor_cents(amount: &Decimal) -> Result<Decimal> {
    Ok(amount.quantize(-2, Rounding::Down)?)
}

fn compute_12_f_base(
    payment_type: &str,
    payment: &Decimal,
    facts: &Facts,
    steps: &mut Vec<Value>,
) -> Result<Decimal> {
    if payment_type == "dividend" {
        let franked = money(facts.get(0))?;
        let cfi = money(facts.get(1))?;
        let exempt_parts = franked.add(&cfi)?;
        if exempt_parts.gt(payment)? {
            return refuse(
                "non_resident_withholding.exempt_exceeds_payment",
                "franked_amount plus conduit_foreign_income exceeds payment.",
            );
        }
        let unfranked = payment.sub(&exempt_parts)?;
        steps.push(step(
            "withholding_base",
            "ITAA 1936 s 128B(3)(ga)(i) and ITAA 1997 ss 802-15 and 802-17: dividend less its franked part and conduit foreign income (TAA Sch 1 s 12-220; TAR 2017 s 39)",
            &unfranked,
        ));
        return Ok(unfranked);
    }
    let auth = if payment_type == "interest" {
        "TAA Sch 1 ss 12-245, 12-250 and 12-255; ITAA 1936 s 128B(2), (2A) and (5): interest payment subject to Subdivision 12-F"
    } else {
        "TAA Sch 1 ss 12-280 and 12-285; ITAA 1936 s 128B(2B), (2C) and (5A): royalty payment subject to Subdivision 12-F"
    };
    steps.push(step("withholding_base", auth, payment));
    Ok(payment.clone())
}

fn reg_12_f_auth(payment_type: &str) -> &'static str {
    match payment_type {
        "dividend" => {
            "TAR 2017 s 40 (30% or stated treaty rate) and s 26(1)(b), (2) and (3): cents in the worked-out amount disregarded"
        }
        "interest" => {
            "TAR 2017 s 41 (10% of the interest) and s 26(1)(b), (2) and (3): cents in the worked-out amount disregarded"
        }
        _ => {
            "TAR 2017 s 42 (30% or stated treaty rate) and s 26(1)(b), (2) and (3): cents in the worked-out amount disregarded"
        }
    }
}

fn compute_12_f_non_exempt(
    payment_type: &str,
    base: &Decimal,
    used: &Decimal,
    applied_rate: &Decimal,
    already: &Decimal,
) -> Result<(Decimal, Decimal, Vec<Value>)> {
    // TAR 2017 s 40 (dividend) and s 42 (royalty) apply the treaty rate directly;
    // s 41 (interest) specifies 10%, with TAA Sch 1 s 12-300 capping at treaty liability.
    let reg_rate = if payment_type == "interest" {
        d(INTEREST_RATE)
    } else {
        applied_rate.clone()
    };
    let reg = whole_dollars(&reg_rate.mul(used)?)?;
    let liability = applied_rate.mul(base)?;
    let cap = floor_cents(&non_negative(liability.sub(already)?)?)?;
    let final_wh = if cap.lt(&reg)? {
        cap.clone()
    } else {
        reg.clone()
    };
    let extra = vec![
        step("regulation_withholding", reg_12_f_auth(payment_type), &reg),
        step(
            "s_12_300_cap",
            "TAA Sch 1 s 12-300(a) and (b); ITAA 1936 s 128B; IT(DIRWHT)A 1974 s 7; International Tax Agreements Act 1953 s 4: withholding tax payable less amounts already withheld under Subdivision 12-F (floored to whole cents so the statutory limit is not exceeded)",
            &cap,
        ),
        step(
            "withholding",
            "TAA Sch 1 s 12-300: lesser of the regulation withholding amount and the s 12-300(b) limit",
            &final_wh,
        ),
    ];
    Ok((reg, final_wh, extra))
}

struct Sub12fInputs {
    domestic_str: &'static str,
    domestic: Decimal,
    treaty: Option<Decimal>,
    exempt: bool,
    already: Decimal,
}

fn parse_12_f_setup(payment_type: &str, payment: &Decimal, facts: &Facts) -> Result<Sub12fInputs> {
    let domestic_str = match payment_type {
        "dividend" => DIVIDEND_RATE,
        "interest" => INTEREST_RATE,
        _ => ROYALTY_RATE,
    };
    let domestic = d(domestic_str);
    let Some(treaty) = parse_optional_rate(facts.get(2), &domestic)? else {
        return refuse(
            "non_resident_withholding.treaty_rate_invalid",
            format!(
                "treaty_rate must be null or a decimal string from 0 to {domestic_str} for {payment_type}."
            ),
        );
    };
    let exempt = flag(facts.get(11), "exempt")?;
    let already = money(facts.get(12))?;
    check_already_withheld(&already, payment)?;
    Ok(Sub12fInputs {
        domestic_str,
        domestic,
        treaty,
        exempt,
        already,
    })
}

struct Sub12fAmounts {
    base: Decimal,
    used: Decimal,
    reg_amount: Decimal,
    withholding: Decimal,
}

fn finish_12_f_worked(
    payment: &Decimal,
    inputs: Sub12fInputs,
    amounts: Sub12fAmounts,
    applied_rate: &Decimal,
    mut steps: Vec<Value>,
) -> Result<Worked> {
    let net = payment.sub(&inputs.already)?.sub(&amounts.withholding)?;
    steps.push(step(
        "net_payment",
        "Payment less amounts already withheld and withholding required now",
        &net,
    ));
    let applied_str = if inputs.exempt {
        "0".to_string()
    } else {
        applied_rate.to_string()
    };
    let mut rates = vec![
        rate("domestic_rate", inputs.domestic_str),
        rate("applied_rate", applied_str),
    ];
    if let Some(tr) = inputs.treaty {
        rates.push(rate("treaty_rate", tr.to_string()));
    }
    Ok(Worked {
        subdivision: "12-F",
        final_tax: true,
        amounts: vec![
            ("withholding_base", amounts.base),
            ("payment_used", amounts.used),
            ("regulation_withholding", amounts.reg_amount),
            ("withholding", amounts.withholding),
            ("net_payment", net),
        ],
        rates,
        steps,
        components: None,
    })
}

/// Subdivision 12-F: dividend (ss 12-210, 12-215, 12-220), interest
/// (ss 12-245, 12-250, 12-255) and royalty (ss 12-280, 12-285), with
/// TAR 2017 ss 26, 39 to 42 and the s 12-300 ceiling.
pub(super) fn subdivision_12_f(
    payment_type: &str,
    payment: &Decimal,
    facts: &Facts,
) -> Result<Worked> {
    let inputs = parse_12_f_setup(payment_type, payment, facts)?;
    let mut steps = Vec::new();
    let base = compute_12_f_base(payment_type, payment, facts, &mut steps)?;
    let applied_rate = inputs
        .treaty
        .clone()
        .unwrap_or_else(|| inputs.domestic.clone());
    let used = whole_dollars(&base)?;
    steps.push(step(
        "payment_used",
        "Taxation Administration Regulations 2017 s 26(1)(a), (2) and (3): cents in the payment disregarded, and an amount under $1 disregarded",
        &used,
    ));
    let (reg_amount, withholding) = if inputs.exempt {
        let zero = int(0);
        steps.push(step(
            "withholding",
            "TAA Sch 1 s 12-300(a): no amount is required to be withheld when no withholding tax is payable",
            &zero,
        ));
        (zero.clone(), zero)
    } else {
        let (reg, final_wh, extra) =
            compute_12_f_non_exempt(payment_type, &base, &used, &applied_rate, &inputs.already)?;
        steps.extend(extra);
        (reg, final_wh)
    };
    finish_12_f_worked(
        payment,
        inputs,
        Sub12fAmounts {
            base,
            used,
            reg_amount,
            withholding,
        },
        &applied_rate,
        steps,
    )
}
