// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `psi_test`: whether the personal services income rules (ITAA 1997
//! Part 2-42) apply to one individual's personal services income for an
//! income year from 2020-21 to 2026-27, step by step as the site's PSI page
//! (ryanduguid.github.io `assets/psi-determination.mjs`, 17 November 2026)
//! works them: s 84-5, s 85-35, a personal services business determination
//! (s 87-15(1)), the results test (s 87-18), the 80% rule (s 87-15(3), with
//! s 87-40 for agents) and the other 3 tests (ss 87-20, 87-25 and 87-30).
//! The citations are to ITAA 1997 compilation 268 (1 October 2026),
//! volume 3, read on 22 November 2026; Part 2-42 reads the same in
//! compilations 210 (1 July 2020) and 223 (1 July 2021). Whether income is
//! mainly a reward for personal efforts or skills, is for a result or came
//! from offers to the public, whether a payer is an associate, and each
//! market value are the operator's answers: the worksheet counts them.

use serde_json::{Map, Value, json};

use super::stated;
use super::{Arg, Result, flag, int, money, precise, refuse, result, scope};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "psi_test";

/// The largest number of payers the worksheet takes.
const MAX_PAYERS: usize = 50;
const PSB: &str = "a personal services business, so Divisions 85 and 86 do not apply";
const PAYER_KEYS: [&str; 6] = ["amount", "associate", "offer", "result", "tools", "defects"];
const PREMISES_KEYS: [&str; 5] = [
    "at_all_times",
    "mainly_for_psi",
    "exclusive_use",
    "separate_from_private",
    "separate_from_clients",
];

/// The facts the worksheet takes only for an agent, in [`PsiTestArgs::optional`].
pub const PSI_OPTIONAL: [&str; 2] = ["commission", "principal_payments"];

/// The facts as the command passes them; see the README for each.
pub struct PsiTestArgs<'a> {
    pub earner: &'a Arg,
    pub mainly_personal_efforts: &'a Arg,
    pub determination_in_force: &'a Arg,
    pub payers: &'a Arg,
    pub agent: &'a Arg,
    pub principal_work_by_others: &'a Arg,
    pub apprentice: &'a Arg,
    pub premises: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 2],
}

struct Payer {
    amount: Decimal,
    associate: bool,
    offer: bool,
    result: bool,
    tools: bool,
    defects: bool,
}

fn boolean(fields: &Map<String, Value>, item: &str, key: &str) -> Result<bool> {
    match stated::required(fields, item, key)? {
        Value::Bool(b) => Ok(*b),
        _ => refuse(
            "input.flag_invalid",
            format!("{item}: {key} must be a boolean."),
        ),
    }
}

fn bad_payers<T>() -> Result<T> {
    refuse(
        "psi_test.payers_invalid",
        format!(
            "payers must be a list of 1 to {MAX_PAYERS} objects, each with amount, associate, offer, result, tools and defects."
        ),
    )
}

fn payers(given: &Arg) -> Result<Vec<Payer>> {
    let Arg::Json(Value::Array(items)) = given else {
        return bad_payers();
    };
    if items.is_empty() || items.len() > MAX_PAYERS {
        return bad_payers();
    }
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let name = format!("payer {}", i + 1);
            let Value::Object(fields) = item else {
                return bad_payers();
            };
            stated::known_keys(fields, &name, "a payer", &PAYER_KEYS)?;
            Ok(Payer {
                amount: stated::amount(fields, &name, "amount")?,
                associate: boolean(fields, &name, "associate")?,
                offer: boolean(fields, &name, "offer")?,
                result: boolean(fields, &name, "result")?,
                tools: boolean(fields, &name, "tools")?,
                defects: boolean(fields, &name, "defects")?,
            })
        })
        .collect()
}

/// Whether all 4 conditions of s 87-30(1) held at all times in the year.
fn premises(given: &Arg) -> Result<bool> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "psi_test.premises_invalid",
            "premises must be an object with at_all_times, mainly_for_psi, exclusive_use, separate_from_private and separate_from_clients.",
        );
    };
    stated::known_keys(fields, "premises", "premises", &PREMISES_KEYS)?;
    let mut all = true;
    for key in PREMISES_KEYS {
        all &= boolean(fields, "premises", key)?;
    }
    Ok(all)
}

