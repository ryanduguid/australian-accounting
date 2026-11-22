// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `payg_instalment`: one quarterly PAYG instalment under Division 45 in
//! Schedule 1 to the Taxation Administration Act 1953, by the 4 methods of
//! the activity statement: the notified amount (s 45-112), an amount varied
//! from estimated benchmark tax (s 45-410, with the s 45-420 credit),
//! instalment income times the rate (s 45-110), and a varied rate (s 45-205,
//! with the s 45-215 credit). The citations are to compilation 227 (1 October
//! 2026), read on 22 November 2026; Division 45 reads the same for these
//! sections in compilation 219 (1 July 2025). The steps and rounding follow the
//! ATO's 'How to vary your PAYG instalments' and 'PAYG instalments: how to
//! complete your activity statement' pages, read the same day.

use serde_json::{Value, json};

use super::{Arg, Result, int, money, precise, rate, refuse, result, scope};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "payg_instalment";

/// The facts each method takes beside `method`, `year` and
/// `scope_confirmed`, in [`PaygInstalmentArgs::optional`].
pub const PAYG_INSTALMENT_OPTIONAL: [&str; 9] = [
    "notified_amount",
    "instalment_number",
    "estimated_tax",
    "earlier_instalments",
    "credits_claimed",
    "instalment_income",
    "instalment_rate",
    "estimated_instalment_income",
    "earlier_instalment_income",
];

/// The facts as the command passes them; see the README for each.
pub struct PaygInstalmentArgs<'a> {
    pub method: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 9],
}

/// The indices into [`PAYG_INSTALMENT_OPTIONAL`] each method takes.
fn fields(method: &str) -> Option<&'static [usize]> {
    Some(match method {
        "amount" => &[0],
        "varied_amount" => &[1, 2, 3, 4],
        "rate" => &[5, 6],
        "varied_rate" => &[2, 3, 4, 5, 6, 7, 8],
        _ => return None,
    })
}

/// Each of the method's facts given, and no other.
fn checked<'a>(args: &PaygInstalmentArgs<'a>, wanted: &[usize]) -> Result<[Option<&'a Arg>; 9]> {
    for (i, (key, given)) in PAYG_INSTALMENT_OPTIONAL
        .iter()
        .zip(args.optional)
        .enumerate()
    {
        match (wanted.contains(&i), given) {
            (true, None) => {
                return refuse(
                    "input.missing_field",
                    format!("{key} is required for this method."),
                );
            }
            (false, Some(_)) => {
                return refuse(
                    "input.field_unknown",
                    format!("{key} is not an input for this method"),
                );
            }
            _ => {}
        }
    }
    Ok(args.optional)
}

/// The method's facts, each present once [`checked`] has passed.
struct Facts<'a>([Option<&'a Arg>; 9]);

impl<'a> Facts<'a> {
    fn get(&self, i: usize) -> &'a Arg {
        self.0[i].expect("checked above")
    }
}

