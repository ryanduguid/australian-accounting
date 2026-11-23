// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! National black coal mining industry portable LSL levy (Coal LSL), mirroring
//! `ryanduguid.github.io/tools/coal-lsl-levy` (`assets/levy.mjs` and
//! `service/coal-lsl-levy/src/calculate.mjs`):
//! - Eligible wages under s 3B of the Coal Mining Industry (Long Service Leave)
//!   Payroll Levy Collection Act 1992 (Compilation No 12, C2026C00364, in force
//!   1 September 2026; checked via Federal Register of Legislation API on 24
//!   November 2026).
//! - Prescribed levy rate of 2.7% under s 6 of the Coal Mining Industry (Long
//!   Service Leave) Payroll Levy Regulations 2018 (Compilation No 1,
//!   F2023C00557, effective from 1 July 2023; checked on 24 November 2026).

use serde_json::{Map, Value, json};

use super::super::{
    Arg, Result, Step, cents, d, derived_result, flag, int, money, precise, py_max, rate, refuse,
    step,
};
use super::{KIND, PortableLslLevyArgs, optional_money, reject_present, required_money};
use crate::decimal::{Decimal, Rounding};

const MAX_BONUSES: usize = 500;
const LEVY_RATE_FROM_MONTH: &str = "2023-07";
const CASUAL_METHOD_CHANGE_MONTH: &str = "2024-01";

const BONUSES_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Collection Act 1992 s 3B(4)(c) and (d) (C2026C00364): incentive payments and bonuses count only when paid at least monthly (weekly, fortnightly or monthly); less frequent amounts are dropped rather than spread";
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
const LEVY_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Act 1992 s 5 and Payroll Levy Regulations 2018 s 6 (F2023C00557, in force from 1 July 2023, verified 24 November 2026): 2.7% of eligible wages, rounded half up to the nearest cent once at the final step";

struct ParsedBonuses {
    counted: Vec<Decimal>,
    excluded: Vec<Value>,
}

fn validate_reporting_month<'a>(arg: Option<&'a Arg>, year: &str) -> Result<&'a str> {
    let Some(raw) = arg else {
        return refuse(
            "input.missing_field",
            "reporting_month is required for Coal LSL (YYYY-MM).",
        );
    };
    let Some(month) = raw.text() else {
        return refuse(
            "portable_lsl_levy.reporting_month_invalid",
            "reporting_month must be a YYYY-MM string (months 01 to 12).",
        );
    };
    let bytes = month.as_bytes();
    let valid_shape = bytes.len() == 7
        && bytes[4] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..].iter().all(u8::is_ascii_digit);
    if !valid_shape {
        return refuse(
            "portable_lsl_levy.reporting_month_invalid",
            format!("reporting_month '{month}' must be a YYYY-MM string (months 01 to 12)."),
        );
    }
    let y: i32 = month[..4].parse().expect("4 ASCII digits");
    let m: u32 = month[5..].parse().expect("2 ASCII digits");
    if !(1..=12).contains(&m) {
        return refuse(
            "portable_lsl_levy.reporting_month_invalid",
            format!("reporting_month '{month}' must have a month from 01 to 12."),
        );
    }
    if month < LEVY_RATE_FROM_MONTH {
        return refuse(
            "portable_lsl_levy.reporting_month_invalid",
            format!(
                "Reporting month {month} is not supported: the 2.7% levy rate under F2023L00772 applies to eligible wages paid on or after 1 July 2023."
            ),
        );
    }
    let fy_start: i32 = year[..4].parse().expect("financial year start");
    let in_fy = (y == fy_start && m >= 7) || (y == fy_start + 1 && m <= 6);
    if !in_fy {
        return refuse(
            "portable_lsl_levy.reporting_month_invalid",
            format!("reporting_month '{month}' does not fall within financial year '{year}'."),
        );
    }
    Ok(month)
}

