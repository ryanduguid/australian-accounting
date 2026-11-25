// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `debt-forgiveness`: the commercial debt forgiveness rules of ITAA 1997
//! Division 245 for one debtor and one forgiveness income year. Each debt's
//! gross forgiven amount is its stated value less the stated amount offset
//! (s 245-75), shared equally between joint debtors (s 245-77); its net
//! forgiven amount is that less the amounts other provisions bring to account
//! (s 245-85) and any amount a creditor under common ownership agreed to forgo
//! (s 245-90). The total (s 245-105) reduces, in order and each only to the
//! extent of the amounts in it, tax losses (s 245-115), net capital losses of
//! earlier years (s 245-130), the expenditure in the table in s 245-145 and
//! the cost bases of CGT assets, investments in associates last (ss 245-175
//! and 245-185); what is left is disregarded (s 245-195). Read on
//! 26 November 2026 in ITAA 1997 compilation 268 (C2026C00471): Division 245
//! as substituted in 2010 (No. 79, 2010), s 245-145 last amended in 2011
//! (No. 93, 2011).

use serde_json::{Map, Value, json};

use super::{Arg, Result, Step, d, derived_result, flag, money, precise, refuse, scope, step};
use crate::decimal::{Calc, Decimal, Rounding};

const KIND: &str = "debt_forgiveness";
const MOST: usize = 200;

/// The worksheet's inputs, as the caller gave them.
pub struct DebtForgivenessArgs<'a> {
    pub debts: &'a Arg,
    pub tax_losses: &'a Arg,
    pub net_capital_losses: &'a Arg,
    pub expenditures: &'a Arg,
    pub cost_bases: &'a Arg,
    pub partnership: &'a Arg,
    pub attributable_income: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One forgiven debt as stated.
struct Debt {
    label: String,
    applies: bool,
    value: Decimal,
    offset: Decimal,
    debtors: i64,
    brought_to_account: Decimal,
    forgone: Decimal,
}

/// One amount the total net forgiven amount may reduce.
struct Item {
    label: String,
    amount: Decimal,
    /// The table item of s 245-145 (expenditure), or 1 for an investment in
    /// an associate (cost bases, s 245-185); 0 otherwise.
    tag: i64,
}

fn amount(o: &Map<String, Value>, key: &str, default: &str) -> Option<Decimal> {
    match o.get(key) {
        None => Some(d(default)),
        Some(v) => {
            let parsed = Arg::Decimal(Decimal::parse_input(v.as_str()?).ok()?);
            money(&parsed).ok()
        }
    }
}

const DEBT_KEYS: [&str; 10] = [
    "label",
    "commercial_debt",
    "excluded",
    "value",
    "amount_offset",
    "debtors",
    "assessable_income",
    "deductions_reduced",
    "cost_bases_reduced",
    "forgone_by_creditor",
];

fn debt(v: &Value, n: usize) -> Result<Debt> {
    let wrong = |why: &str| refuse("debt_forgiveness.debt_invalid", format!("debt {n}: {why}"));
    let Some(o) = v.as_object() else {
        return wrong("must be an object.");
    };
    if !o.keys().all(|k| DEBT_KEYS.contains(&k.as_str())) {
        return wrong(&format!("takes {} only.", DEBT_KEYS.join(", ")));
    }
    let label = o
        .get("label")
        .and_then(Value::as_str)
        .filter(|l| !l.trim().is_empty());
    let commercial = o.get("commercial_debt").and_then(Value::as_bool);
    let excluded = o.get("excluded").and_then(Value::as_bool);
    let debtors = match o.get("debtors") {
        None => Some(1),
        Some(v) => v.as_i64().filter(|n| (1..=100).contains(n)),
    };
    let reductions = [
        amount(o, "assessable_income", "0"),
        amount(o, "deductions_reduced", "0"),
        amount(o, "cost_bases_reduced", "0"),
    ];
    let brought = reductions
        .into_iter()
        .try_fold(d("0"), |sum, r| sum.add(&r?).ok());
    let read = (
        label,
        commercial,
        excluded,
        o.get("value").and_then(|_| amount(o, "value", "0")),
        amount(o, "amount_offset", "0"),
        debtors,
        brought,
        amount(o, "forgone_by_creditor", "0"),
    );
    let (Some(l), Some(c), Some(x), Some(value), Some(offset), Some(debtors), Some(b), Some(f)) =
        read
    else {
        return wrong(
            "needs a label, commercial_debt and excluded true or false and a value in AUD from 0; optionally amount_offset, assessable_income, deductions_reduced, cost_bases_reduced and forgone_by_creditor in AUD from 0 and debtors a whole number from 1 to 100.",
        );
    };
    Ok(Debt {
        label: l.to_string(),
        applies: c && !x,
        value,
        offset,
        debtors,
        brought_to_account: b,
        forgone: f,
    })
}

