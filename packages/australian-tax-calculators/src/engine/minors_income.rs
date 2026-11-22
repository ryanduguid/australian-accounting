// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `minors_income`: the tax on a minor's taxable income under Division 6AA of
//! Part III of the ITAA 1936 (ss 102AA to 102AGA) and the Income Tax Rates Act
//! 1986 ss 13 and 15 and Schedule 11, for 2024-25 to 2026-27. The citations
//! are to ITAA 1936 compilation 193 (1 October 2026) and the Income Tax Rates
//! Act 1986 compilation 66 (1 July 2026), read on 22 November 2026; Division
//! 6AA, ss 13 and 15 and Schedule 11 read the same in the 1 July 2020
//! compilations (169 and 55). Whether a person is excepted, which income is
//! excepted and every amount the Act leaves to the Commissioner's opinion are
//! the operator's stated facts. The ordinary rates are `resident-tax`'s and
//! `foreign-resident-tax`'s scales.

use serde_json::{Map, Value, json};

use super::lump_sum_arrears::scale_tax;
use super::stated;
use super::{Arg, Result, d, flag, int, lookup, money, precise, py_max, py_min, rate, refuse};
use super::{result, scope, whole_dollars};
use crate::decimal::{Decimal, Rounding};
use crate::metadata as meta;

const KIND: &str = "minors_income";
/// ITRA s 13(1)(b), s 15(2): eligible taxable income up to this is not
/// taxed at the special rate (residents) or is capped (foreign residents).
const NIL_BAND: &str = "416";
/// ITRA s 13(2)(a), s 15(2)(b)(i).
const PHASE_IN: &str = "0.66";
/// ITRA Schedule 11 Part I clause 2 and Part II clause 2.
const SPECIAL_RATE: &str = "0.45";
/// The most excepted income items or trust shares the command takes.
const MAX_ITEMS: usize = 50;

/// The facts the command takes only for some excepted person grounds.
pub const MINORS_OPTIONAL: [&str; 2] = ["full_time_occupation", "dependent_on_relatives"];

/// The facts as the command passes them; see the README for each.
pub struct MinorsIncomeArgs<'a> {
    pub taxable_income: &'a Arg,
    pub residency: &'a Arg,
    pub under_18_on_30_june: &'a Arg,
    pub excepted_person: &'a Arg,
    pub excepted_income: &'a Arg,
    pub trust_shares: &'a Arg,
    pub other_assessable_income: &'a Arg,
    pub deductions_exclusive: &'a Arg,
    pub deductions_related: &'a Arg,
    pub apportionable_deductions: &'a Arg,
    pub special_income_component: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 2],
}

/// ITAA 1936 s 102AC(2): each excepted person ground and its paragraph.
const PERSON_GROUNDS: [(&str, &str); 8] = [
    ("full_time_occupation", "s 102AC(2)(b), (6) to (8)"),
    ("carer_allowance", "s 102AC(2)(c)(i)"),
    ("disability_support_pension", "s 102AC(2)(c)(ii)"),
    ("disability_certificate", "s 102AC(2)(d)"),
    ("special_disability_trust", "s 102AC(2)(da)"),
    ("double_orphan_pension", "s 102AC(2)(e), (3)"),
    ("double_orphan_pension_s1003", "s 102AC(2)(f), (3)"),
    ("permanent_disability_certificate", "s 102AC(2)(g), (4)"),
];

/// The grounds s 102AC(3) and (4) deny to a minor dependent on relatives.
const DEPENDENCY_GROUNDS: [&str; 3] = [
    "double_orphan_pension",
    "double_orphan_pension_s1003",
    "permanent_disability_certificate",
];

/// ITAA 1936 s 102AE(2): each excepted assessable income ground (other than
/// trust shares, paragraph (e)) and its paragraph.
const INCOME_GROUNDS: [(&str, &str); 10] = [
    ("employment", "s 102AE(2)(a), s 102AF(1)"),
    ("business", "s 102AE(2)(a), (5), s 102AF(3)"),
    ("damages_or_compensation", "s 102AE(2)(b)(i) to (iii)"),
    ("death_benefit", "s 102AE(2)(b)(iv) to (vi)"),
    ("public_fund", "s 102AE(2)(b)(vii)"),
    ("family_breakdown", "s 102AE(2)(b)(viii), s 102AGA"),
    ("deceased_estate", "s 102AE(2)(c)(i), (ii), (10)"),
    ("lottery_prize", "s 102AE(2)(c)(iii)"),
    ("partnership", "s 102AE(2)(d), (3)"),
    ("accumulations", "s 102AE(2)(f)"),
];

