// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `trust-loss-tests`: which ITAA 1936 Schedule 2F tests apply to a trust's
//! deduction of a prior-year tax loss, and whether each is passed on the facts
//! the caller states. Added after the Python port.
use crate::api::{array, integer, is_absent, text};
use crate::decimal::Decimal;
use crate::engine::d;
use crate::figures::{Result, fact, percentage, refuse, year};
use serde_json::{Map, Value, json};

/// One test: whether it applies, whether it is passed, and the law.
struct Test {
    name: &'static str,
    provision: &'static str,
    applies: Option<bool>,
    passed: Option<bool>,
    basis: String,
}

impl Test {
    fn json(&self) -> Value {
        json!({"test": self.name, "provision": self.provision, "applies": self.applies,
            "passed": self.passed, "basis": self.basis})
    }
}

/// A test that does not apply, with the reason.
fn outside(name: &'static str, provision: &'static str, basis: String) -> Test {
    Test {
        name,
        provision,
        applies: Some(false),
        passed: None,
        basis,
    }
}

const STAKE: &str = "ss 266-40, 267-40, 269-55 Sch 2F ITAA 1936";
const PATTERN: &str = "ss 267-30, 269-60 Sch 2F ITAA 1936";
const CONTROL: &str = "ss 267-45, 269-95 Sch 2F ITAA 1936";

fn excepted_tests(para: &str) -> Vec<Test> {
    let why = || {
        format!(
            "an excepted trust ({para}) at all times in the test period is outside ss 266-25 and 267-20"
        )
    };
    vec![
        outside("fifty_percent_stake", STAKE, why()),
        outside("pattern_of_distributions", PATTERN, why()),
        outside("control", CONTROL, why()),
    ]
}

fn not(x: Option<bool>) -> Option<bool> {
    x.map(|b| !b)
}

