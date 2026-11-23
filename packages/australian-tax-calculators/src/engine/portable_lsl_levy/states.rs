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
    years: &'static [&'static str],
    rate_str: &'static str,
    authority: &'static str,
}

fn payroll_rule(jurisdiction: &str, scheme: &str, year: &str) -> Option<PayrollRule> {
    let rule = match (jurisdiction, scheme) {
        ("nsw", "contract_cleaning") => PayrollRule {
            board: "Long Service Corporation (NSW)",
            years: &["2023-24", "2024-25", "2025-26", "2026-27"],
            rate_str: "0.01",
            authority: "Contract Cleaning Industry (Portable Long Service Leave Scheme) Act 2010 (NSW) and NSW Long Service Corporation 'Managing employer returns and paying levies in the contract cleaning industry' (read 24 November 2026): 1% of gross ordinary wages from 1 July 2021, rounded half up to the cent",
        },
        ("nsw", "community_services") => PayrollRule {
            board: "Long Service Corporation (NSW)",
            years: &["2025-26", "2026-27"],
            rate_str: if year == "2025-26" { "0.013" } else { "0.017" },
            authority: if year == "2025-26" {
                "Community Services Sector (Portable Long Service Leave) Act 2024 (NSW) and NSW Long Service Corporation 'Making levy contributions as a community services employer' (read 24 November 2026): 1.3% transitional levy rate on gross ordinary wages for 2025-26, rounded half up to the cent"
            } else {
                "Community Services Sector (Portable Long Service Leave) Act 2024 (NSW) and NSW Long Service Corporation 'Making levy contributions as a community services employer' (read 24 November 2026): 1.7% of gross ordinary wages from 1 July 2026, rounded half up to the cent"
            },
        },
        ("vic", "construction") => PayrollRule {
            board: "LeavePlus (Victoria)",
            years: &["2026-27"],
            rate_str: "0.027",
            authority: "Construction Industry Long Service Leave Act 1997 (Vic) and LeavePlus 'Long service leave charges' (read 24 November 2026): 2.7% of gross reportable ordinary pay, rounded half up to the cent",
        },
        ("vic", "community_services") => PayrollRule {
            board: "Portable Long Service Authority (Victoria)",
            years: &["2026-27"],
            rate_str: "0.0165",
            authority: "Long Service Benefits Portability Act 2018 (Vic) and Portable Long Service Authority 'About the Portable Long Service Benefits Scheme' (read 24 November 2026): 1.65% of ordinary pay for community services, rounded half up to the cent",
        },
        ("vic", "contract_cleaning") => PayrollRule {
            board: "Portable Long Service Authority (Victoria)",
            years: &["2026-27"],
            rate_str: "0.018",
            authority: "Long Service Benefits Portability Act 2018 (Vic) and Portable Long Service Authority 'About the Portable Long Service Benefits Scheme' (read 24 November 2026): 1.80% of ordinary pay for contract cleaning, rounded half up to the cent",
        },
        ("vic", "security") => PayrollRule {
            board: "Portable Long Service Authority (Victoria)",
            years: &["2026-27"],
            rate_str: "0.018",
            authority: "Long Service Benefits Portability Act 2018 (Vic) and Portable Long Service Authority 'About the Portable Long Service Benefits Scheme' (read 24 November 2026): 1.80% of ordinary pay for security, rounded half up to the cent",
        },
        ("qld", "contract_cleaning") => PayrollRule {
            board: "QLeave (Queensland)",
            years: &["2026-27"],
            rate_str: "0.005",
            authority: "Contract Cleaning Industry (Portable Long Service Leave) Act 2005 (Qld) s 83 and QLeave 'Employer obligations' (read 24 November 2026): 0.5% of workers' ordinary wages, rounded half up to the cent",
        },
        ("qld", "community_services") => PayrollRule {
            board: "QLeave (Queensland)",
            years: &["2026-27"],
            rate_str: "0.0135",
            authority: "Community Services Industry (Portable Long Service Leave) Act 2020 (Qld) and QLeave 'Paying the levy' (read 24 November 2026): 1.35% of workers' ordinary wages, rounded half up to the cent",
        },
        ("sa", "construction") => PayrollRule {
            board: "SA Portable Long Service Leave",
            years: &["2026-27"],
            rate_str: "0.02",
            authority: "Construction Industry Long Service Leave Act 1987 (SA) s 26 and SA Portable Long Service Leave 'How to complete Employer Returns' (read 24 November 2026): 2.00% of declared gross remuneration (excluding levy-free apprentices), rounded half up to the cent",
        },
        ("sa", "community_services") => PayrollRule {
            board: "SA Portable Long Service Leave",
            years: &["2026-27"],
            rate_str: "0.022",
            authority: "Portable Long Service Leave Act 2024 (SA) and SA Portable Long Service Leave 'What do I have to do?' (read 24 November 2026): 2.2% of gross remuneration (excluding exempt trainees), rounded half up to the cent",
        },
        ("wa", "construction") => PayrollRule {
            board: "MyLeave (Western Australia)",
            years: &["2026-27"],
            rate_str: "0.0075",
            authority: "Construction Industry Portable Paid Long Service Leave Act 1985 (WA) s 34 and MyLeave public notice '2027 Employer Contribution Rate' (4 November 2026, read 24 November 2026): 0.75% of ordinary pay for the 2026 and 2027 calendar years (excluding apprentices), rounded half up to the cent",
        },
        ("act", "construction") => PayrollRule {
            board: "ACT Leave",
            years: &["2025-26", "2026-27"],
            rate_str: "0.0275",
            authority: "Long Service Leave (Portable Schemes) Act 2009 (ACT) s 51 and Building and Construction Industry Employer Levy Determination 2024 (DI2024-12, in force from 1 October 2024, read 24 November 2026): 2.75% of gross ordinary wages, rounded half up to the cent",
        },
        ("act", "community_services") => PayrollRule {
            board: "ACT Leave",
            years: &["2024-25", "2025-26", "2026-27"],
            rate_str: "0.0185",
            authority: "Long Service Leave (Portable Schemes) Act 2009 (ACT) s 56A and Community Sector Industry Employer Levy Determination 2024 (DI2024-16, in force from 1 July 2024, read 24 November 2026): 1.85% of gross ordinary wages, rounded half up to the cent",
        },
        ("act", "security") => PayrollRule {
            board: "ACT Leave",
            years: &["2024-25", "2025-26", "2026-27"],
            rate_str: "0.0147",
            authority: "Long Service Leave (Portable Schemes) Act 2009 (ACT) s 56A and Security Industry Employer Levy Determination 2024 (DI2024-15, in force from 1 July 2024, read 24 November 2026): 1.47% of gross ordinary wages, rounded half up to the cent",
        },
        ("act", "contract_cleaning") => PayrollRule {
            board: "ACT Leave",
            years: &["2025-26", "2026-27"],
            rate_str: "0.0107",
            authority: "Long Service Leave (Portable Schemes) Act 2009 (ACT) s 51 and Services Industry Employer Levy Determinations 2024 (DI2024-224, from 1 April 2025) and 2026 (DI2026-228, from 1 January 2027), read 24 November 2026: 1.07% of gross ordinary wages for contract cleaning under the Services Industry Scheme, rounded half up to the cent",
        },
        _ => return None,
    };
    Some(rule)
}

