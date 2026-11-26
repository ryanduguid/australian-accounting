// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Subdivisions 820-B and 820-C: financial entities (non-ADI) by their
//! maximum allowable debt.

use super::super::inputs::{Class, Deduction, not_used, required};
use super::super::{ThinCapitalisationArgs, Worked, max0, value_of};
use super::{
    all_or_none, dd_step, line, not_worked_out, only, proportionate, stated_flag, whole_year,
};
use crate::decimal::Decimal;
use crate::engine::{Result, Step, d, precise, rate, step};

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

/// The stated averages a case's method statements use.
struct Facts {
    debt_capital: Decimal,
    assoc_debt: Decimal,
    pe_assoc_debt: Option<Decimal>,
    borrowed: Decimal,
    cost_free: Decimal,
    assets: Decimal,
    excluded: Decimal,
    assoc_equity: Decimal,
    ndl: Decimal,
    zero: Decimal,
    excess_amount: Decimal,
    on_lent: Decimal,
    /// Controlled foreign entity debt and equity, for an outward entity.
    cfe: Option<(Decimal, Decimal)>,
    /// The worldwide figures, when stated.
    worldwide: Option<Vec<Decimal>>,
    audited: bool,
}

/// The authorities of a case's steps, in the order the steps are worked.
struct Auth {
    aad: &'static str,
    sh: &'static str,
    tda: &'static str,
    ola: &'static str,
    mad: &'static str,
    dis: &'static str,
}

fn uses(case: Fe) -> Vec<&'static str> {
    let mut uses: Vec<&str> = FE_COMMON.to_vec();
    if !matches!(case, Fe::Outward | Fe::OutwardVehicle) {
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
    uses
}

/// The worldwide figures, when stated, and whether audited consolidated
/// financial statements exist.
fn worldwide(a: &ThinCapitalisationArgs, case: Fe) -> Result<(Option<Vec<Decimal>>, bool)> {
    let gearing_why = "the worldwide gearing debt amount uses all of them";
    if case == Fe::Outward {
        return Ok((all_or_none(a, &AVERAGE_WORLDWIDE, gearing_why)?, true));
    }
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
    Ok((s, audited))
}

fn facts(a: &ThinCapitalisationArgs, case: Fe) -> Result<Facts> {
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
    let cfe = match case {
        Fe::Outward | Fe::OutwardVehicle => Some((
            need("average_controlled_foreign_entity_debt")?,
            need("average_controlled_foreign_entity_equity")?,
        )),
        _ => None,
    };
    let (worldwide, audited) = worldwide(a, case)?;
    Ok(Facts {
        debt_capital,
        assoc_debt,
        pe_assoc_debt,
        borrowed,
        cost_free,
        assets,
        excluded,
        assoc_equity,
        ndl,
        zero,
        excess_amount,
        on_lent,
        cfe,
        worldwide,
        audited,
    })
}

fn outward_auth(case: Fe) -> Auth {
    Auth {
        aad: "s 820-85(3): average debt capital less associate entity debt and controlled foreign entity debt, plus borrowed securities and cost-free debt capital",
        sh: "s 820-100(1): the lesser of the total debt amount and the adjusted on-lent amount",
        tda: "s 820-100(2) steps 1 to 7: average assets less excluded equity interests, associate entity debt and equity, controlled foreign entity debt and equity, non-debt liabilities and the zero-capital amount, nil if negative",
        ola: "s 820-100(3) steps 1 to 6: average assets less excluded equity interests, associate entity equity, controlled foreign entity debt and equity, non-debt liabilities and the on-lent amount, nil if negative",
        mad: if case == Fe::Outward {
            "s 820-90(1): the greatest of the safe harbour debt amount and (unless worldwide equity is nil or negative) the worldwide gearing debt amount"
        } else {
            "s 820-90(2): the greatest of the safe harbour debt amount and (unless s 820-90(3) applies) the worldwide gearing debt amount"
        },
        dis: "s 820-115(3): each debt deduction x excess debt / average debt",
    }
}

fn inward_auth(case: Fe) -> Auth {
    let mad = "s 820-190(1): the greatest of the safe harbour debt amount and (unless s 820-190(2) applies) the worldwide gearing debt amount";
    let dis = "s 820-220(3): each debt deduction x excess debt / average debt";
    if case == Fe::Vehicle {
        return Auth {
            aad: "s 820-185(3): average debt capital less associate entity debt, plus borrowed securities and cost-free debt capital",
            sh: "s 820-200(1): the lesser of the total debt amount and the adjusted on-lent amount",
            tda: "s 820-200(2) steps 1 to 5: average assets less excluded equity interests, associate entity debt and equity, non-debt liabilities and the zero-capital amount, nil if negative",
            ola: "s 820-200(3) steps 1 to 4: average assets less excluded equity interests, associate entity equity, non-debt liabilities and the on-lent amount, nil if negative",
            mad,
            dis,
        };
    }
    Auth {
        aad: "s 820-185(3): average debt capital less associate entity debt attributable to Australian permanent establishments, plus borrowed securities and cost-free debt capital",
        sh: "s 820-210(1): the lesser of the total debt amount and the adjusted on-lent amount",
        tda: "s 820-210(2) steps 1 to 5: the Australian investments less excluded equity interests and the associate entity debt and equity, non-debt liabilities and zero-capital amount arising from them, nil if negative",
        ola: "s 820-210(3) steps 1 to 4: the Australian investments less excluded equity interests, associate entity equity, non-debt liabilities and the on-lent amount, nil if negative",
        mad,
        dis,
    }
}

