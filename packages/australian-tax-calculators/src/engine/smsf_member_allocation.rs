// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `smsf_member_allocation`: a complying SMSF's net earnings and income tax for
//! one year shared across its member accounts, with each pension account's
//! minimum payment checked, where `smsf_tax` works the fund's tax alone.
//!
//! SISR reg 5.03(2) (compilation of 1 July 2026, read on 26 November 2026)
//! requires only that the trustee credit or debit investment returns in a way
//! that is fair and reasonable between members and between their kinds of
//! benefit; it sets no method. The worksheet applies one method the trustee
//! states, daily weighted balances. Contributions tax is 15% of each account's
//! assessable contributions (Income Tax Rates Act 1986 s 26(1)(a)); the rest of
//! the fund's stated tax falls on the accounts whose earnings are not exempt
//! current pension income (ITAA 1997 ss 295-385, 295-390 and 307-80(3)). The
//! minimum reuses `pension_minimum`'s SISR Schedule 7 steps, and the ATO's
//! 'Income stream (pension) rules and payments' (last updated 30 April 2026)
//! and 'Exception to minimum pension payment requirements' (5 November 2025)
//! give what follows a shortfall.

use serde_json::{Value, json};

use super::{
    Arg, Result, d, int, minimum_steps, money, pension_factor, precise, rate, refuse, result,
    scope, signed_money, ttr_maximum,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "smsf_member_allocation";
const MOST_ACCOUNTS: usize = 30;
const MOST_TRANSACTIONS: usize = 1000;
/// Income Tax Rates Act 1986 s 26(1)(a): the low tax component of a complying
/// superannuation fund, which assessable contributions form part of.
const CONTRIBUTIONS_TAX_RATE: &str = "0.15";
const METHOD: &str = "daily_weighted_balance";

/// The worksheet's inputs, as the caller gave them.
pub struct SmsfMemberAllocationArgs<'a> {
    pub accounts: &'a Arg,
    pub net_earnings: &'a Arg,
    pub income_tax: &'a Arg,
    pub allocation_method: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Accumulation,
    Pension,
    TransitionToRetirement,
}

/// Each transaction type and whether it adds to the balance. Contributions
/// and roll-overs cannot add to a pension's capital once it has started (the
/// ATO's minimum pension standards); a pension payment is paid from a pension.
const TYPES: [(&str, bool); 6] = [
    ("concessional_contribution", true),
    ("non_concessional_contribution", true),
    ("rollover_in", true),
    ("pension_payment", false),
    ("lump_sum", false),
    ("rollover_out", false),
];

struct Account {
    label: String,
    kind: Kind,
    opening: Decimal,
    age: Option<i64>,
    exception: bool,
    /// (day, type index, amount), in date order.
    moves: Vec<(i64, usize, Decimal)>,
}

/// The year's first and last days.
fn year_days(year: &str) -> (i64, i64) {
    let first: i64 = year[..4].parse().expect("a supported year");
    let day = |y: i64, md: &str| dates::parse(&format!("{y}-{md}")).expect("a date");
    (day(first, "07-01"), day(first + 1, "06-30"))
}

const KIND_NAMES: [(Kind, &str); 3] = [
    (Kind::Accumulation, "accumulation"),
    (Kind::Pension, "account_based_pension"),
    (Kind::TransitionToRetirement, "transition_to_retirement"),
];

fn kind_of(text: Option<&str>) -> Option<Kind> {
    let text = text?;
    KIND_NAMES.iter().find(|k| k.1 == text).map(|k| k.0)
}

fn kind_name(kind: Kind) -> &'static str {
    KIND_NAMES.iter().find(|k| k.0 == kind).expect("a kind").1
}

fn amount_of(v: Option<&Value>) -> Option<Decimal> {
    let t = v?.as_str()?;
    money(&Arg::Decimal(Decimal::parse_input(t).ok()?)).ok()
}

/// One transaction: a date in the year, a known type and an amount above nil.
fn transaction(v: &Value, span: (i64, i64), at: &str) -> Result<(i64, usize, Decimal)> {
    let o = v.as_object();
    let field = |k: &str| o.and_then(|o| o.get(k));
    let date = field("date").and_then(Value::as_str).and_then(dates::parse);
    let ty = field("type")
        .and_then(Value::as_str)
        .and_then(|t| TYPES.iter().position(|(name, ..)| *name == t));
    let amount = amount_of(field("amount")).filter(|a| !a.is_zero());
    let known = o.is_some_and(|o| {
        o.keys()
            .all(|k| matches!(k.as_str(), "date" | "type" | "amount"))
    });
    match (date, ty, amount) {
        (Some(day), Some(t), Some(a)) if known && span.0 <= day && day <= span.1 => Ok((day, t, a)),
        _ => refuse(
            "smsf_member_allocation.transaction_invalid",
            format!(
                "{at}: needs a date in the year, a type ({}) and an amount above nil to 1000000000000.00, at most 2 dp.",
                TYPES.map(|t| t.0).join(", ")
            ),
        ),
    }
}