fn parse_bonuses(arg: Option<&Arg>) -> Result<ParsedBonuses> {
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
        let parsed = obj
            .get("amount")
            .and_then(Value::as_str)
            .and_then(|t| Decimal::parse_input(t).ok())
            .map(|v| money(&Arg::Decimal(v)));
        let amount = match parsed {
            Some(Ok(v)) => v,
            _ => {
                return refuse(
                    "portable_lsl_levy.bonus_invalid",
                    format!(
                        "bonuses[{idx}].amount must be a finite non-negative decimal string, at most 2dp and 1e12."
                    ),
                );
            }
        };
        let freq = match obj.get("frequency").and_then(Value::as_str) {
            Some(
                f @ ("weekly" | "fortnightly" | "monthly" | "quarterly" | "half_yearly"
                | "halfYearly" | "annually"),
            ) => f,
            _ => {
                return refuse(
                    "portable_lsl_levy.bonus_invalid",
                    format!(
                        "bonuses[{idx}].frequency must be weekly, fortnightly, monthly, quarterly, half_yearly (or halfYearly) or annually."
                    ),
                );
            }
        };
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
    }
    Ok(ParsedBonuses { counted, excluded })
}

fn bonuses_step(counted: &[Decimal]) -> Result<Step> {
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

fn gross_up_step(
    id: &'static str,
    authority: &'static str,
    pay: (&str, &Decimal, bool),
    sacrificed: (&Decimal, bool),
) -> Result<Step> {
    let (pay_name, pay_val, pay_supplied) = pay;
    let (sac_val, sac_supplied) = sacrificed;
    let total = pay_val.add(sac_val)?;
    let s = match (pay_supplied, sac_supplied) {
        (true, true) => step(
            id,
            authority,
            &format!("{pay_name} + salary_sacrificed"),
            &[pay_val, sac_val],
            total,
        ),
        (true, false) => step(id, authority, pay_name, &[pay_val], total),
        (false, true) => step(id, authority, "salary_sacrificed", &[sac_val], total),
        (false, false) => step(id, authority, "0", &[], int(0)),
    };
    Ok(s)
}

struct CoalContext<'a> {
    bonuses: &'a ParsedBonuses,
    sacrificed: (&'a Decimal, bool),
    levy_rate: &'a Decimal,
}

struct CasualPay {
    base: (Decimal, bool),
    loading: (Decimal, bool),
    ordinary: (Decimal, bool),
}

struct CoalWorked {
    coal_branch: &'static str,
    winner: Option<&'static str>,
    eligible_wages: Decimal,
    levy_exact: Decimal,
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
}

fn work_base_rate(args: &PortableLslLevyArgs, ctx: &CoalContext) -> Result<CoalWorked> {
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
    )?;
    let base_pay = required_money(
        args.base_rate_of_pay,
        "base_rate_of_pay",
        "Coal LSL base_rate",
    )?;
    let (overtime, overtime_supplied) = optional_money(args.overtime_and_penalty_rates)?;
    let (allowances, allowances_supplied) = optional_money(args.allowances)?;
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

    let factor = d("0.75");
    let bonus_s = bonuses_step(&ctx.bonuses.counted)?;
    let gross_s = gross_up_step(
        "base_rate_grossed_up",
        GROSS_UP_BASE_AUTH,
        ("base_rate_of_pay", &base_pay, true),
        ctx.sacrificed,
    )?;
    let fa_val = gross_s.value.add(&bonus_s.value)?;
    let formula_a = step(
        "formula_a",
        FORMULA_A_AUTH,
        "base_rate_grossed_up + counted_bonuses",
        &[&gross_s.value, &bonus_s.value],
        fa_val,
    );

    let mut agg_terms = vec!["base_rate_grossed_up", "counted_bonuses"];
    let mut agg_ops: Vec<&Decimal> = vec![&gross_s.value, &bonus_s.value];
    let mut agg_val = formula_a.value.clone();
    if overtime_supplied {
        agg_terms.push("overtime_and_penalty_rates");
        agg_ops.push(&overtime);
        agg_val = agg_val.add(&overtime)?;
    }
    if allowances_supplied {
        agg_terms.push("allowances");
        agg_ops.push(&allowances);
        agg_val = agg_val.add(&allowances)?;
    }
    agg_ops.push(&factor);
    let fb_val = agg_val.mul(&factor)?;
    let fb_formula = format!("({}) x formula_b_factor", agg_terms.join(" + "));
    let formula_b = step("formula_b", FORMULA_B_AUTH, &fb_formula, &agg_ops, fb_val);

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
    let levy_exact = elig_val.mul(ctx.levy_rate)?;
    let levy_rounded = cents(levy_exact.clone())?;
    let levy_s = step(
        "levy",
        LEVY_AUTH,
        "cents(eligible_wages x rate)",
        &[&eligible.value, ctx.levy_rate],
        levy_rounded,
    );
    Ok(CoalWorked {
        coal_branch: "s 3B(1)",
        winner: Some(winner),
        eligible_wages: elig_val,
        levy_exact,
        steps: vec![bonus_s, gross_s, formula_a, formula_b, eligible, levy_s],
        rates: vec![
            rate("rate", ctx.levy_rate),
            rate("formula_b_factor", &factor),
        ],
    })
}