/// A share in per cent to 2 places, rounded down, so a share under a
/// threshold never shows as reaching it.
fn share(part: &Decimal, whole: &Decimal) -> Result<String> {
    precise(|| {
        let percent = part
            .mul(&int(100))?
            .div(whole)?
            .quantize(-2, Rounding::Down)?;
        Ok(percent.to_string())
    })
}

fn said(answer: bool) -> &'static str {
    if answer {
        "Yes, as you answered"
    } else {
        "No, as you answered"
    }
}

fn step(label: &str, finding: impl Into<String>) -> Value {
    json!({"step": label, "finding": finding.into()})
}

/// The optional facts: required for an agent and refused otherwise.
fn optional_facts<'a>(args: &PsiTestArgs<'a>, agent: bool) -> Result<[Option<&'a Arg>; 2]> {
    for (key, given) in PSI_OPTIONAL.iter().zip(args.optional) {
        match (agent, given) {
            (true, None) => {
                return refuse(
                    "input.missing_field",
                    format!("{key} is required for an agent."),
                );
            }
            (false, Some(_)) => {
                return refuse(
                    "input.field_unknown",
                    format!("{key} is an input only for an agent"),
                );
            }
            _ => {}
        }
    }
    Ok(args.optional)
}

/// The outcome: the provision that settles it, its effect and whether the
/// income is from a personal services business (`None` when Part 2-42 or
/// Division 85 does not reach that question).
struct Outcome {
    rule: String,
    effect: String,
    psb: Option<bool>,
}

fn outcome(rule: impl Into<String>, effect: impl Into<String>, psb: Option<bool>) -> Outcome {
    Outcome {
        rule: rule.into(),
        effect: effect.into(),
        psb,
    }
}

const DETERMINATION: &str = "ITAA 1997 ss 87-60 and 87-65: whether the Commissioner would make a personal services business determination is not decided; determination_in_force is your answer";

/// `None` for an employee, else whether a company, partnership or trust
/// (a personal services entity) receives the income.
fn earner(given: &Arg) -> Result<Option<bool>> {
    match given.text() {
        Some("employee") => Ok(None),
        Some("sole_trader") => Ok(Some(false)),
        Some("entity") => Ok(Some(true)),
        _ => refuse(
            "psi_test.earner_invalid",
            "earner must be employee, sole_trader or entity.",
        ),
    }
}

pub fn psi_test(args: &PsiTestArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let entity = earner(args.earner)?;
    let mainly = flag(args.mainly_personal_efforts, "mainly_personal_efforts")?;
    let determination = flag(args.determination_in_force, "determination_in_force")?;
    let payers = payers(args.payers)?;
    let agent = flag(args.agent, "agent")?;
    let [commission, principal] = optional_facts(args, agent)?;
    let others = money(args.principal_work_by_others)?;
    let apprentice = flag(args.apprentice, "apprentice")?;
    let premises_met = premises(args.premises)?;
    let total = payers
        .iter()
        .try_fold(int(0), |sum, p| sum.add(&p.amount))?;

    let mut work = Work {
        steps: vec![step(
            "Subsection 84-5(1): income mainly a reward for the individual's personal efforts or skills",
            said(mainly),
        )],
        figures: Map::new(),
    };
    let decided = work.settle(&Facts {
        entity,
        mainly,
        determination,
        payers: &payers,
        total: &total,
        agent: commission.zip(principal),
        others: &others,
        apprentice,
        premises: premises_met,
    })?;

    let mut amounts = Vec::new();
    if work.figures.contains_key("results_test_percent") {
        amounts.push(("personal_services_income", total));
        amounts.push(("principal_work_by_others", others));
    }
    let mut out = result(KIND, &year, amounts, Vec::new())?;
    out["rule"] = json!(decided.rule);
    out["effect"] = json!(decided.effect);
    out["personal_services_business"] = json!(decided.psb);
    out["tests"] = Value::Object(work.figures);
    out["steps"] = json!(work.steps);
    out["determination"] = json!({"decided": false, "authority": DETERMINATION});
    Ok(out)
}

