// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `residential_vacancy_fee`: the Australian federal annual vacancy fee for a
//! foreign owner of a residential dwelling under Part 6A of the Foreign
//! Acquisitions and Takeovers Act 1975 (FATA 1975, ss 115A to 115T), Division
//! 4A of the Foreign Acquisitions and Takeovers Regulation 2015 (FATR 2015,
//! ss 43C and 43D), the Foreign Acquisitions and Takeovers Fees Imposition Act
//! 2015 (ss 5 to 9) and the Foreign Acquisitions and Takeovers Fees Imposition
//! Regulations 2020 (Part 2 Division 2, Part 3 ss 42 to 46, ss 53, 57 and 59
//! to 61A, and s 67).
//!
//! Primary sources read on 24 November 2026: FATA 1975 compilation 35
//! (C2025C00672, 5 December 2025), FATR 2015 compilation 20 (F2025C01038,
//! 1 November 2025), Fees Imposition Act 2015 compilations 4 to 6
//! (C2021C00021, C2022C00334, C2024C00141), Fees Imposition Amendment Act 2024
//! (No. 17, 2024, C2024A00017), Fees Imposition Regulations 2020 compilations
//! 1, 2 and 4 (F2021C00683, F2022C00757, F2024C01252), the ATO 'Vacancy fee
//! return for foreign owners', 'Residential fees for a foreign person',
//! 'Request waiver of an application fee', 'Types of property a foreign person
//! can buy' and 'Consumer price index' pages, and Treasury Guidance Note 6
//! (v5, 1 July 2026), Guidance Note 10 (v12, 18 September 2026) and Schedule
//! of Fees (v8, June 2026).

use serde_json::{Value, json};

use super::{Arg, Result, Step, derived_result, flag, int, precise, rate, refuse, scope, step};
use crate::dates;
use crate::decimal::Decimal;
use fee_base::{VacancyWindow, resolve_fee_base};
use inputs::{
    AcquisitionBasis, DwellingKind, Ownership, Waiver, occupied_days, parse_acquisition_basis,
    parse_exemption, parse_ownership, parse_waiver, vacancy_start_in_year,
};
use schedules::{CAP_FLOOR_2026_27, parse_day, statutory_cap_for};

mod fee_base;
mod inputs;
mod schedules;

const KIND: &str = "residential_vacancy_fee";

/// Optional fee-basis inputs, in the order the command passes them.
pub const RESIDENTIAL_VACANCY_FEE_OPTIONAL: [&str; 5] = [
    "application_fee_payable",
    "consideration",
    "dwelling_kind",
    "application_date",
    "ownership_share",
];

/// FATA s 115C(1): fewer than 183 days residentially occupied triggers a fee.
const MIN_OCCUPIED_DAYS: i64 = 183;

/// FATA s 115D(1): vacancy fee return due within 30 days after year end.
const RETURN_DUE_DAYS: i64 = 30;

/// FATA ss 115D(1), 115DA(1) and 115G(1): 500 penalty units civil penalty.
const CIVIL_PENALTY_UNITS: i64 = 500;

/// Fees Imposition Amendment Act 2024 (No. 17, 2024) Sch 2 commencement:
/// doubles vacancy fees for vacancy years starting on or after 9 April 2024
/// (Regs s 67(2)) and triples application fees for established dwellings
/// notified on or after 9 April 2024 (Regs ss 10(2) and 53(1)(c)).
pub(super) const APRIL_2024_AMENDMENT: &str = "2024-04-09";

/// Government ban on foreign purchases of established dwellings (limited
/// exceptions only): 1 April 2025 to 30 June 2029 (ATO 'Types of property a
/// foreign person can buy' and Treasury GN6 v5).
const ESTABLISHED_BAN_FROM: &str = "2025-04-01";
const ESTABLISHED_BAN_TO: &str = "2029-06-30";