fn check_year(jurisdiction: &str, scheme: &str, year: &str, allowed: &[&str]) -> Result<()> {
    if allowed.contains(&year) {
        return Ok(());
    }
    refuse(
        "portable_lsl_levy.scheme_year_unsupported",
        format!(
            "{jurisdiction} {scheme} portable LSL levy rates are verified for {}, not {year}.",
            allowed.join(", ")
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
    year: &str,
    jurisdiction: &str,
    scheme: &str,
    basis: &str,
    board: &str,
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
) -> Result<Value> {
    let mut out = derived_result(KIND, year, steps, rates)?;
    out["jurisdiction"] = json!(jurisdiction);
    out["scheme"] = json!(scheme);
    out["basis"] = json!(basis);
    out["authority_board"] = json!(board);
    Ok(out)
}

fn calculate_tas_construction(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    check_year("tas", "construction", year, &["2026-27"])?;
    reject_present(
        &[
            ("project_cost", args.project_cost),
            ("partial_exemption", args.partial_exemption),
        ],
        "Tasmania construction portable LSL levy",
    )?;
    let on_time = match args.paid_on_time {
        Some(v) => flag(v, "paid_on_time")?,
        None => {
            return refuse(
                "input.missing_field",
                "paid_on_time is required for Tasmania construction (true for the 1.8% on-time discounted rate, false for the 2.5% standard rate).",
            );
        }
    };
    let wages = required_money(
        args.ordinary_wages,
        "ordinary_wages",
        "Tasmania construction portable LSL levy",
    )?;
    let (rate_str, auth) = if on_time {
        (
            "0.018",
            "Construction Industry (Long Service) Act 1997 (Tas) s 7 and TasBuild 'Employers Information' FAQ (read 24 November 2026): 1.8% discounted rate on ordinary pay when returns and invoices are lodged and paid by the due dates, rounded half up to the cent",
        )
    } else {
        (
            "0.025",
            "Construction Industry (Long Service) Act 1997 (Tas) s 7 and TasBuild 'Employers Information' FAQ (read 24 November 2026): 2.5% standard contribution rate on ordinary pay, rounded half up to the cent",
        )
    };
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
        year,
        "tas",
        "construction",
        "ordinary_wages",
        "TasBuild (Tasmania)",
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

fn calculate_nsw_construction(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    check_year(
        "nsw",
        "construction",
        year,
        &["2023-24", "2024-25", "2025-26", "2026-27"],
    )?;
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
        year,
        "nsw",
        "construction",
        "project_cost",
        "Long Service Corporation (NSW)",
        steps,
        rates,
    )
}

fn calculate_qld_construction(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    check_year("qld", "construction", year, &["2025-26", "2026-27"])?;
    let cost = check_project_inputs(args, "Queensland building and construction levy")?;
    let (steps, rates) = precise(|| {
        let threshold = d("150000");
        let lsl_rate = d("0.0035");
        let training_rate = d("0.001");
        let whs_rate = d("0.00125");
        let combined_rate = d("0.00575");
        let (lsl, training, whs) = if cost.lt(&threshold)? {
            let below = "Building and Construction Industry (Portable Long Service Leave) Regulation 2024 (Qld) ss 9 and 11 (SL No. 162 of 2024, read 24 November 2026): levy is not payable when the cost of work (excluding GST, s 73(1)) is less than $150,000";
            (
                step("lsl_levy", below, "0", &[], int(0)),
                step("training_levy", below, "0", &[], int(0)),
                step("whs_levy", below, "0", &[], int(0)),
            )
        } else {
            let lsl_val = dollars_half_down(&cost.mul(&lsl_rate)?)?;
            let training_val = dollars_half_down(&cost.mul(&training_rate)?)?;
            let whs_val = dollars_half_down(&cost.mul(&whs_rate)?)?;
            (
                step(
                    "lsl_levy",
                    "Building and Construction Industry (Portable Long Service Leave) Act 1991 (Qld) s 72(1)(b) and (2) and Regulation 2024 s 8(b): 0.35% of cost of work (excluding GST), rounded to the nearest dollar with 50c rounded downwards",
                    "dollars_half_down(project_cost x lsl_rate)",
                    &[&cost, &lsl_rate],
                    lsl_val,
                ),
                step(
                    "training_levy",
                    "Building and Construction Industry (Portable Long Service Leave) Act 1991 (Qld) s 72(1)(a) and (2) and Regulation 2024 s 8(a): 0.1% CSQ training levy on cost of work (excluding GST), rounded to the nearest dollar with 50c rounded downwards",
                    "dollars_half_down(project_cost x training_rate)",
                    &[&cost, &training_rate],
                    training_val,
                ),
                step(
                    "whs_levy",
                    "Building and Construction Industry (Portable Long Service Leave) Act 1991 (Qld) s 72(1)(c) and (2) and Regulation 2024 s 8(c): 0.125% WHSQ levy on cost of work (excluding GST), rounded to the nearest dollar with 50c rounded downwards",
                    "dollars_half_down(project_cost x whs_rate)",
                    &[&cost, &whs_rate],
                    whs_val,
                ),
            )
        };
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
        year,
        "qld",
        "construction",
        "project_cost",
        "QLeave (Queensland)",
        steps,
        rates,
    )
}

fn calculate_nt_construction(args: &PortableLslLevyArgs, year: &str) -> Result<Value> {
    check_year("nt", "construction", year, &["2026-27"])?;
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
        let s = if cost.lt(&threshold)? {
            step(
                "levy",
                "Construction Industry Long Service Leave and Benefits Act 2005 (NT) and NT Build 'Levy Payers' (read 24 November 2026): no levy is payable on construction projects with a total contract price under $1,000,000 (including GST)",
                "0",
                &[],
                int(0),
            )
        } else {
            let levy = dollars_half_down(&cost.mul(&applied)?)?;
            step(
                "levy",
                "Construction Industry Long Service Leave and Benefits Act 2005 (NT) s 33 and NT Build 'Levy Payers' (read 24 November 2026): 0.1% of total cost of construction work (including GST) from $1 million to $5 billion, rounded to the nearest dollar with 50c rounded downwards",
                "dollars_half_down(project_cost x rate)",
                &[&cost, &applied],
                levy,
            )
        };
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
        year,
        "nt",
        "construction",
        "project_cost",
        "NT Build (Northern Territory)",
        steps,
        rates,
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
        ("nsw", "construction") => return calculate_nsw_construction(args, year),
        ("qld", "construction") => return calculate_qld_construction(args, year),
        ("nt", "construction") => return calculate_nt_construction(args, year),
        ("tas", "construction") => return calculate_tas_construction(args, year),
        _ => {}
    }
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
        &context,
    )?;
    let wages = required_money(args.ordinary_wages, "ordinary_wages", &context)?;
    let (steps, rates) = precise(|| {
        let applied = d(rule.rate_str);
        let levy = cents(wages.mul(&applied)?)?;
        let s = step(
            "levy",
            rule.authority,
            "cents(ordinary_wages x rate)",
            &[&wages, &applied],
            levy,
        );
        Ok((vec![s], vec![rate("rate", &applied)]))
    })?;
    finish_state(
        year,
        jurisdiction,
        scheme,
        "ordinary_wages",
        rule.board,
        steps,
        rates,
    )
}
