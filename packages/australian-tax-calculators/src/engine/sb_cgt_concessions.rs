// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `sb_cgt_concessions`: which small business CGT concessions in ITAA 1997
//! Division 152 are open for a capital gain from CGT event A1, condition by
//! condition, with the provision for each. The citations are to ITAA 1997
//! compilation 264 (21 May 2026): Division 152 in volume 4, s 104-190 in
//! volume 3 and s 328-110 in volume 7, read on 19 November 2026. That is the
//! text for 2025-26 and 2026-27: Act No. 49 of 2026 amends Division 152 for
//! the income year that includes 1 July 2027 and later years (Schedule 1
//! item 82). Whether a business was carried on, each aggregated turnover, the
//! net value of the CGT assets, the days the asset was active, retirement,
//! incapacity and the significant individual are established facts: the
//! worksheet applies the tests to them.

use serde_json::{Value, json};

use super::business_capital::year_first;
use super::{Arg, Result, date_arg, flag, int, money, rate, refuse, result, scope, signed_money};

mod business;
use crate::dates;
use crate::decimal::Decimal;
use business::{Active, active_asset, anniversary, choice, optional_date, small_business};

const KIND: &str = "sb_cgt_concessions";

/// The $2 million that s 152-10(1AA)(b) puts in place of each $10 million in
/// s 328-110. A business under it is also under $10 million on each rule of
/// s 328-110, so s 152-10(1AA)(a) adds nothing to the test.
pub(super) const TURNOVER: &str = "2000000";
/// Section 152-15.
const NET_ASSETS: &str = "6000000";
/// Section 152-320(1).
const RETIREMENT_LIMIT: &str = "500000";
/// The day CGT began: an asset acquired before it is a pre-CGT asset.
const CGT_START: &str = "1985-09-20";

const SMALL_BUSINESS: &str = "ITAA 1997 s 152-10(1AA) and s 328-110(1), (3) and (4), each $10 million read as $2 million: a business carried on in the income year (s 328-110(5) for one being wound up) and an aggregated turnover under $2 million for the previous year, likely for the year (not after 2 years at $2 million or more), or for the year at its end";
const NET_ASSET_VALUE: &str = "ITAA 1997 s 152-15: just before the CGT event the net value of the CGT assets of yours, of entities connected with you and of your affiliates and entities connected with them (s 152-20) does not exceed $6,000,000";
const NOT_OWN_BUSINESS_AFFILIATE: &str = "ITAA 1997 s 152-10(1A)(b): you do not carry on a business in the income year (other than in partnership)";
const NOT_OWN_BUSINESS_PARTNER: &str = "ITAA 1997 s 152-10(1B)(c): you do not carry on a business in the income year (other than in partnership)";
const GATEWAY: &str = "ITAA 1997 s 152-10(1)(c): you are a CGT small business entity, you satisfy the maximum net asset value test, you are a partner in a partnership that is a CGT small business entity and the asset is an interest in its asset, or s 152-10(1A) or (1B) is satisfied for a passively held asset";
const ACTIVE_ASSET: &str = "ITAA 1997 s 152-35: the asset was an active asset for at least half the test period, or for at least 7.5 years of it if owned for more than 15 years; the period runs from acquisition to the CGT event, or to the business's cessation if it ceased in the 12 months before the event or a longer period the Commissioner allows";
const BASIC: &str = "ITAA 1997 s 152-10(1): a CGT event happens to your CGT asset in the income year and gives a capital gain, s 152-10(1)(c) is satisfied and the asset satisfies the active asset test";
const OWNED_15_INDIVIDUAL: &str = "ITAA 1997 s 152-105(b): you continuously owned the CGT asset for the 15-year period ending just before the CGT event";
const OWNED_15_ENTITY: &str = "ITAA 1997 s 152-110(1)(b): the entity continuously owned the CGT asset for the 15-year period ending just before the CGT event";
const SIGNIFICANT_15: &str = "ITAA 1997 s 152-110(1)(c): the entity had a significant individual (s 152-55, a small business participation percentage of at least 20%) for a total of at least 15 years during which it owned the asset";
const SIGNIFICANT_BEFORE: &str = "ITAA 1997 ss 152-50, 152-110(1)(d) and 152-305(2)(b): the entity had at least one significant individual just before the CGT event";
const RETIRING_INDIVIDUAL: &str = "ITAA 1997 s 152-105(d): you are 55 or over at the time of the CGT event and it happens in connection with your retirement, or you are permanently incapacitated at that time";
const RETIRING_ENTITY: &str = "ITAA 1997 s 152-110(1)(d): an individual who was a significant individual just before the CGT event was 55 or over then and the event happened in connection with their retirement, or was permanently incapacitated then";
const LIMIT_LEFT: &str = "ITAA 1997 s 152-320(1): the CGT retirement exemption limit, $500,000 less the CGT exempt amounts of earlier choices, is not used up";