/// ITAA 1936 s 102AC(6) to (8): the facts of the full-time occupation test.
const FULL_TIME_FACTS: [&str; 4] = [
    "engaged_on_last_day",
    "three_months_in_year",
    "intends_full_time_occupation_next_year",
    "intends_full_time_education_next_year",
];

/// Whether the excepted person ground holds: the full-time occupation test
/// from its 4 stated facts, or for ground (e), (f) or (g) no dependence on
/// relatives (s 102AC(3), (4)).
fn ground_holds(ground: &str, optional: [Option<&Arg>; 2]) -> Result<bool> {
    let [full_time, dependent] = optional;
    let needs_dependency = DEPENDENCY_GROUNDS.contains(&ground);
    if full_time.is_some() && ground != "full_time_occupation" {
        return unknown("full_time_occupation", "the full_time_occupation ground");
    }
    if dependent.is_some() && !needs_dependency {
        return unknown("dependent_on_relatives", "grounds (e), (f) and (g)");
    }
    if ground == "full_time_occupation" {
        return full_time_test(full_time);
    }
    match (needs_dependency, dependent) {
        (true, None) => refuse(
            "input.missing_field",
            "dependent_on_relatives is required for this ground (s 102AC(3), (4)).",
        ),
        (true, Some(dependent)) => Ok(!flag(dependent, "dependent_on_relatives")?),
        (false, _) => Ok(true),
    }
}

fn unknown<T>(key: &str, only: &str) -> Result<T> {
    refuse(
        "input.field_unknown",
        format!("{key} is an input only for {only}"),
    )
}

/// s 102AC(6) to (8): full time on the last day or for 3 months in the year,
/// intending full-time occupation and not full-time education next year.
fn full_time_test(given: Option<&Arg>) -> Result<bool> {
    let Some(Arg::Json(Value::Object(fields))) = given else {
        return refuse(
            "input.missing_field",
            "full_time_occupation must be an object with engaged_on_last_day, three_months_in_year, intends_full_time_occupation_next_year and intends_full_time_education_next_year: the test is not run on facts not stated.",
        );
    };
    let name = "full_time_occupation";
    stated::known_keys(fields, name, name, &FULL_TIME_FACTS)?;
    let mut facts = [false; 4];
    for (fact, key) in facts.iter_mut().zip(FULL_TIME_FACTS) {
        *fact = match stated::required(fields, name, key)? {
            Value::Bool(b) => *b,
            _ => {
                return refuse(
                    "input.flag_invalid",
                    format!("{name}: {key} must be a boolean."),
                );
            }
        };
    }
    let [on_last_day, three_months, occupation_next, education_next] = facts;
    Ok((on_last_day || three_months) && occupation_next && !education_next)
}

/// The ground given and whether the minor is an excepted person by it; for
/// a person 18 or over on 30 June, who is not prescribed whatever the ground
/// (s 102AC(1)(a)), the ground is not tested.
fn excepted_person(
    args: &MinorsIncomeArgs,
    under_18: bool,
) -> Result<Option<(&'static str, bool)>> {
    let ground = args.excepted_person.text();
    if ground == Some("none") {
        if let Some(key) = MINORS_OPTIONAL
            .iter()
            .zip(args.optional)
            .find_map(|(key, given)| given.map(|_| *key))
        {
            return unknown(key, "an excepted person ground");
        }
        return Ok(None);
    }
    let Some(&(name, citation)) = PERSON_GROUNDS.iter().find(|(n, _)| Some(*n) == ground) else {
        return refuse(
            "minors_income.excepted_person_invalid",
            "excepted_person must be none or a ground of ITAA 1936 s 102AC(2) the README lists.",
        );
    };
    if !under_18 {
        return Ok(None);
    }
    Ok(Some((citation, ground_holds(name, args.optional)?)))
}

/// A JSON list of at most [`MAX_ITEMS`] objects, or the refusal `code`.
fn items<'a>(
    given: &'a Arg,
    code: &'static str,
    what: &str,
) -> Result<Vec<&'a Map<String, Value>>> {
    if let Arg::Json(Value::Array(list)) = given
        && list.len() <= MAX_ITEMS
        && let Some(objects) = list
            .iter()
            .map(Value::as_object)
            .collect::<Option<Vec<_>>>()
    {
        return Ok(objects);
    }
    refuse(
        code,
        format!("{what} must be a list of at most {MAX_ITEMS} objects."),
    )
}

