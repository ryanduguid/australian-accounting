// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::APRIL_2024_AMENDMENT;
use super::inputs::{
    AcquisitionBasis, DwellingKind, Ownership, Waiver, parse_application_date, parse_dwelling_kind,
    parse_ownership_share,
};
use super::schedules::{
    DE_MINIMIS_THRESHOLD, RESIDENTIAL_FEE_CONSTANT, Schedule, parse_day, schedule_for,
};
use crate::dates;
use crate::decimal::{Decimal, Rounding};
use crate::engine::{Arg, Result, Step, cents, int, money, rate, refuse, step};

pub(super) struct FeeBase {
    pub(super) steps: Vec<Step>,
    pub(super) rates: Vec<(String, String)>,
    pub(super) base_fee: Decimal,
    pub(super) fee_basis: &'static str,
    pub(super) dwelling_kind: Option<DwellingKind>,
    pub(super) application_date: Option<i64>,
}

#[derive(Clone, Copy)]
pub(super) struct VacancyWindow {
    pub(super) start: i64,
    pub(super) end: i64,
    pub(super) return_due: i64,
    pub(super) lodged_on_time: bool,
}

/// Computes the s 11(1) / s 11(2) integer multiple for consideration strictly
/// above the $1,000,000 residential fee constant.
fn tier_multiple(consideration: &Decimal) -> Result<Decimal> {
    let fc = int(RESIDENTIAL_FEE_CONSTANT);
    let quot = consideration.div(&fc)?;
    let floored = quot.quantize(0, Rounding::Down)?;
    let exact_multiple = floored.mul(&fc)?.compare(consideration)? == std::cmp::Ordering::Equal;
    if exact_multiple {
        // s 10(1) item 2 & s 11(1): floor((consideration - $1) / $1,000,000).
        Ok(floored.sub(&int(1))?)
    } else {
        // s 10(1) item 3 & s 11(2): floor(consideration / $1,000,000).
        Ok(floored)
    }
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

fn stated_fee_authority(basis: AcquisitionBasis) -> &'static str {
    match basis {
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
    }
}

fn stated_application_fee_base(
    basis: AcquisitionBasis,
    waiver: Waiver,
    fee_arg: &Arg,
    other_given: bool,
) -> Result<FeeBase> {
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
    if other_given {
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
    let s = step(
        "application_fee_base",
        stated_fee_authority(basis),
        "application_fee_payable",
        &[&fee],
        fee.clone(),
    );
    Ok(FeeBase {
        steps: vec![s],
        rates: Vec::new(),
        base_fee: fee,
        fee_basis: "stated_application_fee",
        dwelling_kind: None,
        application_date: None,
    })
}

/// The notional fee's authority where the post-2024 fee is waived: only a
/// notice or an order reaches it.
fn waived_fee_authority(basis: AcquisitionBasis) -> &'static str {
    match basis {
        AcquisitionBasis::NotifiableAction => {
            "Fees Regulations 2020 s 43(b) (as amended by Act No. 17 of 2024) and s 10(1) item 1, (2): notional tier-1 fee in force when the notice was given"
        }
        AcquisitionBasis::NoNoticeOrder => {
            "Fees Regulations 2020 s 46(b) (as amended by Act No. 17 of 2024) and s 10(1) item 1, (2): notional tier-1 fee in force when the notice was given"
        }
        _ => unreachable!(),
    }
}

fn waived_post_2024_base(
    basis: AcquisitionBasis,
    window: VacancyWindow,
    optional: &[Option<&Arg>; 5],
    share: Option<&Decimal>,
) -> Result<FeeBase> {
    let dk = parse_dwelling_kind(optional[2], basis)?;
    let (app_day, sched) = parse_application_date(optional[3], basis, window.start)?;
    let estab_mult = if dk == DwellingKind::Established && sched.established_triple {
        3
    } else {
        1
    };
    let tier_one = int(sched.tier_one);
    let em = int(estab_mult);
    let whole = tier_one.mul(&em)?;
    let authority = waived_fee_authority(basis);
    let whole_step = step(
        "whole_property_application_fee",
        authority,
        "tier_one_fee x established_multiplier",
        &[&tier_one, &em],
        whole.clone(),
    );
    let (steps, base_fee) = whole_and_share_steps(whole_step, whole, share)?;
    Ok(FeeBase {
        steps,
        rates: vec![
            rate("tier_one_fee", sched.tier_one),
            rate("established_multiplier", estab_mult),
        ],
        base_fee,
        fee_basis: "waived_tier_one",
        dwelling_kind: Some(dk),
        application_date: Some(app_day),
    })
}

/// A vacancy year starting from 1 July 2020 (2020-21, the first supported
/// year) to 8 April 2024 ends from 30 June 2021 and has its return due by 7 May
/// 2025, inside the schedule's rows.
fn pre_2024_waived_tier_one(window: VacancyWindow) -> Result<i64> {
    let fee = |day| schedule_for(day).expect("a pre-9 April 2024 schedule day");
    let (se, sd) = (fee(window.end), fee(window.return_due));
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
    Ok(se.tier_one)
}

