// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `gst_margin_scheme`: whether the margin scheme of GST Act Division 75 can
//! apply to a taxable supply of real property, condition by condition, and the
//! GST on the margin against the GST without it. The citations are to the A New
//! Tax System (Goods and Services Tax) Act 1999 ss 9-90, 11-5, 75-5, 75-10,
//! 75-11, 75-12, 75-13, 75-14, 75-15, 75-20 and 75-35, as the ATO's legal
//! database held them, and GSTR 2006/8, all read on 19 November 2026.

use serde_json::{Value, json};

use super::business_capital::year_first;
use super::{
    Arg, Result, Step, date_arg, derived_result, flag, int, money, precise, py_max, rate, refuse,
    scope, step,
};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "gst_margin_scheme";
const ITEM_4_SCOPE: &str = "for government land under item 4";

/// The facts as the command passes them; see the README for each.
pub struct MarginSchemeArgs<'a> {
    pub interest: &'a Arg,
    pub agreement: &'a Arg,
    pub acquisition: &'a Arg,
    pub consideration: &'a Arg,
    pub method: &'a Arg,
    pub recipient_associate: &'a Arg,
    pub acquisition_consideration: Option<&'a Arg>,
    pub unpaid_acquisition_consideration: Option<&'a Arg>,
    pub approved_valuation: Option<&'a Arg>,
    pub valuation_day: Option<&'a Arg>,
    pub registered_on_1_july_2000: Option<&'a Arg>,
    pub government_unimproved_on_1_july_2000: Option<&'a Arg>,
    pub improved_on_1_july_2000: Option<&'a Arg>,
    pub registration_effective_day: Option<&'a Arg>,
    pub applied_for_registration: Option<&'a Arg>,
    pub registration_application_day: Option<&'a Arg>,
    pub supply_day: Option<&'a Arg>,
    pub subdivision_share: Option<&'a Arg>,
    pub subdivision_whole: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const INTEREST: &str = "GST Act s 75-5(1): the supply is made by selling a freehold interest in land or a stratum unit, or by granting or selling a long-term lease (at least 50 years: s 195-1)";
const AGREEMENT: &str = "GST Act s 75-5(1) and (1A): you and the recipient agreed in writing that the margin scheme is to apply, on or before making the supply or within a further period the Commissioner allows (GSTR 2006/8 paras 18 and 27: a contract applying it if the supply is taxable is enough)";
const ACQUISITION: &str = "GST Act s 75-5(2) and (3)(a): you did not acquire the entire interest, unit or lease through a taxable supply on which the GST was worked out without applying the margin scheme";

/// An acquisition the command does not work, with the provisions that govern
/// it instead.
const UNSUPPORTED: [(&str, &str); 8] = [
    (
        "gst_group_member",
        "from a fellow member of a GST group: eligibility under s 75-5(3)(c) and the margin under s 75-11(1) and (2)",
    ),
    (
        "joint_venture_operator",
        "from the operator of a GST joint venture you participated in: s 75-5(3)(d) and s 75-11(2A) and (2B)",
    ),
    (
        "inheritance",
        "by inheriting it: s 75-5(3)(b) and s 75-11(3) and (4)",
    ),
    (
        "gst_free_going_concern",
        "as or as part of a GST-free supply of a going concern: s 75-5(3)(e) and s 75-11(5)",
    ),
    (
        "gst_free_farm_land",
        "as or as part of a GST-free supply of farm land under Subdivision 38-O: s 75-5(3)(f) and s 75-11(5)",
    ),
    (
        "associate",
        "from your associate: s 75-5(3)(g) and s 75-11(6) to (8)",
    ),
    (
        "gst_free_government_land",
        "on or after 1 July 2000 through a supply GST-free under s 38-445(1A) relating to a pre-1 July 2000 lease: s 75-10(3) item 2A and GSTR 2006/7",
    ),
    (
        "part_through_ineligible_supply",
        "in part through a supply ineligible for the margin scheme: s 75-5(2) and the increasing adjustment of s 75-22",
    ),
];

