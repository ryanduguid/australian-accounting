// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Australian state and territory portable long service leave levies across
//! construction, community services, contract cleaning and security. Every
//! rate, threshold and statutory rounding rule was read from the administering
//! statutory board's pages and in-force legislation on 24 November 2026.

use serde_json::{Value, json};

use super::super::{
    Result, Step, cents, d, derived_result, flag, int, precise, rate, refuse, step,
};
use super::{KIND, PortableLslLevyArgs, dollars_half_down, reject_present, required_money};
use crate::decimal::{Decimal, Rounding};

struct PayrollRule {
    board: &'static str,
    years: &'static str,
    rate_str: &'static str,
    authority: &'static str,
}

struct StateMeta<'a> {
    year: &'a str,
    jurisdiction: &'a str,
    scheme: &'a str,
    basis: &'a str,
    board: &'a str,
}

const PAYROLL_ROWS: [&str; 14] = [
    "nsw|contract_cleaning|Long Service Corporation (NSW)|2023-24, 2024-25, 2025-26, 2026-27|0.01|Contract Cleaning Industry (Portable Long Service Leave Scheme) Act 2010 (NSW) and NSW Long Service Corporation 'Managing employer returns and paying levies in the contract cleaning industry' (read 24 November 2026): 1% of gross ordinary wages from 1 July 2021, rounded half up to the cent",
    "vic|construction|LeavePlus (Victoria)|2026-27|0.027|Construction Industry Long Service Leave Act 1997 (Vic) and LeavePlus 'Long service leave charges' (read 24 November 2026): 2.7% of gross reportable ordinary pay, rounded half up to the cent",
    "vic|community_services|Portable Long Service Authority (Victoria)|2026-27|0.0165|Long Service Benefits Portability Act 2018 (Vic) and Portable Long Service Authority 'About the Portable Long Service Benefits Scheme' (read 24 November 2026): 1.65% of ordinary pay for community services, rounded half up to the cent",
    "vic|contract_cleaning|Portable Long Service Authority (Victoria)|2026-27|0.018|Long Service Benefits Portability Act 2018 (Vic) and Portable Long Service Authority 'About the Portable Long Service Benefits Scheme' (read 24 November 2026): 1.80% of ordinary pay for contract cleaning, rounded half up to the cent",
    "vic|security|Portable Long Service Authority (Victoria)|2026-27|0.018|Long Service Benefits Portability Act 2018 (Vic) and Portable Long Service Authority 'About the Portable Long Service Benefits Scheme' (read 24 November 2026): 1.80% of ordinary pay for security, rounded half up to the cent",
    "qld|contract_cleaning|QLeave (Queensland)|2026-27|0.005|Contract Cleaning Industry (Portable Long Service Leave) Act 2005 (Qld) s 83 and QLeave 'Employer obligations' (read 24 November 2026): 0.5% of workers' ordinary wages, rounded half up to the cent",
    "qld|community_services|QLeave (Queensland)|2026-27|0.0135|Community Services Industry (Portable Long Service Leave) Act 2020 (Qld) and QLeave 'Paying the levy' (read 24 November 2026): 1.35% of workers' ordinary wages, rounded half up to the cent",
    "sa|construction|SA Portable Long Service Leave|2026-27|0.02|Construction Industry Long Service Leave Act 1987 (SA) s 26 and SA Portable Long Service Leave 'How to complete Employer Returns' (read 24 November 2026): 2.00% of declared gross remuneration (excluding levy-free apprentices), rounded half up to the cent",
    "sa|community_services|SA Portable Long Service Leave|2026-27|0.022|Portable Long Service Leave Act 2024 (SA) and SA Portable Long Service Leave 'What do I have to do?' (read 24 November 2026): 2.2% of gross remuneration (excluding exempt trainees), rounded half up to the cent",
    "wa|construction|MyLeave (Western Australia)|2026-27|0.0075|Construction Industry Portable Paid Long Service Leave Act 1985 (WA) s 34 and MyLeave public notice '2027 Employer Contribution Rate' (4 November 2026, read 24 November 2026): 0.75% of ordinary pay for the 2026 and 2027 calendar years (excluding apprentices), rounded half up to the cent",
    "act|construction|ACT Leave|2025-26, 2026-27|0.0275|Long Service Leave (Portable Schemes) Act 2009 (ACT) s 51 and Building and Construction Industry Employer Levy Determination 2024 (DI2024-12, in force from 1 October 2024, read 24 November 2026): 2.75% of gross ordinary wages, rounded half up to the cent",
    "act|community_services|ACT Leave|2024-25, 2025-26, 2026-27|0.0185|Long Service Leave (Portable Schemes) Act 2009 (ACT) s 56A and Community Sector Industry Employer Levy Determination 2024 (DI2024-16, in force from 1 July 2024, read 24 November 2026): 1.85% of gross ordinary wages, rounded half up to the cent",
    "act|security|ACT Leave|2024-25, 2025-26, 2026-27|0.0147|Long Service Leave (Portable Schemes) Act 2009 (ACT) s 56A and Security Industry Employer Levy Determination 2024 (DI2024-15, in force from 1 July 2024, read 24 November 2026): 1.47% of gross ordinary wages, rounded half up to the cent",
    "act|contract_cleaning|ACT Leave|2025-26, 2026-27|0.0107|Long Service Leave (Portable Schemes) Act 2009 (ACT) s 51 and Services Industry Employer Levy Determinations 2024 (DI2024-224, from 1 April 2025) and 2026 (DI2026-228, from 1 January 2027), read 24 November 2026: 1.07% of gross ordinary wages for contract cleaning under the Services Industry Scheme, rounded half up to the cent",
];