fn transactions(
    v: Option<&Value>,
    span: (i64, i64),
    at: &str,
) -> Result<Vec<(i64, usize, Decimal)>> {
    let items = match v {
        None => return Ok(Vec::new()),
        Some(Value::Array(items)) if items.len() <= MOST_TRANSACTIONS => items,
        Some(_) => {
            return refuse(
                "smsf_member_allocation.transaction_invalid",
                format!(
                    "{at}: transactions must be a list of at most {MOST_TRANSACTIONS} objects."
                ),
            );
        }
    };
    let mut moves = items
        .iter()
        .enumerate()
        .map(|(i, t)| transaction(t, span, &format!("{at}, transaction {}", i + 1)))
        .collect::<Result<Vec<_>>>()?;
    moves.sort_by_key(|m| m.0);
    Ok(moves)
}

/// The age on 1 July and the exception flag, given on a pension account only.
fn pension_facts(o: &serde_json::Map<String, Value>, kind: Kind) -> Option<(Option<i64>, bool)> {
    let age = o
        .get("age")
        .map(|a| a.as_i64().filter(|n| (0..=150).contains(n)));
    let exception = o.get("minimum_exception").map(Value::as_bool);
    match (kind, age, exception) {
        (Kind::Accumulation, None, None) => Some((None, false)),
        (Kind::Accumulation, ..) => None,
        (_, Some(Some(n)), None) => Some((Some(n), false)),
        (_, Some(Some(n)), Some(Some(e))) => Some((Some(n), e)),
        _ => None,
    }
}

fn account(v: &Value, span: (i64, i64), n: usize) -> Result<Account> {
    let at = format!("account {n}");
    let bad = || {
        refuse(
            "smsf_member_allocation.account_invalid",
            format!(
                "{at}: needs a label, a kind (accumulation, account_based_pension or transition_to_retirement), an opening_balance from 0 and, on a pension only, an age on 1 July from 0 to 150 and an optional minimum_exception boolean."
            ),
        )
    };
    let Some(o) = v.as_object() else { return bad() };
    let label = o
        .get("label")
        .and_then(Value::as_str)
        .filter(|l| !l.trim().is_empty());
    let kind = kind_of(o.get("kind").and_then(Value::as_str));
    let opening = amount_of(o.get("opening_balance"));
    let known = o.keys().all(|k| {
        matches!(
            k.as_str(),
            "label" | "kind" | "opening_balance" | "age" | "minimum_exception" | "transactions"
        )
    });
    let (Some(label), Some(kind), Some(opening), true) = (label, kind, opening, known) else {
        return bad();
    };
    let Some((age, exception)) = pension_facts(o, kind) else {
        return bad();
    };
    // ITAA 1997 s 307-80(2)(c)(iv) and (3)(b): attaining 65 puts a transition
    // to retirement income stream in the retirement phase; at 64 on 1 July
    // that is almost always within the year.
    if kind == Kind::TransitionToRetirement && age.is_some_and(|n| n >= 64) {
        return refuse(
            "smsf_member_allocation.transition_reaches_65",
            format!(
                "{at}: a transition to retirement pension whose recipient is 64 or older on 1 July may reach 65 by the end of the year, when it enters the retirement phase (ITAA 1997 s 307-80(3)(b)) and its 10% maximum ends; state it as account_based_pension if 65 or older on 1 July, or allocate the year by hand."
            ),
        );
    }
    let moves = transactions(o.get("transactions"), span, &at)?;
    Ok(Account {
        label: label.to_owned(),
        kind,
        opening,
        age,
        exception,
        moves,
    })
}

fn accounts(given: &Arg, span: (i64, i64)) -> Result<Vec<Account>> {
    match given {
        Arg::Json(Value::Array(items)) if (1..=MOST_ACCOUNTS).contains(&items.len()) => items
            .iter()
            .enumerate()
            .map(|(i, v)| account(v, span, i + 1))
            .collect(),
        _ => refuse(
            "smsf_member_allocation.account_invalid",
            format!("accounts must be a list of 1 to {MOST_ACCOUNTS} objects."),
        ),
    }
}