/// The excepted income items (s 102AE(2) other than paragraph (e)), summed.
fn excepted_items(given: &Arg) -> Result<Decimal> {
    let code = "minors_income.excepted_income_invalid";
    let mut total = int(0);
    for (i, fields) in items(given, code, "excepted_income")?
        .into_iter()
        .enumerate()
    {
        let name = format!("excepted income item {}", i + 1);
        stated::known_keys(
            fields,
            &name,
            "an excepted income item",
            &["ground", "amount"],
        )?;
        let ground = stated::required(fields, &name, "ground")?.as_str();
        if !INCOME_GROUNDS.iter().any(|(g, _)| Some(*g) == ground) {
            return refuse(
                "minors_income.excepted_income_invalid",
                format!("{name}: ground must be one the README lists from ITAA 1936 s 102AE(2)."),
            );
        }
        total = total.add(&stated::amount(fields, &name, "amount")?)?;
    }
    Ok(total)
}

/// Trust shares (s 97 or 100): the excepted parts (s 102AE(2)(e), (4)) and
/// the parts to which Division 6AA applies (s 102AG(1)), each summed.
fn trust_parts(given: &Arg) -> Result<(Decimal, Decimal)> {
    let code = "minors_income.trust_shares_invalid";
    let (mut excepted, mut eligible) = (int(0), int(0));
    for (i, fields) in items(given, code, "trust_shares")?.into_iter().enumerate() {
        let name = format!("trust share {}", i + 1);
        stated::known_keys(
            fields,
            &name,
            "a trust share",
            &["amount", "division_6aa_part"],
        )?;
        let amount = stated::amount(fields, &name, "amount")?;
        let part = stated::amount(fields, &name, "division_6aa_part")?;
        if part.gt(&amount)? {
            return refuse(
                "minors_income.trust_shares_invalid",
                format!("{name}: division_6aa_part exceeds amount."),
            );
        }
        excepted = excepted.add(&amount.sub(&part)?)?;
        eligible = eligible.add(&part)?;
    }
    Ok((excepted, eligible))
}

/// The ordinary scale for the year: Schedule 7 Part I or Part II.
fn scale(resident: bool, year: &str) -> &'static [(i64, Option<i64>, &'static str)] {
    let table = if resident {
        meta::RESIDENT_TAX_SCALES
    } else {
        meta::FOREIGN_RESIDENT_TAX_SCALES
    };
    lookup(table, year)
}

/// The phase-out limit, rounded down to the dollar: ITRA s 13(10) for a
/// resident, s 15(8) for a foreign resident.
fn phase_out(resident: bool, year: &str) -> Result<Decimal> {
    let ordinary = scale(resident, year);
    let highest = d(ordinary.last().expect("a band").2);
    let numerator = if resident {
        d(PHASE_IN)
    } else {
        d(PHASE_IN).sub(&d(ordinary[0].2))?
    };
    precise(|| {
        Ok(d(NIL_BAND)
            .mul(&numerator)?
            .div(&d(PHASE_IN).sub(&highest)?)?
            .quantize(0, Rounding::Down)?)
    })
}

/// ITRA s 3(1): item 2 of the table in Schedule 7 Part I clause 1.
fn second_resident_rate(year: &str) -> Decimal {
    d(scale(true, year)[1].2)
}

/// The tax on the other income, the tax on eligible taxable income and the
/// rule that set the second (ITRA s 13, s 15 and Schedule 11).
struct Split {
    other: Decimal,
    eligible: Decimal,
    rule: &'static str,
}

fn split(resident: bool, year: &str, taxable: &Decimal, eligible: &Decimal) -> Result<Split> {
    let ordinary = scale(resident, year);
    let rest = taxable.sub(eligible)?;
    let limit = phase_out(resident, year)?;
    precise(|| {
        let other = scale_tax(ordinary, &rest, "x")?.0;
        let top = scale_tax(ordinary, taxable, "x")?.0.sub(&other)?;
        let nil_band = d(NIL_BAND);
        let excess = eligible.sub(&nil_band)?;
        let at_special = d(SPECIAL_RATE).mul(eligible)?;
        let capped = |floor: Decimal| py_min(at_special.clone(), py_max(floor, top.clone())?);
        let (tax, rule) = if eligible.is_zero() || (resident && eligible.le(&nil_band)?) {
            (top.clone(), ORDINARY)
        } else if resident && eligible.le(&limit)? {
            (capped(d(PHASE_IN).mul(&excess)?)?, RESIDENT_CAP)
        } else if !resident && eligible.le(&nil_band)? {
            (
                capped(second_resident_rate(year).mul(eligible)?)?,
                FOREIGN_NIL_CAP,
            )
        } else if !resident && eligible.le(&limit)? {
            let floor = second_resident_rate(year)
                .mul(&nil_band)?
                .add(&d(PHASE_IN).mul(&excess)?)?;
            (capped(floor)?, FOREIGN_CAP)
        } else {
            (at_special.clone(), SPECIAL)
        };
        Ok(Split {
            other,
            eligible: tax,
            rule,
        })
    })
}

