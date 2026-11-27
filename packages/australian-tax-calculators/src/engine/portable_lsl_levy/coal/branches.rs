// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::super::{PortableLslLevyArgs, optional_money, reject_present, required_money};
use super::bonuses::bonuses_step;
use super::{
    CoalContext, CoalWorked, LEVY_AUTH, PayField, SimpleBranch, finish_simple_worked, gross_up_step,
};
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, Step, cents, d, flag, int, py_max, rate, refuse, step};

const CASUAL_METHOD_CHANGE_MONTH: &str = "2024-01";

const GROSS_UP_BASE_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(4)(a) (C2026C00364): base rate of pay grossed up for salary sacrificed amounts";
const GROSS_UP_SALARY_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(4)(b) (C2026C00364): annual salary paid grossed up for salary sacrificed amounts";
const GROSS_UP_ORDINARY_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(4)(aa) (C2026C00364): ordinary rate of pay grossed up for salary sacrificed amounts";
const FORMULA_A_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(1)(a) (C2026C00364): Formula A is the grossed-up base rate of pay plus at-least-monthly incentive payments and bonuses";
const FORMULA_B_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(1)(b) (C2026C00364): Formula B is 75% of the sum of the grossed-up base rate of pay, at-least-monthly bonuses, overtime and penalty rates and allowances other than expense reimbursements, kept exact in quarter cents";
const BASE_RATE_ELIGIBLE_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(1) (C2026C00364): eligible wages for a base-rate employee are the greater of Formula A and Formula B";
const ANNUAL_SALARY_ELIGIBLE_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(2) (C2026C00364): eligible wages for an annual-salary employee are the grossed-up annual salary paid plus at-least-monthly bonuses; overtime, penalty rates and shift loading are excluded";
const CASUAL_3A_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(3)(a) (C2026C00364, from 1 January 2024): eligible wages for a casual employee with a quantifiable casual loading specified by the instrument are the grossed-up base rate of pay plus at-least-monthly bonuses plus casual loading";
const CASUAL_3B_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(3)(b) (C2026C00364, from 1 January 2024): eligible wages for a casual employee whose loading is not specified or not quantifiable are the grossed-up ordinary rate of pay plus at-least-monthly bonuses";
const CASUAL_PRE_2024_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B (as in force before 1 January 2024): legacy casual eligible wages are the grossed-up rate of pay plus at-least-monthly bonuses, excluding casual loading";

struct CasualPay {
    base: (Decimal, bool),
    loading: (Decimal, bool),
    ordinary: (Decimal, bool),
}

fn check_allowances_flag(args: &PortableLslLevyArgs, allowances: &Decimal) -> Result<()> {
    if allowances.gt(&int(0))? {
        match args.allowances_exclude_expense_reimbursements {
            Some(Arg::Bool(true)) => {}
            _ => {
                return refuse(
                    "portable_lsl_levy.reimbursements_unconfirmed",
                    "allowances_exclude_expense_reimbursements must be true when allowances are supplied: expense reimbursements are excluded by s 3B(1)(b)(iii) and are not separated by this worksheet.",
                );
            }
        }
    } else if let Some(v) = args.allowances_exclude_expense_reimbursements {
        let _ = flag(v, "allowances_exclude_expense_reimbursements")?;
    }
    Ok(())
}

struct FormulaBExtras<'a> {
    overtime: &'a (Decimal, bool),
    allowances: &'a (Decimal, bool),
}

fn formula_b_step(
    gross_val: &Decimal,
    bonus_val: &Decimal,
    extras: FormulaBExtras<'_>,
    factor: &Decimal,
) -> Result<Step> {
    let mut agg_terms = vec!["base_rate_grossed_up", "counted_bonuses"];
    let mut agg_ops: Vec<&Decimal> = vec![gross_val, bonus_val];
    let mut agg_val = gross_val.add(bonus_val)?;
    if extras.overtime.1 {
        agg_terms.push("overtime_and_penalty_rates");
        agg_ops.push(&extras.overtime.0);
        agg_val = agg_val.add(&extras.overtime.0)?;
    }
    if extras.allowances.1 {
        agg_terms.push("allowances");
        agg_ops.push(&extras.allowances.0);
        agg_val = agg_val.add(&extras.allowances.0)?;
    }
    agg_ops.push(factor);
    let fb_val = agg_val.mul(factor)?;
    let fb_formula = format!("({}) x formula_b_factor", agg_terms.join(" + "));
    Ok(step(
        "formula_b",
        FORMULA_B_AUTH,
        &fb_formula,
        &agg_ops,
        fb_val,
    ))
}