/// A financial entity's maximum allowable debt test (Subdivision 820-B or
/// 820-C).
pub(super) fn financial(
    a: &ThinCapitalisationArgs,
    c: Class,
    items: &[Deduction],
) -> Result<Worked> {
    let case = fe_case(a, c)?;
    only(
        a,
        &uses(case),
        "it is not in the method statements for the entity's case (ss 820-85 to 820-111 or 820-185 to 820-219)",
    )?;
    whole_year(a)?;
    let f = facts(a, case)?;
    let auth = match case {
        Fe::Outward | Fe::OutwardVehicle => outward_auth(case),
        Fe::Vehicle | Fe::Investor => inward_auth(case),
    };
    precise(|| {
        let mut steps = vec![dd_step(items)?, adjusted_average_debt(&f, auth.aad)?];
        let (base, tda) = total_debt(&mut steps, &f, auth.tda)?;
        let ola = on_lent(&mut steps, &f, case, auth.ola)?;
        let mad = maximum(&mut steps, case, &f, (&base, &tda, &ola), &auth)?;
        excess(&mut steps, &mad, f.worldwide.is_some())?;
        steps.push(step(
            "average_debt",
            "ss 820-115(3) and 820-220(3): average debt capital plus cost-free debt capital",
            "average_debt_capital + average_cost_free_debt_capital",
            &[&f.debt_capital, &f.cost_free],
            f.debt_capital.add(&f.cost_free)?,
        ));
        proportionate(&mut steps, "excess_debt", auth.dis)?;
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

fn adjusted_average_debt(f: &Facts, authority: &'static str) -> Result<Step> {
    let mut terms: Vec<(char, &str, &Decimal)> =
        vec![('+', "average_debt_capital", &f.debt_capital)];
    match &f.pe_assoc_debt {
        Some(v) => terms.push(('-', "average_pe_associate_entity_debt", v)),
        None => terms.push(('-', "average_associate_entity_debt", &f.assoc_debt)),
    }
    if let Some((cd, _)) = &f.cfe {
        terms.push(('-', "average_controlled_foreign_entity_debt", cd));
    }
    terms.push(('+', "average_borrowed_securities_amount", &f.borrowed));
    terms.push(('+', "average_cost_free_debt_capital", &f.cost_free));
    Ok(line("adjusted_average_debt", authority, &terms, false)?)
}

/// The total debt base and the total debt amount.
fn total_debt(
    steps: &mut Vec<Step>,
    f: &Facts,
    authority: &'static str,
) -> Result<(Decimal, Decimal)> {
    let mut terms: Vec<(char, &str, &Decimal)> = vec![
        ('+', "average_assets", &f.assets),
        ('-', "average_excluded_equity_interests", &f.excluded),
        ('-', "average_associate_entity_debt", &f.assoc_debt),
        ('-', "average_associate_entity_equity", &f.assoc_equity),
    ];
    if let Some((cd, ce)) = &f.cfe {
        terms.push(('-', "average_controlled_foreign_entity_debt", cd));
        terms.push(('-', "average_controlled_foreign_entity_equity", ce));
    }
    terms.push(('-', "average_non_debt_liabilities", &f.ndl));
    terms.push(('-', "average_zero_capital_amount", &f.zero));
    let base = line("total_debt_base", authority, &terms, true)?;
    let base_v = base.value.clone();
    steps.push(base);
    let tda = base_v
        .mul(&d("15"))?
        .div(&d("16"))?
        .add(&f.zero)?
        .add(&f.excess_amount)?;
    steps.push(step(
        "total_debt_amount",
        "the total debt amount's last steps: x 15/16, plus the zero-capital amount and the associate entity excess amount",
        "total_debt_base x 15 / 16 + average_zero_capital_amount + average_associate_entity_excess_amount",
        &[&base_v, &f.zero, &f.excess_amount],
        tda.clone(),
    ));
    Ok((base_v, tda))
}

/// The adjusted on-lent amount.
fn on_lent(steps: &mut Vec<Step>, f: &Facts, case: Fe, authority: &'static str) -> Result<Decimal> {
    let mut terms: Vec<(char, &str, &Decimal)> = vec![
        ('+', "average_assets", &f.assets),
        ('-', "average_excluded_equity_interests", &f.excluded),
        ('-', "average_associate_entity_equity", &f.assoc_equity),
    ];
    if let Some((cd, ce)) = &f.cfe {
        terms.push(('-', "average_controlled_foreign_entity_debt", cd));
        terms.push(('-', "average_controlled_foreign_entity_equity", ce));
    }
    terms.push(('-', "average_non_debt_liabilities", &f.ndl));
    terms.push(('-', "average_on_lent_amount", &f.on_lent));
    let ol_base = line("on_lent_base", authority, &terms, true)?;
    let ol_v = ol_base.value.clone();
    steps.push(ol_base);
    let after = ol_v
        .mul(&d("3"))?
        .div(&d("5"))?
        .add(&f.on_lent)?
        .sub(&f.assoc_debt)?;
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
    let ola = after.add(&f.excess_amount)?;
    steps.push(step(
        "adjusted_on_lent_amount",
        "the adjusted on-lent amount's last steps: x 3/5, plus the on-lent amount, less associate entity debt (nil if negative for an inward investor, s 820-210(3) step 7), plus the associate entity excess amount",
        formula,
        &[&ol_v, &f.on_lent, &f.assoc_debt, &f.excess_amount],
        ola.clone(),
    ));
    Ok(ola)
}

/// The safe harbour debt amount, the lesser of the total debt amount and the
/// adjusted on-lent amount, then the maximum allowable debt: the greatest of
/// it and the worldwide gearing debt amount, where available and stated.
fn maximum(
    steps: &mut Vec<Step>,
    case: Fe,
    f: &Facts,
    (base, tda, ola): (&Decimal, &Decimal, &Decimal),
    auth: &Auth,
) -> Result<Decimal> {
    let sh = if ola.lt(tda)? {
        ola.clone()
    } else {
        tda.clone()
    };
    steps.push(step(
        "safe_harbour_debt_amount",
        auth.sh,
        "min(total_debt_amount, adjusted_on_lent_amount)",
        &[tda, ola],
        sh.clone(),
    ));
    let gearing = match &f.worldwide {
        None => None,
        Some(v) => gearing(steps, case, v, f.audited, base, &f.zero, &f.excess_amount)?,
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
        auth.mad,
        formula,
        &operands,
        mad.clone(),
    ));
    Ok(mad)
}

/// Excess debt. Adjusted average debt of nil or less does not exceed the
/// maximum allowable debt (ss 820-85(4) and 820-185(4)).
fn excess(steps: &mut Vec<Step>, mad: &Decimal, worldwide_stated: bool) -> Result<()> {
    let aad = value_of(steps, "adjusted_average_debt").clone();
    let excess = if aad.gt(&d("0"))? {
        max0(aad.sub(mad)?)?
    } else {
        d("0")
    };
    if !excess.is_zero() && !worldwide_stated {
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
            &[&aad, mad],
            excess,
        )
    } else {
        step("excess_debt", excess_auth, "0", &[], excess)
    });
    Ok(())
}

