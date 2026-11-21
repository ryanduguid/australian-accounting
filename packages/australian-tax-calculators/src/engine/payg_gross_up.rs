// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `payg_gross_up`: the gross pay whose Schedule 1 withholding leaves a stated
//! net pay, found by searching `payg_withholding`'s own results, or
//! `study_loan_withholding`'s for a payee with a study and training support
//! loan.
//!
//! Schedule 1 withholds whole dollars and never withholds less from a higher
//! gross (`tests/properties.rs`), so a gross leaves `net` exactly when it is
//! `net + w` for a whole-dollar `w` that `payg_withholding` withholds from
//! `net + w`: a fixed point of `h(w) = withholding(net + w)`. `h` never falls,
//! so repeating it from 0 rises to the least fixed point and repeating it from
//! a bound above every fixed point falls to the greatest; any others lie
//! between. Every coefficient `a` is at most 0.47 and every `b` at least -1
//! (`tests/payg_gross_up.rs`), so a week's amount is at most `0.47 x + 1.5`
//! and the withholding on any gross `g` at most `g / 2 + 26` dollars, the 26
//! being a quarter's 13 weeks of that $1.50 and of 47% of the 99 cents added
//! to weekly earnings; so every fixed point is at most `net + 52`.
//!
//! With a study loan, Schedule 8's component is added, rounded on its own.
//! Every component `a` is at most 0.17 and every `b` at least 0
//! (`tests/payg_gross_up.rs`), so a week's component is at most `0.17 x + 0.5`,
//! a week's total at most `0.64 x + 2`, and the withholding on a gross `g` at
//! most `0.64 g + 35`, the 35 covering a quarter's 13 weeks of that $2 and of
//! 64% of the 99 cents. A fixed point `w` then has `w <= 0.64 (net + w) + 35`,
//! so `w <= (0.64 net + 35) / 0.36`, below `(0.64 (m + 1) + 35) / 0.36`,
//! which is `16 m / 9 + 99`, for `m` the whole dollars of `net`; a
//! whole-dollar `w` is then at most `2 m + 98`. Withholding with the component also never falls
//! as pay rises (`tests/properties.rs`).

use serde_json::Value;

use super::{
    Arg, PaygOptions, Result, StudyLoan, d, int, money, payg_withholding, precise, refuse, result,
    scope, study_loan_flag, study_loan_withholding,
};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "payg_gross_up";

/// The largest net searched: the search tries grosses up to twice the net
/// plus $52, which must stay within the $1,000,000,000,000 an amount may be.
const MOST_NET: &str = "499999999974";

/// The largest net searched with a study loan: grosses up to 3 times the net
/// plus $98.
const MOST_NET_WITH_STUDY_LOAN: &str = "333333333300";

/// One pay's withholding by `payg_withholding`, on the caller's pay period,
/// scale, year and residency, or by `study_loan_withholding` with the pay date
/// for a payee with a study loan.
struct Pay<'a> {
    net: Decimal,
    pay_period: &'a Arg,
    scale: &'a Arg,
    year: &'a Arg,
    foreign_resident: Option<&'a Arg>,
    /// `Some(pay_date)` for a payee with a study loan.
    study_loan: Option<Option<&'a Arg>>,
}