fn finish_base_rate_worked(
    bonus_s: Step,
    gross_s: Step,
    formula_a: Step,
    formula_b: Step,
    levy_rate: &Decimal,
) -> Result<CoalWorked> {
    let factor = d("0.75");
    let winner = if formula_b.value.gt(&formula_a.value)? {
        "formula_b"
    } else {
        "formula_a"
    };
    let elig_val = py_max(formula_a.value.clone(), formula_b.value.clone())?;
    let eligible = step(
        "eligible_wages",
        BASE_RATE_ELIGIBLE_AUTH,
        "max(formula_a, formula_b)",
        &[&formula_a.value, &formula_b.value],
        elig_val.clone(),
    );
    let levy_exact = elig_val.mul(levy_rate)?;
    let levy_s = step(
        "levy",
        LEVY_AUTH,
        "cents(eligible_wages x rate)",
        &[&eligible.value, levy_rate],
        cents(levy_exact.clone())?,
    );
    Ok(CoalWorked {
        coal_branch: "s 3B(1)",
        winner: Some(winner),
        eligible_wages: elig_val,
        levy_exact,
        steps: vec![bonus_s, gross_s, formula_a, formula_b, eligible, levy_s],
        rates: vec![rate("rate", levy_rate), rate("formula_b_factor", &factor)],
    })
}

fn reject_base_rate_fields(args: &PortableLslLevyArgs) -> Result<()> {
    reject_present(
        &[
            ("annual_salary_paid", args.annual_salary_paid),
            ("casual_loading", args.casual_loading),
            ("ordinary_rate_of_pay", args.ordinary_rate_of_pay),
            (
                "instrument_specifies_loading",
                args.instrument_specifies_loading,
            ),
            ("loading_quantifiable", args.loading_quantifiable),
        ],
        "Coal LSL base_rate branch",
    )
}

pub(super) fn work_base_rate(args: &PortableLslLevyArgs, ctx: &CoalContext) -> Result<CoalWorked> {
    reject_base_rate_fields(args)?;
    let base_pay = required_money(
        args.base_rate_of_pay,
        "base_rate_of_pay",
        "Coal LSL base_rate",
    )?;
    let overtime = optional_money(args.overtime_and_penalty_rates)?;
    let allowances = optional_money(args.allowances)?;
    check_allowances_flag(args, &allowances.0)?;

    let factor = d("0.75");
    let bonus_s = bonuses_step(&ctx.bonuses.counted)?;
    let gross_s = gross_up_step(
        "base_rate_grossed_up",
        GROSS_UP_BASE_AUTH,
        PayField {
            name: "base_rate_of_pay",
            value: &base_pay,
            supplied: true,
        },
        ctx.sacrificed,
    );
    let gross_s = gross_s?;
    let fa_val = gross_s.value.add(&bonus_s.value)?;
    let formula_a = step(
        "formula_a",
        FORMULA_A_AUTH,
        "base_rate_grossed_up + counted_bonuses",
        &[&gross_s.value, &bonus_s.value],
        fa_val,
    );
    let extras = FormulaBExtras {
        overtime: &overtime,
        allowances: &allowances,
    };
    let formula_b = formula_b_step(&gross_s.value, &bonus_s.value, extras, &factor)?;
    finish_base_rate_worked(bonus_s, gross_s, formula_a, formula_b, ctx.levy_rate)
}