fn waived_pre_2024_base(
    window: VacancyWindow,
    optional: &[Option<&Arg>; 5],
    share: Option<&Decimal>,
) -> Result<FeeBase> {
    if !window.lodged_on_time {
        return refuse(
            "residential_vacancy_fee.pre_2024_waiver_boundary_unsettled",
            "For a pre-9 April 2024 vacancy year with a waived application fee and a late return, the s 10 item 1 amount turns on when the Commissioner issues the s 115E notice or the fee becomes payable, which is outside this worksheet.",
        );
    }
    if optional[3].is_some() || optional[2].is_some() {
        return refuse(
            "residential_vacancy_fee.application_fee_invalid",
            "For a vacancy year starting before 9 April 2024 with a waived application fee, omit application_date and dwelling_kind: pre-9 April 2024 Fees Regulations ss 43(b) and 46(b) use the s 10 item 1 amount in force when the vacancy fee becomes payable.",
        );
    }
    let tier_one_val = pre_2024_waived_tier_one(window)?;
    let tier_one = int(tier_one_val);
    let whole_step = step(
        "whole_property_application_fee",
        "Fees Regulations 2020 (pre-9 April 2024 compilation) s 43(b) or s 46(b) and s 10 item 1: lowest residential tier fee",
        "tier_one_fee",
        &[&tier_one],
        tier_one.clone(),
    );
    let (steps, base_fee) = whole_and_share_steps(whole_step, tier_one, share)?;
    Ok(FeeBase {
        steps,
        rates: vec![rate("tier_one_fee", tier_one_val)],
        base_fee,
        fee_basis: "waived_tier_one",
        dwelling_kind: None,
        application_date: None,
    })
}

fn waived_fee_base(
    basis: AcquisitionBasis,
    ownership: Ownership,
    window: VacancyWindow,
    optional: &[Option<&Arg>; 5],
) -> Result<FeeBase> {
    if optional[1].is_some() {
        return refuse(
            "residential_vacancy_fee.consideration_invalid",
            "Omit consideration when application_fee_waiver is full: Fees Regulations ss 43(b) and 46(b) substitute s 10 item 1 regardless of consideration.",
        );
    }
    let share = parse_ownership_share(optional[4], ownership)?;
    if window.start >= parse_day(APRIL_2024_AMENDMENT) {
        waived_post_2024_base(basis, window, optional, share.as_ref())
    } else {
        waived_pre_2024_base(window, optional, share.as_ref())
    }
}

struct WholeFee {
    step: Step,
    amount: Decimal,
    rates: Vec<(String, String)>,
}

fn notional_above_million(cons: &Decimal, sched: Schedule, estab_mult: i64) -> Result<WholeFee> {
    let mult = tier_multiple(cons)?;
    let ts = int(sched.tier_step);
    let rcap = int(sched.residential_cap);
    let em = int(estab_mult);
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
    Ok(WholeFee {
        step: ws,
        amount: whole,
        rates: vec![
            rate("tier_step", sched.tier_step),
            rate("tier_multiple", mult),
            rate("residential_fee_cap", sched.residential_cap),
            rate("established_multiplier", estab_mult),
        ],
    })
}

fn notional_de_minimis_fee(
    basis: AcquisitionBasis,
    sched: Schedule,
    estab_mult: i64,
) -> Result<WholeFee> {
    if basis == AcquisitionBasis::DeveloperExemptionCertificate {
        return refuse(
            "residential_vacancy_fee.de_minimis_developer_ec_unsettled",
            "Consideration below $75,000 under a developer exemption certificate hits the unsettled boundary between Fees Regulations s 44(2) (Division 2 of Part 2) and s 53(1) (de minimis rule despite Part 2).",
        );
    }
    let em = int(estab_mult);
    let dm = int(sched.de_minimis);
    let whole = dm.mul(&em)?;
    let ws = step(
        "whole_property_application_fee",
        "Fees Regulations 2020 s 53(1): de minimis residential application fee for consideration below $75,000 (3x for established dwellings from 9 April 2024)",
        "de_minimis_fee x established_multiplier",
        &[&dm, &em],
        whole.clone(),
    );
    Ok(WholeFee {
        step: ws,
        amount: whole,
        rates: vec![
            rate("de_minimis_fee", sched.de_minimis),
            rate("established_multiplier", estab_mult),
        ],
    })
}