/// Whether transaction type `t` adds to the balance, refused where the
/// account's kind does not take it.
fn allowed(a: &Account, t: usize) -> Result<bool> {
    let (name, adds) = TYPES[t];
    let pension = a.kind != Kind::Accumulation;
    if (adds && pension) || (name == "pension_payment" && !pension) {
        return refuse(
            "smsf_member_allocation.transaction_not_allowed",
            format!(
                "{}: {name} is not allowed on a {} account: contributions and roll-overs cannot add to a pension's capital once it has started, and a pension payment is paid from a pension.",
                a.label,
                if pension { "pension" } else { "accumulation" }
            ),
        );
    }
    Ok(adds)
}

/// Each transaction's type is allowed on the account's kind, and the balance
/// never falls below nil, or for a pension to nil, before earnings and tax.
fn check_moves(a: &Account) -> Result<()> {
    let pension = a.kind != Kind::Accumulation;
    let ceased = || {
        refuse(
            "smsf_member_allocation.pension_not_whole_year",
            format!(
                "{}: a pension must be held on 1 July with a balance above nil and not be exhausted or fully commuted in the year; for a pension that starts or ends in the year use pension-minimum with the commencement day's balance.",
                a.label
            ),
        )
    };
    if pension && a.opening.is_zero() {
        return ceased();
    }
    let mut balance = a.opening.clone();
    for (i, (day, t, amount)) in a.moves.iter().enumerate() {
        balance = moved(&balance, allowed(a, *t)?, amount)?;
        let last_today = a.moves.get(i + 1).is_none_or(|next| next.0 != *day);
        if last_today && balance.lt(&int(0))? {
            return refuse(
                "smsf_member_allocation.balance_negative",
                format!(
                    "{}: withdrawals take the balance below nil on {}.",
                    a.label,
                    dates::format(*day)
                ),
            );
        }
        if last_today && pension && balance.is_zero() {
            return ceased();
        }
    }
    Ok(())
}

/// The opening balance and each transaction weighted by the days from its
/// date to 30 June, both included, over the days in the year.
fn weighted(a: &Account, span: (i64, i64)) -> Result<Decimal> {
    let length = int(span.1 - span.0 + 1);
    precise(|| {
        let mut sum = a.opening.mul(&length)?;
        for (day, t, amount) in &a.moves {
            sum = moved(&sum, TYPES[*t].1, &amount.mul(&int(span.1 - day + 1))?)?;
        }
        Ok(sum.div(&length)?)
    })
}

/// `total` shared by `weights`, each share rounded half up to the cent
/// cumulatively so that the shares add to `total`.
fn shares(total: &Decimal, weights: &[Decimal]) -> Result<Vec<Decimal>> {
    precise(|| {
        let mut whole = int(0);
        for w in weights {
            whole = whole.add(w)?;
        }
        let (mut running, mut before) = (int(0), d("0.00"));
        let mut out = Vec::with_capacity(weights.len());
        for w in weights {
            running = running.add(w)?;
            let upto = if whole.is_zero() {
                d("0.00")
            } else {
                total
                    .mul(&running)?
                    .div(&whole)?
                    .quantize(-2, Rounding::HalfUp)?
            };
            out.push(upto.sub(&before)?);
            before = upto;
        }
        Ok(out)
    })
}

/// `balance` moved up by `by` when `adds`, else down.
fn moved(balance: &Decimal, adds: bool, by: &Decimal) -> Result<Decimal> {
    Ok(if adds {
        balance.add(by)?
    } else {
        balance.sub(by)?
    })
}

/// The total of one transaction type on an account.
fn sum_of(a: &Account, name: &str) -> Result<Decimal> {
    let mut sum = d("0.00");
    for (_, t, amount) in &a.moves {
        if TYPES[*t].0 == name {
            sum = sum.add(amount)?;
        }
    }
    Ok(sum)
}