fn items(given: &Arg, name: &str, tag: Option<&str>) -> Result<Vec<Item>> {
    let wrong = |why: String| refuse("debt_forgiveness.item_invalid", format!("{name}: {why}"));
    let Arg::Json(Value::Array(list)) = given else {
        return wrong(format!("must be a list of at most {MOST} objects."));
    };
    if list.len() > MOST {
        return wrong(format!("must be a list of at most {MOST} objects."));
    }
    let mut out = Vec::with_capacity(list.len());
    for (i, v) in list.iter().enumerate() {
        let o = v.as_object();
        let known = o.is_some_and(|o| {
            o.keys()
                .all(|k| matches!(k.as_str(), "label" | "amount") || Some(k.as_str()) == tag)
        });
        let label = o
            .and_then(|o| o.get("label"))
            .and_then(Value::as_str)
            .filter(|l| !l.trim().is_empty());
        let value = o
            .filter(|o| o.contains_key("amount"))
            .and_then(|o| amount(o, "amount", "0"));
        let given = tag.and_then(|t| o.and_then(|o| o.get(t)));
        let tagged = match (tag, given) {
            (None, _) | (Some("associate"), None) => Some(0),
            (Some("item"), Some(v)) => v.as_i64().filter(|n| (1..=8).contains(n)),
            (Some("associate"), Some(Value::Bool(b))) => Some(i64::from(*b)),
            _ => None,
        };
        match (label, value, tagged) {
            (Some(l), Some(amount), Some(tag)) if known => out.push(Item {
                label: l.to_string(),
                amount,
                tag,
            }),
            _ => {
                let extra = match tag {
                    Some("item") => {
                        ", and item a whole number from 1 to 8 (the row of the table in s 245-145)"
                    }
                    Some(_) => ", and optionally associate true or false (s 245-185)",
                    None => "",
                };
                return wrong(format!(
                    "entry {}: needs a label and an amount in AUD from 0{extra}, and nothing else.",
                    i + 1
                ));
            }
        }
    }
    Ok(out)
}

/// One debt's figures in cents: (gross forgiven amount, its share for this
/// debtor, net forgiven amount).
fn net_of(x: &Debt) -> Result<(Decimal, Decimal, Decimal)> {
    let calc = || -> Calc<(Decimal, Decimal, Decimal)> {
        let gross = if x.applies && x.value.gt(&x.offset)? {
            x.value.sub(&x.offset)?
        } else {
            d("0")
        };
        let share = gross
            .div(&Decimal::from_int(x.debtors))?
            .quantize(-2, Rounding::HalfUp)?;
        let provisional = share.sub(&x.brought_to_account)?;
        let provisional = if provisional.gt(&d("0"))? {
            provisional
        } else {
            d("0.00")
        };
        Ok((gross, share, provisional))
    };
    let (gross, share, provisional) = precise(|| Ok(calc()?))?;
    if x.forgone.gt(&provisional)? {
        return refuse(
            "debt_forgiveness.forgone_exceeds_provisional",
            format!(
                "{}: forgone_by_creditor {} is more than the provisional net forgiven amount {provisional}; s 245-90(2)(a) and (3)(a) cap the amount a creditor may agree to forgo at it.",
                x.label, x.forgone
            ),
        );
    }
    Ok((gross, share, precise(|| Ok(provisional.sub(&x.forgone)?))?))
}

fn cents(v: &Decimal) -> Calc<String> {
    Ok(v.quantize(-2, Rounding::HalfUp)?.to_string())
}