fn notional_whole_fee(
    basis: AcquisitionBasis,
    cons: &Decimal,
    sched: Schedule,
    estab_mult: i64,
) -> Result<WholeFee> {
    if cons.lt(&int(DE_MINIMIS_THRESHOLD))? {
        return notional_de_minimis_fee(basis, sched, estab_mult);
    }
    if !cons.gt(&int(RESIDENTIAL_FEE_CONSTANT))? {
        let em = int(estab_mult);
        let t1 = int(sched.tier_one);
        let whole = t1.mul(&em)?;
        let ws = step(
            "whole_property_application_fee",
            "Fees Regulations 2020 s 9 item 1 and s 10(1) item 1, (2): residential land fee for consideration up to $1 million (3x for established dwellings from 9 April 2024)",
            "tier_one_fee x established_multiplier",
            &[&t1, &em],
            whole.clone(),
        );
        return Ok(WholeFee {
            step: ws,
            amount: whole,
            rates: vec![
                rate("tier_one_fee", sched.tier_one),
                rate("established_multiplier", estab_mult),
            ],
        });
    }
    notional_above_million(cons, sched, estab_mult)
}

fn notional_fee_base(
    basis: AcquisitionBasis,
    ownership: Ownership,
    window: VacancyWindow,
    optional: &[Option<&Arg>; 5],
) -> Result<FeeBase> {
    if basis == AcquisitionBasis::OtherExemptionCertificate {
        return refuse(
            "residential_vacancy_fee.application_fee_invalid",
            "Supply application_fee_payable for other_exemption_certificate: Fees Regulations s 45(a) uses the Division 7 fee payable for that certificate (ss 32, 33 or 34).",
        );
    }
    let Some(cons_arg) = optional[1] else {
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
    let dk = parse_dwelling_kind(optional[2], basis)?;
    let (app_day, sched) = parse_application_date(optional[3], basis, window.start)?;
    let share = parse_ownership_share(optional[4], ownership)?;
    let estab_mult = if dk == DwellingKind::Established && sched.established_triple {
        3
    } else {
        1
    };
    let wf = notional_whole_fee(basis, &cons, sched, estab_mult)?;
    let (steps, base_fee) = whole_and_share_steps(wf.step, wf.amount, share.as_ref())?;
    Ok(FeeBase {
        steps,
        rates: wf.rates,
        base_fee,
        fee_basis: "notional_schedule",
        dwelling_kind: Some(dk),
        application_date: Some(app_day),
    })
}

pub(super) fn resolve_fee_base(
    basis: AcquisitionBasis,
    ownership: Ownership,
    waiver: Waiver,
    window: VacancyWindow,
    optional: &[Option<&Arg>; 5],
) -> Result<FeeBase> {
    if let Some(fee_arg) = optional[0] {
        let other_given = optional[1..].iter().any(Option::is_some);
        return stated_application_fee_base(basis, waiver, fee_arg, other_given);
    }
    if waiver == Waiver::Full {
        return waived_fee_base(basis, ownership, window, optional);
    }
    notional_fee_base(basis, ownership, window, optional)
}

#[cfg(test)]
mod tests {
    use super::{
        APRIL_2024_AMENDMENT, AcquisitionBasis, VacancyWindow, parse_day, pre_2024_waived_tier_one,
        stated_fee_authority, waived_fee_authority,
    };
    use crate::dates;

    /// Every vacancy year starting in a supported year before 9 April 2024
    /// ends, and has its return due, on days the schedule's rows cover: the
    /// fee is worked or the crossing refused, never a missing row.
    #[test]
    fn each_pre_2024_vacancy_year_has_its_schedule_days() {
        let first = crate::metadata::supported_periods("residential_vacancy_fee")[0];
        assert_eq!(first, "2020-21");
        let mut worked = 0;
        for start in parse_day("2020-07-01")..parse_day(APRIL_2024_AMENDMENT) {
            let end = dates::months_end(start, 12);
            let window = VacancyWindow {
                start,
                end,
                return_due: end + super::super::RETURN_DUE_DAYS,
                lodged_on_time: true,
            };
            worked += usize::from(pre_2024_waived_tier_one(window).is_ok());
        }
        assert!(worked > 1000, "{worked}");
    }

    #[test]
    #[should_panic(expected = "internal error: entered unreachable code")]
    fn a_developer_certificate_has_no_stated_fee() {
        stated_fee_authority(AcquisitionBasis::DeveloperExemptionCertificate);
    }

    /// Only a notice or an order is waived under the post-2024 rules, so no
    /// input brings an exemption certificate here.
    #[test]
    #[should_panic(expected = "internal error: entered unreachable code")]
    fn an_exemption_certificate_has_no_waived_fee() {
        waived_fee_authority(AcquisitionBasis::OtherExemptionCertificate);
    }

    #[test]
    fn a_no_notice_order_takes_the_s_46_fees() {
        let stated = stated_fee_authority(AcquisitionBasis::NoNoticeOrder);
        assert!(stated.starts_with("Fees Regulations 2020 s 46(a): "));
        let waived = waived_fee_authority(AcquisitionBasis::NoNoticeOrder);
        assert!(waived.starts_with("Fees Regulations 2020 s 46(b) "));
    }
}