fn payroll_rule(jurisdiction: &str, scheme: &str, year: &str) -> Option<PayrollRule> {
    if (jurisdiction, scheme) == ("nsw", "community_services") {
        let (rate_str, authority) = if year == "2025-26" {
            (
                "0.013",
                "Community Services Sector (Portable Long Service Leave) Act 2024 (NSW) and NSW Long Service Corporation 'Making levy contributions as a community services employer' (read 24 November 2026): 1.3% transitional levy rate on gross ordinary wages for 2025-26, rounded half up to the cent",
            )
        } else {
            (
                "0.017",
                "Community Services Sector (Portable Long Service Leave) Act 2024 (NSW) and NSW Long Service Corporation 'Making levy contributions as a community services employer' (read 24 November 2026): 1.7% of gross ordinary wages from 1 July 2026, rounded half up to the cent",
            )
        };
        return Some(PayrollRule {
            board: "Long Service Corporation (NSW)",
            years: "2025-26, 2026-27",
            rate_str,
            authority,
        });
    }
    for row in PAYROLL_ROWS {
        let mut parts = row.split('|');
        if parts.next() == Some(jurisdiction) && parts.next() == Some(scheme) {
            return Some(PayrollRule {
                board: parts.next().expect("board"),
                years: parts.next().expect("years"),
                rate_str: parts.next().expect("rate"),
                authority: parts.next().expect("authority"),
            });
        }
    }
    None
}

fn check_year(jurisdiction: &str, scheme: &str, year: &str, allowed: &str) -> Result<()> {
    if allowed.split(", ").any(|y| y == year) {
        return Ok(());
    }
    refuse(
        "portable_lsl_levy.scheme_year_unsupported",
        format!(
            "{jurisdiction} {scheme} portable LSL levy rates are verified for {allowed}, not {year}."
        ),
    )
}

fn reject_coal_fields(args: &PortableLslLevyArgs, context: &str) -> Result<()> {
    reject_present(
        &[
            ("reporting_month", args.reporting_month),
            ("branch", args.branch),
            ("eligible_employee", args.eligible_employee),
            ("base_rate_of_pay", args.base_rate_of_pay),
            ("annual_salary_paid", args.annual_salary_paid),
            ("casual_loading", args.casual_loading),
            ("ordinary_rate_of_pay", args.ordinary_rate_of_pay),
            ("salary_sacrificed", args.salary_sacrificed),
            (
                "overtime_and_penalty_rates",
                args.overtime_and_penalty_rates,
            ),
            ("allowances", args.allowances),
            (
                "allowances_exclude_expense_reimbursements",
                args.allowances_exclude_expense_reimbursements,
            ),
            ("bonuses", args.bonuses),
            (
                "instrument_specifies_loading",
                args.instrument_specifies_loading,
            ),
            ("loading_quantifiable", args.loading_quantifiable),
        ],
        context,
    )
}

