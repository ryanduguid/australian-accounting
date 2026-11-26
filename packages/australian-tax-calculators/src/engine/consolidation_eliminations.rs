// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `consolidation-eliminations`: a parent's and its directly held
//! subsidiaries' stated trial balances combined line by line (AASB 10 para
//! B86(a)), with the elimination journals: the parent's investment against
//! each subsidiary's equity at acquisition, recognising goodwill and the
//! non-controlling interest at acquisition (para B86(b); AASB 3 paras 19 and
//! 32); the non-controlling interest's share of each subsidiary's equity since
//! (paras 22 and B94); stated intra-group balances and income and expenses,
//! eliminated in full (para B86(c)); and stated unrealised profit in closing
//! inventory, eliminated in full with its stated tax effect (para B86(c);
//! AASB 112). Profit is attributed to the owners of the parent and to the
//! non-controlling interests (para B94), a subsidiary's share reduced by its
//! own unrealised profit. AASB 10 compilation ACOMPnov24_01-26 (paras 19 to 24
//! and B86 to B99 unamended since July 2015 in its table of amendments) and
//! AASB 3 compilation no. 5 (F2023C00389), read on 26 November 2026. The
//! Standards' wording is not reproduced.

use serde_json::{Map, Value, json};

use super::{Arg, Result, d, flag, money, precise, refuse, result, scope, signed_money};
use crate::decimal::{Calc, Decimal, Rounding};

mod read;
use read::*;

mod totals;
use totals::*;

const KIND: &str = "consolidation_eliminations";
const MOST_SUBSIDIARIES: usize = 20;
const MOST_LINES: usize = 500;
const MOST_ITEMS: usize = 500;
const LONGEST_LABEL: usize = 200;
const NCI: &str = "Non-controlling interests";
const GOODWILL: &str = "Goodwill";
const CLASSES: [&str; 5] = ["asset", "liability", "equity", "income", "expense"];

/// The worksheet's inputs, as the caller gave them.
pub struct ConsolidationEliminationsArgs<'a> {
    pub parent: &'a Arg,
    pub subsidiaries: &'a Arg,
    pub intra_group_balances: &'a Arg,
    pub intra_group_transactions: &'a Arg,
    pub unrealised_profits: &'a Arg,
    pub tax_rate: &'a Arg,
    pub acquired_in_period: &'a Arg,
    pub fair_value_adjustments: &'a Arg,
    pub previously_held_interest: &'a Arg,
    pub ownership_changed: &'a Arg,
    pub intra_group_dividends: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One trial balance line: its account, class and amount on its natural side.
struct Line {
    account: String,
    class: &'static str,
    amount: Decimal,
}

/// One group entity: its name, lines and, for a subsidiary, the acquisition.
struct Entity {
    name: String,
    lines: Vec<Line>,
    acquisition: Option<Acquisition>,
}

#[derive(Clone)]
struct Acquisition {
    ownership: Decimal,
    investment_account: String,
    consideration: Decimal,
    nci: Decimal,
    equity: Vec<(String, Decimal)>,
}

/// A journal line: account, class and the change on the natural side.
struct Posting {
    account: String,
    class: &'static str,
    change: Decimal,
}

struct Journal {
    id: String,
    authority: &'static str,
    postings: Vec<Posting>,
}

fn bad<T>(code: &'static str, why: String) -> Result<T> {
    refuse(code, why)
}

/// The group's entities with the eliminations posted to each so far, so no
/// elimination takes more from an entity's line than the line holds.
struct Group {
    entities: Vec<Entity>,
    used: Vec<Vec<Decimal>>,
    journals: Vec<Journal>,
}

impl Group {
    fn entity(&self, name: &str) -> Result<usize> {
        match self.entities.iter().position(|e| e.name == name) {
            Some(i) => Ok(i),
            None => bad(
                "consolidation_eliminations.entity_unknown",
                format!("{name} is not the parent or a subsidiary given."),
            ),
        }
    }

    fn has(&self, e: usize, account: &str, class: &str) -> Result<()> {
        let entity = &self.entities[e];
        if entity
            .lines
            .iter()
            .any(|l| l.account == account && l.class == class)
        {
            return Ok(());
        }
        bad(
            "consolidation_eliminations.account_invalid",
            format!("{} has no {class} line {account}.", entity.name),
        )
    }

