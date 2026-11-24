// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `non_resident_withholding`: PAYG withholding from payments to foreign
//! residents under TAA 1953 Schedule 1 Subdivision 12-F (dividends, interest
//! and royalties), Subdivision 12-H (managed investment trust fund payments)
//! and Subdivision 12-FB / s 12-315 (casino gaming junkets, entertainment or
//! sports activities, and construction and related activities) for 2024-25 to
//! 2026-27. The citations are to TAA 1953 compilation 227 (C2026C00468,
//! 1 October 2026), ITAA 1936 compilation 193 (C2026C00454, 1 October 2026),
//! Income Tax (Dividends, Interest and Royalties Withholding Tax) Act 1974
//! (C2007C00596, 24 September 2007), Income Tax (Managed Investment Trust
//! Withholding Tax) Act 2008 compilation 3 (C2019C00219, 1 July 2019),
//! Taxation Administration Regulations 2017 compilation 18 (F2026C01210,
//! 14 November 2026) and Income Tax Rates Act 1986 compilation 66 (C2026C00300,
//! 1 July 2026), all read on 24 November 2026. Residency, treaty rates,
//! information exchange country status, exemptions, variations and payment
//! classification are the operator's facts.

use serde_json::{Value, json};

use super::{Arg, Result, int, money, precise, refuse, result, scope};
use crate::decimal::{Decimal, Rounding};

mod subdivision_12_f;
mod subdivision_12_fb;
mod subdivision_12_h;

const KIND: &str = "non_resident_withholding";

/// The per-payment-type facts beside `payment_type`, `payment`, `year` and
/// `scope_confirmed`, in [`NonResidentWithholdingArgs::optional`].
pub const NON_RESIDENT_WITHHOLDING_OPTIONAL: [&str; 13] = [
    "franked_amount",
    "conduit_foreign_income",
    "treaty_rate",
    "information_exchange_country",
    "clean_building_mit_amount",
    "non_concessional_mit_income",
    "company_not_acting_as_agent",
    "payee_entity",
    "base_rate_entity",
    "contract_date",
    "variation_rate",
    "exempt",
    "already_withheld",
];

/// The facts as the command passes them; see the README for each.
pub struct NonResidentWithholdingArgs<'a> {
    pub payment_type: &'a Arg,
    pub payment: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 13],
}

/// The indices into [`NON_RESIDENT_WITHHOLDING_OPTIONAL`] each payment type takes.
fn fields(payment_type: &str) -> Option<&'static [usize]> {
    Some(match payment_type {
        "dividend" => &[0, 1, 2, 11, 12],
        "interest" | "royalty" => &[2, 11, 12],
        "mit_fund_payment" => &[3, 4, 5, 6, 11],
        "gaming_junket" => &[10, 11, 12],
        "construction" => &[9, 10, 11, 12],
        "entertainment_or_sports" => &[7, 8, 10, 11, 12],
        _ => return None,
    })
}

/// Each of the payment type's facts given, and no other.
fn checked<'a>(
    args: &NonResidentWithholdingArgs<'a>,
    wanted: &[usize],
) -> Result<[Option<&'a Arg>; 13]> {
    for (i, (key, given)) in NON_RESIDENT_WITHHOLDING_OPTIONAL
        .iter()
        .zip(args.optional)
        .enumerate()
    {
        match (wanted.contains(&i), given) {
            (true, None) => {
                return refuse(
                    "input.missing_field",
                    format!("{key} is required for this payment_type."),
                );
            }
            (false, Some(_)) => {
                return refuse(
                    "input.field_unknown",
                    format!("{key} is not an input for this payment_type"),
                );
            }
            _ => {}
        }
    }
    Ok(args.optional)
}

/// The payment type's optional facts, each present once [`checked`] has passed.
pub(super) struct Facts<'a>([Option<&'a Arg>; 13]);

impl<'a> Facts<'a> {
    pub(super) fn get(&self, i: usize) -> &'a Arg {
        self.0[i].expect("checked above")
    }
}

pub(super) struct Worked {
    pub(super) subdivision: &'static str,
    pub(super) final_tax: bool,
    pub(super) amounts: Vec<(&'static str, Decimal)>,
    pub(super) rates: Vec<(String, String)>,
    pub(super) steps: Vec<Value>,
    pub(super) components: Option<Value>,
}

pub(super) fn step(id: &str, authority: &str, value: &Decimal) -> Value {
    json!({"step": id, "authority": authority, "value": value.to_string()})
}

pub(super) fn is_null(given: &Arg) -> bool {
    matches!(given, Arg::Other(none) if none == "None")
}

/// Taxation Administration Regulations 2017 s 26(2) and (3): disregard an
/// amount under $1 and disregard cents above whole dollars.
pub(super) fn whole_dollars(amount: &Decimal) -> Result<Decimal> {
    if amount.lt(&int(1))? {
        Ok(int(0))
    } else {
        Ok(amount.quantize(0, Rounding::Down)?)
    }
}

pub(super) fn non_negative(amount: Decimal) -> Result<Decimal> {
    if amount.lt(&int(0))? {
        Ok(int(0))
    } else {
        Ok(amount)
    }
}

/// Parse an optional decimal rate string in `[0, max]`, or JSON `null`.
/// Returns `Ok(None)` when the input is invalid so callers emit literal refusal codes.
pub(super) fn parse_optional_rate(given: &Arg, max: &Decimal) -> Result<Option<Option<Decimal>>> {
    if is_null(given) {
        return Ok(Some(None));
    }
    let parsed = match given.text() {
        Some(text) if text.len() <= 32 => Decimal::parse_input(text).ok(),
        _ => None,
    };
    match parsed {
        Some(r) if r.is_finite() && !r.lt(&int(0))? && !r.gt(max)? => Ok(Some(Some(r))),
        _ => Ok(None),
    }
}

pub(super) fn check_already_withheld(already: &Decimal, payment: &Decimal) -> Result<()> {
    if already.gt(payment)? {
        return refuse(
            "non_resident_withholding.already_withheld_exceeds_payment",
            "already_withheld exceeds payment.",
        );
    }
    Ok(())
}

pub fn non_resident_withholding(args: &NonResidentWithholdingArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let payment_type = args.payment_type.text().unwrap_or_default();
    let Some(wanted) = fields(payment_type) else {
        let hint = if payment_type == "capital_gains" {
            " Use foreign-resident-withholding for foreign resident capital gains withholding (TAA Sch 1 s 14-200)."
        } else {
            ""
        };
        return refuse(
            "non_resident_withholding.payment_type_invalid",
            format!(
                "payment_type must be dividend, interest, royalty, mit_fund_payment, gaming_junket, entertainment_or_sports or construction.{hint}"
            ),
        );
    };
    let payment = money(args.payment)?;
    let facts = Facts(checked(args, wanted)?);
    let worked = precise(|| match payment_type {
        "dividend" | "interest" | "royalty" => {
            subdivision_12_f::subdivision_12_f(payment_type, &payment, &facts)
        }
        "mit_fund_payment" => subdivision_12_h::subdivision_12_h(&payment, &facts),
        _ => subdivision_12_fb::subdivision_12_fb(payment_type, &payment, &year, &facts),
    })?;
    let mut out = result(KIND, &year, worked.amounts, worked.rates)?;
    out["payment_type"] = json!(payment_type);
    out["subdivision"] = json!(worked.subdivision);
    out["final_tax"] = json!(worked.final_tax);
    out["steps"] = json!(worked.steps);
    if let Some(components) = worked.components {
        out["components"] = components;
    }
    Ok(out)
}