/// One of `allowed`, or a refusal naming them.
fn choice<'a>(arg: &'a Arg, key: &str, allowed: &[&str]) -> Result<&'a str> {
    match arg.text() {
        Some(value) if allowed.contains(&value) => Ok(value),
        _ => refuse(
            "gst_margin_scheme.fact_invalid",
            format!("{key} must be one of {}.", allowed.join(", ")),
        ),
    }
}

/// How you acquired the interest, refusing the acquisitions s 75-11 governs.
fn acquisition(arg: &Arg) -> Result<&str> {
    if let Some((_, why)) = UNSUPPORTED.iter().find(|(k, _)| Some(*k) == arg.text()) {
        return refuse(
            "gst_margin_scheme.acquisition_unsupported",
            format!("An interest acquired {why}; this command does not work it."),
        );
    }
    let mut allowed = vec![
        "taxable_supply_under_margin_scheme",
        "taxable_supply_without_margin_scheme",
        "other_supply_from_1_july_2000",
        "before_1_july_2000",
    ];
    allowed.extend(UNSUPPORTED.iter().map(|(k, _)| *k));
    choice(arg, "acquisition", &allowed)
}

/// An input that `needed` decides: required when needed, refused when not.
fn when<'a>(needed: bool, given: Option<&'a Arg>, key: &str, why: &str) -> Result<Option<&'a Arg>> {
    match (needed, given) {
        (true, None) => refuse("input.missing_field", format!("{key} is required {why}.")),
        (false, Some(_)) => refuse(
            "input.field_unknown",
            format!("{key} is an input only {why}"),
        ),
        (_, given) => Ok(given),
    }
}

fn conflict<T>(message: &str) -> Result<T> {
    refuse("gst_margin_scheme.fact_conflict", message)
}

const CONSIDERATION_BASE: &str = "GST Act ss 75-10(2), 75-12 and 75-14: the consideration for your acquisition of the interest, unit or lease, treated as reduced by any of it unpaid at the supply, disregarding improvements, development work and other acquisitions in relation to the property";
const ITEM_1_OR_3: &str = "GST Act s 75-10(3) items 1 and 3: acquired before 1 July 2000 by a supplier registered or required to be registered then, an approved valuation (s 75-35) as at 1 July 2000";
const ITEM_2: &str = "GST Act s 75-10(3) item 2: acquired before 1 July 2000 by a supplier not registered or required to be registered until after then, an approved valuation (s 75-35) as at the date of effect of the registration, or the day of applying for it if earlier";
const ITEM_4: &str = "GST Act s 75-10(3) item 4 and (3A): the Commonwealth, a State or a Territory, holding the interest since before 1 July 2000 with no improvements on the land then, an approved valuation (s 75-35) as at the day of the supply, as if there were no improvements on the land that day";

/// The day a supported financial year's `first` July starts and its June ends.
fn year_days(year: &str) -> (i64, i64) {
    let first = year_first(year).expect("a supported year");
    let day = |text: String| dates::parse(&text).expect("a date literal");
    (
        day(format!("{first}-07-01")),
        day(format!("{}-06-30", first + 1)),
    )
}

fn required<'a>(given: Option<&'a Arg>, key: &str, why: &str) -> Result<&'a Arg> {
    when(true, given, key, why).map(|given| given.expect("required"))
}

/// Facts required only when item 2 determines the statutory valuation date.
fn later_registration_facts<'a>(
    args: &MarginSchemeArgs<'a>,
) -> [(Option<&'a Arg>, &'static str); 4] {
    [
        (args.improved_on_1_july_2000, "improved_on_1_july_2000"),
        (
            args.registration_effective_day,
            "registration_effective_day",
        ),
        (args.applied_for_registration, "applied_for_registration"),
        (
            args.registration_application_day,
            "registration_application_day",
        ),
    ]
}

fn later_registration_valuation_day(args: &MarginSchemeArgs, start: i64) -> Result<i64> {
    let why = "for a supplier first registered or required to register after 1 July 2000";
    let key = "improved_on_1_july_2000";
    if flag(required(args.improved_on_1_july_2000, key, why)?, key)? {
        return refuse(
            "gst_margin_scheme.valuation_item_unsettled",
            "Both s 75-10(3) item 2 (later registration) and item 3 (improvements on 1 July 2000) describe this supply, and their valuation days can differ.",
        );
    }
    let key = "registration_effective_day";
    let effective = date_arg(required(args.registration_effective_day, key, why)?, key)?;
    if effective <= start {
        return conflict(
            "registration_effective_day must be after 1 July 2000 for later registration.",
        );
    }
    let key = "applied_for_registration";
    let applied = flag(required(args.applied_for_registration, key, why)?, key)?;
    let key = "registration_application_day";
    let application = when(
        applied,
        args.registration_application_day,
        key,
        "when the supplier applied for GST registration",
    )?;
    Ok(match application {
        Some(arg) => effective.min(date_arg(arg, key)?),
        None => effective,
    })
}