    /// Take `take` from an entity's line of `class` (its natural side down).
    fn take(
        &mut self,
        e: usize,
        account: &str,
        class: &'static str,
        take: &Decimal,
    ) -> Result<Posting> {
        let name = &self.entities[e].name;
        let Some(i) = self.entities[e]
            .lines
            .iter()
            .position(|l| l.account == account && l.class == class)
        else {
            return bad(
                "consolidation_eliminations.account_invalid",
                format!("{name} has no {class} line {account}."),
            );
        };
        let left = self.entities[e].lines[i].amount.sub(&self.used[e][i])?;
        let wanted = take.quantize(-2, Rounding::HalfUp)?;
        if left.lt(&wanted)? && wanted.gt(&d("0"))? {
            return bad(
                "consolidation_eliminations.exceeds_line",
                format!("{name}'s {account} holds {left} left to eliminate, less than {wanted}."),
            );
        }
        self.used[e][i] = self.used[e][i].add(&wanted)?;
        Ok(Posting {
            account: account.to_string(),
            class,
            change: wanted.neg()?,
        })
    }

    fn post(&mut self, id: String, authority: &'static str, postings: Vec<Posting>) {
        self.journals.push(Journal {
            id,
            authority,
            postings,
        });
    }
}

fn add(account: &str, class: &'static str, change: Decimal) -> Posting {
    Posting {
        account: account.to_string(),
        class,
        change,
    }
}

/// Para B86(b) with AASB 3 para 32: the investment against equity at acquisition.
fn investment(g: &mut Group, s: usize) -> Result<Decimal> {
    let acq = g.entities[s].acquisition.clone().expect("a subsidiary");
    let name = g.entities[s].name.clone();
    let mut net = d("0");
    let mut postings = Vec::new();
    for (k, (account, value)) in acq.equity.iter().enumerate() {
        net = net.add(value)?;
        if acq.equity[..k].iter().any(|(a, _)| a == account) {
            return bad(
                "consolidation_eliminations.account_invalid",
                format!("{name}: equity_at_acquisition names {account} twice."),
            );
        }
        g.has(s, account, "equity")?;
        postings.push(add(account, "equity", value.neg()?));
    }
    let goodwill = acq.consideration.add(&acq.nci)?.sub(&net)?;
    if goodwill.lt(&d("0"))? {
        return bad(
            "consolidation_eliminations.bargain_purchase",
            format!(
                "{name}: net assets at acquisition exceed consideration and the non-controlling interest, a bargain purchase recognised only after the reassessment of AASB 3 paras 34 to 36, which this worksheet does not apply."
            ),
        );
    }
    postings.push(g.take(0, &acq.investment_account, "asset", &acq.consideration)?);
    if !goodwill.is_zero() {
        postings.push(add(GOODWILL, "asset", goodwill.clone()));
    }
    if !acq.nci.is_zero() {
        postings.push(add(NCI, "equity", acq.nci.clone()));
    }
    g.post(
        format!("{name}: investment and equity at acquisition"),
        "AASB 10 para B86(b); AASB 3 paras 19 and 32",
        postings,
    );
    Ok(goodwill)
}

/// Para B86(b): each investment line naming a subsidiary is eliminated in full,
/// so its carrying amount must equal the stated consideration (or, for a line
/// carrying more than one subsidiary, their sum).
fn investments_eliminated(g: &Group) -> Result<()> {
    let parent = &g.entities[0];
    for s in g.entities.iter().skip(1) {
        let account = &s
            .acquisition
            .as_ref()
            .expect("a subsidiary")
            .investment_account;
        let i = parent
            .lines
            .iter()
            .position(|l| &l.account == account)
            .expect("taken");
        if !parent.lines[i].amount.equals(&g.used[0][i])? {
            return bad(
                "consolidation_eliminations.investment_not_eliminated",
                format!(
                    "{account} carries {}, not the consideration of the subsidiaries it holds; eliminating it in full (AASB 10 para B86(b)) needs the carrying amount at the consideration, which this worksheet does not reconcile.",
                    parent.lines[i].amount
                ),
            );
        }
    }
    Ok(())
}

/// Paras 22 and B94: the non-controlling interest's share of each equity line's
/// movement since acquisition.
fn nci_since(g: &mut Group, s: usize) -> Result<()> {
    let e = &g.entities[s];
    let acq = e.acquisition.as_ref().expect("a subsidiary");
    let share = d("1").sub(&acq.ownership)?;
    if share.is_zero() {
        return Ok(());
    }
    let mut postings = Vec::new();
    let mut total = d("0");
    for l in e.lines.iter().filter(|l| l.class == "equity") {
        let at = acq
            .equity
            .iter()
            .find(|(a, _)| *a == l.account)
            .map_or_else(|| d("0"), |(_, v)| v.clone());
        let part = l
            .amount
            .sub(&at)?
            .mul(&share)?
            .quantize(-2, Rounding::HalfUp)?;
        if !part.is_zero() {
            total = total.add(&part)?;
            postings.push(add(&l.account, "equity", part.neg()?));
        }
    }
    if !postings.is_empty() {
        postings.push(add(NCI, "equity", total));
        let id = format!(
            "{}: non-controlling interest in equity since acquisition",
            e.name
        );
        g.post(id, "AASB 10 paras 22 and B94", postings);
    }
    Ok(())
}

