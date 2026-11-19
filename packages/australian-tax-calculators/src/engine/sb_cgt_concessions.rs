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
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "sb_cgt_concessions";

/// The $2 million that s 152-10(1AA)(b) puts in place of each $10 million in
/// s 328-110. A business under it is also under $10 million on each rule of
/// s 328-110, so s 152-10(1AA)(a) adds nothing to the test.
const TURNOVER: &str = "2000000";
/// Section 152-15.
const NET_ASSETS: &str = "6000000";
/// Section 152-320(1).
const RETIREMENT_LIMIT: &str = "500000";
/// The shortest and longest 7.5 years in days: 7 years with 1 leap day and
/// the shortest 6 months (2,556 + 181), and 7 years with 2 leap days and the
/// longest 6 months (2,557 + 184). A total of active days between them may
/// or may not be 7.5 years, depending on the periods.
const SEVEN_AND_A_HALF: (i64, i64) = (2737, 2741);
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

fn conflict<T>(message: impl Into<String>) -> Result<T> {
    refuse("sb_cgt_concessions.fact_conflict", message)
}

fn invalid<T>(message: &str) -> Result<T> {
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

fn choice<'a>(value: &'a Arg, options: &[&'a str], message: &str) -> Result<&'a str> {
    match value.text() {
        Some(text) if options.contains(&text) => Ok(text),
        _ => invalid(message),
    }
}

/// A turnover, or `null` for a year no business was carried on.
fn turnover(value: &Arg) -> Result<Option<Decimal>> {
    match value {
        Arg::Other(none) if none == "None" => Ok(None),
        other => money(other).map(Some),
    }
}

/// A date, or `null`.
fn optional_date(value: &Arg, name: &str) -> Result<Option<i64>> {
    match value {
        Arg::Other(none) if none == "None" => Ok(None),
        other => date_arg(other, name).map(Some),
    }
}

/// The same day `years` later or earlier; 29 February becomes 1 March.
fn anniversary(day: i64, years: i64) -> i64 {
    let moved = dates::add_months(day, 12 * years);
    if dates::to_civil(moved).2 == dates::to_civil(day).2 {
        moved
    } else {
        moved + 1
    }
}

fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

/// Whether the business entity is a CGT small business entity, and by
/// which rule of s 328-110.
struct SmallBusiness {
    met: bool,
    rule: Option<&'static str>,
}

fn small_business(args: &SbCgtConcessionsArgs, stopped: Option<&Arg>) -> Result<SmallBusiness> {
    let business = choice(
        args.business,
        &["carried_on", "winding_up", "not_carried_on"],
        "business must be carried_on, winding_up or not_carried_on.",
    )?;
    let carried = match (business, stopped) {
        ("carried_on", _) => true,
        // s 328-110(5): a business being wound up counts as carried on if
        // the entity was a small business entity in the year it stopped.
        ("winding_up", Some(stopped)) => flag(stopped, "small_business_when_stopped")?,
        _ => false,
    };
    let previous = turnover(args.previous_year_turnover)?;
    let before = turnover(args.year_before_turnover)?;
    let likely = money(args.likely_turnover)?;
    let at_end = money(args.year_turnover)?;
    let limit = Decimal::parse(TURNOVER)?;
    let under = |amount: &Decimal| amount.lt(&limit);
    let previous_under = previous.as_ref().map(under).transpose()?;
    let barred = match (&previous, &before) {
        (Some(p), Some(b)) => !under(p)? && !under(b)?,
        _ => false,
    };
    let rule = if !carried {
        None
    } else if previous_under == Some(true) {
        Some("s 328-110(1)(b)(i)")
    } else if under(&likely)? && !barred {
        Some("s 328-110(1)(b)(ii)")
    } else if under(&at_end)? {
        Some("s 328-110(4)")
    } else {
        None
    };
    Ok(SmallBusiness {
        met: rule.is_some(),
        rule,
    })
}

/// The test period of s 152-35(2) and whether the asset was active for
/// long enough in it. Both ends are counted.
struct Active {
    met: bool,
    from: i64,
    to: i64,
    days: i64,
    active: i64,
    over_15: bool,
}

fn active_asset(
    args: &SbCgtConcessionsArgs,
    acquired: i64,
    event: i64,
    ceased: Option<(i64, &Arg)>,
) -> Result<Active> {
    let mut to = event;
    if let Some((ceased, longer)) = ceased {
        if ceased < acquired || ceased > event {
            return conflict("business_ceased must be from acquired to event_date.");
        }
        // s 152-35(2)(b)(ii): a business that ceased in the 12 months before
        // the event, or a longer period the Commissioner allows.
        if anniversary(ceased, 1) >= event || flag(longer, "longer_period_allowed")? {
            to = ceased;
        }
    }
    let days = to - acquired + 1;
    let active = match args.active_days.int() {
        Some(Some(n)) if (0..=days).contains(&n) => n,
        _ => {
            return conflict(format!(
                "active_days must be a whole number of days from 0 to the {days} days of the test period."
            ));
        }
    };
    let over_15 = anniversary(acquired, 15) < event;
    let (shortest, longest) = SEVEN_AND_A_HALF;
    if over_15 && (shortest..longest).contains(&active) {
        return refuse(
            "sb_cgt_concessions.active_period_unclear",
            "A total of 7.5 years is 2,737 to 2,741 days, depending on the leap days and months it spans: count the active periods in years and months to see whether they reach 7.5 years (s 152-35(1)(b)).",
        );
    }
    let met = if over_15 {
        active >= longest
    } else {
        2 * active >= days
    };
    Ok(Active {
        met,
        from: acquired,
        to,
        days,
        active,
        over_15,
    })
}

fn concession(id: &str, authority: &str, available: bool, unmet: Vec<&str>) -> Value {
    json!({"concession": id, "authority": authority, "available": available, "unmet": unmet})
}

/// `sb-cgt-concessions`.
pub fn sb_cgt_concessions(args: &SbCgtConcessionsArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let entity = match args.entity.text() {
        Some(e @ ("individual" | "company" | "trust")) => e,
        _ => {
            return refuse(
                "sb_cgt_concessions.entity_invalid",
                "entity must be individual, company or trust.",
            );
        }
    };
    let individual = entity == "individual";
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
    let business_ceased = optional_date(args.business_ceased, "business_ceased")?;
    let optional = optional_facts(
        args,
        [
            args.business.text() == Some("winding_up"),
            business_ceased.is_some(),
            passively_held != "no",
            !individual,
            !individual,
            individual,
            individual,
        ],
    )?;
    let [
        stopped,
        longer,
        own_business,
        before,
        fifteen,
        used,
        choice_date,
    ] = optional;

    let first = year_first(&year).expect("a supported period is an income year");
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

    let business = small_business(args, stopped)?;
    let net_value = signed_money(args.net_asset_value)?;
    let net_assets = !net_value.gt(&Decimal::parse(NET_ASSETS)?)?;
    let active = active_asset(args, acquired, event, business_ceased.zip(longer))?;

    let mut conditions = Vec::new();
    let (business_id, business_authority) = match (partnership_asset, passively_held) {
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
    };
    let mut row = condition(
        business_id,
        &format!("{business_authority}; {SMALL_BUSINESS}"),
        business.met,
    );
    row["rule"] = json!(business.rule);
    conditions.push(row);
    let mut gateway = business.met;
    if let Some(own) = own_business {
        let own = flag(own, "carries_on_own_business")?;
        let authority = if passively_held == "partnership" {
            NOT_OWN_BUSINESS_PARTNER
        } else {
            NOT_OWN_BUSINESS_AFFILIATE
        };
        conditions.push(condition("no_business_of_your_own", authority, !own));
        gateway &= !own;
    }
    conditions.push(condition(
        "maximum_net_asset_value_test",
        NET_ASSET_VALUE,
        net_assets,
    ));
    gateway |= net_assets;
    conditions.push(condition("small_business_or_net_assets", GATEWAY, gateway));
    conditions.push(condition("active_asset_test", ACTIVE_ASSET, active.met));
    let basic = gateway && active.met;
    conditions.push(condition("basic_conditions", BASIC, basic));

    let born = date_arg(args.date_of_birth, "date_of_birth")?;
    if born >= event {
        return conflict("date_of_birth must be before event_date.");
    }
    let fifty_fifth = anniversary(born, 55);
    let retires = flag(args.retirement, "retirement")?;
    let incapacitated = flag(args.permanently_incapacitated, "permanently_incapacitated")?;
    let retiring = (fifty_fifth <= event && retires) || incapacitated;
    let owned_15 = active.over_15;
    let mut unmet_15 = Vec::new();
    if !basic {
        unmet_15.push("basic_conditions");
    }
    conditions.push(condition(
        "owned_for_15_years",
        if individual {
            OWNED_15_INDIVIDUAL
        } else {
            OWNED_15_ENTITY
        },
        owned_15,
    ));
    if !owned_15 {
        unmet_15.push("owned_for_15_years");
    }
    let mut significant = true;
    if let (Some(before), Some(fifteen)) = (before, fifteen) {
        let fifteen = flag(fifteen, "significant_individual_15_years")?;
        significant = flag(before, "significant_individual_before_event")?;
        conditions.push(condition(
            "significant_individual_for_15_years",
            SIGNIFICANT_15,
            fifteen,
        ));
        conditions.push(condition(
            "significant_individual_before_event",
            SIGNIFICANT_BEFORE,
            significant,
        ));
        if !fifteen {
            unmet_15.push("significant_individual_for_15_years");
        }
        if !significant {
            unmet_15.push("significant_individual_before_event");
        }
    }
    conditions.push(condition(
        "aged_55_and_retiring_or_permanently_incapacitated",
        if individual {
            RETIRING_INDIVIDUAL
        } else {
            RETIRING_ENTITY
        },
        retiring,
    ));
    if !retiring {
        unmet_15.push("aged_55_and_retiring_or_permanently_incapacitated");
    }
    let fifteen_year = unmet_15.is_empty();

    let limit = Decimal::parse(RETIREMENT_LIMIT)?;
    let mut limit_left = None;
    let mut contribution = None;
    if let (Some(used), Some(choice_date)) = (used, choice_date) {
        let used = money(used)?;
        if used.gt(&limit)? {
            return conflict(
                "retirement_exemption_used cannot exceed the $500,000 CGT retirement exemption limit (s 152-320(1)).",
            );
        }
        let choice_day = date_arg(choice_date, "choice_date")?;
        if choice_day < event {
            return conflict("choice_date must be on or after event_date.");
        }
        let left = limit.sub(&used)?;
        conditions.push(condition(
            "retirement_exemption_limit_left",
            LIMIT_LEFT,
            left.gt(&int(0))?,
        ));
        // s 152-305(1)(b): under 55 just before the choice.
        contribution = Some(choice_day < fifty_fifth);
        limit_left = Some(left);
    }
    let left_met = match &limit_left {
        Some(left) => left.gt(&int(0))?,
        None => true,
    };

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
    let mut retirement_extra = Vec::new();
    if !individual && !significant {
        retirement_extra.push("significant_individual_before_event");
    }
    if !left_met {
        retirement_extra.push("retirement_exemption_limit_left");
    }
    let reduction_unmet = after_15(Vec::new());
    let retirement_unmet = after_15(retirement_extra);
    let rollover_unmet = after_15(Vec::new());
    let concessions = vec![
        concession(
            "fifteen_year_exemption",
            FIFTEEN_YEAR,
            fifteen_year,
            unmet_15,
        ),
        concession(
            "active_asset_reduction",
            REDUCTION,
            reduction_unmet.is_empty(),
            reduction_unmet,
        ),
        concession(
            "retirement_exemption",
            RETIREMENT,
            retirement_unmet.is_empty(),
            retirement_unmet,
        ),
        concession(
            "rollover",
            ROLLOVER,
            rollover_unmet.is_empty(),
            rollover_unmet.clone(),
        ),
    ];

    let mut amounts = vec![("net_value_of_cgt_assets", net_value)];
    if let Some(left) = limit_left {
        amounts.push(("retirement_exemption_limit_left", left));
    }
    let rates = vec![
        rate("cgt_small_business_turnover_threshold", TURNOVER),
        rate("maximum_net_asset_value", NET_ASSETS),
        rate("retirement_exemption_limit", RETIREMENT_LIMIT),
    ];
    let mut out = result(KIND, &year, amounts, rates)?;
    out["conditions"] = json!(conditions);
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
    // s 104-190(1A)(a) and (b)(i): one year before to 2 years after the last
    // CGT event in the year for which the roll-over is obtained, this one.
    out["replacement_asset_period"] = if rollover_unmet.is_empty() {
        json!({"from": dates::format(anniversary(event, -1)), "to": dates::format(anniversary(event, 2))})
    } else {
        Value::Null
    };
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::anniversary;
    use crate::dates;

    fn day(text: &str) -> i64 {
        dates::parse(text).expect("a date")
    }

    #[test]
    fn an_anniversary_of_29_february_is_1_march() {
        assert_eq!(anniversary(day("2012-02-29"), 15), day("2027-03-01"));
        assert_eq!(anniversary(day("2024-02-29"), -1), day("2023-03-01"));
        assert_eq!(anniversary(day("2011-03-15"), 15), day("2026-03-15"));
        assert_eq!(anniversary(day("2026-02-28"), 2), day("2028-02-28"));
    }
}
