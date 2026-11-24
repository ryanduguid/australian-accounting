// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use serde_json::{Value, json};

use crate::decimal::{Decimal, Rounding};
use crate::engine::{Arg, Result, Step, int, money, refuse, step};

const MAX_BONUSES: usize = 500;
const BONUSES_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(4)(c) and (d) (C2026C00364): incentive payments and bonuses count only when paid at least monthly (weekly, fortnightly or monthly); less frequent amounts are dropped rather than spread";

pub(super) struct ParsedBonuses {
    pub(super) counted: Vec<Decimal>,
    pub(super) excluded: Vec<Value>,
}

fn parse_bonus_amount(idx: usize, obj: &serde_json::Map<String, Value>) -> Result<Decimal> {
    let parsed = obj
        .get("amount")
        .and_then(Value::as_str)
        .and_then(|t| Decimal::parse_input(t).ok())
        .map(|v| money(&Arg::Decimal(v)));
    match parsed {
        Some(Ok(v)) => Ok(v),
        _ => refuse(
            "portable_lsl_levy.bonus_invalid",
            format!(
                "bonuses[{idx}].amount must be a finite non-negative decimal string, at most 2dp and 1e12."
            ),
        ),
    }
}

fn parse_bonus_frequency(idx: usize, obj: &serde_json::Map<String, Value>) -> Result<&str> {
    match obj.get("frequency").and_then(Value::as_str) {
        Some(
            f @ ("weekly" | "fortnightly" | "monthly" | "quarterly" | "half_yearly" | "halfYearly"
            | "annually"),
        ) => Ok(f),
        _ => refuse(
            "portable_lsl_levy.bonus_invalid",
            format!(
                "bonuses[{idx}].frequency must be weekly, fortnightly, monthly, quarterly, half_yearly (or halfYearly) or annually."
            ),
        ),
    }
}

fn parse_one_bonus(
    idx: usize,
    item: &Value,
    counted: &mut Vec<Decimal>,
    excluded: &mut Vec<Value>,
) -> Result<()> {
    let Some(obj) = item.as_object() else {
        return refuse(
            "portable_lsl_levy.bonus_invalid",
            format!("bonuses[{idx}] must be an object with amount and frequency."),
        );
    };
    if let Some(bad_key) = obj
        .keys()
        .find(|k| !matches!(k.as_str(), "amount" | "frequency"))
    {
        return refuse(
            "portable_lsl_levy.bonus_invalid",
            format!("bonuses[{idx}] has unknown field '{bad_key}'."),
        );
    }
    let amount = parse_bonus_amount(idx, obj)?;
    let freq = parse_bonus_frequency(idx, obj)?;
    if matches!(freq, "weekly" | "fortnightly" | "monthly") {
        counted.push(amount);
    } else {
        excluded.push(json!({
            "component": format!("bonuses[{idx}]"),
            "amount": amount.quantize(-2, Rounding::HalfUp)?.to_string(),
            "frequency": freq,
            "reason": "s 3B(4)(c) and (d): an incentive payment or bonus counts only where it is paid at least monthly. It is dropped, not spread across the year."
        }));
    }
    Ok(())
}

pub(super) fn parse_bonuses(arg: Option<&Arg>) -> Result<ParsedBonuses> {
    let Some(given) = arg else {
        return Ok(ParsedBonuses {
            counted: Vec::new(),
            excluded: Vec::new(),
        });
    };
    let Arg::Json(Value::Array(items)) = given else {
        return refuse(
            "portable_lsl_levy.bonus_invalid",
            format!("bonuses must be a list of at most {MAX_BONUSES} bonus objects."),
        );
    };
    if items.len() > MAX_BONUSES {
        return refuse(
            "portable_lsl_levy.bonus_invalid",
            format!("bonuses must contain at most {MAX_BONUSES} items."),
        );
    }
    let mut counted = Vec::new();
    let mut excluded = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        parse_one_bonus(idx, item, &mut counted, &mut excluded)?;
    }
    Ok(ParsedBonuses { counted, excluded })
}

pub(super) fn bonuses_step(counted: &[Decimal]) -> Result<Step> {
    if counted.is_empty() {
        return Ok(step("counted_bonuses", BONUSES_AUTH, "0", &[], int(0)));
    }
    let mut total = int(0);
    let mut parts = Vec::with_capacity(counted.len());
    for amount in counted {
        total = total.add(amount)?;
        parts.push(amount.to_string());
    }
    let formula = parts.join(" + ");
    Ok(step("counted_bonuses", BONUSES_AUTH, &formula, &[], total))
}
