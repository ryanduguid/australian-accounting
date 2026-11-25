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
    Arg, Result, Step, cents, d, derived_result, flag, int, precise, rate, refuse, step,
};
use super::{KIND, PortableLslLevyArgs, optional_money, reject_present};
use crate::decimal::{Decimal, Rounding};
use bonuses::{ParsedBonuses, parse_bonuses};
use branches::{work_annual_salary, work_base_rate, work_casual};

mod bonuses;
mod branches;

const LEVY_RATE_FROM_MONTH: &str = "2023-07";

const LEVY_AUTH: &str = "Coal Mining Industry (Long Service Leave) Payroll Levy Act 1992 s 5 and Payroll Levy Regulations 2018 s 6 (F2023C00557, in force from 1 July 2023, verified 24 November 2026): 2.7% of eligible wages, rounded half up to the nearest cent once at the final step";

struct CoalContext<'a> {
    bonuses: &'a ParsedBonuses,
    sacrificed: (&'a Decimal, bool),
    levy_rate: &'a Decimal,
}

struct CoalWorked {
    coal_branch: &'static str,
    winner: Option<&'static str>,
    eligible_wages: Decimal,
    levy_exact: Decimal,
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
}

struct PayField<'a> {
    name: &'a str,
    value: &'a Decimal,
    supplied: bool,
}

struct SimpleBranch<'a> {
    coal_branch: &'static str,
    auth: &'static str,
    formula: &'a str,
    loading: Option<&'a Decimal>,
}

fn parse_month_parts(month: &str) -> Result<(i32, u32)> {
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
    Ok((y, m))
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
    let (y, m) = parse_month_parts(month)?;
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

fn gross_up_step(
    id: &'static str,
    authority: &'static str,
    pay: PayField<'_>,
    sacrificed: (&Decimal, bool),
) -> Result<Step> {
    let (sac_val, sac_supplied) = sacrificed;
    let total = pay.value.add(sac_val)?;
    let s = match (pay.supplied, sac_supplied) {
        (true, true) => step(
            id,
            authority,
            &format!("{} + salary_sacrificed", pay.name),
            &[pay.value, sac_val],
            total,
        ),
        (true, false) => step(id, authority, pay.name, &[pay.value], total),
        (false, true) => step(id, authority, "salary_sacrificed", &[sac_val], total),
        (false, false) => step(id, authority, "0", &[], int(0)),
    };
    Ok(s)
}

fn finish_simple_worked(
    spec: SimpleBranch<'_>,
    bonus_s: Step,
    gross_s: Step,
    levy_rate: &Decimal,
) -> Result<CoalWorked> {
    let mut elig_val = gross_s.value.add(&bonus_s.value)?;
    let mut deps: Vec<&Decimal> = vec![&gross_s.value, &bonus_s.value];
    if let Some(load) = spec.loading {
        elig_val = elig_val.add(load)?;
        deps.push(load);
    }
    let eligible = step(
        "eligible_wages",
        spec.auth,
        spec.formula,
        &deps,
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
        coal_branch: spec.coal_branch,
        winner: None,
        eligible_wages: elig_val,
        levy_exact,
        steps: vec![bonus_s, gross_s, eligible, levy_s],
        rates: vec![rate("rate", levy_rate)],
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

fn parse_coal_header<'a>(
    args: &'a PortableLslLevyArgs,
    year: &str,
) -> Result<(&'a str, &'static str)> {
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
        Some("base_rate") => "base_rate",
        Some("annual_salary") => "annual_salary",
        Some("casual") => "casual",
        _ => {
            return refuse(
                "portable_lsl_levy.branch_invalid",
                "branch is required for Coal LSL and must be base_rate, annual_salary or casual.",
            );
        }
    };
    Ok((reporting_month, branch))
}

struct CoalOutput<'a> {
    year: &'a str,
    reporting_month: &'a str,
    eligible_exact: &'a str,
    levy_before: &'a str,
}

fn build_coal_output(
    meta: &CoalOutput<'_>,
    worked: CoalWorked,
    excluded: Vec<Value>,
) -> Result<Value> {
    let mut out = derived_result(KIND, meta.year, worked.steps, worked.rates)?;
    let mut details = Map::new();
    details.insert("jurisdiction".into(), json!("cth"));
    details.insert("scheme".into(), json!("coal"));
    details.insert("basis".into(), json!("eligible_wages"));
    details.insert("authority_board".into(), json!("Coal LSL"));
    details.insert("reporting_month".into(), json!(meta.reporting_month));
    details.insert("coal_branch".into(), json!(worked.coal_branch));
    if let Some(winner) = worked.winner {
        details.insert("winner".into(), json!(winner));
    }
    details.insert("eligible_wages_exact".into(), json!(meta.eligible_exact));
    details.insert("levy_before_rounding".into(), json!(meta.levy_before));
    details.insert("excluded_bonuses".into(), Value::Array(excluded));
    for (k, v) in details {
        out[k] = v;
    }
    Ok(out)
}

pub(super) fn calculate(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    let (reporting_month, branch) = parse_coal_header(args, year)?;
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

    let meta = CoalOutput {
        year,
        reporting_month,
        eligible_exact: &eligible_exact_str,
        levy_before: &levy_before_str,
    };
    build_coal_output(&meta, worked, bonuses.excluded)
}
