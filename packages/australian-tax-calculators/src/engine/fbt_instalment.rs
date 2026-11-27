// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `fbt_instalment`: one quarter's FBT instalment on the activity statement,
//! as the ATO gives it at F1 or as varied on the employer's estimate of the
//! year's tax (F2 to F4), with labels 6A and 6B. The citations are to FBTAA
//! Part VII Division 2 (ss 101 to 112B), compilation 97 (1 July 2026,
//! C2026C00332), and the ATO's 'Fringe benefits tax (FBT) instalment' page
//! (last updated 1 May 2025), both read on 22 November 2026.

use serde_json::{Value, json};

use crate::decimal::Decimal;

use super::{
    Arg, Result, Step, cents, d, derived_result, fbt_year, int, money, rate, refuse, step,
};

const KIND: &str = "fbt_instalment";

/// The optional inputs, in the order the command passes them.
pub const FBT_INSTALMENT_OPTIONAL: [&str; 5] = [
    "ato_instalment",
    "estimated_tax",
    "previous_instalments",
    "previous_credits",
    "reason_code",
];

/// FBTAA s 109, 'relevant fraction', by instalment (quarters ending 30 June,
/// 30 September, 31 December and 31 March).
const FRACTIONS: [&str; 4] = ["0.25", "0.50", "0.75", "1.00"];

/// The ATO's reason codes for a variation (label F4).
const REASONS: [(i64, &str); 4] = [
    (22, "current business structure not continuing"),
    (30, "change in fringe benefits for employees"),
    (31, "change in employees with fringe benefits"),
    (32, "fringe benefits rebate now claimed"),
];

const F1: &str = "Label F1: the ATO instalment amount as the activity statement shows it, worked out by the ATO from the notional tax amount (FBTAA ss 110(1) and 111(1)); whether instalments are payable at all is the ATO's (s 111(2))";
const COPIED: &str =
    "Label 6A: F1 copied, as the employer does not vary the instalment (FBTAA s 104)";
const SHARE: &str = "FBTAA s 110(5): the employer's estimate (label F2, s 112) is the notional tax amount, times the relevant fraction for the quarter (s 109), in cents 0.5 up";
const FORMULA: &str = "FBTAA s 111(1): notional tax amount x relevant fraction - (previous instalments - previous credits); the ATO's F3 formula";
const VARIED: &str = "FBTAA s 111(1): the instalment is the formula's amount where positive, otherwise nil; label F3, copied to 6A";
const SHARE_FORMULA: &str = "cents(estimated_tax x relevant_fraction)";
const NET_FORMULA: &str = "notional_share - (previous_instalments - previous_credits)";
const CREDIT: &str = "FBTAA s 112A(1): a negative amount from the s 111(1) formula is a credit the employer may claim, as a positive amount; label 6B";

/// The facts as the command passes them; see the README for each.
pub struct FbtInstalmentArgs<'a> {
    pub quarter: &'a Arg,
    pub year_ended: &'a Arg,
    pub scope_confirmed: &'a Arg,
    /// [`FBT_INSTALMENT_OPTIONAL`], each where given.
    pub optional: [Option<&'a Arg>; 5],
}

fn quarter(arg: &Arg) -> Result<usize> {
    match arg.int() {
        Some(Some(n)) if (1..=4).contains(&n) => Ok(usize::try_from(n).expect("1 to 4")),
        _ => refuse(
            "fbt_instalment.quarter_invalid",
            "quarter must be a JSON integer from 1 to 4: the instalment for the quarter ending 30 June (1), 30 September (2), 31 December (3) or 31 March (4) of the FBT year (FBTAA ss 103 and 109).",
        ),
    }
}

fn variation_invalid<T>(why: &str) -> Result<T> {
    refuse(
        "fbt_instalment.variation_invalid",
        format!("{why} (FBTAA ss 111(1) and 112(1); the ATO's labels F2 to F4)."),
    )
}

/// Label F4: one of the ATO's reason codes.
fn reason(arg: Option<&Arg>) -> Result<i64> {
    let codes = REASONS
        .map(|(code, why)| format!("{code} ({why})"))
        .join(", ");
    match arg.and_then(Arg::int) {
        Some(Some(code)) if REASONS.iter().any(|(c, _)| *c == code) => Ok(code),
        _ => variation_invalid(&format!(
            "A variation needs reason_code, the label F4 code: one of {codes}"
        )),
    }
}

/// Not varying: label 6A is F1.
fn unvaried(optional: &[Option<&Arg>; 5]) -> Result<Vec<Step>> {
    let Some(given) = optional[0] else {
        return refuse(
            "fbt_instalment.ato_instalment_missing",
            "Give ato_instalment, the amount at label F1, or estimated_tax (label F2) to vary it.",
        );
    };
    if optional[2..].iter().any(Option::is_some) {
        return variation_invalid(
            "previous_instalments, previous_credits and reason_code are given only with estimated_tax, to vary the instalment",
        );
    }
    let f1 = money(given)?;
    Ok(vec![
        step("ato_instalment", F1, "ato_instalment", &[&f1], f1.clone()),
        step("instalment", COPIED, "ato_instalment", &[&f1], f1.clone()),
    ])
}