/// The facts as the command passes them; see the README for each.
pub struct ResidentialVacancyFeeArgs<'a> {
    pub vacancy_year_start: &'a Arg,
    pub held_full_vacancy_year: &'a Arg,
    pub residentially_occupied_days: &'a Arg,
    pub return_lodged_on_time: &'a Arg,
    pub exemption: &'a Arg,
    pub acquisition_basis: &'a Arg,
    pub ownership: &'a Arg,
    pub application_fee_waiver: &'a Arg,
    pub vacancy_fee_waiver_or_remission: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    /// [`RESIDENTIAL_VACANCY_FEE_OPTIONAL`], each where given.
    pub optional: [Option<&'a Arg>; 5],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LiabilityStatus {
    NotRequired,
    DeemedLiableLateReturn,
    NotLiableOccupied,
    Exempt,
    LiableVacant,
}

impl LiabilityStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::NotRequired => "not_required",
            Self::DeemedLiableLateReturn => "deemed_liable_late_return",
            Self::NotLiableOccupied => "not_liable_occupied",
            Self::Exempt => "exempt",
            Self::LiableVacant => "liable_vacant",
        }
    }

    fn liable(self) -> bool {
        matches!(self, Self::DeemedLiableLateReturn | Self::LiableVacant)
    }
}

struct ParsedVacancy {
    year: String,
    start: i64,
    end: i64,
    days_in_year: i64,
    return_due: i64,
    retention_until: i64,
    held_full: bool,
    lodged_on_time: bool,
    exemption: Option<(&'static str, &'static str)>,
    basis: AcquisitionBasis,
    ownership: Ownership,
    waiver: Waiver,
    status: LiabilityStatus,
}

fn determine_liability(
    held_full: bool,
    lodged_on_time: bool,
    occ_days: i64,
    has_exemption: bool,
) -> LiabilityStatus {
    if !held_full {
        LiabilityStatus::NotRequired
    } else if !lodged_on_time {
        LiabilityStatus::DeemedLiableLateReturn
    } else if occ_days >= MIN_OCCUPIED_DAYS {
        LiabilityStatus::NotLiableOccupied
    } else if has_exemption {
        LiabilityStatus::Exempt
    } else {
        LiabilityStatus::LiableVacant
    }
}

fn parse_vacancy_inputs(args: &ResidentialVacancyFeeArgs) -> Result<ParsedVacancy> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let start = vacancy_start_in_year(args.vacancy_year_start, &year)?;
    let end = dates::months_end(start, 12);
    let days_in_year = end - start + 1;
    let held_full = flag(args.held_full_vacancy_year, "held_full_vacancy_year")?;
    let occ_days = occupied_days(args.residentially_occupied_days, days_in_year)?;
    let lodged_on_time = flag(args.return_lodged_on_time, "return_lodged_on_time")?;
    let exemption = parse_exemption(args.exemption)?;
    let basis = parse_acquisition_basis(args.acquisition_basis)?;
    let ownership = parse_ownership(args.ownership)?;
    let waiver = parse_waiver(args.application_fee_waiver, basis)?;
    if flag(
        args.vacancy_fee_waiver_or_remission,
        "vacancy_fee_waiver_or_remission",
    )? {
        return refuse(
            "residential_vacancy_fee.discretionary_waiver_unsupported",
            "Discretionary waiver or remission of a vacancy fee under FATA s 115H is decided case by case by the Treasurer or Commissioner and cannot be calculated here.",
        );
    }
    let status = determine_liability(held_full, lodged_on_time, occ_days, exemption.is_some());
    Ok(ParsedVacancy {
        year,
        start,
        end,
        days_in_year,
        return_due: end + RETURN_DUE_DAYS,
        retention_until: dates::add_months(end, 60),
        held_full,
        lodged_on_time,
        exemption,
        basis,
        ownership,
        waiver,
        status,
    })
}