/// A pension's minimum (and a transition to retirement pension's maximum)
/// against the payments made, and whether its earnings stay exempt.
fn pension_check(a: &Account, days_in_year: i64) -> Result<(Value, bool)> {
    let ttr = a.kind == Kind::TransitionToRetirement;
    let factor = pension_factor(a.age.expect("a pension's age"));
    let steps = minimum_steps(&a.opening, &factor, days_in_year, days_in_year)?;
    let steps = ttr_maximum(steps, &a.opening, ttr)?;
    let value_of = |id: &str| steps.iter().find(|s| s.id == id).map(|s| s.value.clone());
    let minimum = value_of("minimum_payment").expect("the minimum");
    let maximum = value_of("maximum_payment");
    let paid = sum_of(a, "pension_payment")?;
    let (shortfall, twelfth, minimum, maximum) = precise(|| {
        let short = if paid.lt(&minimum)? {
            minimum.sub(&paid)?
        } else {
            d("0.00")
        };
        let cents = |v: &Decimal| v.quantize(-2, Rounding::HalfUp);
        Ok((
            short,
            cents(&minimum.div(&int(12))?)?,
            cents(&minimum)?,
            maximum.as_ref().map(cents).transpose()?,
        ))
    })?;
    let met = shortfall.is_zero();
    let over = match &maximum {
        Some(m) => paid.gt(m)?,
        None => false,
    };
    let exempt = a.kind == Kind::Pension && (met || a.exception);
    let mut check = json!({
        "account_balance_used": a.opening.to_string(),
        "percentage_factor": factor.to_string(),
        "minimum_payment": minimum.to_string(),
        "pension_payments": paid.to_string(),
        "shortfall": shortfall.to_string(),
        "one_twelfth_of_minimum": twelfth.to_string(),
        "minimum_met": met,
        "minimum_exception_stated": a.exception,
    });
    if let Some(m) = maximum {
        check["maximum_payment"] = json!(m.to_string());
        check["maximum_exceeded"] = json!(over);
    }
    Ok((check, exempt))
}

const NOT_MET: &str = "did not meet the pension standards for the year: under the ATO's 'Income stream (pension) rules and payments' the income stream is taken to have ceased at the start of the year for income tax purposes, its payments are not super income stream benefits, the fund cannot claim exempt current pension income on it for this or later years and there are transfer balance account consequences, so this worksheet charges it earnings tax. For an account-based pension the Commissioner's exception may apply (an honest mistake giving an underpayment of no more than one-twelfth of the minimum, or matters outside the trustee's control, with a catch-up payment): state minimum_exception only once the trustee has self-assessed it or the ATO has allowed it.";

/// Every account's movements, weighted balance and earnings exemption.
#[derive(Default)]
struct Worked {
    weights: Vec<Decimal>,
    taxed: Vec<bool>,
    checks: Vec<Option<Value>>,
    warnings: Vec<String>,
}

fn work(all: &[Account], span: (i64, i64)) -> Result<Worked> {
    let days_in_year = span.1 - span.0 + 1;
    let mut w = Worked::default();
    for a in all {
        check_moves(a)?;
        w.weights.push(weighted(a, span)?);
        if a.kind == Kind::Accumulation {
            w.taxed.push(true);
            w.checks.push(None);
            continue;
        }
        let (check, exempt) = pension_check(a, days_in_year)?;
        let failed =
            check["minimum_met"] == json!(false) || check["maximum_exceeded"] == json!(true);
        if failed {
            w.warnings.push(format!("{} {NOT_MET}", a.label));
        }
        // Only a transition to retirement pension has a maximum, and its
        // earnings are taxed in any case (ITAA 1997 s 307-80(3)).
        w.taxed.push(!exempt);
        w.checks.push(Some(check));
    }
    Ok(w)
}

/// The fund's tax less contributions tax, shared by the taxed accounts'
/// weighted balances.
fn earnings_tax(tax: &Decimal, w: &Worked) -> Result<Vec<Decimal>> {
    let taxed: Vec<Decimal> = w
        .weights
        .iter()
        .zip(&w.taxed)
        .map(|(x, t)| if *t { x.clone() } else { int(0) })
        .collect();
    let any = taxed.iter().any(|x| !x.is_zero());
    let exempt_held = w
        .weights
        .iter()
        .zip(&w.taxed)
        .any(|(x, t)| !t && !x.is_zero());
    if !tax.is_zero() && !any {
        return refuse(
            "smsf_member_allocation.tax_unallocable",
            "The fund's income tax less contributions tax is not nil but no account whose earnings are taxed held a balance in the year: allocate it by hand.",
        );
    }
    if tax.lt(&int(0))? && exempt_held {
        return refuse(
            "smsf_member_allocation.refund_with_retirement_phase",
            "The fund's income tax less contributions tax is below nil (offsets such as franking credits exceed it) whilst a retirement phase pension is held: whether the benefit belongs to pension or accumulation accounts is the trustee's fair and reasonable determination (SISR reg 5.03(2)); allocate it by hand.",
        );
    }
    shares(tax, &taxed)
}

fn check_method(given: &Arg) -> Result<()> {
    if given.text() == Some(METHOD) {
        return Ok(());
    }
    refuse(
        "smsf_member_allocation.method_unsupported",
        format!(
            "allocation_method must be {METHOD}, the trustee's stated method under SISR reg 5.03(2); this worksheet works no other."
        ),
    )
}

