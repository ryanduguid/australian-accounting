// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `gst_annual_apportionment`: GST Act Division 131, whether an entity is
//! eligible to make an annual apportionment election (s 131-5), the input tax
//! credit on a partly creditable acquisition while one has effect (s 131-40)
//! and the later increasing adjustment (s 131-55), against the credit
//! s 11-30 gives without it.

use serde_json::{Value, json};

use super::super::{
    Arg, Result, Step, d, derived_result, flag, money, precise, rate, refuse, scope, step,
};
use super::{condition, failing, fraction};
use crate::decimal::Decimal;

const KIND: &str = "gst_annual_apportionment";

/// The annual apportionment turnover threshold, s 131-5(2)(a): no higher
/// amount is specified in the GST Regulations 2019, read on 22 November 2026.
const TURNOVER_THRESHOLD: &str = "2000000";

/// The facts as the command passes them; see the README for each.
pub struct AnnualApportionmentArgs<'a> {
    pub small_business_entity: &'a Arg,
    pub carries_on_business: &'a Arg,
    pub gst_turnover: Option<&'a Arg>,
    pub instalment_election: &'a Arg,
    pub annual_tax_period_election: &'a Arg,
    pub election_made: &'a Arg,
    pub gst_on_acquisition: &'a Arg,
    pub creditable_purpose: &'a Arg,
    pub non_input_taxed_extent: Option<&'a Arg>,
    pub consideration_extent: Option<&'a Arg>,
    pub car: &'a Arg,
    pub reduced_credit_acquisition: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const ENTITY: &str = "GST Act s 131-5(1)(a): a small business entity (other than because of ITAA 1997 s 328-110(4)) for the income year, or an entity that does not carry on a business with a GST turnover not over the $2 million annual apportionment turnover threshold (s 131-5(2))";
const NO_INSTALMENTS: &str =
    "GST Act s 131-5(1)(b): no election to pay GST by instalments (s 162-15) in effect";
const NO_ANNUAL_PERIOD: &str =
    "GST Act s 131-5(1)(c): no annual tax period election (s 151-10) in effect";

/// The refusals for an acquisition outside s 131-40 or s 131-50's simple case.
fn excluded(args: &AnnualApportionmentArgs) -> Result<()> {
    if flag(args.car, "car")? {
        return refuse(
            "gst_annual_apportionment.car",
            "An acquisition of a car: s 131-50 limits the credit to the s 69-10 car limit amount, which this command does not work.",
        );
    }
    if flag(
        args.reduced_credit_acquisition,
        "reduced_credit_acquisition",
    )? {
        return refuse(
            "gst_annual_apportionment.reduced_credit_acquisition",
            "An acquisition that is to any extent a reduced credit acquisition: s 131-40(5) excludes it from annual apportionment.",
        );
    }
    Ok(())
}

/// Whether s 131-5(1)(a) is met, refusing a turnover given or missing out of
/// turn and a small business entity that carries on no business.
fn entity_test(args: &AnnualApportionmentArgs) -> Result<bool> {
    let small = flag(args.small_business_entity, "small_business_entity")?;
    let business = flag(args.carries_on_business, "carries_on_business")?;
    if small && !business {
        return refuse(
            "gst_annual_apportionment.fact_conflict",
            "A small business entity carries on a business (ITAA 1997 s 328-110(1)).",
        );
    }
    let needed = !small && !business;
    match (needed, args.gst_turnover) {
        (true, None) => refuse(
            "input.missing_field",
            "gst_turnover is required for an entity that does not carry on a business.",
        ),
        (false, Some(_)) => refuse(
            "input.field_unknown",
            "gst_turnover is an input only for an entity that does not carry on a business.",
        ),
        (false, None) => Ok(small),
        (true, Some(turnover)) => Ok(money(turnover)?.le(&d(TURNOVER_THRESHOLD))?),
    }
}