struct Facts<'a> {
    entity: Option<bool>,
    mainly: bool,
    determination: bool,
    payers: &'a [Payer],
    total: &'a Decimal,
    agent: Option<(&'a Arg, &'a Arg)>,
    others: &'a Decimal,
    apprentice: bool,
    premises: bool,
}

/// Whether `part` is at least `num / den` of `whole`, compared exactly.
fn reaches(part: &Decimal, whole: &Decimal, num: i64, den: i64) -> Result<bool> {
    Ok(!part.mul(&int(den))?.lt(&whole.mul(&int(num))?)?)
}

fn pick<'a>(yes: bool, met: &'a str, not: &'a str) -> &'a str {
    if yes { met } else { not }
}

/// The steps shown and the figures the tests worked out.
struct Work {
    steps: Vec<Value>,
    figures: Map<String, Value>,
}

impl Work {
    fn push(&mut self, label: &str, finding: impl Into<String>) {
        self.steps.push(step(label, finding));
    }

    fn settle(&mut self, f: &Facts) -> Result<Outcome> {
        if !f.mainly {
            return Ok(outcome(
                "subsection 84-5(1)",
                "not personal services income, so Part 2-42 does not apply",
                None,
            ));
        }
        let Some(entity) = f.entity else {
            self.push(
                "Subsection 85-35(1): received as an employee, office holder or religious practitioner",
                said(true),
            );
            return Ok(outcome(
                "subsection 85-35(1)",
                "income as an employee, so the PSI rules do not apply",
                None,
            ));
        };
        let paragraph = if entity { "b" } else { "a" };
        self.push(
            &format!(
                "Paragraph 87-15(1)({paragraph}): a personal services business determination in force"
            ),
            said(f.determination),
        );
        if f.determination {
            let rule = format!("paragraph 87-15(1)({paragraph})");
            return Ok(outcome(rule, PSB, Some(true)));
        }
        if f.total.is_zero() {
            return refuse("psi_test.no_psi", "Enter the PSI from at least one payer.");
        }
        if f.others.gt(f.total)? {
            return refuse(
                "psi_test.others_exceed_psi",
                "The market value of principal work by others cannot exceed the PSI entered.",
            );
        }
        if let Some(met) = self.results_test(f, entity)? {
            return Ok(met);
        }
        if let Some((commission, principal)) = f.agent {
            self.agent(&money(commission)?, &money(principal)?)?;
        }
        let one_source = self.one_source(f)?;
        let passes = self.other_tests(f)?;
        Ok(conclude(entity, one_source, &passes))
    }

    /// s 87-18: at least 75% of the PSI meets all 3 conditions.
    fn results_test(&mut self, f: &Facts, entity: bool) -> Result<Option<Outcome>> {
        let met = f
            .payers
            .iter()
            .filter(|p| p.result && p.tools && p.defects)
            .try_fold(int(0), |sum, p| sum.add(&p.amount))?;
        let results = share(&met, f.total)?;
        let passed = reaches(&met, f.total, 3, 4)?;
        let subsection = if entity { 3 } else { 1 };
        self.push(
            &format!(
                "Subsection 87-18({subsection}): PSI paid for a result, with the tools supplied and liability for defects"
            ),
            format!(
                "{results}%, {}",
                pick(passed, "at least 75%: met", "under 75%: not met")
            ),
        );
        self.figures
            .insert("results_test_percent".into(), json!(results));
        let rule = format!("subsection 87-18({subsection})");
        Ok(passed.then(|| outcome(rule, PSB, Some(true))))
    }