const ORDINARY: &str = "ITRA Schedule 7 (s 12): no special rate applies (eligible taxable income is nil, or for a resident $416 or less, s 13(1)(b)); the part attributed to it is the top slice of the ordinary tax";
const RESIDENT_CAP: &str = "ITRA s 13(2): 45% of eligible taxable income (Schedule 11 Part I clause 2), but no more than the greater of 66% of the excess over $416 and the top-slice tax at Schedule 7 Part I rates, up to the resident phase-out limit";
const FOREIGN_NIL_CAP: &str = "ITRA s 15(2)(a): 45% of eligible taxable income (Schedule 11 Part II clause 2), but no more than the greater of the second resident personal tax rate x eligible taxable income and the top-slice tax at Schedule 7 Part II rates";
const FOREIGN_CAP: &str = "ITRA s 15(2)(b): 45% of eligible taxable income (Schedule 11 Part II clause 2), but no more than the greater of the second resident personal tax rate x $416 plus 66% of the excess over $416 and the top-slice tax at Schedule 7 Part II rates, up to the non-resident phase-out limit";
const SPECIAL: &str =
    "ITRA Schedule 11 clause 2: 45% of eligible taxable income above the phase-out limit";

/// Eligible assessable income less the s 102AD deductions, nil if negative.
/// Returns the apportioned deductions (s 102AD(c)) and the result.
fn eligible_taxable_income(
    eligible_assessable: &Decimal,
    [exclusive, related, apportionable]: &[Decimal; 3],
    taxable: &Decimal,
) -> Result<(Decimal, Decimal)> {
    precise(|| {
        let before = eligible_assessable.sub(exclusive)?.sub(related)?;
        if !before.gt(&int(0))? {
            return Ok((int(0), int(0)));
        }
        let base = taxable.add(apportionable)?;
        let apportioned = if base.is_zero() {
            int(0)
        } else {
            apportionable.mul(&before)?.div(&base)?
        };
        Ok((
            apportioned.clone(),
            py_max(int(0), before.sub(&apportioned)?)?,
        ))
    })
}

fn residency(given: &Arg) -> Result<bool> {
    match given.text() {
        Some("resident") => Ok(true),
        Some("foreign_resident") => Ok(false),
        _ => refuse(
            "minors_income.residency_invalid",
            "residency must be resident or foreign_resident.",
        ),
    }
}

/// The checks before the figures: scope, the special income component and
/// residency; returns the year and whether the minor is a resident.
fn preliminaries(args: &MinorsIncomeArgs) -> Result<(String, bool)> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    if flag(args.special_income_component, "special_income_component")? {
        return refuse(
            "minors_income.special_income_component",
            "Taxable income that includes a special income component (an abnormal income amount, Income Tax Rates Act 1986 s 3(1)) is taxed under Schedule 11 clause 3's formula, which this worksheet does not take.",
        );
    }
    Ok((year, residency(args.residency)?))
}

fn inconsistent<T>(message: &str) -> Result<T> {
    refuse("minors_income.facts_inconsistent", message.to_string())
}

/// Excepted and eligible assessable income: s 102AG(1) applies only to a
/// prescribed beneficiary, so otherwise the whole trust share is excepted
/// (s 102AE(2)(e)).
fn assessable(args: &MinorsIncomeArgs, prescribed: bool) -> Result<(Decimal, Decimal)> {
    let excepted = excepted_items(args.excepted_income)?;
    let (trust_excepted, trust_eligible) = trust_parts(args.trust_shares)?;
    let other = money(args.other_assessable_income)?;
    Ok(if prescribed {
        (excepted.add(&trust_excepted)?, other.add(&trust_eligible)?)
    } else {
        (excepted.add(&trust_excepted)?.add(&trust_eligible)?, other)
    })
}