fn debt_rows(debts: &[Debt], worked: &[(Decimal, Decimal, Decimal)]) -> Calc<Value> {
    let mut rows = Vec::with_capacity(debts.len());
    for (x, (gross, share, net)) in debts.iter().zip(worked) {
        rows.push(json!({
            "label": x.label,
            "division_245_applies": x.applies,
            "value": cents(&x.value)?,
            "amount_offset": cents(&x.offset)?,
            "gross_forgiven_amount": cents(gross)?,
            "debtors": x.debtors,
            "gross_forgiven_amount_share": cents(share)?,
            "brought_to_account": cents(&x.brought_to_account)?,
            "forgone_by_creditor": cents(&x.forgone)?,
            "net_forgiven_amount": cents(net)?,
        }));
    }
    Ok(Value::Array(rows))
}

/// A step adding `terms`, written as numbers; `0` when there are none.
fn sum_step(id: &'static str, authority: &'static str, terms: &[&Decimal]) -> Calc<Step> {
    let mut value = d("0");
    for t in terms {
        value = value.add(t)?;
    }
    let formula = if terms.is_empty() {
        "0".to_string()
    } else {
        terms
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" + ")
    };
    Ok(step(id, authority, &formula, &[], value))
}

/// One class's reduction: the lesser of what is left and the class's total,
/// spread over its items in order (the caller's choice, ss 245-120, 245-135,
/// 245-150 and 245-180).
fn reduce(
    id: &'static str,
    authority: &'static str,
    earlier: &[&'static str],
    steps: &[Step],
    class: &[&Item],
) -> Calc<(Step, Vec<Decimal>)> {
    let mut left = steps[3].value.clone();
    let mut formula = String::from("min(total_net_forgiven_amount");
    let mut operands = vec![&steps[3].value];
    for e in earlier {
        let s = steps.iter().find(|s| s.id == *e).expect("an earlier class");
        left = left.sub(&s.value)?;
        formula.push_str(&format!(" - {e}"));
        operands.push(&s.value);
    }
    let mut sum = d("0");
    for x in class {
        sum = sum.add(&x.amount)?;
    }
    let terms = if class.is_empty() {
        "0".to_string()
    } else {
        class
            .iter()
            .map(|x| x.amount.to_string())
            .collect::<Vec<_>>()
            .join(" + ")
    };
    formula.push_str(&format!(", {terms})"));
    let applied = if left.lt(&sum)? { left } else { sum };
    let mut rest = applied.clone();
    let mut each = Vec::with_capacity(class.len());
    for x in class {
        let take = if rest.lt(&x.amount)? {
            rest.clone()
        } else {
            x.amount.clone()
        };
        rest = rest.sub(&take)?;
        each.push(take);
    }
    Ok((step(id, authority, &formula, &operands, applied), each))
}

fn item_rows(class: &[&Item], applied: &[Decimal], key: Option<&str>) -> Calc<Value> {
    let mut rows = Vec::with_capacity(class.len());
    for (x, a) in class.iter().zip(applied) {
        let mut row = json!({
            "label": x.label,
            "amount": cents(&x.amount)?,
            "reduced_by": cents(a)?,
            "remaining": cents(&x.amount.sub(a)?)?,
        });
        match key {
            Some("item") => row["item"] = json!(x.tag),
            Some(k) => row[k] = json!(x.tag == 1),
            None => {}
        }
        rows.push(row);
    }
    Ok(Value::Array(rows))
}

const CLASSES: [(&str, &str); 4] = [
    (
        "tax_losses_reduced",
        "s 245-115: applied first, to the maximum extent possible, to tax losses deductible in the forgiveness income year or later, in the order chosen (s 245-120)",
    ),
    (
        "net_capital_losses_reduced",
        "s 245-130: what remains, to net capital losses of earlier income years, in the order chosen (s 245-135)",
    ),
    (
        "expenditure_reduced",
        "s 245-145: what remains, to the expenditure in its table incurred before the forgiveness income year, in the order chosen (s 245-150)",
    ),
    (
        "cost_bases_reduced",
        "s 245-175: what remains, to the cost bases and reduced cost bases of CGT assets, each up to the limit in s 245-190(3), investments in associates last (s 245-185)",
    ),
];

/// Division 245 for one debtor and one forgiveness income year.
pub fn debt_forgiveness(a: &DebtForgivenessArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    if flag(a.partnership, "partnership")? {
        return refuse(
            "debt_forgiveness.partnership",
            "A partnership's unapplied net forgiven amount passes to its partners (Subdivision 245-F), which this worksheet does not apply.",
        );
    }
    if flag(a.attributable_income, "attributable_income")? {
        return refuse(
            "debt_forgiveness.attributable_income",
            "Subdivision 245-E does not apply in working out the attributable income of a non-resident trust estate or a CFC (s 245-100).",
        );
    }
    let Arg::Json(Value::Array(list)) = a.debts else {
        return refuse(
            "debt_forgiveness.debt_invalid",
            format!("debts must be a list of 1 to {MOST} objects."),
        );
    };
    if list.is_empty() || list.len() > MOST {
        return refuse(
            "debt_forgiveness.debt_invalid",
            format!("debts must be a list of 1 to {MOST} objects."),
        );
    }
    let debts = list
        .iter()
        .enumerate()
        .map(|(i, v)| debt(v, i + 1))
        .collect::<Result<Vec<_>>>()?;
    let losses = items(a.tax_losses, "tax_losses", None)?;
    let capital = items(a.net_capital_losses, "net_capital_losses", None)?;
    let spend = items(a.expenditures, "expenditures", Some("item"))?;
    let bases = items(a.cost_bases, "cost_bases", Some("associate"))?;
    let worked = debts.iter().map(net_of).collect::<Result<Vec<_>>>()?;
    // Section 245-185: investments in associates after the other assets.
    let ordered: Vec<&Item> = bases
        .iter()
        .filter(|x| x.tag == 0)
        .chain(bases.iter().filter(|x| x.tag == 1))
        .collect();
    let classes: [Vec<&Item>; 4] = [
        losses.iter().collect(),
        capital.iter().collect(),
        spend.iter().collect(),
        ordered,
    ];
    precise(|| {
        let shares: Vec<&Decimal> = worked.iter().map(|w| &w.1).collect();
        let brought: Vec<&Decimal> = debts.iter().map(|x| &x.brought_to_account).collect();
        let forgone: Vec<&Decimal> = debts.iter().map(|x| &x.forgone).collect();
        let nets: Vec<&Decimal> = worked.iter().map(|w| &w.2).collect();
        let mut steps = vec![
            sum_step(
                "gross_forgiven_amount",
                "ss 245-75 and 245-77: each debt's value less the amount offset, nil if not more, shared equally between the debtors (in cents, half up), nil where Division 245 does not apply (ss 245-10 and 245-40)",
                &shares,
            )?,
            sum_step(
                "brought_to_account",
                "s 245-85(1): amounts included in assessable income, deductions reduced and cost bases reduced by other provisions because of the forgiveness",
                &brought,
            )?,
            sum_step(
                "forgone_by_creditors",
                "s 245-90: capital losses and deductions creditors under common ownership agreed to forgo",
                &forgone,
            )?,
            sum_step(
                "total_net_forgiven_amount",
                "ss 245-85(2), 245-90 and 245-105(1): the debts' net forgiven amounts, each nil rather than negative",
                &nets,
            )?,
        ];
        let mut applied = Vec::with_capacity(4);
        let mut earlier = Vec::with_capacity(3);
        for ((id, authority), class) in CLASSES.iter().zip(&classes) {
            let (s, each) = reduce(id, authority, &earlier, &steps, class)?;
            steps.push(s);
            applied.push(each);
            earlier.push(*id);
        }
        let mut formula = String::from("total_net_forgiven_amount");
        let mut operands = vec![&steps[3].value];
        let mut left = steps[3].value.clone();
        for s in &steps[4..] {
            formula.push_str(&format!(" - {}", s.id));
            operands.push(&s.value);
            left = left.sub(&s.value)?;
        }
        let unapplied = step(
            "unapplied_net_forgiven_amount",
            "s 245-195(1): any part left after the 4 reductions is disregarded",
            &formula,
            &operands,
            left,
        );
        steps.push(unapplied);
        let mut out = derived_result(KIND, &year, steps, vec![])?;
        out["debts"] = debt_rows(&debts, &worked)?;
        out["tax_losses"] = item_rows(&classes[0], &applied[0], None)?;
        out["net_capital_losses"] = item_rows(&classes[1], &applied[1], None)?;
        out["expenditures"] = item_rows(&classes[2], &applied[2], Some("item"))?;
        out["cost_bases"] = item_rows(&classes[3], &applied[3], Some("associate"))?;
        Ok(out)
    })
}
