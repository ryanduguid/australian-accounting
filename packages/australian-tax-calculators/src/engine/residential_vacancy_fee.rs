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

use super::business_capital::year_first;
use super::{
    Arg, Result, Step, cents, date_arg, derived_result, flag, int, money, precise, rate, refuse,
    scope, step,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

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
const APRIL_2024_AMENDMENT: &str = "2024-04-09";

/// Government ban on foreign purchases of established dwellings (limited
/// exceptions only): 1 April 2025 to 30 June 2029 (ATO 'Types of property a
/// foreign person can buy' and Treasury GN6 v5).
const ESTABLISHED_BAN_FROM: &str = "2025-04-01";
const ESTABLISHED_BAN_TO: &str = "2029-06-30";

/// De minimis consideration threshold (Fees Regs s 53(1)).
const DE_MINIMIS_THRESHOLD: i64 = 75_000;

/// Fee constant for acquiring an interest in residential land (Fees Regs s 9
/// item 1).
const RESIDENTIAL_FEE_CONSTANT: i64 = 1_000_000;

/// Verified floor for the Fees Imposition Act 2015 s 6(3) statutory fee cap in
/// 2026-27 (and later unindexed years, since s 7(3) prevents downward
/// indexation): $7,000,000 x 1.113 = $7,791,000.
const CAP_FLOOR_2026_27: i64 = 7_791_000;

/// Verified Fees Imposition Regulations 2020 residential fee parameters by
/// inclusive effective date range (`2021-01-01` to `2027-06-30`).
#[derive(Clone, Copy)]
struct Schedule {
    from: &'static str,
    to: &'static str,
    de_minimis: i64,
    tier_one: i64,
    tier_step: i64,
    residential_cap: i64,
    established_triple: bool,
}

const SCHEDULES: [Schedule; 9] = [
    // F2020L01648 base rates (1 January 2021 to 30 June 2021).
    Schedule {
        from: "2021-01-01",
        to: "2021-06-30",
        de_minimis: 2_000,
        tier_one: 6_350,
        tier_step: 12_700,
        residential_cap: 500_000,
        established_triple: false,
    },
    // 2021-22: indexation factor 1.006 (s 59(4)(a) holds $6,350, $12,700, $2,000).
    Schedule {
        from: "2021-07-01",
        to: "2022-06-30",
        de_minimis: 2_000,
        tier_one: 6_350,
        tier_step: 12_700,
        residential_cap: 503_000,
        established_triple: false,
    },
    // 2022-23 before Fee Doubling: indexation factor 1.045.
    Schedule {
        from: "2022-07-01",
        to: "2022-07-28",
        de_minimis: 2_000,
        tier_one: 6_600,
        tier_step: 13_200,
        residential_cap: 522_500,
        established_triple: false,
    },
    // F2022L00999 Fee Doubling Regulations 2022 (from 29 July 2022).
    Schedule {
        from: "2022-07-29",
        to: "2023-06-30",
        de_minimis: 4_000,
        tier_one: 13_200,
        tier_step: 26_400,
        residential_cap: 1_045_000,
        established_triple: false,
    },
    // 2023-24 before Act No. 17 of 2024: indexation factor 1.071.
    Schedule {
        from: "2023-07-01",
        to: "2024-04-08",
        de_minimis: 4_200,
        tier_one: 14_100,
        tier_step: 28_200,
        residential_cap: 1_119_100,
        established_triple: false,
    },
    // Act No. 17 of 2024 Sch 2 (from 9 April 2024): established dwellings 3x.
    Schedule {
        from: "2024-04-09",
        to: "2024-06-30",
        de_minimis: 4_200,
        tier_one: 14_100,
        tier_step: 28_200,
        residential_cap: 1_119_100,
        established_triple: true,
    },
    // 2024-25: indexation factor 1.047 (verified against GN10 Example 34: $88,500).
    Schedule {
        from: "2024-07-01",
        to: "2025-06-30",
        de_minimis: 4_300,
        tier_one: 14_700,
        tier_step: 29_500,
        residential_cap: 1_171_600,
        established_triple: true,
    },
    // 2025-26: indexation factor 1.077 (verified against ATO variation example: $15,100).
    Schedule {
        from: "2025-07-01",
        to: "2026-06-30",
        de_minimis: 4_500,
        tier_one: 15_100,
        tier_step: 30_300,
        residential_cap: 1_205_200,
        established_triple: true,
    },
    // 2026-27: ATO Residential fees tables 1 & 2 and Treasury Schedule of Fees v8 (factor 1.113).
    Schedule {
        from: "2026-07-01",
        to: "2027-06-30",
        de_minimis: 4_600,
        tier_one: 15_600,
        tier_step: 31_300,
        residential_cap: 1_245_500,
        established_triple: true,
    },
];

/// Verified Fees Imposition Act 2015 s 6(3) statutory fee cap (enforced by
/// Fees Regulations s 61A) by inclusive effective date range.
const STATUTORY_CAPS: [(&str, &str, i64); 8] = [
    ("2021-01-01", "2021-06-30", 1_000_000),
    ("2021-07-01", "2022-06-30", 1_006_000),
    ("2022-07-01", "2023-06-30", 1_045_000),
    ("2023-07-01", "2024-04-08", 1_119_100),
    ("2024-04-09", "2024-06-30", 7_000_000),
    ("2024-07-01", "2025-06-30", 7_329_000),
    ("2025-07-01", "2026-06-30", 7_539_000),
    ("2026-07-01", "2027-06-30", CAP_FLOOR_2026_27),
];

const EXEMPTIONS: [(&str, &str); 4] = [
    (
        "damaged_unsafe_or_unsuitable",
        "FATR 2015 s 43D(2)(a) and (3)(a): dwelling damaged, unsafe or otherwise unsuitable to be occupied as a residence",
    ),
    (
        "substantial_repairs_or_renovations",
        "FATR 2015 s 43D(2)(a) and (3)(b): dwelling undergoing substantial repairs or renovations",
    ),
    (
        "legal_restriction_or_court_order",
        "FATR 2015 s 43D(2)(a) and (3)(c): occupation prohibited or made impracticable by a court or tribunal order or a Commonwealth, State or Territory law",
    ),
    (
        "medical_or_residential_care",
        "FATR 2015 s 43D(2)(b): ordinary occupant absent to receive medical care, residential care or funded aged care services in a residential care home, supported by evidence",
    ),
];

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
enum AcquisitionBasis {
    NotifiableAction,
    DeveloperExemptionCertificate,
    OtherExemptionCertificate,
    NoNoticeOrder,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ownership {
    Sole,
    JointTenants,
    TenantsInCommon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Waiver {
    None,
    Full,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DwellingKind {
    NewOrNearNew,
    Established,
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

fn parse_day(text: &str) -> i64 {
    dates::parse(text).expect("valid date literal")
}

fn schedule_for(day: i64) -> Option<Schedule> {
    SCHEDULES
        .iter()
        .copied()
        .find(|s| (parse_day(s.from)..=parse_day(s.to)).contains(&day))
}

fn statutory_cap_for(day: i64) -> Option<i64> {
    STATUTORY_CAPS
        .iter()
        .find(|(from, to, _)| (parse_day(from)..=parse_day(to)).contains(&day))
        .map(|(_, _, cap)| *cap)
}

fn vacancy_start_in_year(given: &Arg, year: &str) -> Result<i64> {
    let first = year_first(year).expect("a supported period is a financial year");
    let start = parse_day(&format!("{first}-07-01"));
    let end = parse_day(&format!("{}-06-30", first + 1));
    let day = date_arg(given, "vacancy_year_start")?;
    if !(start..=end).contains(&day) {
        return refuse(
            "residential_vacancy_fee.date_outside_year",
            format!(
                "vacancy_year_start must be in the {year} financial year, from {} to {}.",
                dates::format(start),
                dates::format(end),
            ),
        );
    }
    Ok(day)
}

fn occupied_days(given: &Arg, days_in_year: i64) -> Result<i64> {
    match given.int() {
        Some(Some(n)) if (0..=days_in_year).contains(&n) => Ok(n),
        _ => refuse(
            "residential_vacancy_fee.days_invalid",
            format!(
                "residentially_occupied_days must be a JSON integer from 0 to {days_in_year} for this vacancy year."
            ),
        ),
    }
}

fn parse_exemption(given: &Arg) -> Result<Option<(&'static str, &'static str)>> {
    if matches!(given, Arg::Other(none) if none == "None") {
        return Ok(None);
    }
    match EXEMPTIONS
        .iter()
        .find(|(key, _)| given.text() == Some(*key))
    {
        Some(&pair) => Ok(Some(pair)),
        None => refuse(
            "residential_vacancy_fee.exemption_invalid",
            "exemption must be null or one of damaged_unsafe_or_unsuitable, substantial_repairs_or_renovations, legal_restriction_or_court_order or medical_or_residential_care.",
        ),
    }
}

fn parse_acquisition_basis(given: &Arg) -> Result<AcquisitionBasis> {
    match given.text() {
        Some("notifiable_action") => Ok(AcquisitionBasis::NotifiableAction),
        Some("developer_exemption_certificate") => {
            Ok(AcquisitionBasis::DeveloperExemptionCertificate)
        }
        Some("other_exemption_certificate") => Ok(AcquisitionBasis::OtherExemptionCertificate),
        Some("no_notice_order") => Ok(AcquisitionBasis::NoNoticeOrder),
        Some("build_to_rent") => refuse(
            "residential_vacancy_fee.build_to_rent_unsupported",
            "Build-to-rent developments receive case-by-case commercial-rate fee waivers under FATA s 115 and bespoke Treasury vacancy-fee reporting conditions (Guidance Note 6 Section I, Guidance Note 10 Section L) and are outside this worksheet.",
        ),
        _ => refuse(
            "residential_vacancy_fee.acquisition_basis_invalid",
            "acquisition_basis must be notifiable_action, developer_exemption_certificate, other_exemption_certificate, no_notice_order or build_to_rent.",
        ),
    }
}

fn parse_ownership(given: &Arg) -> Result<Ownership> {
    match given.text() {
        Some("sole") => Ok(Ownership::Sole),
        Some("joint_tenants") => Ok(Ownership::JointTenants),
        Some("tenants_in_common") => Ok(Ownership::TenantsInCommon),
        _ => refuse(
            "residential_vacancy_fee.ownership_invalid",
            "ownership must be sole, joint_tenants or tenants_in_common.",
        ),
    }
}

fn parse_waiver(given: &Arg, basis: AcquisitionBasis) -> Result<Waiver> {
    match given.text() {
        Some("none") => Ok(Waiver::None),
        Some("partial") => refuse(
            "residential_vacancy_fee.partial_waiver_unsupported",
            "Partial waivers of the original application fee fall outside Fees Regulations 2020 Part 3's full-waiver substitution rules and depend on case-by-case Treasury waiver or remission under FATA s 115H.",
        ),
        Some("full") => match basis {
            AcquisitionBasis::DeveloperExemptionCertificate => refuse(
                "residential_vacancy_fee.waiver_invalid",
                "application_fee_waiver cannot be full for developer_exemption_certificate: under Fees Regulations s 44 the purchaser paid no personal application fee to waive.",
            ),
            AcquisitionBasis::OtherExemptionCertificate => refuse(
                "residential_vacancy_fee.other_ec_waiver_unsupported",
                "Under Fees Regulations s 45(b), a waived fee for another exemption certificate uses the lowest Division 7 fee for that specific certificate kind, which depends on the Division 7 category (s 32, s 33 or s 34) and is outside this worksheet.",
            ),
            AcquisitionBasis::NotifiableAction | AcquisitionBasis::NoNoticeOrder => {
                Ok(Waiver::Full)
            }
        },
        _ => refuse(
            "residential_vacancy_fee.waiver_invalid",
            "application_fee_waiver must be none, full or partial.",
        ),
    }
}

fn parse_dwelling_kind(given: Option<&Arg>, basis: AcquisitionBasis) -> Result<DwellingKind> {
    let kind = match given.and_then(Arg::text) {
        Some("new_or_near_new") => DwellingKind::NewOrNearNew,
        Some("established") => DwellingKind::Established,
        _ => {
            return refuse(
                "residential_vacancy_fee.dwelling_kind_invalid",
                "dwelling_kind is required and must be new_or_near_new or established.",
            );
        }
    };
    if basis == AcquisitionBasis::DeveloperExemptionCertificate && kind == DwellingKind::Established
    {
        return refuse(
            "residential_vacancy_fee.dwelling_kind_invalid",
            "dwelling_kind cannot be established for developer_exemption_certificate: Fees Regulations s 44 covers only s 57 new dwellings and reg 43A near-new dwelling interests certificates.",
        );
    }
    Ok(kind)
}

fn parse_application_date(
    given: Option<&Arg>,
    basis: AcquisitionBasis,
    vacancy_start: i64,
) -> Result<(i64, Schedule)> {
    let Some(arg) = given else {
        return refuse(
            "residential_vacancy_fee.application_date_unsupported",
            "application_date (YYYY-MM-DD, from 2021-01-01 to 2027-06-30) is required to determine the statutory residential fee schedule.",
        );
    };
    let day = date_arg(arg, "application_date")?;
    let Some(sched) = schedule_for(day) else {
        return refuse(
            "residential_vacancy_fee.application_date_unsupported",
            "application_date must be from 2021-01-01 to 2027-06-30, where the Fees Imposition Regulations 2020 residential fee schedule is verified. For a pre-2021 notified acquisition, supply application_fee_payable directly.",
        );
    };
    if matches!(
        basis,
        AcquisitionBasis::NotifiableAction | AcquisitionBasis::DeveloperExemptionCertificate
    ) && day > vacancy_start
    {
        return refuse(
            "residential_vacancy_fee.application_date_unsupported",
            "application_date cannot be after vacancy_year_start for a notified acquisition or developer exemption certificate.",
        );
    }
    Ok((day, sched))
}

fn parse_ownership_share(given: Option<&Arg>, ownership: Ownership) -> Result<Option<Decimal>> {
    match ownership {
        Ownership::TenantsInCommon => {
            let Some(arg) = given else {
                return refuse(
                    "residential_vacancy_fee.ownership_invalid",
                    "ownership_share (a decimal string strictly between 0 and 1) is required for tenants_in_common when deriving the application fee from the statutory schedule.",
                );
            };
            let share = match arg.text() {
                Some(text) if text.len() <= 32 => Decimal::parse_input(text).ok(),
                _ => None,
            };
            match share {
                Some(s) if s.is_finite() && s.gt(&int(0))? && s.lt(&int(1))? => Ok(Some(s)),
                _ => refuse(
                    "residential_vacancy_fee.ownership_invalid",
                    "ownership_share must be a decimal string strictly between 0 and 1 for tenants_in_common (use sole for a 100% owner).",
                ),
            }
        }
        Ownership::Sole | Ownership::JointTenants => {
            if given.is_some() {
                return refuse(
                    "residential_vacancy_fee.ownership_invalid",
                    "ownership_share applies only to tenants_in_common (Fees Regulations s 57(2)); omit it for sole or joint_tenants.",
                );
            }
            Ok(None)
        }
    }
}

/// Computes the s 11(1) / s 11(2) integer multiple for consideration strictly
/// above the $1,000,000 residential fee constant.
fn tier_multiple(consideration: &Decimal) -> Result<Decimal> {
    let fc = int(RESIDENTIAL_FEE_CONSTANT);
    let quot = consideration.div(&fc)?;
    let floored = quot.quantize(0, Rounding::Down)?;
    let exact_multiple = floored.mul(&fc)?.compare(consideration)? == std::cmp::Ordering::Equal;
    if exact_multiple {
        // s 10(1) item 2 & s 11(1): floor((consideration - $1) / $1,000,000)
        // which equals consideration / $1,000,000 - 1 for any positive integer multiple.
        Ok(floored.sub(&int(1))?)
    } else {
        // s 10(1) item 3 & s 11(2): floor(consideration / $1,000,000).
        Ok(floored)
    }
}

struct FeeBase {
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
    base_fee: Decimal,
    fee_basis: &'static str,
    dwelling_kind: Option<DwellingKind>,
    application_date: Option<i64>,
}

fn whole_and_share_steps(
    whole_step: Step,
    whole_fee: Decimal,
    share: Option<&Decimal>,
) -> Result<(Vec<Step>, Decimal)> {
    match share {
        None => {
            let base = step(
                "application_fee_base",
                "Fees Regulations 2020 Part 3 (ss 43 to 46) and s 57(1): full residential application fee basis for a sole owner or joint tenancy",
                "whole_property_application_fee",
                &[&whole_fee],
                whole_fee.clone(),
            );
            Ok((vec![whole_step, base], whole_fee))
        }
        Some(share) => {
            let apportioned = cents(whole_fee.mul(share)?)?;
            let base = step(
                "application_fee_base",
                "Fees Regulations 2020 s 57(2): tenant in common's proportionate share of the application fee",
                "cents(whole_property_application_fee x ownership_share)",
                &[&whole_fee, share],
                apportioned.clone(),
            );
            Ok((vec![whole_step, base], apportioned))
        }
    }
}

#[derive(Clone, Copy)]
struct VacancyWindow {
    start: i64,
    end: i64,
    return_due: i64,
    lodged_on_time: bool,
}

fn resolve_fee_base(
    basis: AcquisitionBasis,
    ownership: Ownership,
    waiver: Waiver,
    window: VacancyWindow,
    optional: &[Option<&Arg>; 5],
) -> Result<FeeBase> {
    let [
        stated_fee,
        consideration,
        dwelling_arg,
        app_date_arg,
        share_arg,
    ] = *optional;
    let post_april_2024 = window.start >= parse_day(APRIL_2024_AMENDMENT);

    // Mode 1: Stated application_fee_payable.
    if let Some(fee_arg) = stated_fee {
        if waiver != Waiver::None {
            return refuse(
                "residential_vacancy_fee.application_fee_invalid",
                "Omit application_fee_payable when application_fee_waiver is full.",
            );
        }
        if basis == AcquisitionBasis::DeveloperExemptionCertificate {
            return refuse(
                "residential_vacancy_fee.application_fee_invalid",
                "Omit application_fee_payable for developer_exemption_certificate: Fees Regulations s 44 derives the notional Division 2 fee from consideration, dwelling_kind and application_date.",
            );
        }
        if consideration.is_some()
            || dwelling_arg.is_some()
            || app_date_arg.is_some()
            || share_arg.is_some()
        {
            return refuse(
                "residential_vacancy_fee.application_fee_invalid",
                "Supply either application_fee_payable alone, or consideration, dwelling_kind and application_date (with ownership_share for tenants_in_common), not both.",
            );
        }
        let fee = money(fee_arg)?;
        if !fee.gt(&int(0))? {
            return refuse(
                "residential_vacancy_fee.application_fee_invalid",
                "application_fee_payable must be above nil.",
            );
        }
        let authority = match basis {
            AcquisitionBasis::NotifiableAction => {
                "Fees Regulations 2020 s 43(a): Division 2 Part 2 application fee payable by the person for the s 81 notice"
            }
            AcquisitionBasis::OtherExemptionCertificate => {
                "Fees Regulations 2020 s 45(a): Division 7 Part 2 application fee payable by the person for the exemption certificate"
            }
            AcquisitionBasis::NoNoticeOrder => {
                "Fees Regulations 2020 s 46(a): s 23 application fee payable by the person for the Division 2 Part 3 order or no objection notification"
            }
            AcquisitionBasis::DeveloperExemptionCertificate => unreachable!(),
        };
        let s = step(
            "application_fee_base",
            authority,
            "application_fee_payable",
            &[&fee],
            fee.clone(),
        );
        return Ok(FeeBase {
            steps: vec![s],
            rates: Vec::new(),
            base_fee: fee,
            fee_basis: "stated_application_fee",
            dwelling_kind: None,
            application_date: None,
        });
    }

    // Mode 3: Full waiver of original application fee (s 43(b) / s 46(b)).
    if waiver == Waiver::Full {
        if consideration.is_some() {
            return refuse(
                "residential_vacancy_fee.consideration_invalid",
                "Omit consideration when application_fee_waiver is full: Fees Regulations ss 43(b) and 46(b) substitute s 10 item 1 regardless of consideration.",
            );
        }
        let share = parse_ownership_share(share_arg, ownership)?;
        if post_april_2024 {
            let dk = parse_dwelling_kind(dwelling_arg, basis)?;
            let (app_day, sched) = parse_application_date(app_date_arg, basis, window.start)?;
            let estab_mult = if dk == DwellingKind::Established && sched.established_triple {
                3
            } else {
                1
            };
            let tier_one = int(sched.tier_one);
            let em = int(estab_mult);
            let whole = tier_one.mul(&em)?;
            let authority = match basis {
                AcquisitionBasis::NotifiableAction => {
                    "Fees Regulations 2020 s 43(b) (as amended by Act No. 17 of 2024) and s 10(1) item 1, (2): notional tier-1 fee in force when the notice was given"
                }
                AcquisitionBasis::NoNoticeOrder => {
                    "Fees Regulations 2020 s 46(b) (as amended by Act No. 17 of 2024) and s 10(1) item 1, (2): notional tier-1 fee in force when the notice was given"
                }
                _ => unreachable!(),
            };
            let whole_step = step(
                "whole_property_application_fee",
                authority,
                "tier_one_fee x established_multiplier",
                &[&tier_one, &em],
                whole.clone(),
            );
            let (steps, base_fee) = whole_and_share_steps(whole_step, whole, share.as_ref())?;
            return Ok(FeeBase {
                steps,
                rates: vec![
                    rate("tier_one_fee", sched.tier_one),
                    rate("established_multiplier", estab_mult),
                ],
                base_fee,
                fee_basis: "waived_tier_one",
                dwelling_kind: Some(dk),
                application_date: Some(app_day),
            });
        }

        // Pre-9 April 2024 s 43(b) / s 46(b): item 1 of s 10 table in force when
        // the vacancy fee becomes payable (no notice-date freeze, no 3x rule).
        if !window.lodged_on_time {
            return refuse(
                "residential_vacancy_fee.pre_2024_waiver_boundary_unsettled",
                "For a pre-9 April 2024 vacancy year with a waived application fee and a late return, the s 10 item 1 amount turns on when the Commissioner issues the s 115E notice or the fee becomes payable, which is outside this worksheet.",
            );
        }
        if app_date_arg.is_some() || dwelling_arg.is_some() {
            return refuse(
                "residential_vacancy_fee.application_fee_invalid",
                "For a vacancy year starting before 9 April 2024 with a waived application fee, omit application_date and dwelling_kind: pre-9 April 2024 Fees Regulations ss 43(b) and 46(b) use the s 10 item 1 amount in force when the vacancy fee becomes payable.",
            );
        }
        let s_end = schedule_for(window.end);
        let s_due = schedule_for(window.return_due);
        let (Some(se), Some(sd)) = (s_end, s_due) else {
            return refuse(
                "residential_vacancy_fee.pre_2024_waiver_boundary_unsettled",
                "The vacancy year end or 30-day return window lies outside the verified Fees Regulations 2020 schedule dates.",
            );
        };
        if se.tier_one != sd.tier_one {
            return refuse(
                "residential_vacancy_fee.pre_2024_waiver_boundary_unsettled",
                format!(
                    "The 30-day return window ({} to {}) crosses a change in the s 10 item 1 fee (${} to ${}), so the pre-9 April 2024 waived-fee amount depends on the exact date the fee became payable.",
                    dates::format(window.end),
                    dates::format(window.return_due),
                    se.tier_one,
                    sd.tier_one,
                ),
            );
        }
        let tier_one = int(se.tier_one);
        let whole_step = step(
            "whole_property_application_fee",
            "Fees Regulations 2020 (pre-9 April 2024 compilation) s 43(b) or s 46(b) and s 10 item 1: lowest residential tier fee",
            "tier_one_fee",
            &[&tier_one],
            tier_one.clone(),
        );
        let (steps, base_fee) = whole_and_share_steps(whole_step, tier_one, share.as_ref())?;
        return Ok(FeeBase {
            steps,
            rates: vec![rate("tier_one_fee", se.tier_one)],
            base_fee,
            fee_basis: "waived_tier_one",
            dwelling_kind: None,
            application_date: None,
        });
    }

    // Mode 2: Derived from statutory residential fee schedule.
    if basis == AcquisitionBasis::OtherExemptionCertificate {
        return refuse(
            "residential_vacancy_fee.application_fee_invalid",
            "Supply application_fee_payable for other_exemption_certificate: Fees Regulations s 45(a) uses the Division 7 fee payable for that certificate (ss 32, 33 or 34).",
        );
    }
    let Some(cons_arg) = consideration else {
        return refuse(
            "residential_vacancy_fee.consideration_invalid",
            "Supply either application_fee_payable, or consideration with dwelling_kind and application_date.",
        );
    };
    let cons = money(cons_arg)?;
    if !cons.gt(&int(0))? {
        return refuse(
            "residential_vacancy_fee.consideration_invalid",
            "consideration must be above nil.",
        );
    }
    let dk = parse_dwelling_kind(dwelling_arg, basis)?;
    let (app_day, sched) = parse_application_date(app_date_arg, basis, window.start)?;
    let share = parse_ownership_share(share_arg, ownership)?;
    let estab_mult = if dk == DwellingKind::Established && sched.established_triple {
        3
    } else {
        1
    };
    let em = int(estab_mult);

    let (whole_step, whole, rates) = if cons.lt(&int(DE_MINIMIS_THRESHOLD))? {
        if basis == AcquisitionBasis::DeveloperExemptionCertificate {
            return refuse(
                "residential_vacancy_fee.de_minimis_developer_ec_unsettled",
                "Consideration below $75,000 under a developer exemption certificate hits the unsettled boundary between Fees Regulations s 44(2) (Division 2 of Part 2) and s 53(1) (de minimis rule despite Part 2).",
            );
        }
        let dm = int(sched.de_minimis);
        let whole = dm.mul(&em)?;
        let ws = step(
            "whole_property_application_fee",
            "Fees Regulations 2020 s 53(1): de minimis residential application fee for consideration below $75,000 (3x for established dwellings from 9 April 2024)",
            "de_minimis_fee x established_multiplier",
            &[&dm, &em],
            whole.clone(),
        );
        (
            ws,
            whole,
            vec![
                rate("de_minimis_fee", sched.de_minimis),
                rate("established_multiplier", estab_mult),
            ],
        )
    } else if !cons.gt(&int(RESIDENTIAL_FEE_CONSTANT))? {
        let t1 = int(sched.tier_one);
        let whole = t1.mul(&em)?;
        let ws = step(
            "whole_property_application_fee",
            "Fees Regulations 2020 s 9 item 1 and s 10(1) item 1, (2): residential land fee for consideration up to $1 million (3x for established dwellings from 9 April 2024)",
            "tier_one_fee x established_multiplier",
            &[&t1, &em],
            whole.clone(),
        );
        (
            ws,
            whole,
            vec![
                rate("tier_one_fee", sched.tier_one),
                rate("established_multiplier", estab_mult),
            ],
        )
    } else {
        let mult = tier_multiple(&cons)?;
        let ts = int(sched.tier_step);
        let rcap = int(sched.residential_cap);
        let raw_single = ts.mul(&mult)?;
        let capped_single = if raw_single.gt(&rcap)? {
            rcap.clone()
        } else {
            raw_single
        };
        let whole = capped_single.mul(&em)?;
        let ws = step(
            "whole_property_application_fee",
            "Fees Regulations 2020 s 9 item 1, s 10(1) items 2 and 3, s 10(2) and s 11: residential land fee above $1 million, capped at the s 10 maximum (3x for established dwellings from 9 April 2024)",
            "min(tier_step x tier_multiple, residential_fee_cap) x established_multiplier",
            &[&ts, &mult, &rcap, &em],
            whole.clone(),
        );
        (
            ws,
            whole,
            vec![
                rate("tier_step", sched.tier_step),
                rate("tier_multiple", mult),
                rate("residential_fee_cap", sched.residential_cap),
                rate("established_multiplier", estab_mult),
            ],
        )
    };

    let (steps, base_fee) = whole_and_share_steps(whole_step, whole, share.as_ref())?;
    Ok(FeeBase {
        steps,
        rates,
        base_fee,
        fee_basis: "notional_schedule",
        dwelling_kind: Some(dk),
        application_date: Some(app_day),
    })
}

pub fn residential_vacancy_fee(args: &ResidentialVacancyFeeArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let start = vacancy_start_in_year(args.vacancy_year_start, &year)?;
    let end = dates::months_end(start, 12);
    let days_in_year = end - start + 1;
    let return_due = end + RETURN_DUE_DAYS;
    let retention_until = dates::add_months(end, 60);

    let held_full = flag(args.held_full_vacancy_year, "held_full_vacancy_year")?;
    let occ_days = occupied_days(args.residentially_occupied_days, days_in_year)?;
    let lodged_on_time = flag(args.return_lodged_on_time, "return_lodged_on_time")?;
    let exemption = parse_exemption(args.exemption)?;
    let basis = parse_acquisition_basis(args.acquisition_basis)?;
    let ownership = parse_ownership(args.ownership)?;
    let waiver = parse_waiver(args.application_fee_waiver, basis)?;
    let discretionary = flag(
        args.vacancy_fee_waiver_or_remission,
        "vacancy_fee_waiver_or_remission",
    )?;
    if discretionary {
        return refuse(
            "residential_vacancy_fee.discretionary_waiver_unsupported",
            "Discretionary waiver or remission of a vacancy fee under FATA s 115H is decided case by case by the Treasurer or Commissioner and cannot be calculated here.",
        );
    }

    let status = if !held_full {
        LiabilityStatus::NotRequired
    } else if !lodged_on_time {
        LiabilityStatus::DeemedLiableLateReturn
    } else if occ_days >= MIN_OCCUPIED_DAYS {
        LiabilityStatus::NotLiableOccupied
    } else if exemption.is_some() {
        LiabilityStatus::Exempt
    } else {
        LiabilityStatus::LiableVacant
    };

    let window = VacancyWindow {
        start,
        end,
        return_due,
        lodged_on_time,
    };
    let fee_base = precise(|| resolve_fee_base(basis, ownership, waiver, window, &args.optional))?;

    let vac_mult: i64 = if start >= parse_day(APRIL_2024_AMENDMENT) {
        2
    } else {
        1
    };
    let vm = int(vac_mult);

    let mut steps = fee_base.steps;
    let mut rates = fee_base.rates;
    rates.push(rate("vacancy_fee_multiplier", vac_mult));

    let (uncapped_step, payable_step, cap_rate) = precise(|| {
        if !status.liable() {
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
            let u = step("uncapped_vacancy_fee", nil_auth, "0", &[], int(0));
            let p = step("vacancy_fee_payable", nil_auth, "0", &[], int(0));
            return Ok((u, p, None));
        }

        let uncapped = fee_base.base_fee.mul(&vm)?;
        let uncapped_auth = if vac_mult == 2 {
            "Fees Regulations 2020 Part 3 (ss 43 to 46) and s 67(2) (Act No. 17 of 2024): double the application fee basis for vacancy years starting on or after 9 April 2024"
        } else {
            "Fees Regulations 2020 Part 3 (ss 43 to 46, pre-9 April 2024 compilation): equal to the application fee basis for vacancy years starting before 9 April 2024"
        };
        let u_step = step(
            "uncapped_vacancy_fee",
            uncapped_auth,
            "application_fee_base x vacancy_fee_multiplier",
            &[&fee_base.base_fee, &vm],
            uncapped.clone(),
        );

        // Fees Imposition Act 2015 s 7(3) guarantees the s 6(3) statutory cap
        // (enforced by Fees Regulations s 61A) never decreases on later 1 July
        // indexations. Therefore, the cap in force at vacancy_year_end (or
        // $7,791,000 for 2027-28+) is a strict lower bound for every possible
        // FATA s 115E notice date and s 115F payment due date. Whenever the
        // uncapped fee is at or below that floor, the cap cannot bind; whenever
        // it exceeds the floor, the applicable cap depends on when the s 115E
        // notice is issued or the s 115F payment falls due (which may follow a
        // later 1 July indexation), so the worksheet refuses cleanly.
        let floor_cap = statutory_cap_for(end).unwrap_or(CAP_FLOOR_2026_27);
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
        Ok((u_step, p_step, Some(("statutory_fee_cap_floor", floor_cap))))
    })?;

    if let Some((k, v)) = cap_rate {
        rates.push(rate(k, v));
    }
    steps.push(uncapped_step);
    steps.push(payable_step);

    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["vacancy_year_start"] = json!(dates::format(start));
    out["vacancy_year_end"] = json!(dates::format(end));
    out["days_in_vacancy_year"] = json!(days_in_year);
    out["return_required"] = json!(held_full);
    out["return_due_date"] = if held_full {
        json!(dates::format(return_due))
    } else {
        Value::Null
    };
    out["record_retention_until"] = if held_full {
        json!(dates::format(retention_until))
    } else {
        Value::Null
    };
    out["liable"] = json!(status.liable());
    out["liability_status"] = json!(status.as_str());
    out["fee_basis"] = json!(fee_base.fee_basis);
    out["exemption_applied"] = if status == LiabilityStatus::Exempt {
        json!(exemption.map(|(k, _)| k))
    } else {
        Value::Null
    };
    out["payment_due_rule"] = if status.liable() {
        json!(
            "Due on the day specified in the s 115E notice of liability, at least 21 days after the notice is given (FATA 1975 s 115F)."
        )
    } else {
        Value::Null
    };
    out["civil_penalty_units"] = json!(CIVIL_PENALTY_UNITS);
    out["joint_ownership_note"] = json!(match ownership {
        Ownership::Sole =>
            "Sole foreign owner: lodges one return and is liable for the full vacancy fee if payable.",
        Ownership::JointTenants =>
            "Joint tenants (FATA s 115D(4), Fees Regulations s 57(1)): one return by any foreign joint tenant with the same vacancy year satisfies the lodgment obligation, and one vacancy fee is payable jointly.",
        Ownership::TenantsInCommon =>
            "Tenants in common (Fees Regulations s 57(2)): each foreign tenant in common must lodge a separate vacancy fee return and pays only their own proportionate share.",
    });
    let ban_note = match (fee_base.dwelling_kind, fee_base.application_date) {
        (Some(DwellingKind::Established), Some(day))
            if (parse_day(ESTABLISHED_BAN_FROM)..=parse_day(ESTABLISHED_BAN_TO)).contains(&day) =>
        {
            json!(
                "Foreign persons are generally banned from purchasing established dwellings from 1 April 2025 to 30 June 2029 unless a limited exception applies (ATO 'Types of property a foreign person can buy' and Treasury GN6 v5)."
            )
        }
        _ => Value::Null,
    };
    out["established_dwelling_ban_note"] = ban_note;
    Ok(out)
}