fn reject_annual_salary_fields(args: &PortableLslLevyArgs) -> Result<()> {
    reject_present(
        &[
            ("base_rate_of_pay", args.base_rate_of_pay),
            (
                "overtime_and_penalty_rates",
                args.overtime_and_penalty_rates,
            ),
            ("allowances", args.allowances),
            (
                "allowances_exclude_expense_reimbursements",
                args.allowances_exclude_expense_reimbursements,
            ),
            ("casual_loading", args.casual_loading),
            ("ordinary_rate_of_pay", args.ordinary_rate_of_pay),
            (
                "instrument_specifies_loading",
                args.instrument_specifies_loading,
            ),
            ("loading_quantifiable", args.loading_quantifiable),
        ],
        "Coal LSL annual_salary branch",
    )
}

pub(super) fn work_annual_salary(
    args: &PortableLslLevyArgs,
    ctx: &CoalContext,
) -> Result<CoalWorked> {
    reject_annual_salary_fields(args)?;
    let salary = required_money(
        args.annual_salary_paid,
        "annual_salary_paid",
        "Coal LSL annual_salary",
    )?;
    let bonus_s = bonuses_step(&ctx.bonuses.counted)?;
    let gross_s = gross_up_step(
        "annual_salary_grossed_up",
        GROSS_UP_SALARY_AUTH,
        PayField {
            name: "annual_salary_paid",
            value: &salary,
            supplied: true,
        },
        ctx.sacrificed,
    );
    let gross_s = gross_s?;
    finish_simple_worked(
        SimpleBranch {
            coal_branch: "s 3B(2)",
            auth: ANNUAL_SALARY_ELIGIBLE_AUTH,
            formula: "annual_salary_grossed_up + counted_bonuses",
            loading: None,
        },
        bonus_s,
        gross_s,
        ctx.levy_rate,
    )
}

fn select_pre_2024_pay<'a>(
    pay: &'a CasualPay,
    zero: &Decimal,
) -> Result<(PayField<'a>, &'static str)> {
    let use_base = pay.base.0.gt(zero)? || (!pay.ordinary.1 && pay.base.1);
    Ok(if use_base {
        (
            PayField {
                name: "base_rate_of_pay",
                value: &pay.base.0,
                supplied: pay.base.1,
            },
            GROSS_UP_BASE_AUTH,
        )
    } else {
        (
            PayField {
                name: "ordinary_rate_of_pay",
                value: &pay.ordinary.0,
                supplied: pay.ordinary.1,
            },
            GROSS_UP_ORDINARY_AUTH,
        )
    })
}

fn work_casual_pre_2024(
    args: &PortableLslLevyArgs,
    pay: &CasualPay,
    ctx: &CoalContext,
) -> Result<CoalWorked> {
    reject_present(
        &[
            (
                "instrument_specifies_loading",
                args.instrument_specifies_loading,
            ),
            ("loading_quantifiable", args.loading_quantifiable),
        ],
        "Coal LSL pre-2024 casual branch",
    )?;
    let zero = int(0);
    if pay.loading.0.gt(&zero)? {
        return refuse(
            "portable_lsl_levy.branch_field_mismatch",
            "casual_loading is not part of eligible wages for reporting months before January 2024 and cannot be non-zero.",
        );
    }
    if pay.base.0.gt(&zero)? && pay.ordinary.0.gt(&zero)? {
        return refuse(
            "portable_lsl_levy.branch_field_mismatch",
            "Only one pay figure (base_rate_of_pay or ordinary_rate_of_pay) is read for pre-2024 casual months; both were non-zero.",
        );
    }
    let bonus_s = bonuses_step(&ctx.bonuses.counted)?;
    let (field, auth) = select_pre_2024_pay(pay, &zero)?;
    let gross_s = gross_up_step("pay_grossed_up", auth, field, ctx.sacrificed)?;
    finish_simple_worked(
        SimpleBranch {
            coal_branch: "pre-2024",
            auth: CASUAL_PRE_2024_AUTH,
            formula: "pay_grossed_up + counted_bonuses",
            loading: None,
        },
        bonus_s,
        gross_s,
        ctx.levy_rate,
    )
}