struct VacancySteps {
    uncapped: Step,
    payable: Step,
    cap_rate: Option<(&'static str, i64)>,
}

fn nil_vacancy_steps(
    status: LiabilityStatus,
    exemption: Option<(&'static str, &'static str)>,
) -> VacancySteps {
    let nil_auth = match status {
        LiabilityStatus::NotRequired => {
            "FATA 1975 s 115C(2) and s 115D(3) note: no vacancy year completed because the foreign person ceased to hold the interest or ceased to be a foreign person during the year"
        }
        LiabilityStatus::NotLiableOccupied => {
            "FATA 1975 s 115C(1) and (4): dwelling residentially occupied for 183 or more days in the vacancy year and return lodged on time"
        }
        LiabilityStatus::Exempt => {
            exemption
                .expect("exempt status requires an exemption ground")
                .1
        }
        _ => unreachable!(),
    };
    VacancySteps {
        uncapped: step("uncapped_vacancy_fee", nil_auth, "0", &[], int(0)),
        payable: step("vacancy_fee_payable", nil_auth, "0", &[], int(0)),
        cap_rate: None,
    }
}

fn compute_vacancy_steps(
    parsed: &ParsedVacancy,
    base_fee: &Decimal,
    vac_mult: i64,
) -> Result<VacancySteps> {
    if !parsed.status.liable() {
        return Ok(nil_vacancy_steps(parsed.status, parsed.exemption));
    }
    let vm = int(vac_mult);
    let uncapped = base_fee.mul(&vm)?;
    let uncapped_auth = if vac_mult == 2 {
        "Fees Regulations 2020 Part 3 (ss 43 to 46) and s 67(2) (Act No. 17 of 2024): double the application fee basis for vacancy years starting on or after 9 April 2024"
    } else {
        "Fees Regulations 2020 Part 3 (ss 43 to 46, pre-9 April 2024 compilation): equal to the application fee basis for vacancy years starting before 9 April 2024"
    };
    let u_step = step(
        "uncapped_vacancy_fee",
        uncapped_auth,
        "application_fee_base x vacancy_fee_multiplier",
        &[base_fee, &vm],
        uncapped.clone(),
    );
    // Fees Imposition Act 2015 s 7(3) guarantees the s 6(3) statutory cap never
    // decreases on later 1 July indexations; any uncapped fee <= floor is safe.
    let floor_cap = statutory_cap_for(parsed.end).unwrap_or(CAP_FLOOR_2026_27);
    if uncapped.gt(&int(floor_cap))? {
        return refuse(
            "residential_vacancy_fee.cap_unsettled",
            format!(
                "The uncapped vacancy fee (${uncapped}) exceeds the Fees Imposition Act 2015 s 6(3) statutory fee cap floor at vacancy_year_end (${floor_cap}). Because the applicable indexed cap under s 7 and Fees Regulations s 61A can rise if the Commissioner's s 115E notice or s 115F due date falls after a later 1 July indexation, and this worksheet takes no notice date, a fee above the cap floor is refused."
            ),
        );
    }
    let p_step = step(
        "vacancy_fee_payable",
        "Fees Imposition Act 2015 ss 6(3) and 7(3) and Fees Regulations 2020 s 61A: uncapped vacancy fee does not exceed the non-decreasing statutory fee cap floor, so the cap cannot bind regardless of s 115E notice or s 115F due date",
        "uncapped_vacancy_fee",
        &[&uncapped],
        uncapped.clone(),
    );
    Ok(VacancySteps {
        uncapped: u_step,
        payable: p_step,
        cap_rate: Some(("statutory_fee_cap_floor", floor_cap)),
    })
}

fn joint_ownership_note(ownership: Ownership) -> &'static str {
    match ownership {
        Ownership::Sole => {
            "Sole foreign owner: lodges one return and is liable for the full vacancy fee if payable."
        }
        Ownership::JointTenants => {
            "Joint tenants (FATA s 115D(4), Fees Regulations s 57(1)): one return by any foreign joint tenant with the same vacancy year satisfies the lodgment obligation, and one vacancy fee is payable jointly."
        }
        Ownership::TenantsInCommon => {
            "Tenants in common (Fees Regulations s 57(2)): each foreign tenant in common must lodge a separate vacancy fee return and pays only their own proportionate share."
        }
    }
}