fn government_supply_valuation_day(args: &MarginSchemeArgs, year: &str) -> Result<i64> {
    let key = "supply_day";
    let supply = date_arg(required(args.supply_day, key, ITEM_4_SCOPE)?, key)?;
    let (first, last) = year_days(year);
    if !(first..=last).contains(&supply) {
        return conflict("supply_day must be in the supply year.");
    }
    Ok(supply)
}

/// The s 75-10(3) item an approved valuation is under, checked against its day.
fn valuation_item(args: &MarginSchemeArgs, why: &str, year: &str) -> Result<&'static str> {
    let key = "registered_on_1_july_2000";
    let registered = flag(required(args.registered_on_1_july_2000, key, why)?, key)?;
    let key = "government_unimproved_on_1_july_2000";
    let government = flag(
        required(args.government_unimproved_on_1_july_2000, key, why)?,
        key,
    )?;
    let key = "valuation_day";
    let day = date_arg(required(args.valuation_day, key, why)?, key)?;
    let start = dates::parse("2000-07-01").expect("a date literal");
    if registered {
        none_of(
            &later_registration_facts(args),
            "for a later registration under item 2",
        )?;
    }
    if !government {
        when(false, args.supply_day, "supply_day", ITEM_4_SCOPE)?;
    }
    let (item, expected, description) = match (government, registered) {
        (true, false) => {
            return refuse(
                "gst_margin_scheme.valuation_item_unsettled",
                "Both s 75-10(3) item 2 (a supplier not registered until after 1 July 2000) and item 4 (government land unimproved on 1 July 2000) describe this supply, and their valuation days differ; this command does not choose between them.",
            );
        }
        (true, true) => (
            ITEM_4,
            government_supply_valuation_day(args, year)?,
            "the day of the supply",
        ),
        (false, false) => (
            ITEM_2,
            later_registration_valuation_day(args, start)?,
            "the date of effect of your registration or the earlier day you applied",
        ),
        (false, true) => (ITEM_1_OR_3, start, "1 July 2000"),
    };
    if day != expected {
        return refuse(
            "gst_margin_scheme.valuation_day_invalid",
            format!("valuation_day must be {description} for this valuation (s 75-10(3))."),
        );
    }
    Ok(item)
}

/// Refuses each input `given` that only the other method takes.
fn none_of(given: &[(Option<&Arg>, &str)], why: &str) -> Result<()> {
    for (given, key) in given {
        when(false, *given, key, why)?;
    }
    Ok(())
}

/// The base the margin is worked out from, before any s 75-15 proportion.
fn base_step(
    args: &MarginSchemeArgs,
    acquisition: &str,
    year: &str,
) -> Result<(Step, &'static str)> {
    let valuation = choice(
        args.method,
        "method",
        &["consideration", "approved_valuation"],
    )? == "approved_valuation";
    if valuation && acquisition != "before_1_july_2000" {
        return refuse(
            "gst_margin_scheme.valuation_unavailable",
            "An approved valuation is used under s 75-10(3) only for an interest acquired before 1 July 2000 (items 1 to 4); item 2A and s 75-11 are not worked here, so use method consideration.",
        );
    }
    if valuation {
        let consideration = [
            (args.acquisition_consideration, "acquisition_consideration"),
            (
                args.unpaid_acquisition_consideration,
                "unpaid_acquisition_consideration",
            ),
        ];
        none_of(&consideration, "with method consideration")?;
        return valuation_base(args, year);
    }
    let valued = [
        (args.approved_valuation, "approved_valuation"),
        (args.valuation_day, "valuation_day"),
        (args.registered_on_1_july_2000, "registered_on_1_july_2000"),
        (
            args.government_unimproved_on_1_july_2000,
            "government_unimproved_on_1_july_2000",
        ),
    ];
    none_of(&valued, "with method approved_valuation")?;
    none_of(
        &later_registration_facts(args),
        "with method approved_valuation under item 2",
    )?;
    when(
        false,
        args.supply_day,
        "supply_day",
        "with method approved_valuation under item 4",
    )?;
    consideration_base(args)
}