const FIFTEEN_YEAR: &str = "ITAA 1997 Subdivision 152-B, s 152-105 for an individual and s 152-110 for a company or trust: the whole capital gain is disregarded";
const REDUCTION: &str = "ITAA 1997 Subdivision 152-C, ss 152-205, 152-215 and 152-220: the gain left after step 3 of the method statement in s 102-5(1) is reduced by 50%, unless you choose otherwise; not for a gain the 15-year exemption covers";
const RETIREMENT: &str = "ITAA 1997 Subdivision 152-D, ss 152-305, 152-320 and 152-330: all or part of the gain, up to the CGT retirement exemption limit, can be disregarded; not for a gain the 15-year exemption covers";
const ROLLOVER: &str = "ITAA 1997 Subdivision 152-E, ss 152-410 and 152-430, and s 104-190(1A): the gain can be rolled over to a replacement active asset or capital improvement in the replacement asset period; not for a gain the 15-year exemption covers";

/// The facts as the command passes them; see the README for each.
pub struct SbCgtConcessionsArgs<'a> {
    pub entity: &'a Arg,
    pub share_or_trust_interest: &'a Arg,
    pub acquired: &'a Arg,
    pub event_date: &'a Arg,
    pub partnership_asset: &'a Arg,
    pub passively_held: &'a Arg,
    pub business: &'a Arg,
    pub previous_year_turnover: &'a Arg,
    pub year_before_turnover: &'a Arg,
    pub likely_turnover: &'a Arg,
    pub year_turnover: &'a Arg,
    pub net_asset_value: &'a Arg,
    pub active_days: &'a Arg,
    pub business_ceased: &'a Arg,
    pub date_of_birth: &'a Arg,
    pub retirement: &'a Arg,
    pub permanently_incapacitated: &'a Arg,
    pub last_rollover_event: &'a Arg,
    pub earnout_latest_due: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    /// Each fact only some answers call for, in this order:
    /// `small_business_when_stopped` (a business being wound up),
    /// `longer_period_allowed` (a business that ceased),
    /// `carries_on_own_business` (a passively held asset),
    /// `significant_individual_before_event` and
    /// `significant_individual_15_years` (a company or trust), and
    /// `retirement_exemption_used` and `choice_date` (an individual).
    pub optional: [Option<&'a Arg>; 7],
}

/// The optional facts' names, in the order of `SbCgtConcessionsArgs::optional`.
pub const SB_CGT_OPTIONAL: [&str; 7] = [
    "small_business_when_stopped",
    "longer_period_allowed",
    "carries_on_own_business",
    "significant_individual_before_event",
    "significant_individual_15_years",
    "retirement_exemption_used",
    "choice_date",
];

pub(super) fn conflict<T>(message: impl Into<String>) -> Result<T> {
    refuse("sb_cgt_concessions.fact_conflict", message)
}

pub(super) fn invalid<T>(message: &str) -> Result<T> {
    refuse("sb_cgt_concessions.fact_invalid", message.to_string())
}

