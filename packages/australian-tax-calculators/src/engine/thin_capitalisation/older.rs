// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Subdivisions 820-B to 820-E: a financial entity (non-ADI) by its maximum
//! allowable debt (ss 820-85 to 820-115 and 820-185 to 820-220) or the third
//! party debt test it chose (ss 820-85(2C), 820-115(1), 820-185(2C) and
//! 820-220(1)), and an ADI by its minimum capital amount (ss 820-300 to
//! 820-325 and 820-395 to 820-415), from stated average values (Subdivision
//! 820-G). The arm's length capital amount (ss 820-315 and 820-410) is a
//! judgement the worksheet does not make: it is used only as stated, and a
//! shortfall it could remove is refused rather than estimated. Part-year
//! periods (ss 820-120, 820-225, 820-330 and 820-420) are refused. Read on
//! 27 November 2026 in ITAA 1997 compilation 268 (C2026C00471).

use super::inputs::{Class, Deduction, OLDER, Test, fact, not_used, optional_amount, required};
use super::{ALLOWED, ThinCapitalisationArgs, Worked, max0, sum_step, value_of};
use crate::decimal::{Calc, Decimal};
use crate::engine::{Result, Step, d, flag, precise, rate, refuse, step};

const DD: &str = "s 820-40: the year's debt deductions the Subdivision tests, after any disallowed under Subdivision 820-EAA (s 820-31)";

fn stated_flag(a: &ThinCapitalisationArgs, key: &str, why: &str) -> Result<bool> {
    match a.get(key) {
        Some(v) => flag(v, key),
        None => refuse(
            "thin_capitalisation.input_missing",
            format!("{key} is needed: {why}."),
        ),
    }
}

/// Refuses every input of Subdivisions 820-B to 820-E outside `uses`.
fn only(a: &ThinCapitalisationArgs, uses: &[&str], why: &str) -> Result<()> {
    match OLDER
        .iter()
        .find(|k| !uses.contains(k) && a.get(k).is_some())
    {
        Some(k) => not_used(k, why),
        None => Ok(()),
    }
}