fn finish_state(
    meta: StateMeta<'_>,
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
) -> Result<Value> {
    let mut out = derived_result(KIND, meta.year, steps, rates)?;
    out["jurisdiction"] = json!(meta.jurisdiction);
    out["scheme"] = json!(meta.scheme);
    out["basis"] = json!(meta.basis);
    out["authority_board"] = json!(meta.board);
    Ok(out)
}

fn tas_rate_and_auth(on_time: bool) -> (&'static str, &'static str) {
    if on_time {
        (
            "0.018",
            "Construction Industry (Long Service) Act 1997 (Tas) s 7 and TasBuild 'Employers Information' FAQ (read 24 November 2026): 1.8% discounted rate on ordinary pay when returns and invoices are lodged and paid by the due dates, rounded half up to the cent",
        )
    } else {
        (
            "0.025",
            "Construction Industry (Long Service) Act 1997 (Tas) s 7 and TasBuild 'Employers Information' FAQ (read 24 November 2026): 2.5% standard contribution rate on ordinary pay, rounded half up to the cent",
        )
    }
}

fn calculate_tas_construction(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    check_year("tas", "construction", year, "2026-27")?;
    reject_present(
        &[
            ("project_cost", args.project_cost),
            ("partial_exemption", args.partial_exemption),
        ],
        "Tasmania construction portable LSL levy",
    )?;
    let Some(paid_arg) = args.paid_on_time else {
        return refuse(
            "input.missing_field",
            "paid_on_time is required for Tasmania construction (true for the 1.8% on-time discounted rate, false for the 2.5% standard rate).",
        );
    };
    let on_time = flag(paid_arg, "paid_on_time")?;
    let wages = required_money(
        args.ordinary_wages,
        "ordinary_wages",
        "Tasmania construction portable LSL levy",
    )?;
    let (rate_str, auth) = tas_rate_and_auth(on_time);
    let (steps, rates) = precise(|| {
        let applied = d(rate_str);
        let levy = cents(wages.mul(&applied)?)?;
        let s = step(
            "levy",
            auth,
            "cents(ordinary_wages x rate)",
            &[&wages, &applied],
            levy,
        );
        Ok((vec![s], vec![rate("rate", &applied)]))
    })?;
    finish_state(
        StateMeta {
            year,
            jurisdiction: "tas",
            scheme: "construction",
            basis: "ordinary_wages",
            board: "TasBuild (Tasmania)",
        },
        steps,
        rates,
    )
}

fn check_project_inputs(args: &PortableLslLevyArgs, context: &str) -> Result<Decimal> {
    reject_present(
        &[
            ("ordinary_wages", args.ordinary_wages),
            ("paid_on_time", args.paid_on_time),
        ],
        context,
    )?;
    if let Some(exempt) = args.partial_exemption
        && flag(exempt, "partial_exemption")?
    {
        return refuse(
            "portable_lsl_levy.exemption_unsupported",
            format!(
                "Partial or special exemptions (owner-builder, non-profit, church, Crown or local government reductions) are outside {context}."
            ),
        );
    }
    required_money(args.project_cost, "project_cost", context)
}

/// The worksheet's supported years, which `scope` holds `year` to, are the
/// years this levy was verified for: 2023-24 to 2026-27.
fn calculate_nsw_construction(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    let cost = check_project_inputs(args, "NSW building and construction levy")?;
    let (steps, rates) = precise(|| {
        let threshold = d("250000");
        let applied = d("0.0025");
        let s = if cost.lt(&threshold)? {
            step(
                "levy",
                "Building and Construction Industry Long Service Payments Act 1986 (NSW) and NSW Long Service Corporation 'About the long service levy' (read 24 November 2026): no levy is payable on works costing less than $250,000 (including GST)",
                "0",
                &[],
                int(0),
            )
        } else {
            let levy = cost.mul(&applied)?.quantize(0, Rounding::Down)?;
            step(
                "levy",
                "Building and Construction Industry Long Service Payments Act 1986 (NSW) and NSW Long Service Corporation 'About the long service levy for the building and construction industry' and Building Levy Calculator (verified 24 November 2026): 0.25% of the cost of works of $250,000 or more (including GST), rounded down to the whole dollar",
                "floor(project_cost x rate)",
                &[&cost, &applied],
                levy,
            )
        };
        Ok((
            vec![s],
            vec![rate("threshold", &threshold), rate("rate", &applied)],
        ))
    })?;
    finish_state(
        StateMeta {
            year,
            jurisdiction: "nsw",
            scheme: "construction",
            basis: "project_cost",
            board: "Long Service Corporation (NSW)",
        },
        steps,
        rates,
    )
}