/// The optional facts: each required where `wanted` says so and refused
/// otherwise.
fn optional_facts<'a>(
    args: &SbCgtConcessionsArgs<'a>,
    wanted: [bool; 7],
) -> Result<[Option<&'a Arg>; 7]> {
    for ((key, given), want) in SB_CGT_OPTIONAL.iter().zip(args.optional).zip(wanted) {
        match (want, given) {
            (true, None) => {
                return refuse(
                    "input.missing_field",
                    format!("{key} is required for these facts."),
                );
            }
            (false, Some(_)) => {
                return refuse(
                    "input.field_unknown",
                    format!("{key} is not an input for these facts"),
                );
            }
            _ => {}
        }
    }
    Ok(args.optional)
}

fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

/// The facts every test reads, checked against each other and the year.
struct Facts<'a> {
    year: String,
    individual: bool,
    partnership_asset: bool,
    passively_held: &'a str,
    optional: [Option<&'a Arg>; 7],
    acquired: i64,
    event: i64,
    /// The last CGT event in the year for which the roll-over is obtained
    /// (s 104-190(1A)(a)): this one unless a later one is given.
    last_event: i64,
    /// The latest time a financial benefit could become due under a
    /// look-through earnout right for the disposal (s 104-190(1A)(b)(ii)).
    earnout: Option<i64>,
    ceased: Option<i64>,
}

fn facts<'a>(args: &SbCgtConcessionsArgs<'a>) -> Result<Facts<'a>> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let individual = match args.entity.text() {
        Some("individual") => true,
        Some("company" | "trust") => false,
        _ => {
            return refuse(
                "sb_cgt_concessions.entity_invalid",
                "entity must be individual, company or trust.",
            );
        }
    };
    let (partnership_asset, passively_held) = asset(args)?;
    let ceased = optional_date(args.business_ceased, "business_ceased")?;
    let wanted = [
        args.business.text() == Some("winding_up"),
        ceased.is_some(),
        passively_held != "no",
        !individual,
        !individual,
        individual,
        individual,
    ];
    let optional = optional_facts(args, wanted)?;
    let (acquired, event, last_event) = contract_dates(args, &year)?;
    let earnout = optional_date(args.earnout_latest_due, "earnout_latest_due")?;
    if earnout.is_some_and(|due| due < event) {
        return conflict("earnout_latest_due must be on or after event_date.");
    }
    Ok(Facts {
        year,
        individual,
        partnership_asset,
        passively_held,
        optional,
        acquired,
        event,
        last_event,
        earnout,
        ceased,
    })
}

/// Whether the asset is an interest in a partnership asset and whether it
/// is passively held, refusing a share or trust interest.
fn asset<'a>(args: &SbCgtConcessionsArgs<'a>) -> Result<(bool, &'a str)> {
    if flag(args.share_or_trust_interest, "share_or_trust_interest")? {
        return refuse(
            "sb_cgt_concessions.share_or_trust_interest",
            "A share in a company or an interest in a trust must also meet the additional basic conditions in s 152-10(2), which look through to the company or trust: this worksheet does not work them out.",
        );
    }
    let partnership_asset = flag(args.partnership_asset, "partnership_asset")?;
    let passively_held = choice(
        args.passively_held,
        &["no", "affiliate_or_connected_entity", "partnership"],
        "passively_held must be no, affiliate_or_connected_entity or partnership.",
    )?;
    if partnership_asset && passively_held != "no" {
        return conflict(
            "s 152-10(1A)(c) and (1B)(d) are for an asset that is not an interest in an asset of a partnership you are a partner in: give partnership_asset false or passively_held no.",
        );
    }
    Ok((partnership_asset, passively_held))
}