/// An approved valuation as at the day its s 75-10(3) item fixes.
fn valuation_base(args: &MarginSchemeArgs, year: &str) -> Result<(Step, &'static str)> {
    let why = "with method approved_valuation";
    let value = when(true, args.approved_valuation, "approved_valuation", why)?;
    let item = valuation_item(args, why, year)?;
    let value = money(value.expect("required"))?;
    let base = step(
        "margin_base",
        item,
        "approved_valuation",
        &[&value],
        value.clone(),
    );
    Ok((base, item))
}

/// The consideration for the acquisition, less any of it unpaid.
fn consideration_base(args: &MarginSchemeArgs) -> Result<(Step, &'static str)> {
    let why = "with method consideration";
    let key = "acquisition_consideration";
    let paid = when(true, args.acquisition_consideration, key, why)?;
    let key = "unpaid_acquisition_consideration";
    let unpaid = when(true, args.unpaid_acquisition_consideration, key, why)?;
    let (paid, unpaid) = (
        money(paid.expect("required"))?,
        money(unpaid.expect("required"))?,
    );
    if unpaid.gt(&paid)? {
        return conflict(
            "unpaid_acquisition_consideration cannot exceed acquisition_consideration (s 75-12).",
        );
    }
    let formula = "acquisition_consideration - unpaid_acquisition_consideration";
    let base = step(
        "margin_base",
        CONSIDERATION_BASE,
        formula,
        &[&paid, &unpaid],
        paid.sub(&unpaid)?,
    );
    Ok((base, CONSIDERATION_BASE))
}

const PROPORTION: &str = "GST Act s 75-15: a supply relating only to part of the land or premises uses only the corresponding proportion of the consideration or valuation, the share of the whole on a reasonable basis such as area or expected price";

/// The s 75-15 proportion of `base`, for a supply of part of the land.
fn proportion(args: &MarginSchemeArgs, base: &Decimal) -> Result<Option<Step>> {
    let why = "with subdivision_share and subdivision_whole together";
    let (share, whole) = match (args.subdivision_share, args.subdivision_whole) {
        (None, None) => return Ok(None),
        (Some(share), Some(whole)) => (money(share)?, money(whole)?),
        (None, Some(_)) => {
            return refuse(
                "input.missing_field",
                format!("subdivision_share is required {why}."),
            );
        }
        (Some(_), None) => {
            return refuse(
                "input.missing_field",
                format!("subdivision_whole is required {why}."),
            );
        }
    };
    if share.is_zero() || share.gt(&whole)? {
        return conflict(
            "subdivision_share must be more than nil and not more than subdivision_whole (s 75-15).",
        );
    }
    let value = base.mul(&share)?.div(&whole)?;
    let formula = "margin_base x subdivision_share / subdivision_whole";
    Ok(Some(step(
        "apportioned_base",
        PROPORTION,
        formula,
        &[base, &share, &whole],
        value,
    )))
}

const MARGIN: &str = "GST Act s 75-10(2) and (3): the margin is the amount by which the consideration for the supply exceeds the base, nil when it does not";
const MARGIN_GST: &str = "GST Act s 75-10(1) and s 9-90(1): 1/11 of the margin, rounded to the nearest cent (0.5 cents upwards)";
const FULL_GST: &str = "GST Act ss 9-70 and 9-75(1) and s 9-90(1): without the margin scheme, 10% of the value, which is the price x 10 / 11, so 1/11 of the consideration";
const GST_MARGIN: &str = "GST Act s 75-10(1): every condition of s 75-5 is met, so the GST on the supply is the GST on the margin";
const GST_FULL: &str = "GST Act s 75-5: a condition is not met, so the margin scheme does not apply and the GST is worked out without it";