/// 15% of each account's concessional contributions, to the cent.
fn contributions_tax(all: &[Account]) -> Result<Vec<Decimal>> {
    let rate = d(CONTRIBUTIONS_TAX_RATE);
    all.iter()
        .map(|acc| {
            precise(|| {
                Ok(sum_of(acc, "concessional_contribution")?
                    .mul(&rate)?
                    .quantize(-2, Rounding::HalfUp)?)
            })
        })
        .collect()
}

/// The fund's net earnings and income tax shared across its member accounts,
/// with each pension account's minimum checked.
pub fn smsf_member_allocation(a: &SmsfMemberAllocationArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    check_method(a.allocation_method)?;
    let span = year_days(&year);
    let all = accounts(a.accounts, span)?;
    let earnings = signed_money(a.net_earnings)?;
    let tax = signed_money(a.income_tax)?;
    let w = work(&all, span)?;
    if !earnings.is_zero() && w.weights.iter().all(Decimal::is_zero) {
        return refuse(
            "smsf_member_allocation.no_balance",
            "net_earnings is not nil but no account held a balance in the year.",
        );
    }
    let earned = shares(&earnings, &w.weights)?;
    let ct = contributions_tax(&all)?;
    let ct_total = total(&ct)?;
    let rest = precise(|| Ok(tax.sub(&ct_total)?))?;
    let et = earnings_tax(&rest, &w)?;
    let (rows, closing): (Vec<Value>, Vec<Decimal>) = all
        .iter()
        .enumerate()
        .map(|(i, acc)| row(acc, &w, i, [&earned[i], &ct[i], &et[i]]))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .unzip();
    let opening: Vec<Decimal> = all.iter().map(|x| x.opening.clone()).collect();
    let amounts = vec![
        ("net_earnings", earnings),
        ("income_tax", tax),
        ("contributions_tax", ct_total),
        ("earnings_tax", rest),
        ("opening_balances", total(&opening)?),
        ("closing_balances", total(&closing)?),
    ];
    finish(&year, amounts, span, rows, w.warnings)
}

/// The result with the rates, each account's row and any pension warnings.
fn finish(
    year: &str,
    amounts: Vec<(&str, Decimal)>,
    span: (i64, i64),
    rows: Vec<Value>,
    extra: Vec<String>,
) -> Result<Value> {
    let rates = vec![
        rate("contributions_tax_rate", CONTRIBUTIONS_TAX_RATE),
        rate("days_in_year", span.1 - span.0 + 1),
    ];
    let mut out = result(KIND, year, amounts, rates)?;
    out["accounts"] = Value::Array(rows);
    let warnings = out["warnings"].as_array_mut().expect("warnings");
    warnings.extend(extra.into_iter().map(Value::from));
    Ok(out)
}

fn total(values: &[Decimal]) -> Result<Decimal> {
    Ok(values.iter().try_fold(d("0.00"), |sum, v| sum.add(v))?)
}

/// One account's movements, shares and closing balance: its earnings,
/// contributions tax and earnings tax in `parts`. A closing balance below nil
/// is refused, as before earnings and tax.
fn row(acc: &Account, w: &Worked, i: usize, parts: [&Decimal; 3]) -> Result<(Value, Decimal)> {
    let [earned, ct, et] = parts;
    let mut closing = acc.opening.clone();
    let mut row = json!({"label": acc.label, "kind": kind_name(acc.kind),
        "opening_balance": acc.opening.to_string()});
    for (name, adds) in TYPES {
        let s = sum_of(acc, name)?;
        closing = moved(&closing, adds, &s)?;
        row[name] = json!(s.to_string());
    }
    let closing = precise(|| Ok(closing.add(earned)?.sub(ct)?.sub(et)?))?;
    if closing.lt(&int(0))? {
        return refuse(
            "smsf_member_allocation.balance_negative",
            format!(
                "{}: its share of losses and tax takes the closing balance below nil.",
                acc.label
            ),
        );
    }
    let weighted = precise(|| Ok(w.weights[i].quantize(-2, Rounding::HalfUp)?))?;
    row["weighted_balance"] = json!(weighted.to_string());
    row["earnings"] = json!(earned.to_string());
    row["contributions_tax"] = json!(ct.to_string());
    row["earnings_taxed"] = json!(w.taxed[i]);
    row["earnings_tax"] = json!(et.to_string());
    row["closing_balance"] = json!(closing.to_string());
    if let Some(check) = &w.checks[i] {
        row["pension"] = check.clone();
    }
    Ok((row, closing))
}