/// The method statement that gives a case's worldwide gearing ratio.
fn gearing_authority(case: Fe) -> &'static str {
    match case {
        Fe::Outward => {
            "s 820-110: worldwide debt / worldwide equity, then step 1 / (1 + step 1) x total debt base, plus the zero-capital amount and the associate entity excess amount"
        }
        Fe::OutwardVehicle => {
            "s 820-111: statement worldwide debt / statement worldwide equity, then step 1 / (1 + step 1) x total debt base, plus the zero-capital amount and the associate entity excess amount"
        }
        Fe::Vehicle => {
            "s 820-217: statement worldwide debt / statement worldwide equity, then step 1 / (1 + step 1) x total debt base, plus the zero-capital amount and the associate entity excess amount"
        }
        Fe::Investor => {
            "s 820-219: statement worldwide debt / statement worldwide equity, then step 1 / (1 + step 1) x total debt base, plus the zero-capital amount and the associate entity excess amount"
        }
    }
}

/// Whether the worldwide gearing debt amount is available: positive
/// worldwide equity and, from statements, positive worldwide assets, audited
/// statements and Australian assets of no more than half (ss 820-90(1)(c),
/// 820-90(3) and 820-190(2)).
fn gearing_available(
    steps: &mut Vec<Step>,
    case: Fe,
    v: &[Decimal],
    audited: bool,
) -> Result<bool> {
    if !v[1].gt(&d("0"))? {
        return Ok(false);
    }
    if case == Fe::Outward {
        return Ok(true);
    }
    let (assets, australian) = (&v[2], &v[3]);
    if !assets.gt(&d("0"))? || !audited {
        return Ok(false);
    }
    let share = australian.div(assets)?;
    steps.push(step(
        "australian_assets_share",
        "ss 820-90(3)(c) and 820-190(2)(c): average Australian assets / statement worldwide assets; above 0.5 the worldwide gearing debt amount is not available",
        "statement_average_australian_assets / statement_worldwide_assets",
        &[australian, assets],
        share.clone(),
    ));
    Ok(!share.gt(&d("0.5"))?)
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
    if !gearing_available(steps, case, v, audited)? {
        return Ok(None);
    }
    let (debt, equity) = (&v[0], &v[1]);
    let ratio_names = if case == Fe::Outward {
        "average_worldwide_debt / average_worldwide_equity"
    } else {
        "statement_worldwide_debt / statement_worldwide_equity"
    };
    let authority = gearing_authority(case);
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
