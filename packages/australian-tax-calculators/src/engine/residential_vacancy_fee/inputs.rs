// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::schedules::{Schedule, parse_day, schedule_for};
use crate::dates;
use crate::decimal::Decimal;
use crate::engine::business_capital::year_first;
use crate::engine::{Arg, Result, date_arg, int, refuse};

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AcquisitionBasis {
    NotifiableAction,
    DeveloperExemptionCertificate,
    OtherExemptionCertificate,
    NoNoticeOrder,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Ownership {
    Sole,
    JointTenants,
    TenantsInCommon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Waiver {
    None,
    Full,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DwellingKind {
    NewOrNearNew,
    Established,
}

pub(super) fn vacancy_start_in_year(given: &Arg, year: &str) -> Result<i64> {
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

pub(super) fn occupied_days(given: &Arg, days_in_year: i64) -> Result<i64> {
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

pub(super) fn parse_exemption(given: &Arg) -> Result<Option<(&'static str, &'static str)>> {
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

pub(super) fn parse_acquisition_basis(given: &Arg) -> Result<AcquisitionBasis> {
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

pub(super) fn parse_ownership(given: &Arg) -> Result<Ownership> {
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

pub(super) fn parse_waiver(given: &Arg, basis: AcquisitionBasis) -> Result<Waiver> {
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

pub(super) fn parse_dwelling_kind(
    given: Option<&Arg>,
    basis: AcquisitionBasis,
) -> Result<DwellingKind> {
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

pub(super) fn parse_application_date(
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

pub(super) fn parse_ownership_share(
    given: Option<&Arg>,
    ownership: Ownership,
) -> Result<Option<Decimal>> {
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