/// `None` when none of `keys` is given, every value when all are, and a
/// refusal naming the first missing one otherwise.
fn all_or_none(
    a: &ThinCapitalisationArgs,
    keys: &[&str],
    why: &str,
) -> Result<Option<Vec<Decimal>>> {
    if keys.iter().all(|k| a.get(k).is_none()) {
        return Ok(None);
    }
    keys.iter()
        .map(|k| required(a, k, why))
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

/// A step adding and subtracting stated terms: `+` or `-` before each name
/// after the first. With `floor`, a negative result is taken to be nil.
fn line(
    id: &'static str,
    authority: &'static str,
    terms: &[(char, &str, &Decimal)],
    floor: bool,
) -> Calc<Step> {
    let mut formula = String::new();
    let mut value = d("0");
    for (n, (sign, name, v)) in terms.iter().enumerate() {
        if n > 0 {
            formula.push_str(&format!(" {sign} "));
        }
        formula.push_str(name);
        value = if *sign == '-' {
            value.sub(v)?
        } else {
            value.add(v)?
        };
    }
    if floor {
        formula = format!("max({formula}, 0)");
        value = max0(value)?;
    }
    let operands: Vec<&Decimal> = terms.iter().map(|(_, _, v)| *v).collect();
    Ok(step(id, authority, &formula, &operands, value))
}

fn dd_step(items: &[Deduction]) -> Calc<Step> {
    let amounts: Vec<&Decimal> = items.iter().map(|x| &x.amount).collect();
    sum_step("debt_deductions", DD, &amounts)
}

/// The disallowed amount under ss 820-115(3), 820-220(3), 820-325 and
/// 820-415: each debt deduction times the excess (or shortfall) over average
/// debt, and no more than all of it (ss 820-85(1), 820-185(1), 820-300(1) and
/// 820-395(1) disallow 'all or a part' of each).
fn proportionate(
    steps: &mut Vec<Step>,
    excess_id: &'static str,
    authority: &'static str,
) -> Result<()> {
    let dd = value_of(steps, "debt_deductions").clone();
    let excess = value_of(steps, excess_id).clone();
    let avg = value_of(steps, "average_debt").clone();
    if excess.is_zero() {
        steps.push(step("total_disallowed_amount", authority, "0", &[], d("0")));
    } else {
        if !avg.gt(&d("0"))? {
            return fact(format!(
                "average debt is nil, so the {excess_id} cannot be spread over it ({authority})."
            ));
        }
        let formula = format!("min(debt_deductions x {excess_id} / average_debt, debt_deductions)");
        let raw = dd.mul(&excess)?.div(&avg)?;
        let value = if raw.gt(&dd)? { dd.clone() } else { raw };
        steps.push(step(
            "total_disallowed_amount",
            authority,
            &formula,
            &[&dd, &excess, &avg, &dd],
            value,
        ));
    }
    let disallowed = value_of(steps, "total_disallowed_amount").clone();
    steps.push(step(
        "allowed_debt_deductions",
        ALLOWED,
        "debt_deductions - total_disallowed_amount",
        &[&dd, &disallowed],
        dd.sub(&disallowed)?,
    ));
    Ok(())
}

fn not_worked_out<T>(what: &str) -> Result<T> {
    refuse(
        "thin_capitalisation.amount_not_worked_out",
        what.to_string(),
    )
}

/// The part-year rules are not applied.
pub(super) fn whole_year(a: &ThinCapitalisationArgs) -> Result<()> {
    if stated_flag(
        a,
        "whole_year",
        "Subdivisions 820-B to 820-E apply to an entity in its class for all of the income year (ss 820-85(1A), 820-185(1A), 820-300(1) and 820-395(1))",
    )? {
        return Ok(());
    }
    refuse(
        "thin_capitalisation.part_year_not_built",
        "An entity in its class for only part of the income year is tested period by period under ss 820-120, 820-225, 820-330 or 820-420, which this worksheet does not apply.",
    )
}

// Subdivisions 820-B and 820-C: financial entities (non-ADI).

const FE_COMMON: [&str; 15] = [
    "whole_year",
    "inward_investment_vehicle",
    "average_debt_capital",
    "average_associate_entity_debt",
    "average_borrowed_securities_amount",
    "average_cost_free_debt_capital",
    "average_assets",
    "average_excluded_equity_interests",
    "average_associate_entity_equity",
    "average_non_debt_liabilities",
    "average_zero_capital_amount",
    "average_associate_entity_excess_amount",
    "average_on_lent_amount",
    "average_controlled_foreign_entity_debt",
    "average_controlled_foreign_entity_equity",
];
const AVERAGE_WORLDWIDE: [&str; 2] = ["average_worldwide_debt", "average_worldwide_equity"];
const STATEMENT: [&str; 4] = [
    "statement_worldwide_debt",
    "statement_worldwide_equity",
    "statement_worldwide_assets",
    "statement_average_australian_assets",
];

/// Which of the 4 financial entity cases applies.
#[derive(Clone, Copy, PartialEq)]
enum Fe {
    /// Outward, not also an inward investment vehicle (ss 820-100, 820-110).
    Outward,
    /// Outward and also an inward investment vehicle (ss 820-100, 820-111).
    OutwardVehicle,
    /// Inward investment vehicle (financial) (ss 820-200, 820-217).
    Vehicle,
    /// Inward investor (financial), a foreign entity (ss 820-210, 820-219).
    Investor,
}

fn fe_case(a: &ThinCapitalisationArgs, c: Class) -> Result<Fe> {
    let vehicle = stated_flag(
        a,
        "inward_investment_vehicle",
        "it settles which method statements apply (ss 820-90, 820-185(2) and 820-190)",
    )?;
    Ok(match (c, vehicle) {
        (Class::OutwardFinancial, false) => Fe::Outward,
        (Class::OutwardFinancial, true) => Fe::OutwardVehicle,
        (_, true) => Fe::Vehicle,
        (_, false) => Fe::Investor,
    })
}

/// A financial entity's third party debt test, chosen under s 820-85(2C) or
/// 820-185(2C).
pub(super) fn third_party(
    a: &ThinCapitalisationArgs,
    c: Class,
    items: &[Deduction],
) -> Result<Worked> {
    only(
        a,
        &["whole_year"],
        "the third party debt test compares debt deductions with the third party earnings limit alone (ss 820-115(1) and 820-220(1))",
    )?;
    whole_year(a)?;
    let authority = if c == Class::OutwardFinancial {
        "s 820-115(1): debt deductions, other than those attributable to an overseas permanent establishment, above the third party earnings limit"
    } else {
        "s 820-220(1): debt deductions above the third party earnings limit"
    };
    precise(|| {
        let qualifying: Vec<&Decimal> = items
            .iter()
            .filter(|x| x.third_party == Some(true))
            .map(|x| &x.amount)
            .collect();
        let dd = dd_step(items)?;
        let limit = sum_step(
            "third_party_earnings_limit",
            "s 820-427A(1) and (2): the debt deductions attributable to debt interests satisfying the third party debt conditions",
            &qualifying,
        )?;
        let disallowed = max0(dd.value.sub(&limit.value)?)?;
        let total = step(
            "total_disallowed_amount",
            authority,
            "max(debt_deductions - third_party_earnings_limit, 0)",
            &[&dd.value, &limit.value],
            disallowed.clone(),
        );
        let allowed = step(
            "allowed_debt_deductions",
            ALLOWED,
            "debt_deductions - total_disallowed_amount",
            &[&dd.value, &disallowed],
            dd.value.sub(&disallowed)?,
        );
        Ok(Worked {
            steps: vec![dd, limit, total, allowed],
            rates: vec![],
            carried: None,
        })
    })
}

/// A financial entity's maximum allowable debt test (Subdivision 820-B or
/// 820-C).
pub(super) fn financial(
    a: &ThinCapitalisationArgs,
    c: Class,
    items: &[Deduction],
) -> Result<Worked> {
    let case = fe_case(a, c)?;
    let outward = matches!(case, Fe::Outward | Fe::OutwardVehicle);
    let mut uses: Vec<&str> = FE_COMMON.to_vec();
    if !outward {
        uses.retain(|k| !k.contains("controlled_foreign_entity"));
    }
    if case == Fe::Investor {
        uses.push("average_pe_associate_entity_debt");
    }
    if case == Fe::Outward {
        uses.extend(AVERAGE_WORLDWIDE);
    } else {
        uses.extend(STATEMENT);
        uses.push("audited_consolidated_statements");
    }
    only(
        a,
        &uses,
        "it is not in the method statements for the entity's case (ss 820-85 to 820-111 or 820-185 to 820-219)",
    )?;
    whole_year(a)?;
    let need = |key: &str| required(a, key, "the method statement uses it; state 0.00 if none");
    let debt_capital = need("average_debt_capital")?;
    let assoc_debt = need("average_associate_entity_debt")?;
    let pe_assoc_debt = match case {
        Fe::Investor => Some(need("average_pe_associate_entity_debt")?),
        _ => None,
    };
    let borrowed = need("average_borrowed_securities_amount")?;
    let cost_free = need("average_cost_free_debt_capital")?;
    let assets = need("average_assets")?;
    let excluded = need("average_excluded_equity_interests")?;
    let assoc_equity = need("average_associate_entity_equity")?;
    let ndl = need("average_non_debt_liabilities")?;
    let zero = need("average_zero_capital_amount")?;
    let excess_amount = need("average_associate_entity_excess_amount")?;
    let on_lent = need("average_on_lent_amount")?;
    let (cfe_debt, cfe_equity) = if outward {
        (
            Some(need("average_controlled_foreign_entity_debt")?),
            Some(need("average_controlled_foreign_entity_equity")?),
        )
    } else {
        (None, None)
    };
    let gearing_why = "the worldwide gearing debt amount uses all of them";
    let (worldwide, audited) = if case == Fe::Outward {
        (all_or_none(a, &AVERAGE_WORLDWIDE, gearing_why)?, true)
    } else {
        let s = all_or_none(a, &STATEMENT, gearing_why)?;
        let audited = match (&s, a.get("audited_consolidated_statements")) {
            (Some(_), _) => stated_flag(
                a,
                "audited_consolidated_statements",
                "without audited consolidated financial statements the worldwide gearing debt amount is not available (ss 820-90(3)(b) and 820-190(2)(b))",
            )?,
            (None, Some(_)) => {
                return not_used(
                    "audited_consolidated_statements",
                    "it matters only with the statement worldwide figures (ss 820-90(3) and 820-190(2))",
                );
            }
            (None, None) => true,
        };
        (s, audited)
    };
    let (aad_auth, sh_auth, tda_auth, ola_auth, mad_auth, dis_auth) = match case {
        Fe::Outward | Fe::OutwardVehicle => (
            "s 820-85(3): average debt capital less associate entity debt and controlled foreign entity debt, plus borrowed securities and cost-free debt capital",
            "s 820-100(1): the lesser of the total debt amount and the adjusted on-lent amount",
            "s 820-100(2) steps 1 to 7: average assets less excluded equity interests, associate entity debt and equity, controlled foreign entity debt and equity, non-debt liabilities and the zero-capital amount, nil if negative",
            "s 820-100(3) steps 1 to 6: average assets less excluded equity interests, associate entity equity, controlled foreign entity debt and equity, non-debt liabilities and the on-lent amount, nil if negative",
            if case == Fe::Outward {
                "s 820-90(1): the greatest of the safe harbour debt amount and (unless worldwide equity is nil or negative) the worldwide gearing debt amount"
            } else {
                "s 820-90(2): the greatest of the safe harbour debt amount and (unless s 820-90(3) applies) the worldwide gearing debt amount"
            },
            "s 820-115(3): each debt deduction x excess debt / average debt",
        ),
        Fe::Vehicle | Fe::Investor => (
            if case == Fe::Vehicle {
                "s 820-185(3): average debt capital less associate entity debt, plus borrowed securities and cost-free debt capital"
            } else {
                "s 820-185(3): average debt capital less associate entity debt attributable to Australian permanent establishments, plus borrowed securities and cost-free debt capital"
            },
            if case == Fe::Vehicle {
                "s 820-200(1): the lesser of the total debt amount and the adjusted on-lent amount"
            } else {
                "s 820-210(1): the lesser of the total debt amount and the adjusted on-lent amount"
            },
            if case == Fe::Vehicle {
                "s 820-200(2) steps 1 to 5: average assets less excluded equity interests, associate entity debt and equity, non-debt liabilities and the zero-capital amount, nil if negative"
            } else {
                "s 820-210(2) steps 1 to 5: the Australian investments less excluded equity interests and the associate entity debt and equity, non-debt liabilities and zero-capital amount arising from them, nil if negative"
            },
            if case == Fe::Vehicle {
                "s 820-200(3) steps 1 to 4: average assets less excluded equity interests, associate entity equity, non-debt liabilities and the on-lent amount, nil if negative"
            } else {
                "s 820-210(3) steps 1 to 4: the Australian investments less excluded equity interests, associate entity equity, non-debt liabilities and the on-lent amount, nil if negative"
            },
            "s 820-190(1): the greatest of the safe harbour debt amount and (unless s 820-190(2) applies) the worldwide gearing debt amount",
            "s 820-220(3): each debt deduction x excess debt / average debt",
        ),
    };
    precise(|| {
        let mut steps = vec![dd_step(items)?];
        let mut terms: Vec<(char, &str, &Decimal)> =
            vec![('+', "average_debt_capital", &debt_capital)];
        match &pe_assoc_debt {
            Some(v) => terms.push(('-', "average_pe_associate_entity_debt", v)),
            None => terms.push(('-', "average_associate_entity_debt", &assoc_debt)),
        }
        if let Some(v) = &cfe_debt {
            terms.push(('-', "average_controlled_foreign_entity_debt", v));
        }
        terms.push(('+', "average_borrowed_securities_amount", &borrowed));
        terms.push(('+', "average_cost_free_debt_capital", &cost_free));
        steps.push(line("adjusted_average_debt", aad_auth, &terms, false)?);
        // Total debt amount.
        let mut terms: Vec<(char, &str, &Decimal)> = vec![
            ('+', "average_assets", &assets),
            ('-', "average_excluded_equity_interests", &excluded),
            ('-', "average_associate_entity_debt", &assoc_debt),
            ('-', "average_associate_entity_equity", &assoc_equity),
        ];
        if let (Some(cd), Some(ce)) = (&cfe_debt, &cfe_equity) {
            terms.push(('-', "average_controlled_foreign_entity_debt", cd));
            terms.push(('-', "average_controlled_foreign_entity_equity", ce));
        }
        terms.push(('-', "average_non_debt_liabilities", &ndl));
        terms.push(('-', "average_zero_capital_amount", &zero));
        let base = line("total_debt_base", tda_auth, &terms, true)?;
        let base_v = base.value.clone();
        steps.push(base);
        let tda = base_v
            .mul(&d("15"))?
            .div(&d("16"))?
            .add(&zero)?
            .add(&excess_amount)?;
        steps.push(step(
            "total_debt_amount",
            "the total debt amount's last steps: x 15/16, plus the zero-capital amount and the associate entity excess amount",
            "total_debt_base x 15 / 16 + average_zero_capital_amount + average_associate_entity_excess_amount",
            &[&base_v, &zero, &excess_amount],
            tda.clone(),
        ));
        // Adjusted on-lent amount.
        let mut terms: Vec<(char, &str, &Decimal)> = vec![
            ('+', "average_assets", &assets),
            ('-', "average_excluded_equity_interests", &excluded),
            ('-', "average_associate_entity_equity", &assoc_equity),
        ];
        if let (Some(cd), Some(ce)) = (&cfe_debt, &cfe_equity) {
            terms.push(('-', "average_controlled_foreign_entity_debt", cd));
            terms.push(('-', "average_controlled_foreign_entity_equity", ce));
        }
        terms.push(('-', "average_non_debt_liabilities", &ndl));
        terms.push(('-', "average_on_lent_amount", &on_lent));
        let ol_base = line("on_lent_base", ola_auth, &terms, true)?;
        let ol_v = ol_base.value.clone();
        steps.push(ol_base);
        let after = ol_v
            .mul(&d("3"))?
            .div(&d("5"))?
            .add(&on_lent)?
            .sub(&assoc_debt)?;
        let (formula, after) = if case == Fe::Investor {
            (
                "max(on_lent_base x 3 / 5 + average_on_lent_amount - average_associate_entity_debt, 0) + average_associate_entity_excess_amount",
                max0(after)?,
            )
        } else {
            (
                "on_lent_base x 3 / 5 + average_on_lent_amount - average_associate_entity_debt + average_associate_entity_excess_amount",
                after,
            )
        };
        let ola = after.add(&excess_amount)?;
        steps.push(step(
            "adjusted_on_lent_amount",
            "the adjusted on-lent amount's last steps: x 3/5, plus the on-lent amount, less associate entity debt (nil if negative for an inward investor, s 820-210(3) step 7), plus the associate entity excess amount",
            formula,
            &[&ol_v, &on_lent, &assoc_debt, &excess_amount],
            ola.clone(),
        ));
        let sh = if ola.lt(&tda)? {
            ola.clone()
        } else {
            tda.clone()
        };
        steps.push(step(
            "safe_harbour_debt_amount",
            sh_auth,
            "min(total_debt_amount, adjusted_on_lent_amount)",
            &[&tda, &ola],
            sh.clone(),
        ));
        // Worldwide gearing debt amount, where available and stated.
        let gearing = match &worldwide {
            None => None,
            Some(v) => gearing(&mut steps, case, v, audited, &base_v, &zero, &excess_amount)?,
        };
        let mad = match &gearing {
            Some(g) if g.gt(&sh)? => g.clone(),
            _ => sh.clone(),
        };
        let (formula, operands) = match &gearing {
            Some(g) => (
                "max(safe_harbour_debt_amount, worldwide_gearing_debt_amount)",
                vec![&sh, g],
            ),
            None => ("safe_harbour_debt_amount", vec![&sh]),
        };
        steps.push(step(
            "maximum_allowable_debt",
            mad_auth,
            formula,
            &operands,
            mad.clone(),
        ));
        let aad = value_of(&steps, "adjusted_average_debt").clone();
        // Adjusted average debt of nil or less does not exceed the maximum
        // allowable debt (ss 820-85(4) and 820-185(4)).
        let excess = if aad.gt(&d("0"))? {
            max0(aad.sub(&mad)?)?
        } else {
            d("0")
        };
        if !excess.is_zero() && worldwide.is_none() {
            return not_worked_out(
                "Adjusted average debt exceeds the safe harbour debt amount, and the maximum allowable debt is the greatest of it and the worldwide gearing debt amount (ss 820-90 and 820-190), which was not stated: the disallowed amount is not worked out. State the worldwide figures (ss 820-110, 820-111, 820-217 or 820-219).",
            );
        }
        let excess_auth = "ss 820-115(3) and 820-220(3): adjusted average debt above the maximum allowable debt; nil where adjusted average debt is nil or negative (ss 820-85(4) and 820-185(4))";
        steps.push(if aad.gt(&d("0"))? {
            step(
                "excess_debt",
                excess_auth,
                "max(adjusted_average_debt - maximum_allowable_debt, 0)",
                &[&aad, &mad],
                excess,
            )
        } else {
            step("excess_debt", excess_auth, "0", &[], excess)
        });
        steps.push(step(
            "average_debt",
            "ss 820-115(3) and 820-220(3): average debt capital plus cost-free debt capital",
            "average_debt_capital + average_cost_free_debt_capital",
            &[&debt_capital, &cost_free],
            debt_capital.add(&cost_free)?,
        ));
        proportionate(&mut steps, "excess_debt", dis_auth)?;
        Ok(Worked {
            steps,
            rates: vec![
                rate("total_debt_fraction", "15/16"),
                rate("on_lent_fraction", "3/5"),
            ],
            carried: None,
        })
    })
}

/// The worldwide gearing debt amount (ss 820-110, 820-111, 820-217 and
/// 820-219), or `None` where the Act makes it unavailable.
fn gearing(
    steps: &mut Vec<Step>,
    case: Fe,
    v: &[Decimal],
    audited: bool,
    base: &Decimal,
    zero: &Decimal,
    excess_amount: &Decimal,
) -> Result<Option<Decimal>> {
    let (debt, equity) = (&v[0], &v[1]);
    let (ratio_names, authority) = match case {
        Fe::Outward => {
            if !equity.gt(&d("0"))? {
                return Ok(None);
            }
            (
                "average_worldwide_debt / average_worldwide_equity",
                "s 820-110: worldwide debt / worldwide equity, then step 1 / (1 + step 1) x total debt base, plus the zero-capital amount and the associate entity excess amount",
            )
        }
        _ => {
            let (assets, australian) = (&v[2], &v[3]);
            if !equity.gt(&d("0"))? || !assets.gt(&d("0"))? || !audited {
                return Ok(None);
            }
            let share = australian.div(assets)?;
            steps.push(step(
                "australian_assets_share",
                "ss 820-90(3)(c) and 820-190(2)(c): average Australian assets / statement worldwide assets; above 0.5 the worldwide gearing debt amount is not available",
                "statement_average_australian_assets / statement_worldwide_assets",
                &[australian, assets],
                share.clone(),
            ));
            if share.gt(&d("0.5"))? {
                return Ok(None);
            }
            (
                "statement_worldwide_debt / statement_worldwide_equity",
                match case {
                    Fe::OutwardVehicle => {
                        "s 820-111: statement worldwide debt / statement worldwide equity, then step 1 / (1 + step 1) x total debt base, plus the zero-capital amount and the associate entity excess amount"
                    }
                    Fe::Vehicle => {
                        "s 820-217: statement worldwide debt / statement worldwide equity, then step 1 / (1 + step 1) x total debt base, plus the zero-capital amount and the associate entity excess amount"
                    }
                    _ => {
                        "s 820-219: statement worldwide debt / statement worldwide equity, then step 1 / (1 + step 1) x total debt base, plus the zero-capital amount and the associate entity excess amount"
                    }
                },
            )
        }
    };
    let ratio = debt.div(equity)?;
    steps.push(step(
        "worldwide_gearing_ratio",
        authority,
        ratio_names,
        &[debt, equity],
        ratio.clone(),
    ));
    let amount = ratio
        .div(&ratio.add(&d("1"))?)?
        .mul(base)?
        .add(zero)?
        .add(excess_amount)?;
    steps.push(step(
        "worldwide_gearing_debt_amount",
        authority,
        "worldwide_gearing_ratio / (1 + worldwide_gearing_ratio) x total_debt_base + average_zero_capital_amount + average_associate_entity_excess_amount",
        &[&ratio, &ratio, base, zero, excess_amount],
        amount.clone(),
    ));
    Ok(Some(amount))
}

// Subdivisions 820-D and 820-E: ADIs.

/// An ADI's minimum capital amount test (Subdivision 820-D or 820-E).
pub(super) fn adi(a: &ThinCapitalisationArgs, c: Class, items: &[Deduction]) -> Result<Worked> {
    let outward = c == Class::OutwardAdi;
    let mut uses = vec![
        "whole_year",
        "average_debt_capital",
        "average_adi_equity_capital",
        "average_risk_weighted_assets",
        "arm_length_capital_amount",
    ];
    let world = [
        "worldwide_capital_risk_weighted_assets",
        "worldwide_capital_tier_1_prudential_capital_deductions",
        "group_average_tier_1_capital",
        "group_average_risk_weighted_assets",
    ];
    if outward {
        uses.extend([
            "average_controlled_foreign_entity_equity",
            "average_tier_1_prudential_capital_deductions",
            "foreign_controlled_australian_entity",
        ]);
        uses.extend(world);
    } else {
        uses.push("average_interest_free_pe_loans");
    }
    only(
        a,
        &uses,
        "it is not in the provisions for the ADI's class (ss 820-300 to 820-325 or 820-395 to 820-415)",
    )?;
    whole_year(a)?;
    let need = |key: &str| required(a, key, "the provision uses it; state 0.00 if none");
    let debt_capital = need("average_debt_capital")?;
    let equity = need("average_adi_equity_capital")?;
    let rwa = need("average_risk_weighted_assets")?;
    let arm = optional_amount(a, "arm_length_capital_amount")?;
    let (second, t1) = if outward {
        (
            need("average_controlled_foreign_entity_equity")?,
            Some(need("average_tier_1_prudential_capital_deductions")?),
        )
    } else {
        (need("average_interest_free_pe_loans")?, None)
    };
    let worldwide = if outward {
        let controlled = stated_flag(
            a,
            "foreign_controlled_australian_entity",
            "the worldwide capital amount is not available to a foreign controlled Australian entity (s 820-320(1))",
        )?;
        let w = all_or_none(a, &world, "the worldwide capital amount uses all of them")?;
        if controlled && w.is_some() {
            return not_used(
                "worldwide_capital_risk_weighted_assets",
                "the worldwide capital amount does not apply to a foreign controlled Australian entity (s 820-320(1))",
            );
        }
        if let Some(w) = &w
            && !w[3].gt(&d("0"))?
        {
            return fact("group_average_risk_weighted_assets must be above nil (s 820-320(3)).");
        }
        Some((controlled, w))
    } else {
        None
    };
    precise(|| {
        let mut steps = vec![dd_step(items)?];
        let (eq_id, eq_auth, eq_formula) = if outward {
            (
                "adjusted_average_equity_capital",
                "s 820-300(3): ADI equity capital less controlled foreign entity equity, neither attributable to overseas permanent establishments",
                "average_adi_equity_capital - average_controlled_foreign_entity_equity",
            )
        } else {
            (
                "average_equity_capital",
                "s 820-395(3): ADI equity capital of the Australian permanent establishments not allocated to OB activities, plus the loans to them that give rise to no debt deductions",
                "average_adi_equity_capital + average_interest_free_pe_loans",
            )
        };
        let eq = if outward {
            equity.sub(&second)?
        } else {
            equity.add(&second)?
        };
        steps.push(step(
            eq_id,
            eq_auth,
            eq_formula,
            &[&equity, &second],
            eq.clone(),
        ));
        let six = d("0.06");
        let sh = match &t1 {
            Some(t1) => {
                let v = six.mul(&rwa)?.add(t1)?;
                steps.push(step(
                    "safe_harbour_capital_amount",
                    "s 820-310(1): 6% of risk-weighted assets and capitalised software intangibles (step 1's exclusions applied), plus tier 1 prudential capital deductions (step 3's exclusions applied)",
                    "safe_harbour_capital_rate x average_risk_weighted_assets + average_tier_1_prudential_capital_deductions",
                    &[&six, &rwa, t1],
                    v.clone(),
                ));
                v
            }
            None => {
                let v = six.mul(&rwa)?;
                steps.push(step(
                    "safe_harbour_capital_amount",
                    "s 820-405: 6% of the risk-weighted assets of the Australian permanent establishments not attributable to OB activities",
                    "safe_harbour_capital_rate x average_risk_weighted_assets",
                    &[&six, &rwa],
                    v.clone(),
                ));
                v
            }
        };
        let mut least = sh.clone();
        let mut names = vec!["safe_harbour_capital_amount"];
        let mut operands = vec![sh.clone()];
        let mut missing: Vec<&str> = Vec::new();
        if let Some((controlled, w)) = &worldwide {
            match w {
                Some(w) => {
                    let ratio = w[2].div(&w[3])?;
                    steps.push(step(
                        "worldwide_group_capital_ratio",
                        "s 820-320(3): the group's average tier 1 capital / its average risk-weighted assets, under the prudential standards",
                        "group_average_tier_1_capital / group_average_risk_weighted_assets",
                        &[&w[2], &w[3]],
                        ratio.clone(),
                    ));
                    let v = w[0].mul(&ratio)?.add(&w[1])?;
                    steps.push(step(
                        "worldwide_capital_amount",
                        "s 820-320(2): risk-weighted assets (step 1's exclusions applied) x the worldwide group capital ratio, plus tier 1 prudential capital deductions (step 4's exclusions applied)",
                        "worldwide_capital_risk_weighted_assets x worldwide_group_capital_ratio + worldwide_capital_tier_1_prudential_capital_deductions",
                        &[&w[0], &ratio, &w[1]],
                        v.clone(),
                    ));
                    if v.lt(&least)? {
                        least = v.clone();
                    }
                    names.push("worldwide_capital_amount");
                    operands.push(v);
                }
                None if !controlled => missing.push("the worldwide capital amount (s 820-320)"),
                None => {}
            }
        }
        match &arm {
            Some(v) => {
                if v.lt(&least)? {
                    least = v.clone();
                }
                names.push("arm_length_capital_amount");
                operands.push(v.clone());
            }
            None => missing.push(if outward {
                "the arm's length capital amount (s 820-315)"
            } else {
                "the arm's length capital amount (s 820-410)"
            }),
        }
        let formula = names
            .iter()
            .skip(1)
            .fold(names[0].to_string(), |f, n| format!("min({f}, {n})"));
        let refs: Vec<&Decimal> = operands.iter().collect();
        steps.push(step(
            "minimum_capital_amount",
            if outward {
                "s 820-305: the least of the safe harbour, arm's length and (unless a foreign controlled Australian entity) worldwide capital amounts"
            } else {
                "s 820-400: the lesser of the safe harbour and arm's length capital amounts"
            },
            &formula,
            &refs,
            least.clone(),
        ));
        let shortfall = max0(least.sub(&eq)?)?;
        if !shortfall.is_zero() && !missing.is_empty() {
            return not_worked_out(&format!(
                "The ADI's equity capital is below the amounts worked out, but the minimum capital amount is the least of them all and {} was not stated: a judgement this worksheet does not make, so the disallowed amount is not worked out.",
                missing.join(" and ")
            ));
        }
        steps.push(step(
            "capital_shortfall",
            if outward {
                "s 820-325: the amount by which adjusted average equity capital is less than the minimum capital amount"
            } else {
                "s 820-415: the amount by which average equity capital is less than the minimum capital amount"
            },
            &format!("max(minimum_capital_amount - {eq_id}, 0)"),
            &[&least, &eq],
            shortfall,
        ));
        steps.push(step(
            "average_debt",
            if outward {
                "s 820-325: average debt capital giving rise to debt deductions, other than that attributable to overseas permanent establishments"
            } else {
                "s 820-415: average debt capital giving rise to debt deductions other than allowable OB deductions"
            },
            "average_debt_capital",
            &[&debt_capital],
            debt_capital.clone(),
        ));
        proportionate(
            &mut steps,
            "capital_shortfall",
            if outward {
                "s 820-325: each debt deduction x capital shortfall / average debt"
            } else {
                "s 820-415: each debt deduction (other than an allowable OB deduction) x capital shortfall / average debt"
            },
        )?;
        Ok(Worked {
            steps,
            rates: vec![rate("safe_harbour_capital_rate", "0.06")],
            carried: None,
        })
    })
}

/// The test a financial entity or ADI is worked out by.
pub(super) fn work(
    a: &ThinCapitalisationArgs,
    c: Class,
    t: Test,
    items: &[Deduction],
) -> Result<Worked> {
    match t {
        Test::ThirdParty => third_party(a, c, items),
        _ if c.adi() => adi(a, c, items),
        _ => financial(a, c, items),
    }
}