impl Pay<'_> {
    /// The withholding worksheet's result for the gross `net + w`.
    fn at(&self, w: &Decimal) -> Result<Value> {
        let gross = Arg::Decimal(precise(|| Ok(self.net.add(w)?))?);
        let confirmed = Arg::Bool(true);
        if let Some(pay_date) = self.study_loan {
            return study_loan_withholding(
                &gross,
                self.pay_period,
                self.scale,
                self.year,
                &confirmed,
                pay_date,
            );
        }
        let options = PaygOptions {
            foreign_resident: self.foreign_resident,
            ..PaygOptions::default()
        };
        payg_withholding(
            &gross,
            self.pay_period,
            self.scale,
            self.year,
            &confirmed,
            &options,
        )
    }

    /// `h(w)`: the whole dollars withheld from `net + w`.
    fn h(&self, w: &Decimal) -> Result<Decimal> {
        Ok(d(self.at(w)?["amounts"]["withholding"]
            .as_str()
            .expect("withholding amount")))
    }

    /// The fixed point of `h` that repeating it from `w` reaches: the least
    /// at or above `w` when `h(w) >= w`, the greatest at or below it otherwise.
    fn settle(&self, mut w: Decimal) -> Result<Decimal> {
        loop {
            let next = self.h(&w)?;
            if next.equals(&w)? {
                return Ok(w);
            }
            w = next;
        }
    }

    /// Every whole-dollar `w` from the least fixed point to the greatest with
    /// `h(w) = w`, lowest first.
    fn fixed_points(&self) -> Result<Vec<Decimal>> {
        let least = self.settle(int(0))?;
        let (times, plus) = if self.study_loan.is_some() {
            (2, 98)
        } else {
            (1, 52)
        };
        let bound = precise(|| {
            let whole = self.net.quantize(0, Rounding::Down)?;
            Ok(whole.mul(&int(times))?.add(&int(plus))?)
        })?;
        let greatest = self.settle(bound)?;
        let mut found = vec![least.clone()];
        let mut w = precise(|| Ok(least.add(&int(1))?))?;
        while w.le(&greatest)? {
            if self.h(&w)?.equals(&w)? {
                found.push(w.clone());
            }
            w = precise(|| Ok(w.add(&int(1))?))?;
        }
        Ok(found)
    }
}

/// `a`, `b` and `c` as Australian prose lists them.
fn listed(items: &[String]) -> String {
    match items.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => items.concat(),
    }
}

pub fn payg_gross_up(
    net_pay: &Arg,
    pay_period: &Arg,
    scale: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    foreign_resident: Option<&Arg>,
    study_loan: StudyLoan,
) -> Result<Value> {
    let year_text = scope(scope_confirmed, year, KIND)?;
    let pay_date = study_loan.1;
    let study_loan = study_loan_flag(KIND, &year_text, study_loan)?;
    if study_loan && foreign_resident.is_some() {
        return refuse(
            "payg.foreign_resident_not_scale_4",
            "foreign_resident applies to scale 4 only, which has no study loan component: Schedule 8's component does not apply to a payee who gave no TFN.",
        );
    }
    let net = money(net_pay)?;
    let (most, worksheet) = if study_loan {
        (MOST_NET_WITH_STUDY_LOAN, "study-loan-withholding")
    } else {
        (MOST_NET, "payg-withholding")
    };
    if net.gt(&d(most))? {
        return refuse(
            "payg.gross_up_net_too_large",
            format!(
                "net_pay must be at most ${most}.00: a larger net could need a gross beyond the $1000000000000.00 {worksheet} takes."
            ),
        );
    }
    let pay = Pay {
        net: net.clone(),
        pay_period,
        scale,
        year,
        foreign_resident,
        study_loan: study_loan.then_some(pay_date),
    };
    let withheld = pay.fixed_points()?;
    let answer = pay.at(&withheld[0])?;
    let period = answer["period"].as_str().expect("period");
    if withheld.len() > 1 {
        return not_unique(&net, &withheld, period, &answer);
    }
    let gross = precise(|| Ok(net.add(&withheld[0])?))?;
    let mut amounts = vec![("net_pay", net), ("gross_pay", gross)];
    for (key, value) in answer["amounts"].as_object().expect("amounts") {
        amounts.push((key.as_str(), d(value.as_str().expect("amount"))));
    }
    let rates = answer["rates"]
        .as_object()
        .expect("rates")
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().expect("rate").to_string()))
        .collect();
    result(KIND, period, amounts, rates)
}

/// The refusal for a net that more than one gross leaves, listing each gross.
fn not_unique(net: &Decimal, withheld: &[Decimal], period: &str, answer: &Value) -> Result<Value> {
    let grosses: Vec<String> = precise(|| {
        withheld
            .iter()
            .map(|w| Ok(format!("${}", net.add(w)?.fixed(2, false))))
            .collect()
    })?;
    refuse(
        "payg.gross_up_not_unique",
        format!(
            "{} gross pays leave a net of ${} for a {period} pay on scale {}: {}. Schedule 1 withholds whole dollars, so a gross that withholds a dollar more for each dollar more of pay leaves the same net. Take the gross from the pay agreement.",
            withheld.len(),
            net.fixed(2, false),
            answer["rates"]["scale"].as_str().expect("scale"),
            listed(&grosses)
        ),
    )
}