/// The 3 conditions of s 131-5(1), each met or not.
fn eligibility(args: &AnnualApportionmentArgs) -> Result<Vec<Value>> {
    let entity = entity_test(args)?;
    let instalments = flag(args.instalment_election, "instalment_election")?;
    let annual = flag(
        args.annual_tax_period_election,
        "annual_tax_period_election",
    )?;
    Ok(vec![
        condition("entity_eligible", ENTITY, entity),
        condition("no_instalment_election", NO_INSTALMENTS, !instalments),
        condition("no_annual_tax_period_election", NO_ANNUAL_PERIOD, !annual),
    ])
}

/// The acquisition's extents: creditable purpose, and non-input-taxed purpose
/// and consideration when given (1 otherwise), refusing one that is not partly
/// creditable.
struct Extents {
    purpose: Decimal,
    non_input_taxed: Option<Decimal>,
    consideration: Option<Decimal>,
}

impl Extents {
    fn non_input_taxed(&self) -> Decimal {
        self.non_input_taxed.clone().unwrap_or_else(|| d("1"))
    }

    fn consideration(&self) -> Decimal {
        self.consideration.clone().unwrap_or_else(|| d("1"))
    }
}

fn extents(args: &AnnualApportionmentArgs) -> Result<Extents> {
    let given = |value: Option<&Arg>, name| value.map(|v| fraction(v, name)).transpose();
    let e = Extents {
        purpose: fraction(args.creditable_purpose, "creditable_purpose")?,
        non_input_taxed: given(args.non_input_taxed_extent, "non_input_taxed_extent")?,
        consideration: given(args.consideration_extent, "consideration_extent")?,
    };
    if e.purpose.gt(&e.non_input_taxed())? {
        return refuse(
            "gst_annual_apportionment.fact_conflict",
            "creditable_purpose cannot exceed non_input_taxed_extent: an acquisition is not for a creditable purpose to the extent it relates to making input taxed supplies (s 11-15(2)(a)).",
        );
    }
    let (one, nil) = (d("1"), d("0"));
    let whole = e.purpose.equals(&one)? && e.consideration().equals(&one)?;
    if e.purpose.equals(&nil)? || e.consideration().equals(&nil)? || whole {
        return refuse(
            "gst_annual_apportionment.not_partly_creditable",
            "Division 131 is for partly creditable acquisitions (s 11-30(1)): some, but not all, for a creditable purpose, or only part of the consideration provided.",
        );
    }
    Ok(e)
}