    /// s 87-40: refused at 75% or more commission; under it, the principal
    /// stays the one source.
    fn agent(&mut self, commission: &Decimal, principal: &Decimal) -> Result<()> {
        if principal.is_zero() || commission.gt(principal)? {
            return refuse(
                "psi_test.agent_payments_invalid",
                "Enter the principal's payments and the part of them that is commission or performance fees.",
            );
        }
        if reaches(commission, principal, 3, 4)? {
            return refuse(
                "psi_test.agent_rules",
                "At least 75% of the principal's payments are commission or performance fees, so section 87-40 may treat the principal's customers as the sources of the PSI; this worksheet does not apply section 87-40.",
            );
        }
        self.push(
            "Paragraph 87-40(2)(c): commission or performance fees in the principal's payments",
            format!(
                "{}%, under 75%: the agent rules do not apply",
                share(commission, principal)?
            ),
        );
        Ok(())
    }

    /// s 87-15(3): each payer row is one entity with its associates.
    fn one_source(&mut self, f: &Facts) -> Result<bool> {
        let largest = f
            .payers
            .iter()
            .map(|p| &p.amount)
            .try_fold(int(0), |max, a| -> Result<Decimal> {
                Ok(if a.gt(&max)? { a.clone() } else { max })
            })?;
        let source = share(&largest, f.total)?;
        let one_source = reaches(&largest, f.total, 4, 5)?;
        self.push(
            "Subsection 87-15(3): largest share of the PSI from one payer and its associates",
            format!(
                "{source}%, {}",
                pick(one_source, "80% or more", "under 80%")
            ),
        );
        self.figures
            .insert("one_source_percent".into(), json!(source));
        Ok(one_source)
    }

    /// The other 3 tests, which count for self-assessment only under 80%
    /// from one source but are still worked out: the Commissioner weighs
    /// them for a determination (s 87-15(1), note 2). Returns the sections
    /// met.
    fn other_tests(&mut self, f: &Facts) -> Result<Vec<&'static str>> {
        let unrelated = f
            .payers
            .iter()
            .filter(|p| !p.amount.is_zero() && !p.associate && p.offer)
            .count();
        let employment = share(f.others, f.total)?;
        let fifth = reaches(f.others, f.total, 1, 5)?;
        let employs = fifth || f.apprentice;
        let clients = unrelated >= 2;
        self.push(
            "Section 87-20: unrelated payers that engaged the individual or entity as a direct result of offers to the public",
            format!(
                "{unrelated}, {}",
                pick(clients, "at least 2: met", "under 2: not met")
            ),
        );
        self.push(
            "Section 87-25: principal work by others engaged, by market value, or an apprentice",
            format!(
                "{employment}%, {}, {}: {}",
                pick(fifth, "at least 20%", "under 20%"),
                pick(
                    f.apprentice,
                    "an apprentice for at least half the year",
                    "no apprentice for half the year"
                ),
                pick(employs, "met", "not met")
            ),
        );
        self.push(
            "Section 87-30: business premises meeting all 4 conditions at all times in the year",
            pick(f.premises, "Yes: met", "No: not met"),
        );
        self.figures
            .insert("unrelated_clients".into(), json!(unrelated));
        self.figures
            .insert("employment_percent".into(), json!(employment));
        Ok([
            (clients, "87-20"),
            (employs, "87-25"),
            (f.premises, "87-30"),
        ]
        .into_iter()
        .filter_map(|(met, section)| met.then_some(section))
        .collect())
    }
}

/// s 87-15(3) at 80% from one source; otherwise s 87-15(1) by the other 3
/// tests met.
fn conclude(entity: bool, one_source: bool, passes: &[&str]) -> Outcome {
    let division = pick(entity, "Division 86 applies", "Division 85 applies");
    if one_source {
        return outcome(
            "subsection 87-15(3)",
            format!(
                "not a personal services business unless a determination is in force, so {division}"
            ),
            Some(false),
        );
    }
    match passes {
        [] => outcome(
            "subsection 87-15(1)",
            format!("not a personal services business, so {division}"),
            Some(false),
        ),
        [one] => outcome(format!("section {one}"), PSB, Some(true)),
        [first @ .., last] => outcome(
            format!("sections {} and {last}", first.join(", ")),
            PSB,
            Some(true),
        ),
    }
}