/// The instalments that fell due earlier in the year and the credits claimed
/// for them (s 111(1A)); the first quarter has none.
fn previous(at: usize, paid: Option<&Arg>, credits: Option<&Arg>) -> Result<[Decimal; 2]> {
    let paid = match paid {
        Some(paid) => money(paid)?,
        None if at == 0 => int(0),
        None => {
            return variation_invalid(
                "previous_instalments is required to vary the second, third or fourth instalment: the instalments for the year that fell due before this one",
            );
        }
    };
    let credits = credits.map(money).transpose()?.unwrap_or_else(|| int(0));
    if at == 0 && !(paid.is_zero() && credits.is_zero()) {
        return variation_invalid(
            "the first instalment has no earlier instalments or credits in the year: previous_instalments and previous_credits must be nil for quarter 1",
        );
    }
    if credits.gt(&paid)? {
        return variation_invalid(
            "previous_credits exceed previous_instalments: an s 112A credit is at most the instalments paid before it, less earlier credits",
        );
    }
    Ok([paid, credits])
}

/// A variation's steps: id, authority and working.
const VARIATION_STEPS: [(&str, &str, &str); 6] = [
    ("estimated_tax", SHARE, "estimated_tax"),
    ("notional_share", SHARE, SHARE_FORMULA),
    ("instalment_formula", FORMULA, NET_FORMULA),
    ("varied_instalment", VARIED, "max(instalment_formula, 0)"),
    ("instalment", VARIED, "varied_instalment"),
    ("credit", CREDIT, "max(0 - instalment_formula, 0)"),
];

/// The s 111(1) formula's working and operands: a previous amount not given
/// is nil and shows as 0, as `derive` names only the inputs given.
fn net_working<'a>(
    share: &'a Decimal,
    previous: [&'a Decimal; 2],
    given: [bool; 2],
) -> (String, Vec<&'a Decimal>) {
    let mut net = NET_FORMULA.to_string();
    let mut operands = vec![share];
    for ((name, value), given) in ["previous_instalments", "previous_credits"]
        .into_iter()
        .zip(previous)
        .zip(given)
    {
        if given {
            operands.push(value);
        } else {
            net = net.replace(name, "0");
        }
    }
    (net, operands)
}

/// The s 111(1) formula with s 110(5), then F3 (and 6A) and the 6B credit.
fn variation_steps(
    estimate: &Decimal,
    fraction: &Decimal,
    [paid, credits]: &[Decimal; 2],
    given: [bool; 2],
) -> Result<Vec<Step>> {
    let share = cents(estimate.mul(fraction)?)?;
    let formula = share.sub(&paid.sub(credits)?)?;
    let zero = int(0);
    let f3 = if formula.gt(&zero)? {
        formula.clone()
    } else {
        zero.clone()
    };
    let credit = if formula.lt(&zero)? {
        formula.neg()?
    } else {
        zero
    };
    let (net, net_operands) = net_working(&share, [paid, credits], given);
    let operands: [Vec<&Decimal>; 6] = [
        vec![estimate],
        vec![estimate, fraction],
        net_operands,
        vec![&formula],
        vec![&f3],
        vec![&formula],
    ];
    let values = [estimate, &share, &formula, &f3, &f3, &credit];
    Ok(VARIATION_STEPS
        .iter()
        .zip(operands)
        .zip(values)
        .map(|((&(id, rule, working), operands), value)| {
            let working = if id == "instalment_formula" {
                &net
            } else {
                working
            };
            step(id, rule, working, &operands, value.clone())
        })
        .collect())
}

/// Varying on the estimate: the reason code and the steps.
fn varied(at: usize, optional: &[Option<&Arg>; 5]) -> Result<(Vec<Step>, i64)> {
    let [_, Some(estimate), paid, credits, code] = *optional else {
        unreachable!("varied is called with an estimate");
    };
    let code = reason(code)?;
    let given = [paid.is_some(), credits.is_some()];
    let previous = previous(at, paid, credits)?;
    let estimate = money(estimate)?;
    let steps = variation_steps(&estimate, &d(FRACTIONS[at]), &previous, given)?;
    Ok((steps, code))
}

/// The labels on the activity statement, from the steps.
fn labels(out: &Value, code: Option<i64>) -> Value {
    let amount = |key: &str| out["amounts"][key].clone();
    match code {
        None => json!({"F1": amount("ato_instalment"), "6A": amount("instalment")}),
        Some(code) => json!({
            "F2": amount("estimated_tax"),
            "F3": amount("varied_instalment"),
            "F4": code,
            "6A": amount("instalment"),
            "6B": amount("credit"),
        }),
    }
}

pub fn fbt_instalment(args: &FbtInstalmentArgs) -> Result<Value> {
    let (_, period, _) = fbt_year(args.scope_confirmed, args.year_ended, KIND)?;
    let at = quarter(args.quarter)? - 1;
    let (steps, code) = match args.optional[1] {
        None => (unvaried(&args.optional)?, None),
        Some(_) => {
            let (steps, code) = varied(at, &args.optional)?;
            (steps, Some(code))
        }
    };
    let rates = vec![rate("relevant_fraction", FRACTIONS[at])];
    let mut out = derived_result(KIND, &period, steps, rates)?;
    out["activity_statement_labels"] = labels(&out, code);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::varied;

    #[test]
    #[should_panic(expected = "varied is called with an estimate")]
    fn varying_needs_an_estimate() {
        let _ = varied(0, &[None; 5]);
    }
}