fn work_annual_salary(args: &PortableLslLevyArgs, ctx: &CoalContext) -> Result<CoalWorked> {
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
    )?;
    let salary = required_money(
        args.annual_salary_paid,
        "annual_salary_paid",
        "Coal LSL annual_salary",
    )?;
    let bonus_s = bonuses_step(&ctx.bonuses.counted)?;
    let gross_s = gross_up_step(
        "annual_salary_grossed_up",
        GROSS_UP_SALARY_AUTH,
        ("annual_salary_paid", &salary, true),
        ctx.sacrificed,
    )?;
    let elig_val = gross_s.value.add(&bonus_s.value)?;
    let eligible = step(
        "eligible_wages",
        ANNUAL_SALARY_ELIGIBLE_AUTH,
        "annual_salary_grossed_up + counted_bonuses",
        &[&gross_s.value, &bonus_s.value],
        elig_val.clone(),
    );
    let levy_exact = elig_val.mul(ctx.levy_rate)?;
    let levy_s = step(
        "levy",
        LEVY_AUTH,
        "cents(eligible_wages x rate)",
        &[&eligible.value, ctx.levy_rate],
        cents(levy_exact.clone())?,
    );
    Ok(CoalWorked {
        coal_branch: "s 3B(2)",
        winner: None,
        eligible_wages: elig_val,
        levy_exact,
        steps: vec![bonus_s, gross_s, eligible, levy_s],
        rates: vec![rate("rate", ctx.levy_rate)],
    })
}

fn work_casual_pre_2024(
    args: &PortableLslLevyArgs,
    pay: &CasualPay,
    ctx: &CoalContext,
) -> Result<CoalWorked> {
    if let Some(v) = args.instrument_specifies_loading {
        let _ = flag(v, "instrument_specifies_loading")?;
    }
    if let Some(v) = args.loading_quantifiable {
        let _ = flag(v, "loading_quantifiable")?;
    }
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
    let use_base = pay.base.0.gt(&zero)? || (!pay.ordinary.1 && pay.base.1);
    let (pay_tuple, auth) = if use_base {
        (
            ("base_rate_of_pay", &pay.base.0, pay.base.1),
            GROSS_UP_BASE_AUTH,
        )
    } else {
        (
            ("ordinary_rate_of_pay", &pay.ordinary.0, pay.ordinary.1),
            GROSS_UP_ORDINARY_AUTH,
        )
    };
    let gross_s = gross_up_step("pay_grossed_up", auth, pay_tuple, ctx.sacrificed)?;
    let elig_val = gross_s.value.add(&bonus_s.value)?;
    let eligible = step(
        "eligible_wages",
        CASUAL_PRE_2024_AUTH,
        "pay_grossed_up + counted_bonuses",
        &[&gross_s.value, &bonus_s.value],
        elig_val.clone(),
    );
    let levy_exact = elig_val.mul(ctx.levy_rate)?;
    let levy_s = step(
        "levy",
        LEVY_AUTH,
        "cents(eligible_wages x rate)",
        &[&eligible.value, ctx.levy_rate],
        cents(levy_exact.clone())?,
    );
    Ok(CoalWorked {
        coal_branch: "pre-2024",
        winner: None,
        eligible_wages: elig_val,
        levy_exact,
        steps: vec![bonus_s, gross_s, eligible, levy_s],
        rates: vec![rate("rate", ctx.levy_rate)],
    })
}

