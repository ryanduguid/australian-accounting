// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use serde_json::Value;

use super::{
    Facts, Worked, check_already_withheld, is_null, non_negative, parse_optional_rate, step,
    whole_dollars,
};
use crate::dates;
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, d, date_arg, flag, int, lookup, money, rate, refuse};
use crate::metadata as meta;

const GAMING_JUNKET_RATE: &str = "0.03";
const CONSTRUCTION_RATE: &str = "0.05";
const COMPANY_BRE_RATE: &str = "0.25";
const COMPANY_STANDARD_RATE: &str = "0.30";
/// TAR 2017 s 33(1): construction withholding applies only to contracts
/// entered into after 30 June 2004.
const CONSTRUCTION_CONTRACT_CUTOFF: &str = "2004-06-30";

struct FbSetup {
    variation: Option<Decimal>,
    exempt: bool,
    already: Decimal,
    entity_bre: Option<(&'static str, Option<bool>)>,
}

/// Apply ITRA 1986 Schedule 7 Part II clause 1 foreign resident individual
/// marginal rates to `amount` (reusing `FOREIGN_RESIDENT_TAX_SCALES`).
fn foreign_individual_scale(amount: &Decimal, year: &str) -> Result<Decimal> {
    let scale = lookup(meta::FOREIGN_RESIDENT_TAX_SCALES, year);
    let mut total = int(0);
    for (lower, upper, rate_str) in *scale {
        let lo = int(*lower);
        if amount.gt(&lo)? {
            let slice = match upper {
                Some(hi) => {
                    let cap = int(*hi);
                    if amount.gt(&cap)? {
                        cap.sub(&lo)?
                    } else {
                        amount.sub(&lo)?
                    }
                }
                None => amount.sub(&lo)?,
            };
            total = total.add(&slice.mul(&d(rate_str))?)?;
        }
    }
    Ok(total)
}

fn check_construction_cutoff(facts: &Facts) -> Result<()> {
    let day = date_arg(facts.get(9), "contract_date")?;
    let cutoff = dates::parse(CONSTRUCTION_CONTRACT_CUTOFF).expect("a date");
    if day <= cutoff {
        return refuse(
            "non_resident_withholding.construction_contract_before_cutoff",
            "Construction and related activities withholding under TAR 2017 s 33(1) applies only to a payment made under a contract entered into after 30 June 2004.",
        );
    }
    Ok(())
}

fn parse_entertainment_payee(facts: &Facts) -> Result<(&'static str, Option<bool>)> {
    let entity = match facts.get(7).text() {
        Some("individual") => "individual",
        Some("company") => "company",
        _ => {
            return refuse(
                "non_resident_withholding.payee_entity_unsupported",
                "payee_entity must be individual or company for entertainment_or_sports (TAR 2017 s 50(2)); trusts, partnerships and other entities are outside scope.",
            );
        }
    };
    let bre_raw = facts.get(8);
    let bre = match entity {
        "company" => match bre_raw {
            Arg::Bool(b) => Some(*b),
            _ => {
                return refuse(
                    "non_resident_withholding.base_rate_entity_invalid",
                    "base_rate_entity must be a JSON boolean when payee_entity is company (ITRA 1986 s 23(2)).",
                );
            }
        },
        _ if is_null(bre_raw) => None,
        _ => {
            return refuse(
                "non_resident_withholding.base_rate_entity_invalid",
                "base_rate_entity must be null when payee_entity is individual.",
            );
        }
    };
    Ok((entity, bre))
}

fn parse_12_fb_setup(payment_type: &str, payment: &Decimal, facts: &Facts) -> Result<FbSetup> {
    if payment_type == "construction" {
        check_construction_cutoff(facts)?;
    }
    let Some(variation) = parse_optional_rate(facts.get(10), &int(1))? else {
        return refuse(
            "non_resident_withholding.variation_rate_invalid",
            "variation_rate must be null or a decimal string from 0 to 1 (TAA Sch 1 s 15-15).",
        );
    };
    let exempt = flag(facts.get(11), "exempt")?;
    let already = money(facts.get(12))?;
    check_already_withheld(&already, payment)?;
    let entity_bre = if payment_type == "entertainment_or_sports" {
        Some(parse_entertainment_payee(facts)?)
    } else {
        None
    };
    Ok(FbSetup {
        variation,
        exempt,
        already,
        entity_bre,
    })
}

fn raw_12_fb_rate(
    payment_type: &str,
    used: &Decimal,
    year: &str,
    entity_bre: Option<(&'static str, Option<bool>)>,
    rates: &mut Vec<(String, String)>,
) -> Result<(Decimal, &'static str)> {
    match payment_type {
        "gaming_junket" => {
            rates.push(rate("rate", GAMING_JUNKET_RATE));
            Ok((
                d(GAMING_JUNKET_RATE).mul(used)?,
                "TAR 2017 ss 31 and 49(2) (3% for casino gaming junkets) and s 26(1)(b), (2) and (3): cents disregarded",
            ))
        }
        "construction" => {
            rates.push(rate("rate", CONSTRUCTION_RATE));
            Ok((
                d(CONSTRUCTION_RATE).mul(used)?,
                "TAR 2017 ss 33 and 51(2) (5% for construction and related activities) and s 26(1)(b), (2) and (3): cents disregarded",
            ))
        }
        _ => match entity_bre.expect("checked above") {
            ("company", Some(true)) => {
                rates.push(rate("rate", COMPANY_BRE_RATE));
                Ok((
                    d(COMPANY_BRE_RATE).mul(used)?,
                    "TAR 2017 ss 32 and 50(2)(a) and ITRA 1986 s 23(2)(a) (25% base rate entity company rate) and TAR 2017 s 26(1)(b), (2) and (3): cents disregarded",
                ))
            }
            ("company", _) => {
                rates.push(rate("rate", COMPANY_STANDARD_RATE));
                Ok((
                    d(COMPANY_STANDARD_RATE).mul(used)?,
                    "TAR 2017 ss 32 and 50(2)(a) and ITRA 1986 s 23(2)(b) (30% company rate) and TAR 2017 s 26(1)(b), (2) and (3): cents disregarded",
                ))
            }
            _ => {
                rates.push(rate("scale", "ITRA 1986 Sch 7 Pt II (30%/37%/45%)"));
                Ok((
                    foreign_individual_scale(used, year)?,
                    "TAR 2017 ss 32 and 50(2)(b) and ITRA 1986 Sch 7 Pt II cl 1 (foreign resident individual marginal rates applied to payment as taxable income) and TAR 2017 s 26(1)(b), (2) and (3): cents disregarded",
                ))
            }
        },
    }
}

struct FbOutcome {
    before_prior: Decimal,
    withholding: Decimal,
    rates: Vec<(String, String)>,
}

fn compute_12_fb_non_exempt(
    payment_type: &str,
    used: &Decimal,
    year: &str,
    setup: &FbSetup,
    steps: &mut Vec<Value>,
) -> Result<FbOutcome> {
    let mut rates = Vec::new();
    let (raw, raw_auth) = if let Some(vr) = &setup.variation {
        rates.push(rate("variation_rate", vr.to_string()));
        (
            vr.mul(used)?,
            "TAA Sch 1 s 15-15 (Commissioner's variation rate) and TAR 2017 s 26(1)(b), (2) and (3): cents in the worked-out amount disregarded",
        )
    } else {
        raw_12_fb_rate(payment_type, used, year, setup.entity_bre, &mut rates)?
    };
    let gross_wh = whole_dollars(&raw)?;
    steps.push(step("withholding_before_prior", raw_auth, &gross_wh));
    let final_wh = whole_dollars(&non_negative(gross_wh.sub(&setup.already)?)?)?;
    steps.push(step(
        "withholding",
        "TAR 2017 s 48(2) and s 26(1)(b), (2) and (3): reduced by amounts already withheld under Subdivision 12-FB, cents disregarded",
        &final_wh,
    ));
    Ok(FbOutcome {
        before_prior: gross_wh,
        withholding: final_wh,
        rates,
    })
}

fn finish_12_fb_worked(
    payment: &Decimal,
    used: Decimal,
    already: &Decimal,
    outcome: FbOutcome,
    mut steps: Vec<Value>,
) -> Result<Worked> {
    let net = payment.sub(already)?.sub(&outcome.withholding)?;
    steps.push(step(
        "net_payment",
        "Payment less amounts already withheld and withholding required now",
        &net,
    ));
    Ok(Worked {
        subdivision: "12-FB",
        final_tax: false,
        amounts: vec![
            ("withholding_base", payment.clone()),
            ("payment_used", used),
            ("withholding_before_prior", outcome.before_prior),
            ("withholding", outcome.withholding),
            ("net_payment", net),
        ],
        rates: outcome.rates,
        steps,
        components: None,
    })
}

/// Subdivision 12-FB (s 12-315 / s 12-317): gaming junkets (TAR 2017 ss 31, 49),
/// entertainment or sports activities (ss 32, 50) and construction and related
/// activities (ss 33, 51), with s 26 rounding, s 48 prior-withholding reduction,
/// s 12-319 exemption and s 15-15 Commissioner variation.
pub(super) fn subdivision_12_fb(
    payment_type: &str,
    payment: &Decimal,
    year: &str,
    facts: &Facts,
) -> Result<Worked> {
    let setup = parse_12_fb_setup(payment_type, payment, facts)?;
    let used = whole_dollars(payment)?;
    let mut steps = vec![step(
        "payment_used",
        "Taxation Administration Regulations 2017 s 26(1)(a), (2) and (3): cents in the payment disregarded, and an amount under $1 disregarded",
        &used,
    )];
    let outcome = if setup.exempt {
        let zero = int(0);
        steps.push(step(
            "withholding",
            "TAA Sch 1 ss 12-315(1)(d), 12-317(1)(d) and 12-319(1): recipient holds a Commissioner's written exemption",
            &zero,
        ));
        FbOutcome {
            before_prior: zero.clone(),
            withholding: zero,
            rates: vec![rate("applied_rate", "0")],
        }
    } else {
        compute_12_fb_non_exempt(payment_type, &used, year, &setup, &mut steps)?
    };
    finish_12_fb_worked(payment, used, &setup.already, outcome, steps)
}
