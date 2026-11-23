// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Each scheme's premium on one wages figure, in whole cents, worked in
//! integers as the site's calculator works it: wages in cents and rates in
//! millionths of 1 (ten-thousandths of 1%), each step rounded half up to the
//! cent unless the scheme rounds otherwise.

use super::super::{Result, refuse};

/// One scheme's facts beyond the wages, the rate and apprentice wages.
pub(super) enum Facts {
    /// The dust diseases rate (including GST) and the Safe Employer Reward.
    Nsw {
        dust: i128,
        reward: i128,
    },
    Vic(Vic),
    Qld(Qld),
    Sa(Sa),
    /// Western Australia, Tasmania, the ACT and the Northern Territory.
    Private,
}

pub(super) struct Vic {
    /// N in clause 11: the days in 2026-27 the employer is registered.
    pub days: i128,
    pub buy_out: bool,
}

pub(super) struct Qld {
    /// Section 4.1's minimum, including GST and duty, in cents.
    pub minimum: i128,
}

pub(super) struct Sa {
    pub first: bool,
    /// The claims costs (income support) in cents.
    pub claims: i128,
}

/// A scheme's premium: its steps, the premium before GST and duty, the GST,
/// the duty, the total and the scheme's figures.
pub(super) struct Premium {
    pub lines: Vec<(&'static str, i128)>,
    pub premium: i128,
    pub gst: i128,
    pub duty: i128,
    pub total: i128,
    pub rates: Vec<(&'static str, String)>,
}

/// `numerator / denominator` half up, for a fraction that is not negative.
fn half(numerator: i128, denominator: i128) -> i128 {
    (2 * numerator + denominator) / (2 * denominator)
}

/// Cents at a rate in millionths of 1, half up.
fn at(cents: i128, rate: i128) -> i128 {
    half(cents * rate, 1_000_000)
}

fn gst(cents: i128) -> i128 {
    half(cents, 10)
}

/// `$` and the cents as dollars, for a refusal's message.
fn shown(cents: i128) -> String {
    let whole = (cents / 100).to_string();
    let mut grouped = String::new();
    for (i, ch) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    format!("${grouped}.{:02}", cents % 100)
}

/// Apprentice wages are part of the wages.
pub(super) fn within(apprentice: i128, wages: i128) -> Result<()> {
    if apprentice > wages {
        return refuse(
            "workers_compensation_premium.apprentice_above_wages",
            "Apprentice wages are part of the wages: give no more than the wages.",
        );
    }
    Ok(())
}

pub(super) fn premium(facts: &Facts, wages: i128, rate: i128, apprentice: i128) -> Result<Premium> {
    within(apprentice, wages)?;
    match facts {
        Facts::Nsw { dust, reward } => nsw(wages, rate, apprentice, (*dust, *reward)),
        Facts::Vic(vic) => victoria(wages, rate, vic),
        Facts::Qld(qld) => queensland(wages, rate, apprentice, qld),
        Facts::Sa(sa) => south_australia(wages, rate, apprentice, sa),
        Facts::Private => Ok(private(wages, rate)),
    }
}

const NSW_SMALL_LIMIT: i128 = 3_000_000;
const NSW_MINIMUM: i128 = 24_000;

/// New South Wales, a small employer (icare): the average performance
/// premium less the Safe Employer Reward on it and the apprentice incentive,
/// plus GST, then the dust diseases contribution, whose rate includes GST.
fn nsw(wages: i128, rate: i128, apprentice: i128, (dust, reward): (i128, i128)) -> Result<Premium> {
    let app = at(wages, rate);
    if app > NSW_SMALL_LIMIT {
        return refuse(
            "workers_compensation_premium.experience_rated",
            format!(
                "The average performance premium of {} is more than $30,000, so icare experience-rates the premium on its claims model: use the renewal notice.",
                shown(app)
            ),
        );
    }
    let ser = at(app, reward);
    let incentive = at(apprentice, rate);
    let premium = app - ser - incentive;
    if premium < NSW_MINIMUM {
        return refuse(
            "workers_compensation_premium.below_minimum",
            format!(
                "The premium of {} is below icare's minimum premium of $240, and icare does not say whether the minimum includes GST: use the renewal notice.",
                shown(premium)
            ),
        );
    }
    let contribution = at(wages, dust);
    let gst = gst(premium);
    Ok(Premium {
        lines: vec![
            ("average_performance_premium", app),
            ("safe_employer_reward", ser),
            ("apprentice_incentive", incentive),
            ("dust_diseases_contribution", contribution),
        ],
        premium,
        gst,
        duty: 0,
        total: premium + gst + contribution,
        rates: vec![
            ("minimum_premium", "240".into()),
            ("small_employer_limit", "30000".into()),
        ],
    })
}

const VIC_MINIMUM: i128 = 40_000;
const VIC_DEDUCTIBLE_DOLLARS: i128 = 15_500;

/// Victoria, a new or small employer: Premiums Order (No. 34) clauses 6 and
/// 8 to 11 with Gazette S 284's values, remuneration to the dollar, the
/// deductible to the cent, the WorkCover and buy-out premiums down to it.
fn victoria(wages: i128, rate: i128, vic: &Vic) -> Result<Premium> {
    let remuneration = half(wages, 100);
    // Clause 9(2), in millionths of a dollar; clause 11 is the premium rate
    // x 15,500 x N / 365 for one rate, to the cent.
    let preliminary = remuneration * rate;
    let deductible = half(rate * VIC_DEDUCTIBLE_DOLLARS * vic.days, 365 * 10_000);
    // Clause 6(1) rounds 'all numbers' to 6 places; with N / 365 so rounded
    // the deductible can move a cent, and which reading holds is unsettled.
    let ratio = half(vic.days * 1_000_000, 365);
    let six_places = half(rate * VIC_DEDUCTIBLE_DOLLARS * ratio, 10_000_000_000);
    if six_places != deductible {
        return refuse(
            "workers_compensation_premium.rounding_unsettled",
            format!(
                "The deductible is {} worked exactly and {} with N / 365 rounded to 6 decimal places; which Premiums Order (No. 34) clause 6(1) requires is unsettled.",
                shown(deductible),
                shown(six_places)
            ),
        );
    }
    let worked = (preliminary - deductible * 10_000) / 10_000;
    let workcover = worked.max(VIC_MINIMUM);
    let buy_out = if vic.buy_out { workcover / 10 } else { 0 };
    let premium = workcover + buy_out;
    let gst = gst(premium);
    Ok(Premium {
        lines: vec![
            ("remuneration", remuneration * 100),
            ("preliminary_premium", half(preliminary, 10_000)),
            ("deductible_benefit", deductible),
            ("workcover_premium", workcover),
            ("buy_out_premium", buy_out),
        ],
        premium,
        gst,
        duty: 0,
        total: premium + gst,
        rates: vec![
            ("minimum_premium", "400".into()),
            ("buy_out_value", "0.1".into()),
            ("deductible_amount", "15500".into()),
            ("days_registered", vic.days.to_string()),
        ],
    })
}

/// Queensland, a policy that commences in 2026-27: the notice's sections
/// 3.2, 4.1, 5.2, 8.3 and 10.1, with 5% duty on the premium and its GST.
fn queensland(wages: i128, rate: i128, apprentice: i128, qld: &Qld) -> Result<Premium> {
    let estimated = at(wages, rate);
    let discount = at(apprentice, rate);
    let mut premium = estimated - discount;
    let mut gst = gst(premium);
    let mut duty = half((premium + gst) * 5, 100);
    if premium + gst + duty < qld.minimum {
        // The minimum includes GST and duty: the premium within it is the
        // minimum / 1.1 / 1.05.
        premium = half(qld.minimum * 1000, 1155);
        gst = half(premium, 10);
        duty = qld.minimum - premium - gst;
    }
    Ok(Premium {
        lines: vec![
            ("estimated_premium", estimated),
            ("apprentice_discount", discount),
        ],
        premium,
        gst,
        duty,
        total: premium + gst + duty,
        rates: vec![
            ("minimum_premium", format!("{}", qld.minimum / 100)),
            ("duty_rate", "0.05".into()),
        ],
    })
}

/// ReturnToWorkSA's discount factor in hundredths for a base premium below
/// each bound in dollars, then 30.
const SA_BANDS: [(i128, i128); 5] = [
    (10_000, 5),
    (50_000, 10),
    (100_000, 15),
    (500_000, 20),
    (1_000_000, 25),
];
const SA_TOP_BAND: i128 = 30;
const SA_MINIMUM_DOLLARS: i128 = 200;
/// Hundred-millionths of a cent, which hold every base premium exactly.
const UNIT: i128 = 100_000_000;

/// The discount factor for a base premium in hundred-millionths of a cent,
/// refused where it is above a bound less a cent and below the bound.
fn sa_band(base: i128) -> Result<i128> {
    for (bound, band) in SA_BANDS {
        let edge = bound * 100 * UNIT;
        if base > edge - UNIT && base < edge {
            return refuse(
                "workers_compensation_premium.band_edge_unsettled",
                format!(
                    "The base premium is above ${}.99 and below ${bound}: ReturnToWorkSA's table puts it in the lower band and its calculator in the higher one.",
                    bound - 1
                ),
            );
        }
        if base < edge {
            return Ok(band);
        }
    }
    Ok(SA_TOP_BAND)
}

/// South Australia: BP x (1 - D) + C - A with SUR nil, at least $200 and
/// rounded down to the whole dollar, as ReturnToWorkSA's 2026-27 calculator
/// works it.
fn south_australia(wages: i128, rate: i128, apprentice: i128, sa: &Sa) -> Result<Premium> {
    let base = wages * rate * 100;
    let apprentice_base = apprentice * rate * 100;
    let band = if sa.first { 0 } else { sa_band(base)? };
    let discount = wages * rate * band;
    let claims = sa.claims * UNIT;
    let applied = claims.min(3 * discount);
    let sum = base - discount + applied - apprentice_base;
    let floor = SA_MINIMUM_DOLLARS * 100 * UNIT;
    let premium = sum.max(floor) / (100 * UNIT) * 100;
    let gst = gst(premium);
    let cents = |amount: i128| half(amount, UNIT);
    Ok(Premium {
        lines: vec![
            ("base_premium", cents(base)),
            ("discount", cents(discount)),
            ("claims_costs_applied", cents(applied)),
            ("apprentice_incentive", cents(apprentice_base)),
        ],
        premium,
        gst,
        duty: 0,
        total: premium + gst,
        rates: vec![
            ("discount_factor", format!("0.{band:02}")),
            ("minimum_premium", "200".into()),
        ],
    })
}

/// Western Australia, Tasmania, the ACT and the Northern Territory: the
/// insurer's rate x wages, plus GST.
fn private(wages: i128, rate: i128) -> Premium {
    let premium = at(wages, rate);
    let gst = gst(premium);
    Premium {
        lines: Vec::new(),
        premium,
        gst,
        duty: 0,
        total: premium + gst,
        rates: Vec::new(),
    }
}