/// The acquisition, CGT event and last rolled over CGT event dates, refused
/// where the worksheet cannot apply the tests to them.
fn contract_dates(args: &SbCgtConcessionsArgs, year: &str) -> Result<(i64, i64, i64)> {
    let first = year_first(year).expect("a supported period is an income year");
    let (start, end) = (
        dates::parse(&format!("{first}-07-01")).expect("a date"),
        dates::parse(&format!("{}-06-30", first + 1)).expect("a date"),
    );
    let acquired = date_arg(args.acquired, "acquired")?;
    let event = date_arg(args.event_date, "event_date")?;
    if !(start..=end).contains(&event) {
        return conflict(format!(
            "event_date must be in the {year} income year, from {} to {}.",
            dates::format(start),
            dates::format(end)
        ));
    }
    if acquired >= event {
        return conflict("acquired must be before event_date.");
    }
    let last = optional_date(args.last_rollover_event, "last_rollover_event")?.unwrap_or(event);
    if !(event..=end).contains(&last) {
        return conflict(format!(
            "last_rollover_event must be from event_date to the end of the {year} income year, {}.",
            dates::format(end)
        ));
    }
    if acquired < dates::parse(CGT_START).expect("a date") {
        return refuse(
            "sb_cgt_concessions.pre_cgt_asset",
            "An asset acquired before 20 September 1985 is a pre-CGT asset: the capital gain is disregarded (s 104-10(5)), so no concession is needed.",
        );
    }
    if anniversary(acquired, 15) == event {
        return refuse(
            "sb_cgt_concessions.fifteen_years_same_day",
            "Whether an asset acquired on the same date 15 years before the CGT event was owned for the 15 years before it (ss 152-35 and 152-105) turns on the time of day of each contract: this worksheet cannot tell.",
        );
    }
    Ok((acquired, event, last))
}

/// The CGT small business entity row's id and the paragraph that makes the
/// business entity's turnover the test.
fn business_entity(f: &Facts) -> (&'static str, &'static str) {
    match (f.partnership_asset, f.passively_held) {
        (true, _) => (
            "partnership_cgt_small_business_entity",
            "ITAA 1997 s 152-10(1)(c)(iii)",
        ),
        (_, "affiliate_or_connected_entity") => (
            "affiliate_or_connected_entity_cgt_small_business_entity",
            "ITAA 1997 s 152-10(1A)(a) and (d)",
        ),
        (_, "partnership") => (
            "partnership_cgt_small_business_entity",
            "ITAA 1997 s 152-10(1B)(a), (b) and (e)",
        ),
        _ => ("cgt_small_business_entity", "ITAA 1997 s 152-10(1)(c)(i)"),
    }
}

/// The basic conditions' rows, and whether they are all met.
fn basic_conditions(
    args: &SbCgtConcessionsArgs,
    f: &Facts,
    active: &Active,
    net_assets: bool,
    rows: &mut Vec<Value>,
) -> Result<bool> {
    let [stopped, _, own_business, ..] = f.optional;
    let business = small_business(args, stopped)?;
    let (id, authority) = business_entity(f);
    let mut row = condition(id, &format!("{authority}; {SMALL_BUSINESS}"), business.met);
    row["rule"] = json!(business.rule);
    rows.push(row);
    let mut gateway = business.met;
    if let Some(own) = own_business {
        let own = flag(own, "carries_on_own_business")?;
        let authority = if f.passively_held == "partnership" {
            NOT_OWN_BUSINESS_PARTNER
        } else {
            NOT_OWN_BUSINESS_AFFILIATE
        };
        rows.push(condition("no_business_of_your_own", authority, !own));
        gateway &= !own;
    }
    rows.push(condition(
        "maximum_net_asset_value_test",
        NET_ASSET_VALUE,
        net_assets,
    ));
    gateway |= net_assets;
    rows.push(condition("small_business_or_net_assets", GATEWAY, gateway));
    rows.push(condition("active_asset_test", ACTIVE_ASSET, active.met));
    let basic = gateway && active.met;
    rows.push(condition("basic_conditions", BASIC, basic));
    Ok(basic)
}

