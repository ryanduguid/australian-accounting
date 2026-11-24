// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The persons and stated facts that link them, read from the input, and the
//! groups the harmonised grouping Parts form from them (Payroll Tax Act 2007
//! (Vic) ss 70 to 78 and each other Act's equivalent).

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde_json::Value;

use crate::decimal::Decimal;
use crate::engine::{Arg, Result, d, int, money, refuse};

/// One person: an employer, or a holder that pays no wages here.
pub(super) struct Person {
    pub(super) id: String,
    pub(super) taxable: Decimal,
    pub(super) interstate: Decimal,
    pub(super) business: bool,
}

/// The stated facts, with each person by its place in `persons`.
#[derive(Default)]
pub(super) struct Facts {
    pub(super) persons: Vec<Person>,
    pub(super) related: Vec<(usize, usize)>,
    pub(super) employees: Vec<(usize, usize)>,
    /// Holders with a controlling interest in a business (s 72(2)).
    pub(super) control: Vec<(Vec<usize>, usize)>,
    /// Holders' voting power in a corporation, as a fraction (s 76).
    pub(super) voting: Vec<(Vec<usize>, usize, Decimal)>,
}

fn invalid<T>(message: impl Into<String>) -> Result<T> {
    refuse("payroll_tax_group.fact_invalid", message)
}

fn items<'a>(arg: Option<&'a Arg>, name: &str) -> Result<&'a [Value]> {
    match arg {
        None => Ok(&[]),
        Some(Arg::Json(Value::Array(items))) => Ok(items),
        Some(_) => invalid(format!("{name} must be a list.")),
    }
}

fn amount(value: Option<&Value>, what: &str) -> Result<Decimal> {
    let Some(value) = value else {
        return Ok(int(0));
    };
    match value.as_str().and_then(|t| Decimal::parse_input(t).ok()) {
        Some(v) => money(&Arg::Decimal(v)),
        None => invalid(format!("{what} must be an amount as a string.")),
    }
}

/// Refuses a key an object does not take, so a misspelt wage is not nil.
fn keys(item: &Value, what: &str, allowed: &[&str]) -> Result<()> {
    let Some(map) = item.as_object() else {
        return invalid(format!("Each of {what} must be an object."));
    };
    match map.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(key) => refuse(
            "input.field_unknown",
            format!(
                "{what} does not take {key}; it takes {}.",
                allowed.join(", ")
            ),
        ),
        None => Ok(()),
    }
}

fn person(item: &Value) -> Result<Person> {
    let fields = [
        "id",
        "taxable_wages",
        "interstate_wages",
        "carries_on_business",
    ];
    keys(item, "persons", &fields)?;
    let Some(id) = item.get("id").and_then(Value::as_str) else {
        return invalid("Each person needs an id string.");
    };
    let taxable = amount(item.get("taxable_wages"), "taxable_wages")?;
    let interstate = amount(item.get("interstate_wages"), "interstate_wages")?;
    let business = match item.get("carries_on_business") {
        None => !(taxable.is_zero() && interstate.is_zero()),
        Some(Value::Bool(b)) => *b,
        Some(_) => return invalid("carries_on_business must be true or false."),
    };
    Ok(Person {
        id: id.to_string(),
        taxable,
        interstate,
        business,
    })
}

/// Each person's place by id.
pub(super) struct Index(HashMap<String, usize>);

impl Index {
    pub(super) fn of(&self, value: &Value) -> Result<usize> {
        match value.as_str().and_then(|id| self.0.get(id)) {
            Some(&at) => Ok(at),
            None => refuse(
                "payroll_tax_group.person_unknown",
                format!("{value} is not a person listed in persons."),
            ),
        }
    }

    /// A non-empty list of persons, each once, in input order.
    pub(super) fn set(&self, value: Option<&Value>, what: &str) -> Result<Vec<usize>> {
        let Some(Value::Array(ids)) = value else {
            return invalid(format!("{what} must be a list of person ids."));
        };
        let mut set = Vec::new();
        for id in ids {
            let at = self.of(id)?;
            if set.contains(&at) {
                return invalid(format!("{what} names {id} twice."));
            }
            set.push(at);
        }
        if set.is_empty() {
            return invalid(format!("{what} must name at least one person."));
        }
        set.sort_unstable();
        Ok(set)
    }