fn parse_casual_2024_flags(args: &PortableLslLevyArgs) -> Result<(bool, bool)> {
    let specifies = match args.instrument_specifies_loading {
        Some(v) => flag(v, "instrument_specifies_loading")?,
        None => {
            return refuse(
                "input.missing_field",
                "instrument_specifies_loading is required for Coal LSL casual months from January 2024.",
            );
        }
    };
    let quantifiable = match args.loading_quantifiable {
        Some(v) => flag(v, "loading_quantifiable")?,
        None => {
            return refuse(
                "input.missing_field",
                "loading_quantifiable is required for Coal LSL casual months from January 2024.",
            );
        }
    };
    Ok((specifies, quantifiable))
}

fn work_casual_2024_3a(pay: &CasualPay, bonus_s: Step, ctx: &CoalContext) -> Result<CoalWorked> {
    if pay.ordinary.0.gt(&int(0))? {
        return refuse(
            "portable_lsl_levy.branch_field_mismatch",
            "s 3B(3)(a) reads base_rate_of_pay and casual_loading, not ordinary_rate_of_pay; non-zero ordinary_rate_of_pay would be discarded.",
        );
    }
    let gross_s = gross_up_step(
        "base_rate_grossed_up",
        GROSS_UP_BASE_AUTH,
        PayField {
            name: "base_rate_of_pay",
            value: &pay.base.0,
            supplied: pay.base.1,
        },
        ctx.sacrificed,
    );
    let gross_s = gross_s?;
    let formula = if pay.loading.1 {
        "base_rate_grossed_up + counted_bonuses + casual_loading"
    } else {
        "base_rate_grossed_up + counted_bonuses"
    };
    finish_simple_worked(
        SimpleBranch {
            coal_branch: "s 3B(3)(a)",
            auth: CASUAL_3A_AUTH,
            formula,
            loading: pay.loading.1.then_some(&pay.loading.0),
        },
        bonus_s,
        gross_s,
        ctx.levy_rate,
    )
}

fn work_casual_2024_3b(pay: &CasualPay, bonus_s: Step, ctx: &CoalContext) -> Result<CoalWorked> {
    let zero = int(0);
    if pay.base.0.gt(&zero)? || pay.loading.0.gt(&zero)? {
        return refuse(
            "portable_lsl_levy.branch_field_mismatch",
            "s 3B(3)(b) reads ordinary_rate_of_pay only; non-zero base_rate_of_pay or casual_loading would be discarded.",
        );
    }
    let gross_s = gross_up_step(
        "ordinary_rate_grossed_up",
        GROSS_UP_ORDINARY_AUTH,
        PayField {
            name: "ordinary_rate_of_pay",
            value: &pay.ordinary.0,
            supplied: pay.ordinary.1,
        },
        ctx.sacrificed,
    );
    let gross_s = gross_s?;
    finish_simple_worked(
        SimpleBranch {
            coal_branch: "s 3B(3)(b)",
            auth: CASUAL_3B_AUTH,
            formula: "ordinary_rate_grossed_up + counted_bonuses",
            loading: None,
        },
        bonus_s,
        gross_s,
        ctx.levy_rate,
    )
}

pub(super) fn work_casual(
    args: &PortableLslLevyArgs,
    reporting_month: &str,
    ctx: &CoalContext,
) -> Result<CoalWorked> {
    reject_present(
        &[
            ("annual_salary_paid", args.annual_salary_paid),
            (
                "overtime_and_penalty_rates",
                args.overtime_and_penalty_rates,
            ),
            ("allowances", args.allowances),
            (
                "allowances_exclude_expense_reimbursements",
                args.allowances_exclude_expense_reimbursements,
            ),
        ],
        "Coal LSL casual branch",
    )?;
    let pay = CasualPay {
        base: optional_money(args.base_rate_of_pay)?,
        loading: optional_money(args.casual_loading)?,
        ordinary: optional_money(args.ordinary_rate_of_pay)?,
    };
    if reporting_month < CASUAL_METHOD_CHANGE_MONTH {
        return work_casual_pre_2024(args, &pay, ctx);
    }
    let (specifies, quantifiable) = parse_casual_2024_flags(args)?;
    let bonus_s = bonuses_step(&ctx.bonuses.counted)?;
    if specifies && quantifiable {
        work_casual_2024_3a(&pay, bonus_s, ctx)
    } else {
        work_casual_2024_3b(&pay, bonus_s, ctx)
    }
}