/// Three-valued 'or': true if either is true, false if both are false.
fn either(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

/// s 272-100 paragraph of each excepted trust kind the command accepts.
fn excepted(v: &Value) -> Result<Option<(&str, &'static str)>> {
    if is_absent(v, "excepted_trust") {
        return Ok(None);
    }
    let kind = text(v, "excepted_trust")?;
    let para = match kind {
        "family_trust" => "s 272-100(a)",
        "complying_superannuation_fund" => "s 272-100(b)",
        "deceased_estate" => "s 272-100(c)",
        "exempt_entity_unit_trust" => "s 272-100(d)",
        "designated_infrastructure_project_entity" => "s 272-100(f)",
        _ => return Err("excepted_trust must be family_trust, complying_superannuation_fund, deceased_estate, exempt_entity_unit_trust, designated_infrastructure_project_entity or null".into()),
    };
    Ok(Some((kind, para)))
}

fn fixed_tests(v: &Value) -> Result<Vec<Test>> {
    let stake = fact(v, "fifty_percent_stake_test_passed")?;
    let alternative = if stake == Some(true) {
        None
    } else {
        fact(v, "alternative_stake_test_passed")?
    };
    Ok(vec![Test {
        name: "fifty_percent_stake",
        provision: "ss 266-25(2), 266-40, 266-45, 269-55 Sch 2F ITAA 1936",
        applies: Some(true),
        passed: either(stake, alternative),
        basis: "a fixed trust must pass the 50% stake test for the test period or meet the s 266-45 conditions (stated)".into(),
    },
    outside("pattern_of_distributions", PATTERN, "applies only to a non-fixed trust (s 267-20)".into()),
    outside("control", CONTROL, "applies only to a non-fixed trust (s 267-20)".into())])
}

/// Smallest percentage each individual received across the distributions, and their total (s 269-70).
fn minimums(list: &[Value], kind: &str) -> Result<(Map<String, Value>, Decimal)> {
    let dists = list
        .iter()
        .map(|x| distribution(x, kind))
        .collect::<Result<Vec<_>>>()?;
    let mut out = Map::new();
    let mut total = d("0");
    for name in dists.iter().flat_map(|m| m.keys()) {
        if out.contains_key(name) {
            continue;
        }
        let mut least = d("100");
        for m in &dists {
            // An individual absent from a distribution received 0% of it.
            let p = m.get(name).cloned().unwrap_or_else(|| d("0"));
            if p.lt(&least)? {
                least = p;
            }
        }
        total = total.add(&least)?;
        out.insert(name.clone(), json!(least.to_string()));
    }
    Ok((out, total))
}

/// One test year distribution: each individual's percentage, totalling at most 100%.
fn distribution(x: &Value, kind: &str) -> Result<std::collections::BTreeMap<String, Decimal>> {
    let o = x
        .as_object()
        .ok_or_else(|| format!("each {kind} test year distribution must be an object"))?;
    let mut sum = d("0");
    let mut out = std::collections::BTreeMap::new();
    for (name, pct) in o {
        let p = percentage(&json!({ "p": pct }), "p").map_err(|_| {
            format!("{kind} distribution to {name} must be a percentage from 0 to 100")
        })?;
        sum = sum.add(&p)?;
        out.insert(name.clone(), p);
    }
    if sum.gt(&d("100"))? {
        return Err(format!(
            "a {kind} test year distribution totals {sum}%, more than 100%"
        ));
    }
    Ok(out)
}

/// The s 269-60 result from stated test year distributions, with the working.
fn pattern_from(dists: &Value) -> Result<(Option<bool>, String, Value)> {
    let mut results = Vec::new();
    let mut working = Map::new();
    for kind in ["income", "capital"] {
        let list = array(dists, kind)?;
        if list.is_empty() {
            results.push(None);
            continue;
        }
        let (mins, total) = minimums(list, kind)?;
        results.push(Some(total.gt(&d("50"))?));
        working.insert(
            kind.into(),
            json!({"minimum_percentages": mins, "total": total.to_string()}),
        );
    }
    let (income, capital) = (results[0], results[1]);
    let passed = match (income, capital) {
        (None, None) => {
            return Err("test_year_distributions has no income or capital distribution".into());
        }
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    };
    let basis = if passed.is_none() {
        "the distributions of one kind pass, but whether s 269-60 is met with no test year distribution of the other kind is not decided".into()
    } else {
        "each individual counts the smallest percentage they received in any test year distribution; the totals must exceed 50% for income and for capital".into()
    };
    Ok((passed, basis, Value::Object(working)))
}

fn pattern_applies(v: &Value) -> Result<Option<bool>> {
    let mut kinds = Vec::new();
    for kind in ["income", "capital"] {
        let now = fact(
            v,
            &format!("{kind}_distributed_in_income_year_or_2_months_after"),
        )?;
        let before = fact(v, &format!("{kind}_distributed_in_6_earlier_years"))?;
        kinds.push(match (now, before) {
            (Some(true), Some(true)) => Some(true),
            (Some(false), _) | (_, Some(false)) => Some(false),
            _ => None,
        });
    }
    Ok(either(kinds[0], kinds[1]))
}

fn pattern_test(v: &Value) -> Result<(Test, Value)> {
    let (passed, basis, working) = if is_absent(v, "test_year_distributions") {
        let p = fact(v, "pattern_of_distributions_test_passed")?;
        (
            p,
            "the pattern of distributions test result as stated".into(),
            Value::Null,
        )
    } else {
        pattern_from(&v["test_year_distributions"])?
    };
    let test = Test {
        name: "pattern_of_distributions",
        provision: "ss 267-30, 269-60, 269-65, 269-70 Sch 2F ITAA 1936",
        applies: pattern_applies(v)?,
        passed,
        basis,
    };
    Ok((test, working))
}

fn non_fixed_tests(v: &Value) -> Result<(Vec<Test>, Value)> {
    let (pattern, working) = pattern_test(v)?;
    let tests = vec![
        pattern,
        Test {
            name: "pattern_of_distributions_not_previously_failed",
            provision: "s 267-35 Sch 2F ITAA 1936",
            applies: Some(true),
            passed: not(fact(v, "previously_failed_pattern_test")?),
            basis: "the trust must not have been denied the loss in an earlier year for failing s 267-30(2)".into(),
        },
        Test {
            name: "fifty_percent_stake",
            provision: "ss 267-40, 269-50, 269-55 Sch 2F ITAA 1936",
            applies: fact(v, "individuals_had_over_50_percent_stake")?,
            passed: fact(v, "stake_maintained")?,
            basis: "where individuals had more than a 50% stake in income or capital in the test period, the same individuals must keep it to the end (including any s 267-40(3) discretion, as stated)".into(),
        },
        Test {
            name: "control",
            provision: "ss 267-45, 269-95 Sch 2F ITAA 1936",
            applies: Some(true),
            passed: not(fact(v, "group_began_to_control")?),
            basis: "a group must not begin to control the trust in the test period".into(),
        },
    ];
    Ok((tests, working))
}

fn injection_test(v: &Value, para: Option<&str>) -> Result<Test> {
    let exempt = matches!(para, Some("s 272-100(b)" | "s 272-100(c)" | "s 272-100(d)"));
    Ok(Test {
        name: "income_injection",
        provision: "ss 270-10, 270-15, 270-25 Sch 2F ITAA 1936",
        applies: Some(!exempt),
        passed: if exempt {
            None
        } else {
            not(fact(v, "income_injection_scheme")?)
        },
        basis: if exempt {
            "does not apply to an excepted trust under s 272-100(b), (c) or (d) (s 270-10(1)(d))"
                .into()
        } else {
            "where a s 270-10 scheme exists, the deduction is not allowable to the extent it relates to the scheme assessable income (s 270-15)".into()
        },
    })
}

fn deduction(v: &Value) -> Result<()> {
    match text(v, "deduction")? {
        "prior_year_tax_loss" => Ok(()),
        "bad_debt" => Err(refuse(
            "a bad debt or debt/equity swap deduction is not modelled",
            "ss 266-35, 267-25 Sch 2F ITAA 1936",
        )),
        "current_year_loss" => Err(refuse(
            "working out net income and a tax loss by periods is not modelled",
            "s 266-30, Subdiv 267-C, Div 268 Sch 2F ITAA 1936",
        )),
        _ => Err("deduction must be prior_year_tax_loss, bad_debt or current_year_loss".into()),
    }
}

const WIDELY_HELD: [&str; 4] = [
    "unlisted_widely_held",
    "listed_widely_held",
    "unlisted_very_widely_held",
    "wholesale_widely_held",
];

fn trust_kind(v: &Value) -> Result<&str> {
    let kind = text(v, "trust_kind")?;
    if kind == "fixed" || kind == "non_fixed" {
        Ok(kind)
    } else if WIDELY_HELD.contains(&kind) {
        Err(refuse(
            format!("trust_kind {kind} is not modelled"),
            "Subdivs 266-C, 266-D, 266-E Sch 2F ITAA 1936",
        ))
    } else {
        Err("trust_kind must be fixed, non_fixed, unlisted_widely_held, listed_widely_held, unlisted_very_widely_held or wholesale_widely_held".into())
    }
}

fn scope(v: &Value) -> Result<(i64, i64, &str)> {
    let fy = year(v)?;
    let loss_year = integer(v, "loss_year")?;
    if loss_year >= fy {
        return Err(format!(
            "loss_year {loss_year} must be earlier than financial_year {fy}"
        ));
    }
    deduction(v)?;
    Ok((fy, loss_year, trust_kind(v)?))
}

/// True when every applicable test passes, false when an applicable one fails, otherwise not established.
fn verdict(tests: &[Test]) -> Option<bool> {
    if tests
        .iter()
        .any(|t| t.applies == Some(true) && t.passed == Some(false))
    {
        return Some(false);
    }
    if tests
        .iter()
        .any(|t| t.applies != Some(false) && t.passed != Some(true))
    {
        return None;
    }
    Some(true)
}

const NOT_DECIDED: [&str; 3] = [
    "deducting the part of a loss attributable to part of the loss year (ss 266-50, 267-50)",
    "family trust distribution tax (Div 271)",
    "which income years' distributions are test year distributions (s 269-65), and ss 269-75, 269-80 and 269-85: the caller states the percentages after applying them",
];

pub fn evaluate(v: &Value) -> Result<Value> {
    let (fy, loss_year, kind) = scope(v)?;
    let exc = excepted(v)?;
    let (mut tests, working) = match (exc, kind) {
        // An excepted trust at all times in the test period is outside ss 266-25 and 267-20.
        (Some(e), _) => (excepted_tests(e.1), Value::Null),
        (None, "fixed") => (fixed_tests(v)?, Value::Null),
        _ => non_fixed_tests(v)?,
    };
    tests.push(injection_test(v, exc.map(|e| e.1))?);
    let not_established: Vec<&str> = tests
        .iter()
        .filter(|t| match t.applies {
            Some(true) => t.passed.is_none(),
            Some(false) => false,
            None => t.passed != Some(true),
        })
        .map(|t| t.name)
        .collect();
    Ok(
        json!({"financial_year": fy, "loss_year": loss_year, "trust_kind": kind,
        "excepted_trust": exc.map(|e| json!({"kind": e.0, "provision": e.1})),
        "tests": tests.iter().map(Test::json).collect::<Vec<_>>(),
        "pattern_of_distributions_working": working, "deductible": verdict(&tests),
        "not_established": not_established, "not_decided": NOT_DECIDED}),
    )
}