    fn pairs(&self, arg: Option<&Arg>, name: &str) -> Result<Vec<(usize, usize)>> {
        let mut pairs = Vec::new();
        for item in items(arg, name)? {
            match self.set(Some(item), name)?.as_slice() {
                [a, b] => pairs.push((*a, *b)),
                _ => return invalid(format!("Each of {name} must name 2 persons.")),
            }
        }
        Ok(pairs)
    }
}

pub(super) fn persons(arg: &Arg) -> Result<(Vec<Person>, Index)> {
    let mut list = Vec::new();
    let mut index = HashMap::new();
    for item in items(Some(arg), "persons")? {
        let p = person(item)?;
        if index.insert(p.id.clone(), list.len()).is_some() {
            return refuse(
                "payroll_tax_group.person_duplicate",
                format!("{} is listed twice in persons.", p.id),
            );
        }
        list.push(p);
    }
    Ok((list, Index(index)))
}

/// Tests of s 72(2) (WA s 33) passed by a stated fact alone.
const FACT_TESTS: [&str; 4] = [
    "board_composition",
    "directors_obliged",
    "discretionary_object",
    "sole_owner",
];
/// Tests passed only by more than 50%.
const PERCENT_TESTS: [&str; 6] = [
    "board_members",
    "director_votes",
    "partnership_capital",
    "partnership_profits",
    "trust_interests",
    "voting_shares",
];

/// A percentage given as a string, as a fraction from nil (exclusive) to 1.
fn fraction(value: Option<&Value>) -> Result<Decimal> {
    let parsed = value
        .and_then(Value::as_str)
        .and_then(|t| Decimal::parse_input(t).ok());
    match parsed {
        Some(p) if p.gt(&int(0))? && p.le(&int(100))? => Ok(p.div(&int(100))?),
        _ => refuse(
            "payroll_tax_group.percent_invalid",
            "percent must be a string above 0 and at most 100.",
        ),
    }
}

/// Whether one stated controlling-interest fact passes its test.
fn passes(item: &Value) -> Result<bool> {
    let test = item.get("test").and_then(Value::as_str).unwrap_or("");
    if FACT_TESTS.contains(&test) {
        if item.get("percent").is_some() {
            return invalid(format!("The {test} test takes no percent."));
        }
        return Ok(true);
    }
    if PERCENT_TESTS.contains(&test) {
        return Ok(fraction(item.get("percent"))?.gt(&d("0.5"))?);
    }
    refuse(
        "payroll_tax_group.test_invalid",
        "test must be board_composition, directors_obliged, discretionary_object, sole_owner, board_members, director_votes, partnership_capital, partnership_profits, trust_interests or voting_shares.",
    )
}

/// The stated links between persons.
pub(super) struct Stated<'a> {
    pub(super) related: Option<&'a Arg>,
    pub(super) employees: Option<&'a Arg>,
    pub(super) control: Option<&'a Arg>,
    pub(super) voting: Option<&'a Arg>,
}

pub(super) fn facts(persons: Vec<Person>, index: &Index, stated: &Stated) -> Result<Facts> {
    let mut f = Facts {
        related: index.pairs(stated.related, "related_bodies_corporate")?,
        employees: index.pairs(stated.employees, "common_employees")?,
        ..Facts::default()
    };
    for item in items(stated.control, "controlling_interests")? {
        keys(
            item,
            "controlling_interests",
            &["holders", "business", "test", "percent"],
        )?;
        let holders = index.set(item.get("holders"), "holders")?;
        let business = index.of(item.get("business").unwrap_or(&Value::Null))?;
        if passes(item)? {
            f.control.push((holders, business));
        }
    }
    for item in items(stated.voting, "voting_interests")? {
        keys(
            item,
            "voting_interests",
            &["holders", "corporation", "percent"],
        )?;
        let holders = index.set(item.get("holders"), "holders")?;
        let corporation = index.of(item.get("corporation").unwrap_or(&Value::Null))?;
        f.voting
            .push((holders, corporation, fraction(item.get("percent"))?));
    }
    f.persons = persons;
    Ok(f)
}