pub fn minors_income(args: &MinorsIncomeArgs) -> Result<Value> {
    let (year, resident) = preliminaries(args)?;
    let taxable = money(args.taxable_income)?;
    whole_dollars(&taxable, "taxable income")?;
    let under_18 = flag(args.under_18_on_30_june, "under_18_on_30_june")?;
    let person = excepted_person(args, under_18)?;
    let prescribed = under_18 && !person.is_some_and(|(_, holds)| holds);
    let (excepted_total, eligible_assessable) = assessable(args, prescribed)?;
    let deductions = [
        money(args.deductions_exclusive)?,
        money(args.deductions_related)?,
        money(args.apportionable_deductions)?,
    ];
    if taxable.gt(&eligible_assessable.add(&excepted_total)?)? {
        return inconsistent("taxable_income exceeds the assessable income given.");
    }
    let (apportioned, eligible) = if prescribed {
        eligible_taxable_income(&eligible_assessable, &deductions, &taxable)?
    } else {
        (int(0), int(0))
    };
    if eligible.gt(&taxable)? {
        return inconsistent(
            "Eligible taxable income exceeds taxable income: a loss or deduction against excepted income, which this worksheet does not take.",
        );
    }
    let tax = split(resident, &year, &taxable, &eligible)?;
    let figures = Figures {
        excepted_total,
        eligible_assessable,
        apportioned,
        eligible,
    };
    let mut out = render(&year, resident, &taxable, figures, &tax)?;
    explain(&mut out, under_18, person, prescribed, tax.rule);
    Ok(out)
}

struct Figures {
    excepted_total: Decimal,
    eligible_assessable: Decimal,
    apportioned: Decimal,
    eligible: Decimal,
}

fn render(year: &str, resident: bool, taxable: &Decimal, f: Figures, tax: &Split) -> Result<Value> {
    let other_income = taxable.sub(&f.eligible)?;
    let total = tax.other.add(&tax.eligible)?;
    result(
        KIND,
        year,
        vec![
            ("excepted_assessable_income", f.excepted_total),
            ("eligible_assessable_income", f.eligible_assessable),
            ("apportioned_deductions", f.apportioned),
            ("eligible_taxable_income", f.eligible),
            ("other_taxable_income", other_income),
            ("tax_on_other_income", tax.other.clone()),
            ("tax_on_eligible_income", tax.eligible.clone()),
            ("basic_income_tax", total),
        ],
        vec![
            rate("nil_band", NIL_BAND),
            rate("phase_in_rate", PHASE_IN),
            rate("special_rate", SPECIAL_RATE),
            rate("phase_out_limit", phase_out(resident, year)?),
            rate(
                "second_resident_personal_tax_rate",
                second_resident_rate(year),
            ),
        ],
    )
}

const STEPS: [(&str, &str); 6] = [
    (
        "eligible_assessable_income",
        "ITAA 1936 s 102AE(1): assessable income that is not excepted assessable income (s 102AE(2)); of a trust share, the part to which Division 6AA applies (s 102AE(4), s 102AG)",
    ),
    (
        "apportioned_deductions",
        "ITAA 1936 s 102AD(c): apportionable deductions x (eligible assessable income less the s 102AD(a) and (b) deductions) / (taxable income + apportionable deductions)",
    ),
    (
        "eligible_taxable_income",
        "ITAA 1936 s 102AD: eligible assessable income less the deductions in paragraphs (a), (b) and (c), nil if negative; nil for a person who is not a prescribed person (s 102AC)",
    ),
    (
        "tax_on_other_income",
        "ITRA Schedule 11 clause 1: the taxable income other than eligible taxable income at Schedule 7 rates (Part I for a resident, Part II for a foreign resident) as if it were the whole taxable income",
    ),
    ("tax_on_eligible_income", "the rule given in tax_rule"),
    (
        "basic_income_tax",
        "tax_on_other_income + tax_on_eligible_income, before tax offsets and levies (ITAA 1997 s 4-10(3) step 2)",
    ),
];

fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

/// The conditions for a prescribed person, the rule and the authorities.
fn explain(
    out: &mut Value,
    under_18: bool,
    person: Option<(&'static str, bool)>,
    prescribed: bool,
    rule: &str,
) {
    let mut conditions = vec![condition(
        "under_18_on_30_june",
        "ITAA 1936 s 102AC(1)(a): under 18 on the last day of the income year",
        under_18,
    )];
    if let Some((citation, holds)) = person {
        conditions.push(condition(
            "excepted_person",
            &format!("ITAA 1936 {citation}: an excepted person on the ground given"),
            holds,
        ));
    }
    out["conditions"] = json!(conditions);
    out["prescribed_person"] = json!(prescribed);
    out["tax_rule"] = json!(rule);
    out["authorities"] = Value::Object(
        STEPS
            .iter()
            .map(|(step, text)| ((*step).to_owned(), json!(text)))
            .collect::<Map<_, _>>(),
    );
}