fn qld_component_steps(
    cost: &Decimal,
    threshold: &Decimal,
    lsl_rate: &Decimal,
    training_rate: &Decimal,
    whs_rate: &Decimal,
) -> Result<(Step, Step, Step)> {
    if cost.lt(threshold)? {
        let below = "Building and Construction Industry (Portable Long Service Leave) Regulation 2024 (Qld) ss 9 and 11 (SL No. 162 of 2024, read 24 November 2026): levy is not payable when the cost of work (excluding GST, s 73(1)) is less than $150,000";
        return Ok((
            step("lsl_levy", below, "0", &[], int(0)),
            step("training_levy", below, "0", &[], int(0)),
            step("whs_levy", below, "0", &[], int(0)),
        ));
    }
    let lsl_val = dollars_half_down(&cost.mul(lsl_rate)?)?;
    let training_val = dollars_half_down(&cost.mul(training_rate)?)?;
    let whs_val = dollars_half_down(&cost.mul(whs_rate)?)?;
    Ok((
        step(
            "lsl_levy",
            "Building and Construction Industry (Portable Long Service Leave) Act 1991 (Qld) s 72(1)(b) and (2) and Regulation 2024 s 8(b): 0.35% of cost of work (excluding GST), rounded to the nearest dollar with 50c rounded downwards",
            "dollars_half_down(project_cost x lsl_rate)",
            &[cost, lsl_rate],
            lsl_val,
        ),
        step(
            "training_levy",
            "Building and Construction Industry (Portable Long Service Leave) Act 1991 (Qld) s 72(1)(a) and (2) and Regulation 2024 s 8(a): 0.1% CSQ training levy on cost of work (excluding GST), rounded to the nearest dollar with 50c rounded downwards",
            "dollars_half_down(project_cost x training_rate)",
            &[cost, training_rate],
            training_val,
        ),
        step(
            "whs_levy",
            "Building and Construction Industry (Portable Long Service Leave) Act 1991 (Qld) s 72(1)(c) and (2) and Regulation 2024 s 8(c): 0.125% WHSQ levy on cost of work (excluding GST), rounded to the nearest dollar with 50c rounded downwards",
            "dollars_half_down(project_cost x whs_rate)",
            &[cost, whs_rate],
            whs_val,
        ),
    ))
}

fn calculate_qld_construction(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    check_year("qld", "construction", year, "2025-26, 2026-27")?;
    let cost = check_project_inputs(args, "Queensland building and construction levy")?;
    let (steps, rates) = precise(|| {
        let threshold = d("150000");
        let lsl_rate = d("0.0035");
        let training_rate = d("0.001");
        let whs_rate = d("0.00125");
        let combined_rate = d("0.00575");
        let (lsl, training, whs) =
            qld_component_steps(&cost, &threshold, &lsl_rate, &training_rate, &whs_rate)?;
        let total_val = lsl.value.add(&training.value)?.add(&whs.value)?;
        let total = step(
            "levy",
            "Building and Construction Industry (Portable Long Service Leave) Act 1991 (Qld) s 72 and QLeave 'What is the levy?' (read 24 November 2026): combined QLeave portable long service leave, CSQ training and WHSQ levies payable",
            "lsl_levy + training_levy + whs_levy",
            &[&lsl.value, &training.value, &whs.value],
            total_val,
        );
        Ok((
            vec![lsl, training, whs, total],
            vec![
                rate("threshold", &threshold),
                rate("lsl_rate", &lsl_rate),
                rate("training_rate", &training_rate),
                rate("whs_rate", &whs_rate),
                rate("combined_rate", &combined_rate),
            ],
        ))
    })?;
    finish_state(
        StateMeta {
            year,
            jurisdiction: "qld",
            scheme: "construction",
            basis: "project_cost",
            board: "QLeave (Queensland)",
        },
        steps,
        rates,
    )
}