/// A union of persons and the provision that made it.
pub(super) struct Basis {
    pub(super) kind: usize,
    pub(super) persons: Vec<usize>,
}

/// The bases, by their place in `Act::sections`.
pub(super) const CORPORATIONS: usize = 0;
pub(super) const EMPLOYEES: usize = 1;
pub(super) const CONTROL: usize = 2;
pub(super) const TRACING: usize = 3;
pub(super) const SUBSUMED: usize = 4;

/// Persons joined into groups, skipping the persons in `out`.
pub(super) struct Groups {
    parent: Vec<usize>,
    pub(super) bases: Vec<Basis>,
}

impl Groups {
    pub(super) fn root(&mut self, mut at: usize) -> usize {
        while self.parent[at] != at {
            self.parent[at] = self.parent[self.parent[at]];
            at = self.parent[at];
        }
        at
    }

    /// Joins `persons`; true when that joined 2 groups.
    fn join(&mut self, kind: usize, persons: Vec<usize>) -> bool {
        let mut changed = false;
        for pair in persons.windows(2) {
            let (a, b) = (self.root(pair[0]), self.root(pair[1]));
            if a != b {
                self.parent[a] = b;
                changed = true;
            }
        }
        if persons.len() > 1 {
            self.bases.push(Basis { kind, persons });
        }
        changed
    }

    /// Every person in the group that `at` is in, in input order.
    pub(super) fn members(&mut self, at: usize) -> Vec<usize> {
        let root = self.root(at);
        (0..self.parent.len())
            .filter(|&p| self.root(p) == root)
            .collect()
    }
}

/// A related body corporate controls what the other controls (s 72(3)). The
/// pair is already a group (s 70), so only control of a third person's
/// business is passed on.
fn pass_to_related(
    f: &Facts,
    out: &[bool],
    snapshot: &BTreeMap<Vec<usize>, BTreeSet<usize>>,
    held: &mut BTreeMap<Vec<usize>, BTreeSet<usize>>,
) {
    for &(a, b) in &f.related {
        for (from, to) in [(a, b), (b, a)] {
            if let Some(more) = snapshot.get(&vec![from]).filter(|_| !out[to]) {
                let others = more.iter().filter(|&&x| x != from && x != to);
                held.entry(vec![to]).or_default().extend(others);
            }
        }
    }
}

/// The businesses each set of holders controls, with control passed along
/// chains (s 72(4), (5), (7), (8)) and to related bodies corporate (s 72(3)).
fn controlled(f: &Facts, out: &[bool]) -> BTreeMap<Vec<usize>, BTreeSet<usize>> {
    let mut held: BTreeMap<Vec<usize>, BTreeSet<usize>> = BTreeMap::new();
    for (p, person) in f.persons.iter().enumerate() {
        if person.business && !out[p] {
            held.entry(vec![p]).or_default().insert(p);
        }
    }
    for (holders, business) in &f.control {
        if !out[*business] && holders.iter().all(|&h| !out[h]) {
            held.entry(holders.clone()).or_default().insert(*business);
        }
    }
    loop {
        let before: usize = held.values().map(BTreeSet::len).sum();
        let snapshot = held.clone();
        for set in held.values_mut() {
            for b in set.clone() {
                if let Some(more) = snapshot.get(&vec![b]) {
                    set.extend(more);
                }
            }
        }
        pass_to_related(f, out, &snapshot, &mut held);
        if held.values().map(BTreeSet::len).sum::<usize>() == before {
            return held;
        }
    }
}