/// The 15-year exemption's rows and its unmet conditions, with whether a
/// company or trust had a significant individual just before the event.
fn fifteen_year(
    f: &Facts,
    basic: bool,
    owned_15: bool,
    retiring: bool,
    rows: &mut Vec<Value>,
) -> Result<(Vec<&'static str>, bool)> {
    let [.., before, fifteen, _, _] = f.optional;
    let mut unmet = Vec::new();
    if !basic {
        unmet.push("basic_conditions");
    }
    let owned = if f.individual {
        OWNED_15_INDIVIDUAL
    } else {
        OWNED_15_ENTITY
    };
    rows.push(condition("owned_for_15_years", owned, owned_15));
    if !owned_15 {
        unmet.push("owned_for_15_years");
    }
    let significant = match (before, fifteen) {
        (Some(before), Some(fifteen)) => significant_individual(before, fifteen, rows, &mut unmet)?,
        _ => true,
    };
    let age = if f.individual {
        RETIRING_INDIVIDUAL
    } else {
        RETIRING_ENTITY
    };
    rows.push(condition(
        "aged_55_and_retiring_or_permanently_incapacitated",
        age,
        retiring,
    ));
    if !retiring {
        unmet.push("aged_55_and_retiring_or_permanently_incapacitated");
    }
    Ok((unmet, significant))
}

/// A company's or trust's significant individual rows (ss 152-50 and
/// 152-110(1)(c)), and whether it had one just before the event.
fn significant_individual(
    before: &Arg,
    fifteen: &Arg,
    rows: &mut Vec<Value>,
    unmet: &mut Vec<&'static str>,
) -> Result<bool> {
    let fifteen = flag(fifteen, "significant_individual_15_years")?;
    let before = flag(before, "significant_individual_before_event")?;
    for (id, authority, met) in [
        (
            "significant_individual_for_15_years",
            SIGNIFICANT_15,
            fifteen,
        ),
        (
            "significant_individual_before_event",
            SIGNIFICANT_BEFORE,
            before,
        ),
    ] {
        rows.push(condition(id, authority, met));
        if !met {
            unmet.push(id);
        }
    }
    Ok(before)
}

/// The day the individual turns 55, and whether they are 55 or over and
/// retiring, or permanently incapacitated, at the CGT event
/// (ss 152-105(d) and 152-110(1)(d)).
fn retiring(args: &SbCgtConcessionsArgs, event: i64) -> Result<(i64, bool)> {
    let born = date_arg(args.date_of_birth, "date_of_birth")?;
    if born >= event {
        return conflict("date_of_birth must be before event_date.");
    }
    let fifty_fifth = anniversary(born, 55);
    let retires = flag(args.retirement, "retirement")?;
    let incapacitated = flag(args.permanently_incapacitated, "permanently_incapacitated")?;
    Ok((
        fifty_fifth,
        (fifty_fifth <= event && retires) || incapacitated,
    ))
}

/// An individual's CGT retirement exemption limit left (s 152-320(1)) and
/// whether the exempt amount goes to super, being under 55 just before the
/// choice (s 152-305(1)(b)).
fn retirement_limit(
    f: &Facts,
    fifty_fifth: i64,
    rows: &mut Vec<Value>,
) -> Result<(Option<Decimal>, Option<bool>)> {
    let [.., Some(used), Some(choice_date)] = f.optional else {
        return Ok((None, None));
    };
    let limit = Decimal::parse(RETIREMENT_LIMIT)?;
    let used = money(used)?;
    if used.gt(&limit)? {
        return conflict(
            "retirement_exemption_used cannot exceed the $500,000 CGT retirement exemption limit (s 152-320(1)).",
        );
    }
    let choice_day = date_arg(choice_date, "choice_date")?;
    if choice_day < f.event {
        return conflict("choice_date must be on or after event_date.");
    }
    let left = limit.sub(&used)?;
    rows.push(condition(
        "retirement_exemption_limit_left",
        LIMIT_LEFT,
        left.gt(&int(0))?,
    ));
    Ok((Some(left), Some(choice_day < fifty_fifth)))
}