fn nt_levy_step(cost: &Decimal, threshold: &Decimal, applied: &Decimal) -> Result<Step> {
    if cost.lt(threshold)? {
        return Ok(step(
            "levy",
            "Construction Industry Long Service Leave and Benefits Act 2005 (NT) and NT Build 'Levy Payers' (read 24 November 2026): no levy is payable on construction projects with a total contract price under $1,000,000 (including GST)",
            "0",
            &[],
            int(0),
        ));
    }
    let levy = dollars_half_down(&cost.mul(applied)?)?;
    Ok(step(
        "levy",
        "Construction Industry Long Service Leave and Benefits Act 2005 (NT) s 33 and NT Build 'Levy Payers' (read 24 November 2026): 0.1% of total cost of construction work (including GST) from $1 million to $5 billion, rounded to the nearest dollar with 50c rounded downwards",
        "dollars_half_down(project_cost x rate)",
        &[cost, applied],
        levy,
    ))
}

fn calculate_nt_construction(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    check_year("nt", "construction", year, "2026-27")?;
    let cost = check_project_inputs(args, "Northern Territory construction levy")?;
    let upper = d("5000000000");
    if cost.gt(&upper)? {
        return refuse(
            "portable_lsl_levy.project_cost_above_tier",
            "Northern Territory construction projects over $5 billion are subject to a two-tier ministerial levy determination, which this worksheet does not calculate.",
        );
    }
    let (steps, rates) = precise(|| {
        let threshold = d("1000000");
        let applied = d("0.001");
        let s = nt_levy_step(&cost, &threshold, &applied)?;
        Ok((
            vec![s],
            vec![
                rate("threshold", &threshold),
                rate("upper_tier_limit", &upper),
                rate("rate", &applied),
            ],
        ))
    })?;
    finish_state(
        StateMeta {
            year,
            jurisdiction: "nt",
            scheme: "construction",
            basis: "project_cost",
            board: "NT Build (Northern Territory)",
        },
        steps,
        rates,
    )
}

fn finish_payroll_scheme(
    meta: StateMeta<'_>,
    rule: &PayrollRule,
    wages: &Decimal,
) -> Result<Value> {
    let (steps, rates) = precise(|| {
        let applied = d(rule.rate_str);
        let levy = cents(wages.mul(&applied)?)?;
        let s = step(
            "levy",
            rule.authority,
            "cents(ordinary_wages x rate)",
            &[wages, &applied],
            levy,
        );
        Ok((vec![s], vec![rate("rate", &applied)]))
    })?;
    finish_state(meta, steps, rates)
}

fn calculate_payroll_scheme(
    jurisdiction: &str,
    scheme: &str,
    args: &PortableLslLevyArgs,
    year: &str,
    context: &str,
) -> Result<Value> {
    let Some(rule) = payroll_rule(jurisdiction, scheme, year) else {
        return refuse(
            "portable_lsl_levy.scheme_unavailable",
            format!(
                "No portable long service leave scheme is covered for jurisdiction '{jurisdiction}' and scheme '{scheme}'."
            ),
        );
    };
    check_year(jurisdiction, scheme, year, rule.years)?;
    reject_present(
        &[
            ("project_cost", args.project_cost),
            ("partial_exemption", args.partial_exemption),
            ("paid_on_time", args.paid_on_time),
        ],
        context,
    )?;
    let wages = required_money(args.ordinary_wages, "ordinary_wages", context)?;
    finish_payroll_scheme(
        StateMeta {
            year,
            jurisdiction,
            scheme,
            basis: "ordinary_wages",
            board: rule.board,
        },
        &rule,
        &wages,
    )
}

pub(super) fn calculate(
    jurisdiction: &str,
    scheme: &str,
    args: &PortableLslLevyArgs,
    year: &str,
) -> Result<Value> {
    let context = format!("{jurisdiction} {scheme} portable LSL levy");
    reject_coal_fields(args, &context)?;
    match (jurisdiction, scheme) {
        ("nsw", "construction") => calculate_nsw_construction(args, year),
        ("qld", "construction") => calculate_qld_construction(args, year),
        ("nt", "construction") => calculate_nt_construction(args, year),
        ("tas", "construction") => calculate_tas_construction(args, year),
        _ => calculate_payroll_scheme(jurisdiction, scheme, args, year, &context),
    }
}