/// The value of each chain of direct interests from `from` to each
/// corporation linked to it (s 77), summed over chains.
fn linked(
    f: &Facts,
    from: usize,
    out: &[bool],
    path: &mut Vec<usize>,
) -> Result<BTreeMap<usize, Decimal>> {
    let mut reach = BTreeMap::new();
    for (holders, next, value) in &f.voting {
        if holders.as_slice() != [from] || out[*next] {
            continue;
        }
        if path.contains(next) {
            return refuse(
                "payroll_tax_group.interest_cycle",
                "The voting interests hold one another in a cycle, so no chain of corporations ends (Payroll Tax Act 2007 (Vic) s 77(2) and each other Act's equivalent).",
            );
        }
        path.push(*next);
        let beyond = linked(f, *next, out, path)?;
        path.pop();
        add(&mut reach, *next, value)?;
        for (c, v) in beyond {
            add(&mut reach, c, &value.mul(&v)?)?;
        }
    }
    Ok(reach)
}

fn add(reach: &mut BTreeMap<usize, Decimal>, at: usize, value: &Decimal) -> Result<()> {
    let sum = reach
        .get(&at)
        .cloned()
        .unwrap_or_else(|| int(0))
        .add(value)?;
    reach.insert(at, sum);
    Ok(())
}

/// Each entity's aggregate interest in each corporation (ss 76 to 78): its
/// direct interest plus each indirect one through the corporations it holds.
fn traced(f: &Facts, out: &[bool]) -> Result<Vec<(Vec<usize>, usize)>> {
    let mut totals: BTreeMap<(Vec<usize>, usize), Decimal> = BTreeMap::new();
    for (holders, first, value) in &f.voting {
        if out[*first] || holders.iter().any(|&h| out[h]) {
            continue;
        }
        let mut entry = |c: usize, v: Decimal| -> Result<()> {
            let key = (holders.clone(), c);
            let sum = totals
                .get(&key)
                .cloned()
                .unwrap_or_else(|| int(0))
                .add(&v)?;
            totals.insert(key, sum);
            Ok(())
        };
        entry(*first, value.clone())?;
        for (c, v) in linked(f, *first, out, &mut vec![*first])? {
            entry(c, value.mul(&v)?)?;
        }
    }
    let mut controlling = Vec::new();
    for ((holders, c), value) in totals {
        if value.gt(&d("0.5"))? {
            controlling.push((holders, c));
        }
    }
    Ok(controlling)
}

/// The groups `f` forms without the persons in `out` (ss 70 to 74).
pub(super) fn form(f: &Facts, out: &[bool]) -> Result<Groups> {
    let mut g = Groups {
        parent: (0..f.persons.len()).collect(),
        bases: Vec::new(),
    };
    let kept = |pair: &&(usize, usize)| !out[pair.0] && !out[pair.1];
    for &(a, b) in f.related.iter().filter(kept) {
        g.join(CORPORATIONS, vec![a, b]);
    }
    for &(a, b) in f.employees.iter().filter(kept) {
        g.join(EMPLOYEES, vec![a, b]);
    }
    let held = controlled(f, out);
    for set in held.values().filter(|set| set.len() > 1) {
        g.join(CONTROL, set.iter().copied().collect());
    }
    for (mut holders, c) in traced(f, out)? {
        holders.push(c);
        holders.sort_unstable();
        holders.dedup();
        g.join(TRACING, holders);
    }
    subsume(&mut g, &held);
    Ok(g)
}

/// Members of a group that together control a business bring in the persons
/// who carry it on (s 74(2)); repeated until no group grows.
fn subsume(g: &mut Groups, held: &BTreeMap<Vec<usize>, BTreeSet<usize>>) {
    let mut grew = true;
    while grew {
        grew = false;
        for (holders, set) in held.iter().filter(|(h, _)| h.len() > 1) {
            let root = g.root(holders[0]);
            let joined = holders.iter().all(|&h| g.root(h) == root);
            if joined && set.iter().any(|&b| g.root(b) != root) {
                let mut persons = vec![holders[0]];
                persons.extend(set);
                grew |= g.join(SUBSUMED, persons);
            }
        }
    }
}