fn work_casual(
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
    let bonus_s = bonuses_step(&ctx.bonuses.counted)?;
    let zero = int(0);

    if specifies && quantifiable {
        if pay.ordinary.0.gt(&zero)? {
            return refuse(
                "portable_lsl_levy.branch_field_mismatch",
                "s 3B(3)(a) reads base_rate_of_pay and casual_loading, not ordinary_rate_of_pay; non-zero ordinary_rate_of_pay would be discarded.",
            );
        }
        let gross_s = gross_up_step(
            "base_rate_grossed_up",
            GROSS_UP_BASE_AUTH,
            ("base_rate_of_pay", &pay.base.0, pay.base.1),
            ctx.sacrificed,
        )?;
        let elig_val = gross_s.value.add(&bonus_s.value)?.add(&pay.loading.0)?;
        let eligible = if pay.loading.1 {
            step(
                "eligible_wages",
                CASUAL_3A_AUTH,
                "base_rate_grossed_up + counted_bonuses + casual_loading",
                &[&gross_s.value, &bonus_s.value, &pay.loading.0],
                elig_val.clone(),
            )
        } else {
            step(
                "eligible_wages",
                CASUAL_3A_AUTH,
                "base_rate_grossed_up + counted_bonuses",
                &[&gross_s.value, &bonus_s.value],
                elig_val.clone(),
            )
        };
        let levy_exact = elig_val.mul(ctx.levy_rate)?;
        let levy_s = step(
            "levy",
            LEVY_AUTH,
            "cents(eligible_wages x rate)",
            &[&eligible.value, ctx.levy_rate],
            cents(levy_exact.clone())?,
        );
        return Ok(CoalWorked {
            coal_branch: "s 3B(3)(a)",
            winner: None,
            eligible_wages: elig_val,
            levy_exact,
            steps: vec![bonus_s, gross_s, eligible, levy_s],
            rates: vec![rate("rate", ctx.levy_rate)],
        });
    }

    if pay.base.0.gt(&zero)? || pay.loading.0.gt(&zero)? {
        return refuse(
            "portable_lsl_levy.branch_field_mismatch",
            "s 3B(3)(b) reads ordinary_rate_of_pay only; non-zero base_rate_of_pay or casual_loading would be discarded.",
        );
    }
    let gross_s = gross_up_step(
        "ordinary_rate_grossed_up",
        GROSS_UP_ORDINARY_AUTH,
        ("ordinary_rate_of_pay", &pay.ordinary.0, pay.ordinary.1),
        ctx.sacrificed,
    )?;
    let elig_val = gross_s.value.add(&bonus_s.value)?;
    let eligible = step(
        "eligible_wages",
        CASUAL_3B_AUTH,
        "ordinary_rate_grossed_up + counted_bonuses",
        &[&gross_s.value, &bonus_s.value],
        elig_val.clone(),
    );
    let levy_exact = elig_val.mul(ctx.levy_rate)?;
    let levy_s = step(
        "levy",
        LEVY_AUTH,
        "cents(eligible_wages x rate)",
        &[&eligible.value, ctx.levy_rate],
        cents(levy_exact.clone())?,
    );
    Ok(CoalWorked {
        coal_branch: "s 3B(3)(b)",
        winner: None,
        eligible_wages: elig_val,
        levy_exact,
        steps: vec![bonus_s, gross_s, eligible, levy_s],
        rates: vec![rate("rate", ctx.levy_rate)],
    })
}