/// The margin and the GST with and without the margin scheme.
fn gst_steps(consideration: &Decimal, base: &Step, eligible: bool) -> Result<[Step; 4]> {
    let name = if base.id == "apportioned_base" {
        "max(consideration - apportioned_base, 0)"
    } else {
        "max(consideration - margin_base, 0)"
    };
    let margin = py_max(consideration.sub(&base.value)?, int(0))?;
    let margin = step(
        "margin",
        MARGIN,
        name,
        &[consideration, &base.value],
        margin,
    );
    let with = step(
        "gst_under_margin_scheme",
        MARGIN_GST,
        "margin / 11",
        &[&margin.value],
        margin.value.div(&int(11))?,
    );
    let without = step(
        "gst_without_margin_scheme",
        FULL_GST,
        "consideration / 11",
        &[consideration],
        consideration.div(&int(11))?,
    );
    let gst = if eligible {
        step(
            "gst",
            GST_MARGIN,
            "gst_under_margin_scheme",
            &[&with.value],
            with.value.clone(),
        )
    } else {
        step(
            "gst",
            GST_FULL,
            "gst_without_margin_scheme",
            &[&without.value],
            without.value.clone(),
        )
    };
    Ok([margin, with, without, gst])
}

/// One condition row.
fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

const APPLIES: [&str; 2] = [
    "The margin scheme applies: the recipient's acquisition is not a creditable acquisition, so it cannot claim an input tax credit for it (GST Act s 75-20).",
    "The recipient can apply the margin scheme to its own later taxable supply of the interest: an acquisition through a taxable supply under the margin scheme is not ineligible (s 75-5(3)(a)).",
];
const NOT_APPLIES: &str = "The margin scheme does not apply: the GST is 1/11 of the consideration, and a recipient whose acquisition is a creditable acquisition (GST Act s 11-5: for a creditable purpose, for consideration and by a recipient registered or required to be registered) can claim an input tax credit for it.";

/// The facts of s 75-5, checked, as condition rows, with how the interest
/// was acquired.
fn conditions<'a>(args: &MarginSchemeArgs<'a>) -> Result<(Vec<Value>, &'a str)> {
    let interest = choice(
        args.interest,
        "interest",
        &[
            "freehold_interest",
            "stratum_unit",
            "long_term_lease",
            "other",
        ],
    )?;
    let agreement = choice(
        args.agreement,
        "agreement",
        &[
            "written_on_or_before_supply",
            "written_in_further_period_allowed",
            "none",
        ],
    )?;
    let acquisition = acquisition(args.acquisition)?;
    if flag(args.recipient_associate, "recipient_associate")? {
        return refuse(
            "gst_margin_scheme.supply_to_associate",
            "A supply to your associate: s 75-13 works its margin from the GST inclusive market value of the interest at the supply, not the consideration; this command does not work it.",
        );
    }
    let rows = vec![
        condition("real_property_interest", INTEREST, interest != "other"),
        condition("written_agreement", AGREEMENT, agreement != "none"),
        condition(
            "acquisition_eligible",
            ACQUISITION,
            acquisition != "taxable_supply_without_margin_scheme",
        ),
    ];
    Ok((rows, acquisition))
}

pub fn gst_margin_scheme(args: &MarginSchemeArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let (rows, acquisition) = conditions(args)?;
    let consideration = money(args.consideration)?;
    let failing: Vec<Value> = rows
        .iter()
        .filter(|row| row["met"] == json!(false))
        .map(|row| row["condition"].clone())
        .collect();
    let eligible = failing.is_empty();
    let (steps, rule) = precise(|| {
        let (base, rule) = base_step(args, acquisition, &year)?;
        let apportioned = proportion(args, &base.value)?;
        let last = apportioned.as_ref().unwrap_or(&base);
        let gst = gst_steps(&consideration, last, eligible)?;
        let mut steps = vec![base];
        steps.extend(apportioned);
        steps.extend(gst);
        Ok((steps, rule))
    })?;
    let mut out = derived_result(KIND, &year, steps, vec![rate("gst_rate", "0.10")])?;
    out["eligible"] = json!(eligible);
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    out["base_rule"] = json!(rule);
    out["consequences"] = if eligible {
        json!(APPLIES)
    } else {
        json!([NOT_APPLIES])
    };
    Ok(out)
}