/// One worked instalment: the amounts, the rates and the cited steps.
struct Worked {
    amounts: Vec<(&'static str, Decimal)>,
    rates: Vec<(String, String)>,
    steps: Vec<Value>,
}

fn step(id: &str, authority: &str, value: &Decimal) -> Value {
    json!({"step": id, "authority": authority, "value": value.to_string()})
}

pub fn payg_instalment(args: &PaygInstalmentArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let method = args.method.text().unwrap_or_default();
    let Some(wanted) = fields(method) else {
        return refuse(
            "payg_instalment.method_invalid",
            "method must be amount, varied_amount, rate or varied_rate.",
        );
    };
    let facts = Facts(checked(args, wanted)?);
    let worked = precise(|| match method {
        "amount" => notified(&money(facts.get(0))?),
        "varied_amount" => varied_amount(&facts),
        _ => by_rate(method == "varied_rate", &facts),
    })?;
    let mut out = result(KIND, &year, worked.amounts, worked.rates)?;
    out["method"] = json!(method);
    out["steps"] = json!(worked.steps);
    Ok(out)
}

/// Cents dropped, as the activity statement takes them; never negative here.
fn whole(amount: &Decimal) -> Result<Decimal> {
    Ok(amount.quantize(0, Rounding::Down)?)
}

fn cents(amount: &Decimal) -> Result<Decimal> {
    Ok(amount.quantize(-2, Rounding::HalfUp)?)
}

/// The instalment and the credit from an amount that may be negative.
fn split(amount: &Decimal) -> Result<(Decimal, Decimal)> {
    Ok(if amount.lt(&int(0))? {
        (int(0), whole(&amount.neg()?)?)
    } else {
        (whole(amount)?, int(0))
    })
}

/// Option 1: the amount the Commissioner notifies at T7 (s 45-112(1)(a), (c)).
fn notified(amount: &Decimal) -> Result<Worked> {
    let instalment = whole(amount)?;
    Ok(Worked {
        steps: vec![
            step("notified_amount", NOTIFIED_RULE, amount),
            step("instalment", WHOLE_DOLLARS_RULE, &instalment),
        ],
        amounts: vec![
            ("amount", amount.clone()),
            ("instalment", instalment),
            ("credit", int(0)),
        ],
        rates: Vec::new(),
    })
}

/// Which of the year's 4 instalments this is.
fn instalment_number(given: &Arg) -> Result<i64> {
    match given.int() {
        Some(Some(n)) if (1..=4).contains(&n) => Ok(n),
        _ => refuse(
            "payg_instalment.instalment_number_invalid",
            "instalment_number must be a JSON integer from 1 to 4.",
        ),
    }
}

/// Earlier instalments less the credits claimed against them; credits
/// claimed cannot exceed the instalments they came from.
fn net_earlier(earlier: &Decimal, credits: &Decimal) -> Result<Decimal> {
    if credits.gt(earlier)? {
        return refuse(
            "payg_instalment.credits_exceed_earlier",
            "credits_claimed cannot exceed earlier_instalments.",
        );
    }
    Ok(earlier.sub(credits)?)
}

/// Option 1 varied: s 45-410(2) to (5), 25%, 50%, 75% or 100% of the
/// estimate of benchmark tax, less earlier instalments, plus credits claimed.
fn varied_amount(given: &Facts) -> Result<Worked> {
    let number = instalment_number(given.get(1))?;
    let tax = money(given.get(2))?;
    let (earlier, credits) = (money(given.get(3))?, money(given.get(4))?);
    if number == 1 && !earlier.is_zero() {
        return refuse(
            "payg_instalment.first_has_earlier",
            "The first instalment of the year has no earlier instalments.",
        );
    }
    let net = net_earlier(&earlier, &credits)?;
    if number == 2 && !credits.is_zero() {
        return refuse(
            "payg_instalment.credits_before_third",
            "No credit can be claimed before the second instalment: the first instalment is never negative (s 45-410(2)) and s 45-215(1)(b) needs an earlier quarter.",
        );
    }
    let due = cents(&tax.mul(&int(number))?.div(&int(4))?)?;
    let amount = due.sub(&net)?;
    let (instalment, credit) = split(&amount)?;
    Ok(Worked {
        steps: vec![
            step("due_by_this_instalment", SHARE_RULE, &due),
            step("earlier_instalments_less_credits", EARLIER_RULE, &net),
            step("amount", AMOUNT_RULE, &amount),
            step("instalment", WHOLE_DOLLARS_RULE, &instalment),
            step("credit", AMOUNT_CREDIT_RULE, &credit),
        ],
        amounts: vec![
            ("due", due),
            ("amount", amount),
            ("instalment", instalment),
            ("credit", credit),
        ],
        rates: vec![rate("share_of_estimated_tax", format!("{}%", number * 25))],
    })
}

/// A rate in per cent: 0 to 100, at most 2 decimal places.
fn percent(given: &Arg) -> Result<Decimal> {
    let bad = || {
        refuse(
            "payg_instalment.rate_invalid",
            "instalment_rate must be a per cent from 0 to 100, at most 2 decimal places.",
        )
    };
    let Arg::Decimal(r) = given else {
        return bad();
    };
    if !r.is_finite() || r.lt(&int(0))? || r.gt(&int(100))? || r.exponent() < -2 {
        return bad();
    }
    Ok(r.clone())
}

/// s 45-205 as the ATO's step 4 works it: estimated tax over estimated
/// instalment income, times 100, cut to 2 decimal places as its example 2
/// does; nil income varies the rate to nil.
fn varied_rate(tax: &Decimal, income: &Decimal) -> Result<Decimal> {
    if tax.gt(income)? {
        return refuse(
            "payg_instalment.tax_exceeds_income",
            "estimated_tax cannot exceed estimated_instalment_income.",
        );
    }
    if income.is_zero() {
        return Ok(int(0));
    }
    Ok(tax
        .mul(&int(100))?
        .div(income)?
        .quantize(-2, Rounding::Down)?)
}

/// s 45-215(1): earlier instalments less credits claimed, less earlier
/// instalment income times the lower varied rate; a positive result.
fn lower_rate_credit(given: &Facts, varied: &Decimal) -> Result<(Decimal, Decimal)> {
    let net = net_earlier(&money(given.get(3))?, &money(given.get(4))?)?;
    let at_rate = cents(&money(given.get(8))?.mul(varied)?.div(&int(100))?)?;
    let left = net.sub(&at_rate)?;
    Ok((
        at_rate,
        if left.gt(&int(0))? {
            whole(&left)?
        } else {
            int(0)
        },
    ))
}

/// Option 2: instalment income at T1 times the applicable rate (s 45-110),
/// the T2 rate or a varied one with any credit for earlier quarters.
fn by_rate(varied: bool, given: &Facts) -> Result<Worked> {
    let income = money(given.get(5))?;
    let t2 = percent(given.get(6))?;
    let mut steps = Vec::new();
    let (applied, credit) = if varied {
        let rate = varied_rate(&money(given.get(2))?, &money(given.get(7))?)?;
        steps.push(step("varied_rate", VARIED_RATE_RULE, &rate));
        let credit = if rate.lt(&t2)? {
            let (at_rate, credit) = lower_rate_credit(given, &rate)?;
            steps.push(step("earlier_income_at_varied_rate", CREDIT_RULE, &at_rate));
            credit
        } else {
            int(0)
        };
        steps.push(step("credit", CREDIT_RULE, &credit));
        (rate, credit)
    } else {
        (t2.clone(), int(0))
    };
    let amount = cents(&income.mul(&applied)?.div(&int(100))?)?;
    let instalment = whole(&amount)?;
    steps.push(step("amount", RATE_RULE, &amount));
    steps.push(step("instalment", WHOLE_DOLLARS_RULE, &instalment));
    Ok(Worked {
        steps,
        amounts: vec![
            ("amount", amount),
            ("instalment", instalment),
            ("credit", credit),
        ],
        rates: vec![
            rate("instalment_rate", applied.quantize(-2, Rounding::HalfUp)?),
            rate("t2_rate", t2.quantize(-2, Rounding::HalfUp)?),
        ],
    })
}

const NOTIFIED_RULE: &str = "TAA 1953 Sch 1 s 45-112(1)(a) and (c): the instalment is the amount the Commissioner works out and notifies (T7)";

const WHOLE_DOLLARS_RULE: &str = "ATO, Complete your BAS: activity statement amounts round down to whole dollars; the ATO's T1 x T2 example enters $341.70 as 341 at T11 and 5A";

const SHARE_RULE: &str = "TAA 1953 Sch 1 s 45-410(2) to (5): 25%, 50%, 75% or 100% of the estimate of benchmark tax (T8) for the first, second, third or fourth instalment of the year for which you are liable, half up to the cent";

const EARLIER_RULE: &str = "TAA 1953 Sch 1 s 45-410(3) to (5): the earlier instalments of the year are subtracted and the s 45-420 credits claimed for them are added back";

const AMOUNT_RULE: &str = "TAA 1953 Sch 1 s 45-410(1): the amount due by this instalment less earlier instalments net of credits; nil if negative";

const AMOUNT_CREDIT_RULE: &str = "TAA 1953 Sch 1 s 45-420(1): a negative amount is a credit of that amount as a positive figure, claimed at 5B, cents dropped";

const RATE_RULE: &str = "TAA 1953 Sch 1 s 45-110: instalment income for the quarter (T1) times the applicable instalment rate, half up to the cent";

const VARIED_RATE_RULE: &str = "TAA 1953 Sch 1 s 45-205; ATO, How to vary your PAYG instalments, step 4: estimated tax / estimated instalment income x 100, cut to 2 decimal places as the ATO's example 2 gives 12.27%; nil income gives a nil rate";

const CREDIT_RULE: &str = "TAA 1953 Sch 1 s 45-215(1): where the varied rate is lower than the rate used last quarter (T2), earlier instalments less credits claimed, less earlier instalment income times the varied rate, if positive, cents dropped";
