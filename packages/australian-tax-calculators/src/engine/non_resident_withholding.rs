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

use super::{
    Arg, Result, cents, d, date_arg, flag, int, lookup, money, precise, rate, refuse, result, scope,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::metadata as meta;

const KIND: &str = "non_resident_withholding";

const DIVIDEND_RATE: &str = "0.30";
const INTEREST_RATE: &str = "0.10";
const ROYALTY_RATE: &str = "0.30";

const MIT_EOI_RATE: &str = "0.15";
const MIT_CLEAN_BUILDING_RATE: &str = "0.10";
const MIT_NON_CONCESSIONAL_RATE: &str = "0.30";
const MIT_NON_EOI_RATE: &str = "0.30";

const GAMING_JUNKET_RATE: &str = "0.03";
const CONSTRUCTION_RATE: &str = "0.05";
const COMPANY_BRE_RATE: &str = "0.25";
const COMPANY_STANDARD_RATE: &str = "0.30";
/// TAR 2017 s 33(1): construction withholding applies only to contracts
/// entered into after 30 June 2004.
const CONSTRUCTION_CONTRACT_CUTOFF: &str = "2004-06-30";

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
struct Facts<'a>([Option<&'a Arg>; 13]);

impl<'a> Facts<'a> {
    fn get(&self, i: usize) -> &'a Arg {
        self.0[i].expect("checked above")
    }
}