/// Format a non-negative `Decimal` with exactly `scale` fixed decimal places,
/// never in scientific notation (Python-style `Decimal::to_string()` prints
/// `"0E-8"` when `exponent < -6`).
fn format_fixed(value: &Decimal, scale: usize) -> crate::decimal::Calc<String> {
    let mut factor_str = String::with_capacity(scale + 1);
    factor_str.push('1');
    for _ in 0..scale {
        factor_str.push('0');
    }
    let scaled = value.mul(&d(&factor_str))?.quantize(0, Rounding::HalfUp)?;
    let digits = scaled.to_string();
    if digits.len() <= scale {
        let mut out = String::with_capacity(scale + 2);
        out.push_str("0.");
        for _ in 0..(scale - digits.len()) {
            out.push('0');
        }
        out.push_str(&digits);
        Ok(out)
    } else {
        let split = digits.len() - scale;
        Ok(format!("{}.{}", &digits[..split], &digits[split..]))
    }
}

pub(super) fn calculate(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    reject_present(
        &[
            ("ordinary_wages", args.ordinary_wages),
            ("paid_on_time", args.paid_on_time),
            ("project_cost", args.project_cost),
            ("partial_exemption", args.partial_exemption),
        ],
        "Coal LSL",
    )?;
    let reporting_month = validate_reporting_month(args.reporting_month, year)?;
    let eligible = match args.eligible_employee {
        Some(v) => flag(v, "eligible_employee")?,
        None => {
            return refuse(
                "input.missing_field",
                "eligible_employee is required for Coal LSL.",
            );
        }
    };
    if !eligible {
        return refuse(
            "portable_lsl_levy.employee_ineligible",
            "An employee who is not an eligible employee under s 4 of the Coal Mining Industry (Long Service Leave) Administration Act 1992 has no eligible wages under s 3B.",
        );
    }
    let branch = match args.branch.and_then(Arg::text) {
        Some(b @ ("base_rate" | "annual_salary" | "casual")) => b,
        _ => {
            return refuse(
                "portable_lsl_levy.branch_invalid",
                "branch is required for Coal LSL and must be base_rate, annual_salary or casual.",
            );
        }
    };
    let bonuses = parse_bonuses(args.bonuses)?;
    let (sacrificed, sacrificed_supplied) = optional_money(args.salary_sacrificed)?;

    let (worked, eligible_exact_str, levy_before_str) = precise(|| {
        let levy_rate = d("0.027");
        let ctx = CoalContext {
            bonuses: &bonuses,
            sacrificed: (&sacrificed, sacrificed_supplied),
            levy_rate: &levy_rate,
        };
        let worked = match branch {
            "base_rate" => work_base_rate(args, &ctx)?,
            "annual_salary" => work_annual_salary(args, &ctx)?,
            _ => work_casual(args, reporting_month, &ctx)?,
        };
        let eligible_exact_str = format_fixed(&worked.eligible_wages, 4)?;
        let levy_before_str = format_fixed(&worked.levy_exact, 8)?;
        Ok((worked, eligible_exact_str, levy_before_str))
    })?;

    let mut out = derived_result(KIND, year, worked.steps, worked.rates)?;
    let mut details = Map::new();
    details.insert("jurisdiction".into(), json!("cth"));
    details.insert("scheme".into(), json!("coal"));
    details.insert("basis".into(), json!("eligible_wages"));
    details.insert("authority_board".into(), json!("Coal LSL"));
    details.insert("reporting_month".into(), json!(reporting_month));
    details.insert("coal_branch".into(), json!(worked.coal_branch));
    if let Some(winner) = worked.winner {
        details.insert("winner".into(), json!(winner));
    }
    details.insert("eligible_wages_exact".into(), json!(eligible_exact_str));
    details.insert("levy_before_rounding".into(), json!(levy_before_str));
    details.insert("excluded_bonuses".into(), Value::Array(bonuses.excluded));
    for (k, v) in details {
        out[k] = v;
    }
    Ok(out)
}