/// The 4 concessions, each open or not with the conditions not met.
fn concessions(
    fifteen_unmet: Vec<&'static str>,
    basic: bool,
    retirement_extra: Vec<&'static str>,
) -> Vec<Value> {
    let fifteen_year = fifteen_unmet.is_empty();
    let after_15 = |extra: Vec<&'static str>| {
        let mut unmet = Vec::new();
        if fifteen_year {
            unmet.push("fifteen_year_exemption_applies");
        }
        if !basic {
            unmet.push("basic_conditions");
        }
        unmet.extend(extra);
        unmet
    };
    [
        ("fifteen_year_exemption", FIFTEEN_YEAR, fifteen_unmet),
        ("active_asset_reduction", REDUCTION, after_15(Vec::new())),
        ("retirement_exemption", RETIREMENT, after_15(retirement_extra)),
        ("rollover", ROLLOVER, after_15(Vec::new())),
    ]
    .into_iter()
    .map(|(id, authority, unmet)| {
        json!({"concession": id, "authority": authority, "available": unmet.is_empty(), "unmet": unmet})
    })
    .collect()
}

const RATES: [(&str, &str); 3] = [
    ("cgt_small_business_turnover_threshold", TURNOVER),
    ("maximum_net_asset_value", NET_ASSETS),
    ("retirement_exemption_limit", RETIREMENT_LIMIT),
];

/// `sb-cgt-concessions`.
pub fn sb_cgt_concessions(args: &SbCgtConcessionsArgs) -> Result<Value> {
    let f = facts(args)?;
    let longer = f.optional[1];
    let net_value = signed_money(args.net_asset_value)?;
    let net_assets = !net_value.gt(&Decimal::parse(NET_ASSETS)?)?;
    let active = active_asset(args, f.acquired, f.event, f.ceased.zip(longer))?;
    let mut rows = Vec::new();
    let basic = basic_conditions(args, &f, &active, net_assets, &mut rows)?;

    let (fifty_fifth, retiring) = retiring(args, f.event)?;
    let (fifteen_unmet, significant) =
        fifteen_year(&f, basic, active.over_15, retiring, &mut rows)?;
    let (limit_left, contribution) = retirement_limit(&f, fifty_fifth, &mut rows)?;
    let mut retirement_extra = Vec::new();
    if !significant {
        retirement_extra.push("significant_individual_before_event");
    }
    if let Some(left) = &limit_left
        && !left.gt(&int(0))?
    {
        retirement_extra.push("retirement_exemption_limit_left");
    }
    let rollover = basic && !fifteen_unmet.is_empty();
    let concessions = concessions(fifteen_unmet, basic, retirement_extra);

    let mut amounts = vec![("net_value_of_cgt_assets", net_value)];
    amounts.extend(limit_left.map(|left| ("retirement_exemption_limit_left", left)));
    let mut out = result(
        KIND,
        &f.year,
        amounts,
        RATES.map(|(k, v)| rate(k, v)).into(),
    )?;
    out["conditions"] = json!(rows);
    out["concessions"] = json!(concessions);
    out["active_asset_test"] = json!({
        "test_period_from": dates::format(active.from),
        "test_period_to": dates::format(active.to),
        "test_period_days": active.days,
        "active_days": active.active,
        "owned_more_than_15_years": active.over_15,
    });
    // s 152-305(1)(b) and (c): an individual under 55 just before the choice
    // contributes the CGT exempt amount to a complying super fund or RSA.
    out["super_contribution_required"] = json!(contribution);
    // s 104-190(1A): one year before the last CGT event in the year for
    // which the roll-over is obtained to the later of 2 years after it and,
    // for this disposal, 6 months after an earnout's latest due time.
    let two_years = anniversary(f.last_event, 2);
    let to = f
        .earnout
        .map_or(two_years, |due| two_years.max(dates::add_months(due, 6)));
    out["replacement_asset_period"] = if rollover {
        json!({"from": dates::format(anniversary(f.last_event, -1)), "to": dates::format(to)})
    } else {
        Value::Null
    };
    Ok(out)
}