fn established_ban_note(
    dwelling_kind: Option<DwellingKind>,
    application_date: Option<i64>,
) -> Value {
    match (dwelling_kind, application_date) {
        (Some(DwellingKind::Established), Some(day))
            if (parse_day(ESTABLISHED_BAN_FROM)..=parse_day(ESTABLISHED_BAN_TO)).contains(&day) =>
        {
            json!(
                "Foreign persons are generally banned from purchasing established dwellings from 1 April 2025 to 30 June 2029 unless a limited exception applies (ATO 'Types of property a foreign person can buy' and Treasury GN6 v5)."
            )
        }
        _ => Value::Null,
    }
}

struct FeeSummary {
    fee_basis: &'static str,
    dwelling_kind: Option<DwellingKind>,
    application_date: Option<i64>,
}

fn build_vacancy_output(
    parsed: &ParsedVacancy,
    fee_summary: &FeeSummary,
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
) -> Result<Value> {
    let mut out = derived_result(KIND, &parsed.year, steps, rates)?;
    out["vacancy_year_start"] = json!(dates::format(parsed.start));
    out["vacancy_year_end"] = json!(dates::format(parsed.end));
    out["days_in_vacancy_year"] = json!(parsed.days_in_year);
    out["return_required"] = json!(parsed.held_full);
    out["return_due_date"] = if parsed.held_full {
        json!(dates::format(parsed.return_due))
    } else {
        Value::Null
    };
    out["record_retention_until"] = if parsed.held_full {
        json!(dates::format(parsed.retention_until))
    } else {
        Value::Null
    };
    out["liable"] = json!(parsed.status.liable());
    out["liability_status"] = json!(parsed.status.as_str());
    out["fee_basis"] = json!(fee_summary.fee_basis);
    out["exemption_applied"] = if parsed.status == LiabilityStatus::Exempt {
        json!(parsed.exemption.map(|(k, _)| k))
    } else {
        Value::Null
    };
    out["payment_due_rule"] = if parsed.status.liable() {
        json!(
            "Due on the day specified in the s 115E notice of liability, at least 21 days after the notice is given (FATA 1975 s 115F)."
        )
    } else {
        Value::Null
    };
    out["civil_penalty_units"] = json!(CIVIL_PENALTY_UNITS);
    out["joint_ownership_note"] = json!(joint_ownership_note(parsed.ownership));
    out["established_dwelling_ban_note"] =
        established_ban_note(fee_summary.dwelling_kind, fee_summary.application_date);
    Ok(out)
}

pub fn residential_vacancy_fee(args: &ResidentialVacancyFeeArgs) -> Result<Value> {
    let parsed = parse_vacancy_inputs(args)?;
    let window = VacancyWindow {
        start: parsed.start,
        end: parsed.end,
        return_due: parsed.return_due,
        lodged_on_time: parsed.lodged_on_time,
    };
    let fee_base = precise(|| {
        resolve_fee_base(
            parsed.basis,
            parsed.ownership,
            parsed.waiver,
            window,
            &args.optional,
        )
    })?;
    let vac_mult: i64 = if parsed.start >= parse_day(APRIL_2024_AMENDMENT) {
        2
    } else {
        1
    };
    let vac_steps = precise(|| compute_vacancy_steps(&parsed, &fee_base.base_fee, vac_mult))?;

    let summary = FeeSummary {
        fee_basis: fee_base.fee_basis,
        dwelling_kind: fee_base.dwelling_kind,
        application_date: fee_base.application_date,
    };
    let mut steps = fee_base.steps;
    steps.push(vac_steps.uncapped);
    steps.push(vac_steps.payable);
    let mut rates = fee_base.rates;
    rates.push(rate("vacancy_fee_multiplier", vac_mult));
    if let Some((k, v)) = vac_steps.cap_rate {
        rates.push(rate(k, v));
    }
    build_vacancy_output(&parsed, &summary, steps, rates)
}

#[cfg(test)]
mod tests {
    use super::{LiabilityStatus, nil_vacancy_steps};

    /// compute_vacancy_steps calls this only for a status that is not liable,
    /// so no input brings a liable status here.
    #[test]
    #[should_panic(expected = "internal error: entered unreachable code")]
    fn a_liable_status_has_no_nil_steps() {
        nil_vacancy_steps(LiabilityStatus::LiableVacant, None);
    }
}