struct Worked {
    subdivision: &'static str,
    final_tax: bool,
    amounts: Vec<(&'static str, Decimal)>,
    rates: Vec<(String, String)>,
    steps: Vec<Value>,
    components: Option<Value>,
}

fn step(id: &str, authority: &str, value: &Decimal) -> Value {
    json!({"step": id, "authority": authority, "value": value.to_string()})
}

fn is_null(given: &Arg) -> bool {
    matches!(given, Arg::Other(none) if none == "None")
}

/// Taxation Administration Regulations 2017 s 26(2) and (3): disregard an
/// amount under $1 and disregard cents above whole dollars.
fn whole_dollars(amount: &Decimal) -> Result<Decimal> {
    if amount.lt(&int(1))? {
        Ok(int(0))
    } else {
        Ok(amount.quantize(0, Rounding::Down)?)
    }
}

/// Floor to whole cents so a statutory upper limit (TAA Sch 1 s 12-300(b)) is
/// never exceeded by half-cent rounding.
fn floor_cents(amount: &Decimal) -> Result<Decimal> {
    Ok(amount.quantize(-2, Rounding::Down)?)
}

fn non_negative(amount: Decimal) -> Result<Decimal> {
    if amount.lt(&int(0))? {
        Ok(int(0))
    } else {
        Ok(amount)
    }
}

/// Parse an optional decimal rate string in `[0, max]`, or JSON `null`.
/// Returns `Ok(None)` when the input is invalid so callers emit literal refusal codes.
fn parse_optional_rate(given: &Arg, max: &Decimal) -> Result<Option<Option<Decimal>>> {
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

fn check_already_withheld(already: &Decimal, payment: &Decimal) -> Result<()> {
    if already.gt(payment)? {
        return refuse(
            "non_resident_withholding.already_withheld_exceeds_payment",
            "already_withheld exceeds payment.",
        );
    }
    Ok(())
}

/// Subdivision 12-F: dividend (ss 12-210, 12-215, 12-220), interest
/// (ss 12-245, 12-250, 12-255) and royalty (ss 12-280, 12-285), with
/// TAR 2017 ss 26, 39 to 42 and the s 12-300 ceiling.
fn subdivision_12_f(payment_type: &str, payment: &Decimal, facts: &Facts) -> Result<Worked> {
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

    let mut steps = Vec::new();
    let base = if payment_type == "dividend" {
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
        unfranked
    } else {
        let auth = if payment_type == "interest" {
            "TAA Sch 1 ss 12-245, 12-250 and 12-255; ITAA 1936 s 128B(2), (2A) and (5): interest payment subject to Subdivision 12-F"
        } else {
            "TAA Sch 1 ss 12-280 and 12-285; ITAA 1936 s 128B(2B), (2C) and (5A): royalty payment subject to Subdivision 12-F"
        };
        steps.push(step("withholding_base", auth, payment));
        payment.clone()
    };

    let applied_rate = treaty.clone().unwrap_or_else(|| domestic.clone());
    let used = whole_dollars(&base)?;
    steps.push(step(
        "payment_used",
        "Taxation Administration Regulations 2017 s 26(1)(a), (2) and (3): cents in the payment disregarded, and an amount under $1 disregarded",
        &used,
    ));

    let (reg_amount, withholding) = if exempt {
        let zero = int(0);
        steps.push(step(
            "withholding",
            "TAA Sch 1 s 12-300(a): no amount is required to be withheld when no withholding tax is payable",
            &zero,
        ));
        (zero.clone(), zero)
    } else {
        // TAR 2017 s 40 (dividend) and s 42 (royalty) apply the treaty rate directly;
        // s 41 (interest) specifies 10%, with TAA Sch 1 s 12-300 capping at treaty liability.
        let reg_rate = if payment_type == "interest" {
            &domestic
        } else {
            &applied_rate
        };
        let reg = whole_dollars(&reg_rate.mul(&used)?)?;
        let reg_auth = match payment_type {
            "dividend" => {
                "TAR 2017 s 40 (30% or stated treaty rate) and s 26(1)(b), (2) and (3): cents in the worked-out amount disregarded"
            }
            "interest" => {
                "TAR 2017 s 41 (10% of the interest) and s 26(1)(b), (2) and (3): cents in the worked-out amount disregarded"
            }
            _ => {
                "TAR 2017 s 42 (30% or stated treaty rate) and s 26(1)(b), (2) and (3): cents in the worked-out amount disregarded"
            }
        };
        steps.push(step("regulation_withholding", reg_auth, &reg));

        let liability = applied_rate.mul(&base)?;
        let cap = floor_cents(&non_negative(liability.sub(&already)?)?)?;
        let final_wh = if cap.lt(&reg)? {
            cap.clone()
        } else {
            reg.clone()
        };
        steps.push(step(
            "s_12_300_cap",
            "TAA Sch 1 s 12-300(a) and (b); ITAA 1936 s 128B; IT(DIRWHT)A 1974 s 7; International Tax Agreements Act 1953 s 4: withholding tax payable less amounts already withheld under Subdivision 12-F (floored to whole cents so the statutory limit is not exceeded)",
            &cap,
        ));
        steps.push(step(
            "withholding",
            "TAA Sch 1 s 12-300: lesser of the regulation withholding amount and the s 12-300(b) limit",
            &final_wh,
        ));
        (reg, final_wh)
    };

    let net = payment.sub(&already)?.sub(&withholding)?;
    steps.push(step(
        "net_payment",
        "Payment less amounts already withheld and withholding required now",
        &net,
    ));

    let mut rates = vec![
        rate("domestic_rate", domestic_str),
        rate(
            "applied_rate",
            if exempt {
                "0".to_string()
            } else {
                applied_rate.to_string()
            },
        ),
    ];
    if let Some(tr) = treaty {
        rates.push(rate("treaty_rate", tr.to_string()));
    }
    Ok(Worked {
        subdivision: "12-F",
        final_tax: true,
        amounts: vec![
            ("withholding_base", base),
            ("payment_used", used),
            ("regulation_withholding", reg_amount),
            ("withholding", withholding),
            ("net_payment", net),
        ],
        rates,
        steps,
        components: None,
    })
}

/// Subdivision 12-H: withholding MIT fund payments under TAA Sch 1 ss 12-385
/// and 12-390 and Income Tax (Managed Investment Trust Withholding Tax) Act
/// 2008 s 4. TAR 2017 s 26 does not apply (s 15-10(3)); each statutory rate
/// part is rounded half up to the cent.
fn subdivision_12_h(payment: &Decimal, facts: &Facts) -> Result<Worked> {
    let eoi = flag(facts.get(3), "information_exchange_country")?;
    let clean = money(facts.get(4))?;
    let ncmi = money(facts.get(5))?;
    let company_non_agent = flag(facts.get(6), "company_not_acting_as_agent")?;
    let exempt = flag(facts.get(11), "exempt")?;

    let special = clean.add(&ncmi)?;
    if special.gt(payment)? {
        return refuse(
            "non_resident_withholding.mit_parts_exceed_payment",
            "clean_building_mit_amount plus non_concessional_mit_income exceeds payment.",
        );
    }
    let standard = payment.sub(&special)?;

    let mut steps = vec![step(
        "standard_fund_payment",
        "TAA Sch 1 ss 12-385(3)(a)(i) and 12-390(3)(a)(i): fund payment other than clean building MIT and non-concessional MIT income parts",
        &standard,
    )];

    let (withholding, components, rates) = if company_non_agent {
        let zero = int(0);
        steps.push(step(
            "withholding",
            "TAA Sch 1 s 12-390(10)(a): section 12-390 does not apply to a company unless the company would, apart from section 12-420, be acting in the capacity as agent for the recipient",
            &zero,
        ));
        (
            zero,
            json!([{
                "component": "company_not_acting_as_agent_exception",
                "amount": cents(payment.clone())?.to_string(),
                "rate": "0",
                "withholding": "0.00",
                "authority": "TAA Sch 1 s 12-390(10)(a)"
            }]),
            vec![rate("applied_rate", "0")],
        )
    } else if exempt {
        let zero = int(0);
        steps.push(step(
            "withholding",
            "TAA Sch 1 ss 12-385(5) and 12-390(10)(b): no amount is required to be withheld when no managed investment trust withholding tax is payable",
            &zero,
        ));
        (
            zero,
            json!([{
                "component": "exempt_fund_payment",
                "amount": cents(payment.clone())?.to_string(),
                "rate": "0",
                "withholding": "0.00",
                "authority": "TAA Sch 1 ss 12-385(5) and 12-390(10)(b)"
            }]),
            vec![rate("applied_rate", "0")],
        )
    } else if !eoi {
        let r = d(MIT_NON_EOI_RATE);
        let wh = cents(r.mul(payment)?)?;
        steps.push(step(
            "withholding",
            "TAA Sch 1 ss 12-385(2), (3)(b) and 12-390(2), (3)(b), (5), (6)(b); IT(MITWHT)A 2008 s 4(1)(b): 30% for a recipient not in an information exchange country (rounded half up to the cent; s 15-10(3) does not engage TAR 2017 s 26)",
            &wh,
        ));
        (
            wh.clone(),
            json!([{
                "component": "non_eoi_fund_payment",
                "amount": cents(payment.clone())?.to_string(),
                "rate": MIT_NON_EOI_RATE,
                "withholding": wh.to_string(),
                "authority": "TAA Sch 1 ss 12-385(3)(b), 12-390(3)(b) and (6)(b); IT(MITWHT)A 2008 s 4(1)(b)"
            }]),
            vec![rate("non_eoi_rate", MIT_NON_EOI_RATE)],
        )
    } else {
        let std_wh = cents(d(MIT_EOI_RATE).mul(&standard)?)?;
        let clean_wh = cents(d(MIT_CLEAN_BUILDING_RATE).mul(&clean)?)?;
        let ncmi_wh = cents(d(MIT_NON_CONCESSIONAL_RATE).mul(&ncmi)?)?;
        let total = std_wh.add(&clean_wh)?.add(&ncmi_wh)?;
        steps.push(step(
            "standard_withholding",
            "TAA Sch 1 ss 12-385(3)(a)(i) and 12-390(3)(a)(i), (6)(a)(i); IT(MITWHT)A 2008 s 4(1)(a)(i): 15% on standard fund payment (rounded half up to the cent)",
            &std_wh,
        ));
        steps.push(step(
            "clean_building_withholding",
            "TAA Sch 1 ss 12-385(3)(a)(ii), 12-390(3)(a)(ii), (6)(a)(ii) and 12-425; IT(MITWHT)A 2008 s 4(1)(a)(ii): 10% on clean building MIT fund payment (rounded half up to the cent)",
            &clean_wh,
        ));
        steps.push(step(
            "ncmi_withholding",
            "TAA Sch 1 ss 12-385(3)(a)(iii), 12-390(3)(a)(iii), (6)(a)(iii) and 12-435; IT(MITWHT)A 2008 s 4(1)(a)(iii): 30% on non-concessional MIT income (rounded half up to the cent)",
            &ncmi_wh,
        ));
        steps.push(step(
            "withholding",
            "TAA Sch 1 ss 12-385(2) and 12-390(2), (5): sum of the Subdivision 12-H rate parts",
            &total,
        ));
        let comps = json!([
            {
                "component": "standard",
                "amount": cents(standard.clone())?.to_string(),
                "rate": MIT_EOI_RATE,
                "withholding": std_wh.to_string(),
                "authority": "TAA Sch 1 ss 12-385(3)(a)(i), 12-390(3)(a)(i) and (6)(a)(i); IT(MITWHT)A 2008 s 4(1)(a)(i)"
            },
            {
                "component": "clean_building_mit",
                "amount": cents(clean.clone())?.to_string(),
                "rate": MIT_CLEAN_BUILDING_RATE,
                "withholding": clean_wh.to_string(),
                "authority": "TAA Sch 1 ss 12-385(3)(a)(ii), 12-390(3)(a)(ii), (6)(a)(ii) and 12-425; IT(MITWHT)A 2008 s 4(1)(a)(ii)"
            },
            {
                "component": "non_concessional_mit_income",
                "amount": cents(ncmi.clone())?.to_string(),
                "rate": MIT_NON_CONCESSIONAL_RATE,
                "withholding": ncmi_wh.to_string(),
                "authority": "TAA Sch 1 ss 12-385(3)(a)(iii), 12-390(3)(a)(iii), (6)(a)(iii) and 12-435; IT(MITWHT)A 2008 s 4(1)(a)(iii)"
            }
        ]);
        let rates = vec![
            rate("standard_rate", MIT_EOI_RATE),
            rate("clean_building_rate", MIT_CLEAN_BUILDING_RATE),
            rate("non_concessional_rate", MIT_NON_CONCESSIONAL_RATE),
        ];
        (total, comps, rates)
    };

    let net = payment.sub(&withholding)?;
    steps.push(step(
        "net_payment",
        "Fund payment less withholding required under Subdivision 12-H",
        &net,
    ));

    Ok(Worked {
        subdivision: "12-H",
        final_tax: true,
        amounts: vec![
            ("standard_fund_payment", standard),
            ("clean_building_mit_amount", clean),
            ("non_concessional_mit_income", ncmi),
            ("withholding", withholding),
            ("net_payment", net),
        ],
        rates,
        steps,
        components: Some(components),
    })
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

/// Subdivision 12-FB (s 12-315 / s 12-317): gaming junkets (TAR 2017 ss 31, 49),
/// entertainment or sports activities (ss 32, 50) and construction and related
/// activities (ss 33, 51), with s 26 rounding, s 48 prior-withholding reduction,
/// s 12-319 exemption and s 15-15 Commissioner variation.
fn subdivision_12_fb(
    payment_type: &str,
    payment: &Decimal,
    year: &str,
    facts: &Facts,
) -> Result<Worked> {
    if payment_type == "construction" {
        let day = date_arg(facts.get(9), "contract_date")?;
        let cutoff = dates::parse(CONSTRUCTION_CONTRACT_CUTOFF).expect("a date");
        if day <= cutoff {
            return refuse(
                "non_resident_withholding.construction_contract_before_cutoff",
                "Construction and related activities withholding under TAR 2017 s 33(1) applies only to a payment made under a contract entered into after 30 June 2004.",
            );
        }
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

    let entity_and_bre = if payment_type == "entertainment_or_sports" {
        let entity = match facts.get(7).text() {
            Some(e @ ("individual" | "company")) => e,
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
            _ => {
                if is_null(bre_raw) {
                    None
                } else {
                    return refuse(
                        "non_resident_withholding.base_rate_entity_invalid",
                        "base_rate_entity must be null when payee_entity is individual.",
                    );
                }
            }
        };
        Some((entity, bre))
    } else {
        None
    };

    let used = whole_dollars(payment)?;
    let mut steps = vec![step(
        "payment_used",
        "Taxation Administration Regulations 2017 s 26(1)(a), (2) and (3): cents in the payment disregarded, and an amount under $1 disregarded",
        &used,
    )];

    let mut rates = Vec::new();
    let (before_prior, withholding) = if exempt {
        let zero = int(0);
        steps.push(step(
            "withholding",
            "TAA Sch 1 ss 12-315(1)(d), 12-317(1)(d) and 12-319(1): recipient holds a Commissioner's written exemption",
            &zero,
        ));
        rates.push(rate("applied_rate", "0"));
        (zero.clone(), zero)
    } else {
        let (raw, raw_auth) = if let Some(vr) = &variation {
            rates.push(rate("variation_rate", vr.to_string()));
            (
                vr.mul(&used)?,
                "TAA Sch 1 s 15-15 (Commissioner's variation rate) and TAR 2017 s 26(1)(b), (2) and (3): cents in the worked-out amount disregarded",
            )
        } else {
            match payment_type {
                "gaming_junket" => {
                    rates.push(rate("rate", GAMING_JUNKET_RATE));
                    (
                        d(GAMING_JUNKET_RATE).mul(&used)?,
                        "TAR 2017 ss 31 and 49(2) (3% for casino gaming junkets) and s 26(1)(b), (2) and (3): cents disregarded",
                    )
                }
                "construction" => {
                    rates.push(rate("rate", CONSTRUCTION_RATE));
                    (
                        d(CONSTRUCTION_RATE).mul(&used)?,
                        "TAR 2017 ss 33 and 51(2) (5% for construction and related activities) and s 26(1)(b), (2) and (3): cents disregarded",
                    )
                }
                _ => match entity_and_bre.expect("checked above") {
                    ("company", Some(true)) => {
                        rates.push(rate("rate", COMPANY_BRE_RATE));
                        (
                            d(COMPANY_BRE_RATE).mul(&used)?,
                            "TAR 2017 ss 32 and 50(2)(a) and ITRA 1986 s 23(2)(a) (25% base rate entity company rate) and TAR 2017 s 26(1)(b), (2) and (3): cents disregarded",
                        )
                    }
                    ("company", _) => {
                        rates.push(rate("rate", COMPANY_STANDARD_RATE));
                        (
                            d(COMPANY_STANDARD_RATE).mul(&used)?,
                            "TAR 2017 ss 32 and 50(2)(a) and ITRA 1986 s 23(2)(b) (30% company rate) and TAR 2017 s 26(1)(b), (2) and (3): cents disregarded",
                        )
                    }
                    _ => {
                        rates.push(rate("scale", "ITRA 1986 Sch 7 Pt II (30%/37%/45%)"));
                        (
                            foreign_individual_scale(&used, year)?,
                            "TAR 2017 ss 32 and 50(2)(b) and ITRA 1986 Sch 7 Pt II cl 1 (foreign resident individual marginal rates applied to payment as taxable income) and TAR 2017 s 26(1)(b), (2) and (3): cents disregarded",
                        )
                    }
                },
            }
        };

        let gross_wh = whole_dollars(&raw)?;
        steps.push(step("withholding_before_prior", raw_auth, &gross_wh));
        let final_wh = whole_dollars(&non_negative(gross_wh.sub(&already)?)?)?;
        steps.push(step(
            "withholding",
            "TAR 2017 s 48(2) and s 26(1)(b), (2) and (3): reduced by amounts already withheld under Subdivision 12-FB, cents disregarded",
            &final_wh,
        ));
        (gross_wh, final_wh)
    };

    let net = payment.sub(&already)?.sub(&withholding)?;
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
            ("withholding_before_prior", before_prior),
            ("withholding", withholding),
            ("net_payment", net),
        ],
        rates,
        steps,
        components: None,
    })
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
        "dividend" | "interest" | "royalty" => subdivision_12_f(payment_type, &payment, &facts),
        "mit_fund_payment" => subdivision_12_h(&payment, &facts),
        _ => subdivision_12_fb(payment_type, &payment, &year, &facts),
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