/// `first` times each factor given: the formula naming them, its operands and
/// the product.
fn product<'a>(
    first: (&str, &'a Decimal),
    rest: [(&str, Option<&'a Decimal>); 2],
) -> Result<(String, Vec<&'a Decimal>, Decimal)> {
    let (mut formula, mut operands, mut value) =
        (first.0.to_string(), vec![first.1], first.1.clone());
    for (name, factor) in rest {
        if let Some(factor) = factor {
            formula = format!("{formula} x {name}");
            operands.push(factor);
            value = value.mul(factor)?;
        }
    }
    Ok((formula, operands, value))
}

const WITHOUT: &str = "GST Act s 11-30(3): the full input tax credit times the extent of creditable purpose times the extent of consideration";
const FULL: &str = "GST Act s 131-40(1): with an annual apportionment election in effect at the end of the tax period the credit is attributable to, the credit equals the GST payable on the supply";
const PART: &str = "GST Act s 131-40(2): the acquisition relates to making input taxed supplies or only part of the consideration is provided, so the full input tax credit times the extent of non-input-taxed purpose times the extent of consideration";
const NOT_APPLYING: &str = "GST Act s 131-40(1)(a): no annual apportionment election has effect, so the credit is the s 11-30 amount";
const ADJUSTMENT: &str = "GST Act s 131-55(1) and (2): an increasing adjustment of the difference between the credit worked out under Division 131 and the credit without it";

/// The credit under Division 131 when it applies, else the s 11-30 credit.
fn apportioned(gst: &Decimal, e: &Extents, applies: bool, without: &Step) -> Result<Step> {
    const ID: &str = "annual_apportionment_credit";
    if !applies {
        let formula = "credit_without_annual_apportionment";
        let value = without.value.clone();
        return Ok(step(ID, NOT_APPLYING, formula, &[&without.value], value));
    }
    let one = d("1");
    if e.non_input_taxed().equals(&one)? && e.consideration().equals(&one)? {
        return Ok(step(ID, FULL, "gst_on_acquisition", &[gst], gst.clone()));
    }
    let (formula, operands, value) = product(
        ("gst_on_acquisition", gst),
        [
            ("non_input_taxed_extent", e.non_input_taxed.as_ref()),
            ("consideration_extent", e.consideration.as_ref()),
        ],
    )?;
    Ok(step(ID, PART, &formula, &operands, value))
}

/// The 3 steps: the credit without Division 131, with it, and the s 131-55
/// adjustment.
fn credits(gst: &Decimal, e: &Extents, applies: bool) -> Result<Vec<Step>> {
    precise(|| {
        let (formula, operands, plain) = product(
            ("gst_on_acquisition", gst),
            [
                ("creditable_purpose", Some(&e.purpose)),
                ("consideration_extent", e.consideration.as_ref()),
            ],
        )?;
        let id = "credit_without_annual_apportionment";
        let without = step(id, WITHOUT, &formula, &operands, plain);
        let with = apportioned(gst, e, applies, &without)?;
        let formula = "annual_apportionment_credit - credit_without_annual_apportionment";
        let difference = with.value.sub(&without.value)?;
        let operands = [&with.value, &without.value];
        let adjustment = step(
            "increasing_adjustment",
            ADJUSTMENT,
            formula,
            &operands,
            difference,
        );
        Ok(vec![without, with, adjustment])
    })
}

const ATTRIBUTION: &str = "The increasing adjustment is attributable to the tax period in which the income tax return for the income year in which the credit's tax period starts is due, or to the tax period with 31 December of the next financial year without a return, or to an earlier tax period you choose (GST Act s 131-60(1)); to the tax period in which a revocation or disallowance during the financial year takes effect (s 131-60(2)).";
const CEASES: &str = "An election ceases on revocation, disallowance, not being a small business entity for an income year (from the start of the tax period with that income year's first day) or failing s 131-5(1)(a)(ii) on 31 July (from that tax period) (GST Act s 131-20).";
const NOT_ELIGIBLE: &str = "The entity is not eligible to make an annual apportionment election (GST Act s 131-5), so the credit is apportioned in the tax period it is attributable to (s 11-30).";
const NO_ELECTION: &str = "No annual apportionment election is made, so the credit is apportioned in the tax period it is attributable to (GST Act s 11-30).";

/// GST Act Division 131: eligibility, and the credit and adjustment for one
/// partly creditable acquisition.
pub fn gst_annual_apportionment(args: &AnnualApportionmentArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    excluded(args)?;
    let rows = eligibility(args)?;
    let elected = flag(args.election_made, "election_made")?;
    let gst = money(args.gst_on_acquisition)?;
    let e = extents(args)?;
    let failing = failing(&rows);
    let eligible = failing.is_empty();
    // s 131-20(1) ends an election on revocation, disallowance or the entity
    // test failing, not on a later instalment or annual tax period election.
    if elected && rows[0]["met"] == json!(false) {
        return refuse(
            "gst_annual_apportionment.election_status_unsettled",
            "election_made is true but the entity test of s 131-5(1)(a) fails: whether the election ceased turns on s 131-20(1)(c) or (d) and the income year or 31 July test, which this command does not take.",
        );
    }
    let steps = credits(&gst, &e, elected)?;
    let rates = vec![rate("turnover_threshold", TURNOVER_THRESHOLD)];
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["eligible"] = json!(eligible);
    out["applies"] = json!(elected);
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    out["consequences"] = match (eligible, elected) {
        (_, true) => json!([ATTRIBUTION, CEASES]),
        (true, false) => json!([NO_ELECTION]),
        (false, false) => json!([NOT_ELIGIBLE, CEASES]),
    };
    Ok(out)
}