fn side(o: &Map<String, Value>, key: &str, what: &str) -> Result<(String, String)> {
    let s = object(&o[key], &format!("{what} {key}"), &["entity", "account"])?;
    Ok((label(&s["entity"], what)?, label(&s["account"], what)?))
}

/// Para B86(c): a stated intra-group pair eliminated in full.
fn pairs(g: &mut Group, given: &Arg, what: &str, keys: [(&str, &'static str); 2]) -> Result<()> {
    for (i, v) in list(given, what, 0, MOST_ITEMS)?.iter().enumerate() {
        let item = format!("{what} {}", i + 1);
        let o = object(v, &item, &[keys[0].0, keys[1].0, "amount"])?;
        let value = amount(&o["amount"], &item, false)?;
        let (a, b) = (side(o, keys[0].0, &item)?, side(o, keys[1].0, &item)?);
        let (ea, eb) = (g.entity(&a.0)?, g.entity(&b.0)?);
        if ea == eb || value.is_zero() {
            return bad(
                "consolidation_eliminations.item_invalid",
                format!("{item}: needs 2 different entities and an amount above nil."),
            );
        }
        let postings = vec![
            g.take(ea, &a.1, keys[0].1, &value)?,
            g.take(eb, &b.1, keys[1].1, &value)?,
        ];
        g.post(item, "AASB 10 para B86(c)", postings);
    }
    Ok(())
}

fn tax_rate(given: &Arg) -> Result<Decimal> {
    match given {
        Arg::Decimal(r) if r.is_finite() && r.exponent() >= -6 && !r.lt(&d("0"))? && r.lt(&d("1"))? => Ok(r.clone()),
        _ => bad("consolidation_eliminations.tax_rate_invalid", "tax_rate must be a decimal fraction from 0 up to but not including 1, at most 6 places: the stated rate for the deferred tax on eliminated unrealised profit (AASB 112), 0 where none is recognised.".into()),
    }
}

/// Para B86(c): stated unrealised profit in a buyer's closing inventory, and
/// its stated tax effect; gives each seller's after-tax unrealised profit.
fn unrealised(g: &mut Group, given: &Arg, rate: &Decimal) -> Result<Vec<Decimal>> {
    const KEYS: [&str; 7] = [
        "seller",
        "buyer",
        "inventory_account",
        "cost_of_sales_account",
        "profit",
        "income_tax_account",
        "deferred_tax_account",
    ];
    let mut after_tax = vec![d("0"); g.entities.len()];
    for (i, v) in list(given, "unrealised_profits", 0, MOST_ITEMS)?
        .iter()
        .enumerate()
    {
        let item = format!("unrealised profit {}", i + 1);
        let o = object(v, &item, &KEYS)?;
        let (seller, kept) = unrealised_item(g, o, item, rate)?;
        after_tax[seller] = after_tax[seller].add(&kept)?;
    }
    Ok(after_tax)
}

/// One stated unrealised profit's journal; gives the seller and the profit
/// after its tax effect.
fn unrealised_item(
    g: &mut Group,
    o: &Map<String, Value>,
    item: String,
    rate: &Decimal,
) -> Result<(usize, Decimal)> {
    let profit = amount(&o["profit"], &item, false)?;
    let seller = g.entity(&label(&o["seller"], &item)?)?;
    let buyer = g.entity(&label(&o["buyer"], &item)?)?;
    let deferred = label(&o["deferred_tax_account"], &item)?;
    if seller == buyer || profit.is_zero() || [NCI, GOODWILL].contains(&deferred.as_str()) {
        return bad(
            "consolidation_eliminations.item_invalid",
            format!(
                "{item}: needs a seller and a buyer that differ, a profit above nil and a deferred tax account other than {NCI} or {GOODWILL}."
            ),
        );
    }
    let cos = label(&o["cost_of_sales_account"], &item)?;
    g.has(seller, &cos, "expense")?;
    let inventory = label(&o["inventory_account"], &item)?;
    let mut postings = vec![
        add(&cos, "expense", profit.clone()),
        g.take(buyer, &inventory, "asset", &profit)?,
    ];
    let tax = profit.mul(rate)?.quantize(-2, Rounding::HalfUp)?;
    if !tax.is_zero() {
        let expense = label(&o["income_tax_account"], &item)?;
        postings.push(g.take(seller, &expense, "expense", &tax)?);
        postings.push(add(&deferred, "asset", tax.clone()));
    }
    g.post(item, "AASB 10 para B86(c); AASB 112", postings);
    Ok((seller, profit.sub(&tax)?))
}

/// The facts outside the worksheet, each refused when stated true.
fn excluded(a: &ConsolidationEliminationsArgs) -> Result<()> {
    if flag(a.acquired_in_period, "acquired_in_period")? {
        return refuse(
            "consolidation_eliminations.acquired_in_period",
            "A subsidiary acquired during the period is consolidated only from the date control is obtained (AASB 10 paras 20 and B88), which needs its results split at that date; this worksheet does not apply that.",
        );
    }
    if flag(a.fair_value_adjustments, "fair_value_adjustments")? {
        return refuse(
            "consolidation_eliminations.fair_value_adjustments",
            "Identifiable assets and liabilities measured at acquisition-date fair values other than their carrying amounts (AASB 3 para 18) need adjustments and their later depreciation (AASB 10 para B88), which this worksheet does not apply.",
        );
    }
    if flag(a.previously_held_interest, "previously_held_interest")? {
        return refuse(
            "consolidation_eliminations.previously_held_interest",
            "A business combination achieved in stages remeasures the previously held interest (AASB 3 paras 32(a)(iii) and 42), which this worksheet does not apply.",
        );
    }
    if flag(a.ownership_changed, "ownership_changed")? {
        return refuse(
            "consolidation_eliminations.ownership_changed",
            "A change in ownership without loss of control is an equity transaction (AASB 10 paras 23 and B96), which this worksheet does not apply.",
        );
    }
    if flag(a.intra_group_dividends, "intra_group_dividends")? {
        return refuse(
            "consolidation_eliminations.intra_group_dividends",
            "Intra-group dividends, and the non-controlling interest's share of them, are eliminated under AASB 10 para B86(c) by entries this worksheet does not make.",
        );
    }
    Ok(())
}

fn group(a: &ConsolidationEliminationsArgs) -> Result<Group> {
    let mut entities = vec![parent(a.parent)?];
    entities.extend(subsidiaries(a.subsidiaries)?);
    for (i, e) in entities.iter().enumerate() {
        if entities[..i].iter().any(|o| o.name == e.name) {
            return bad(
                "consolidation_eliminations.item_invalid",
                format!("{} names 2 entities.", e.name),
            );
        }
    }
    let used = entities
        .iter()
        .map(|e| vec![d("0"); e.lines.len()])
        .collect();
    Ok(Group {
        entities,
        used,
        journals: Vec::new(),
    })
}

fn eliminate(a: &ConsolidationEliminationsArgs, rate: &Decimal) -> Result<Value> {
    let mut g = group(a)?;
    let mut goodwill = d("0");
    for s in 1..g.entities.len() {
        goodwill = goodwill.add(&investment(&mut g, s)?)?;
        nci_since(&mut g, s)?;
    }
    investments_eliminated(&g)?;
    pairs(
        &mut g,
        a.intra_group_balances,
        "intra-group balance",
        [("receivable", "asset"), ("payable", "liability")],
    )?;
    pairs(
        &mut g,
        a.intra_group_transactions,
        "intra-group transaction",
        [("income", "income"), ("expense", "expense")],
    )?;
    let after_tax = unrealised(&mut g, a.unrealised_profits, rate)?;
    let rows = rows(&g)?;
    let amounts = totals(&g, &rows, &after_tax, goodwill)?;
    let mut out = result(KIND, &scope_year(a)?, amounts, vec![])?;
    out["journals"] = shown_journals(&g.journals)?;
    out["lines"] = shown_rows(&rows)?;
    Ok(out)
}

fn scope_year(a: &ConsolidationEliminationsArgs) -> Result<String> {
    scope(a.scope_confirmed, a.year, KIND)
}

/// AASB 10 consolidation eliminations from stated trial balances.
pub fn consolidation_eliminations(a: &ConsolidationEliminationsArgs) -> Result<Value> {
    scope_year(a)?;
    excluded(a)?;
    let rate = tax_rate(a.tax_rate)?;
    precise(|| eliminate(a, &rate))
}
