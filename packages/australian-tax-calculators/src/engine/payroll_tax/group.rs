// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `payroll_tax_group`: from stated facts, the payroll tax group one employer
//! belongs to in one jurisdiction for 2026-27, the designated group employer
//! and the group's single deduction. Each Act's grouping Part was read on 24
//! November 2026 (New South Wales, Tasmania and the Northern Territory on 27
//! November); the thresholds and tapers are `payroll_tax`'s, applied to the
//! group's wages as each Act's group clause reads.

use serde_json::{Value, json};

use super::{Wages, worked};
use crate::decimal::Decimal;
use crate::engine::{
    Arg, Result, Step, d, derived_result, flag, int, precise, py_min, refuse, scope, step,
};

mod links;
use links::{Facts, Groups, Index, Stated, facts, form, persons};

const KIND: &str = "payroll_tax_group";

/// The facts as the command passes them; see the README for each.
pub struct PayrollTaxGroupArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub employer: &'a Arg,
    pub persons: &'a Arg,
    pub related_bodies_corporate: Option<&'a Arg>,
    pub common_employees: Option<&'a Arg>,
    pub controlling_interests: Option<&'a Arg>,
    pub voting_interests: Option<&'a Arg>,
    pub exclusions: Option<&'a Arg>,
    pub designated_group_employers: Option<&'a Arg>,
    pub excess_deduction_order: Option<&'a Arg>,
    pub annual_calculation_allowed: Option<&'a Arg>,
    pub employed_all_year: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One Act's grouping provisions and group deduction clauses.
struct Act {
    name: &'static str,
    /// Related corporations, common employees, common control, tracing,
    /// subsumed groups, exclusion and designation.
    sections: [&'static str; 7],
    deduction: &'static str,
    claim: &'static str,
    none: &'static str,
    /// Where the Act frees every member when a group with a DGE is under its
    /// threshold, the clause that says so.
    nil: &'static str,
}

const HARMONISED: [&str; 7] = [
    "s 70",
    "s 71",
    "s 72",
    "s 73 and ss 76 to 78",
    "s 74",
    "s 79",
    "s 80",
];

const VIC: Act = Act {
    name: "Payroll Tax Act 2007 (Vic)",
    sections: HARMONISED,
    deduction: "Payroll Tax Act 2007 (Vic) Sch 1 cl 7A (authorised version 041, read 24 November 2026): the lesser of the tapered deduction and GTW / (GTW + GIW) x TA, worked on the group's taxable and interstate wages",
    claim: "Payroll Tax Act 2007 (Vic) Sch 1 cl 9A(3) and 9B(3): only the designated group employer deducts the deductible amount ([TW - D] x R); each other member pays TW x R (cl 9A(4), 9B(4))",
    none: "Payroll Tax Act 2007 (Vic) Sch 1 Pt 4 cl 12: with no designated group employer each member pays TW x R, so no member deducts",
    nil: "",
};
const QLD: Act = Act {
    name: "Payroll Tax Act 1971 (Qld)",
    sections: [
        "s 69",
        "s 70",
        "s 71",
        "s 72 and ss 74E to 74G",
        "s 73",
        "s 74",
        "s 75",
    ],
    deduction: "Payroll Tax Act 1971 (Qld) s 33, annual deduction for the DGE (current reprint, read 24 November 2026): K $1,300,000, falling by $1 for every $7 of the group's taxable and interstate wages above it, x TW / (TW + IW) of the relevant group employers",
    claim: "Payroll Tax Act 1971 (Qld) s 33, annual payroll tax amount, and s 45: the DGE deducts up to its own annual wages; any more is the excess deduction",
    none: "Payroll Tax Act 1971 (Qld) ss 32 and 33 give the annual deduction to the DGE alone (the Queensland Revenue Office's 'Payroll tax deductions': 'only the designated group employer (DGE) can claim a deduction'), so with none no member deducts",
    nil: "",
};
const WA: Act = Act {
    name: "Pay-roll Tax Assessment Act 2002 (WA)",
    sections: [
        "s 30",
        "s 31",
        "ss 32 to 34",
        "ss 35A to 35D",
        "s 36",
        "s 38",
        "s 39",
    ],
    deduction: "Pay-roll Tax Assessment Act 2002 (WA) s 17(2)(c) and (3A) (version 04-d0-03, read 24 November 2026): the group's deductable amount (AT - [(TW - AT) x TV]) x W / TW on the group's Australian and WA taxable wages, deducted from the group's WA taxable wages whether or not a DGE is designated",
    claim: "",
    none: "",
    nil: "",
};
const SA: Act = Act {
    name: "Payroll Tax Act 2009 (SA)",
    sections: HARMONISED,
    deduction: "Payroll Tax Act 2009 (SA) Sch 1 cl 7 and cl 9(2) (version 31.3.2026, read 24 November 2026): GTW / (GTW + GIW) x EA, worked on the group's taxable and interstate wages",
    claim: "Payroll Tax Act 2009 (SA) Sch 1 cl 9(2) and (3): only the designated group employer deducts; each other member pays TW x R",
    none: "",
    nil: "",
};
const ACT: Act = Act {
    name: "Payroll Tax Act 2011 (ACT)",
    sections: HARMONISED,
    deduction: "Payroll Tax Act 2011 (ACT) Sch 1 s 1.7 and s 1.9(2) (current republication, read 24 November 2026): GTW / (GTW + GIW) x TA, worked on the group's taxable and interstate wages",
    claim: "Payroll Tax Act 2011 (ACT) Sch 1 s 1.9(2) and (3): only the designated group employer deducts ([TW - [GTW / (GTW + GIW) x TA]] x R); each other member pays TW x R",
    none: "Payroll Tax Act 2011 (ACT) Sch 1 Pt 1.4: with no designated group employer each member pays TW x R, so no member deducts",
    nil: "",
};
const NSW: Act = Act {
    name: "Payroll Tax Act 2007 (NSW)",
    sections: HARMONISED,
    deduction: "Payroll Tax Act 2007 (NSW) Sch 1 cl 9(2) (AustLII consolidation as at 22 October 2026, read 27 November 2026): GTW / (GTW + GIW) x TA x C / FY, worked on the group's taxable and interstate wages (C / FY is 1 for a group paying wages all year)",
    claim: "Payroll Tax Act 2007 (NSW) Sch 1 cl 9(2) and (3): only the designated group employer deducts ([TW - [GTW / (GTW + GIW) x TA x C / FY]] x R); each other member pays TW x R",
    none: "Payroll Tax Act 2007 (NSW) Sch 1 Pt 4 cl 12: with no designated group employer each member pays TW x R, so no member deducts",
    nil: "Payroll Tax Act 2007 (NSW) Sch 1 cl 8: the group's taxable and interstate wages are not more than the group threshold amount (TA x C / FY), so no member is liable to pay payroll tax for the year",
};
const TAS: Act = Act {
    name: "Payroll Tax Act 2008 (Tas)",
    sections: HARMONISED,
    deduction: "Payroll Tax Act 2008 (Tas) Sch 1 cl 8(1), 9A(2) and 9B(2) (current version, read 27 November 2026): GTW / (GTW + GIW) x TA x C / FY, the initial threshold shared by the group's taxable wages over its taxable and interstate wages; each other member pays TW x R1 (cl 9A(3)), or TW x R2 when the group's wages are more than the subsequent threshold (cl 9B(3))",
    claim: "Payroll Tax Act 2008 (Tas) Sch 1 cl 8(2) and 9B(2): above the subsequent threshold (TTB x C / FY) the designated group employer pays [TTB - TA] x GTW / (GTW + GIW) x C / FY x R1 + [TW - GTW / (GTW + GIW) x TTB x C / FY] x R2; the second deduction is the top bracket threshold shared the same way",
    none: "Payroll Tax Act 2008 (Tas) Sch 1 Pt 4 cl 12: with no designated group employer each member pays TW x R2, so no member deducts",
    nil: "Payroll Tax Act 2008 (Tas) Sch 1 cl 9: the group's taxable and interstate wages are not more than the initial threshold (TA x C / FY, cl 8(1)), so no member is liable to pay payroll tax for the year",
};
const NT: Act = Act {
    name: "Payroll Tax Act 2009 (NT)",
    sections: HARMONISED,
    deduction: "Payroll Tax Act 2009 (NT) Sch 1 cl 7 and 9(2) (as in force at 1 July 2026, read 27 November 2026): GTW / (GTW + GIW) x AD, where AD = GTA - [(GTW + GIW - GTA) / 2], nil if not positive, and GTA = TA x C / FY (cl 8); each other member pays TW x R (cl 9(3))",
    claim: "",
    none: "Payroll Tax Act 2009 (NT) Sch 1 Pt 4 cl 12: with no designated group employer each member pays TW x R, so no member deducts",
    nil: "Payroll Tax Act 2009 (NT) Sch 1 cl 8: the group's taxable and interstate wages are not more than the group threshold amount (TA x C / FY), so no member is liable to pay payroll tax for the year",
};

const BASES: [&str; 5] = [
    "related bodies corporate",
    "common employees",
    "common control",
    "tracing of interests in corporations",
    "smaller groups subsumed",
];

/// The Act for `place`, or the refusal for a jurisdiction not worked.
fn act(place: Option<&str>) -> Result<(&'static str, &'static Act)> {
    match place {
        Some("vic") => Ok(("vic", &VIC)),
        Some("qld") => Ok(("qld", &QLD)),
        Some("wa") => Ok(("wa", &WA)),
        Some("sa") => Ok(("sa", &SA)),
        Some("act") => Ok(("act", &ACT)),
        Some("nsw") => Ok(("nsw", &NSW)),
        Some("tas") => Ok(("tas", &TAS)),
        Some("nt") => Ok(("nt", &NT)),
        _ => refuse(
            "payroll_tax_group.jurisdiction_invalid",
            "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.",
        ),
    }
}

/// The persons each of a list of ids names, for `exclusions` and the like.
fn named(index: &Index, arg: Option<&Arg>, what: &str) -> Result<Vec<usize>> {
    match arg {
        None => Ok(Vec::new()),
        Some(Arg::Json(Value::Array(ids))) if ids.is_empty() => Ok(Vec::new()),
        Some(Arg::Json(value)) => index.set(Some(value), what),
        Some(_) => refuse(
            "payroll_tax_group.fact_invalid",
            format!("{what} must be a list of person ids."),
        ),
    }
}

/// The group `employer` is in once the excluded persons are taken out, with
/// the refusals of s 79(3) and of an exclusion that would split the group.
fn group_of(
    f: &Facts,
    employer: usize,
    excluded: &[usize],
    (place, act): (&str, &Act),
) -> Result<(Vec<usize>, Groups)> {
    // Queensland excludes a person or a set of persons, barred only by a
    // related body corporate outside the set (s 74(4)); the other Acts exclude
    // a person, barred by any related body corporate in the group.
    for &x in excluded {
        let barred = f.related.iter().any(|&(a, b)| {
            let other = if a == x {
                b
            } else if b == x {
                a
            } else {
                return false;
            };
            place != "qld" || !excluded.contains(&other)
        });
        if barred {
            return refuse(
                "payroll_tax_group.exclusion_barred",
                format!(
                    "{} cannot be excluded: it is a related body corporate of another member, and {} {} bars that.",
                    f.persons[x].id, act.name, act.sections[5]
                ),
            );
        }
    }
    // An excluded person is not a member (s 67 'group'), so the group is
    // formed again without it and its links.
    let mut out = vec![false; f.persons.len()];
    excluded.iter().for_each(|&x| out[x] = true);
    let mut rest = form(f, &out)?;
    let members = rest.members(employer);
    if out[employer] || members.len() < 2 {
        return not_grouped(f, employer);
    }
    Ok((members, rest))
}

fn not_grouped<T>(f: &Facts, employer: usize) -> Result<T> {
    refuse(
        "payroll_tax_group.not_grouped",
        format!(
            "{} is not a member of a group on these facts: use payroll-tax.",
            f.persons[employer].id
        ),
    )
}

/// The designated group employer among `members`, if one is stated.
fn designated(
    f: &Facts,
    members: &[usize],
    stated: &[usize],
    groups: &mut Groups,
) -> Result<Option<usize>> {
    let root = groups.root(members[0]);
    let mut dge = None;
    for &p in stated {
        if !members.contains(&p) {
            if groups.root(p) == root || groups.members(p).len() < 2 {
                return refuse(
                    "payroll_tax_group.dge_not_member",
                    format!(
                        "{} is not a member of a group, so it cannot be its designated group employer.",
                        f.persons[p].id
                    ),
                );
            }
            continue;
        }
        if dge.replace(p).is_some() {
            return refuse(
                "payroll_tax_group.dge_conflict",
                "A group has only one designated group employer; 2 are stated for this group.",
            );
        }
    }
    Ok(dge)
}

/// The members' wages as a sum of each, and that sum.
fn summed(
    f: &Facts,
    members: &[usize],
    pick: fn(&links::Person) -> &Decimal,
) -> Result<(String, Decimal)> {
    let mut total = int(0);
    let mut terms = Vec::new();
    for &p in members {
        let wages = pick(&f.persons[p]);
        total = total.add(wages)?;
        terms.push(wages.to_string());
    }
    Ok((terms.join(" + "), total))
}

const GTW: &str = "The taxable wages paid or payable in the jurisdiction by the group's members (GTW), each member's as stated, in the order of persons";
const GIW: &str = "The interstate wages paid or payable by the group's members (GIW), each member's as stated, in the order of persons";
const GAW: &str = "The group's taxable and interstate wages together (GTW + GIW), on which each Act's group clause works the deduction";
const NO_WAGES: &str = "The group pays no taxable or interstate wages, so there is no deduction";

/// The group's taxable, interstate and Australian wages.
fn wage_steps(f: &Facts, members: &[usize]) -> Result<(Vec<Step>, Wages)> {
    let (taxable_terms, gtw) = summed(f, members, |p| &p.taxable)?;
    let (interstate_terms, giw) = summed(f, members, |p| &p.interstate)?;
    let australian = gtw.add(&giw)?;
    let steps = vec![
        step("taxable_wages", GTW, &taxable_terms, &[], gtw.clone()),
        step("interstate_wages", GIW, &interstate_terms, &[], giw.clone()),
        step(
            "australian_wages",
            GAW,
            "taxable_wages + interstate_wages",
            &[&gtw, &giw],
            australian.clone(),
        ),
    ];
    Ok((
        steps,
        Wages {
            taxable: gtw,
            australian,
        },
    ))
}

/// The group's wages, the deduction worked on them and the group deduction,
/// and whether any member is liable: not where the Act frees a group with a
/// DGE whose wages are not more than its threshold (`act.nil`).
fn group_steps(
    place: &str,
    act: &Act,
    f: &Facts,
    members: &[usize],
    claims: bool,
) -> Result<(Table, bool)> {
    let (mut steps, wages) = wage_steps(f, members)?;
    let floored = claims && !act.nil.is_empty();
    if wages.australian.is_zero() {
        steps.push(step("group_deduction", NO_WAGES, "0", &[], int(0)));
        return Ok(((steps, Vec::new()), !floored));
    }
    let worked = worked(place, &wages, false)?;
    let mut it = worked.steps.into_iter();
    let base = it
        .next()
        .expect("each jurisdiction starts with its deduction");
    // Below the threshold the NT's annual deduction is the threshold itself.
    let under = !wages.australian.gt(&base.value)?;
    steps.push(base);
    if floored && under {
        steps.push(step("group_deduction", act.nil, "0", &[], int(0)));
        return Ok(((steps, worked.rates), false));
    }
    let shared = it.next().expect("then shares it");
    let value = shared.value.clone();
    steps.push(shared);
    steps.push(if claims {
        step(
            "group_deduction",
            act.deduction,
            "deduction",
            &[&value],
            value.clone(),
        )
    } else {
        step("group_deduction", act.none, "0", &[], int(0))
    });
    Ok(((steps, worked.rates), true))
}

/// Tasmania above the subsequent threshold: the DGE also deducts the top
/// bracket threshold, shared as the initial one is, from its R2 wages (Sch 1
/// cl 8(2) and 9B(2)); refused when its wages are less.
fn tas_top(f: &Facts, dge: usize, steps: &[Step]) -> Result<Option<Step>> {
    let value = |id: &str| {
        steps
            .iter()
            .find(|s| s.id == id)
            .expect("the group's wages are worked")
            .value
            .clone()
    };
    let (gtw, gaw, top) = (
        value("taxable_wages"),
        value("australian_wages"),
        d(super::TAS_TOP),
    );
    if !gaw.gt(&top)? {
        return Ok(None);
    }
    let share = gtw.mul(&top)?.div(&gaw)?;
    if f.persons[dge].taxable.lt(&share)? {
        return refuse(
            "payroll_tax_group.dge_wages_below_deduction",
            "The designated group employer's taxable wages are less than the top bracket threshold's share: Payroll Tax Act 2008 (Tas) Sch 1 cl 9B(2) would go below nil, and the Act does not say where the rest goes.",
        );
    }
    let operands = [&gtw, &top, &gaw];
    Ok(Some(step(
        "top_bracket_deduction",
        TAS.claim,
        "taxable_wages x top_bracket / australian_wages",
        &operands,
        share,
    )))
}

/// The NT works the year under Sch 1 cl 9 only where the Commissioner allows
/// it in writing (cl 9(1)(c)); the fact is taken in the NT alone.
fn nt_allowed(place: &str, arg: Option<&Arg>) -> Result<Option<bool>> {
    match arg {
        None => Ok(None),
        Some(_) if place != "nt" => refuse(
            "payroll_tax_group.fact_invalid",
            "annual_calculation_allowed is taken in the Northern Territory only (Payroll Tax Act 2009 (NT) Sch 1 cl 9(1)(c)).",
        ),
        Some(a) => flag(a, "annual_calculation_allowed").map(Some),
    }
}

fn nt_unallowed<T>() -> Result<T> {
    refuse(
        "payroll_tax_group.nt_annual_unallowed",
        "Payroll Tax Act 2009 (NT) Sch 1 cl 9 applies to a group with a designated group employer above the group threshold amount only if the Commissioner, by writing, allows the group to calculate payroll tax under it (cl 9(1)(c)); otherwise Sch 2 cl 14 applies, which this worksheet does not work. State annual_calculation_allowed: true when the Commissioner has allowed it.",
    )
}

/// Outside Queensland, refuses a DGE whose wages are below the deduction.
fn below(own: &Decimal, deduction: &Decimal) -> Result<()> {
    if own.lt(deduction)? {
        return refuse(
            "payroll_tax_group.dge_wages_below_deduction",
            "The designated group employer's taxable wages are less than the group deduction: the Act's formula for it would go below nil, and the Act does not say where the rest of the deduction goes.",
        );
    }
    Ok(())
}

/// The DGE's claim and, in Queensland, the excess shared in order (ss 45, 48).
fn claim_steps(
    place: &str,
    f: &Facts,
    dge: usize,
    deduction: &Decimal,
    order: &[usize],
) -> Result<(Vec<Step>, Vec<Value>)> {
    let own = &f.persons[dge].taxable;
    if place != "qld" {
        return below(own, deduction).map(|()| (Vec::new(), Vec::new()));
    }
    let claimed = py_min(deduction.clone(), own.clone())?;
    let excess = deduction.sub(&claimed)?;
    if !excess.is_zero() && order.is_empty() {
        return refuse(
            "payroll_tax_group.excess_unnominated",
            "The group deduction exceeds the designated group employer's wages, and no excess_deduction_order is stated: the DGE nominates the members who share it (Payroll Tax Act 1971 (Qld) s 46), or the commissioner determines them (s 47).",
        );
    }
    let mut left = excess.clone();
    let (mut shares, mut less) = (Vec::new(), String::from("excess_deduction"));
    for &p in order {
        let share = py_min(left.clone(), f.persons[p].taxable.clone())?;
        left = left.sub(&share)?;
        less.push_str(&format!(" - {share}"));
        shares.push(
            json!({"person": f.persons[p].id, "share": crate::engine::cents(share)?.to_string()}),
        );
    }
    let steps = vec![
        step(
            "dge_deduction",
            QLD.claim,
            &format!("min(group_deduction, {own})"),
            &[deduction],
            claimed.clone(),
        ),
        step(
            "excess_deduction",
            EXCESS,
            "group_deduction - dge_deduction",
            &[deduction, &claimed],
            excess.clone(),
        ),
        step("excess_unshared", UNSHARED, &less, &[&excess], left),
    ];
    Ok((steps, shares))
}

const EXCESS: &str = "Payroll Tax Act 1971 (Qld) s 45: the excess deduction, the DGE's annual deduction less its annual wages";
const UNSHARED: &str = "Payroll Tax Act 1971 (Qld) s 48(2): each entitled group member, in the order nominated by the DGE (s 46) or determined by the commissioner (s 47), takes the lesser of what remains and its annual wages; what no member takes is unshared";

fn bases(f: &Facts, groups: &Groups, members: &[usize], act: &Act) -> Vec<Value> {
    groups
        .bases
        .iter()
        .filter(|b| b.persons.iter().any(|p| members.contains(p)))
        .map(|b| {
            json!({
                "basis": BASES[b.kind],
                "persons": b.persons.iter().map(|&p| f.persons[p].id.clone()).collect::<Vec<_>>(),
                "provision": format!("{} {}", act.name, act.sections[b.kind]),
            })
        })
        .collect()
}

/// The persons, the stated facts and the employer whose group is worked.
fn gather(args: &PayrollTaxGroupArgs) -> Result<(Facts, Index, usize)> {
    let (list, index) = persons(args.persons)?;
    let stated = Stated {
        related: args.related_bodies_corporate,
        employees: args.common_employees,
        control: args.controlling_interests,
        voting: args.voting_interests,
    };
    let f = facts(list, &index, &stated)?;
    let employer = index.of(&args.employer.text().map_or(Value::Null, Value::from))?;
    Ok((f, index, employer))
}

type Table = (Vec<Step>, Vec<(String, String)>);
type Worked = (Vec<Step>, Vec<(String, String)>, Vec<Value>, bool);

/// The group's wages and deduction, then the DGE's claim where it has one.
fn worked_steps(
    (place, act): (&str, &Act),
    f: &Facts,
    members: &[usize],
    dge: Option<usize>,
    order: &[usize],
    allowed: Option<bool>,
) -> Result<Worked> {
    let claims = dge.is_some() || place == "wa";
    let ((mut steps, rates), liable) = group_steps(place, act, f, members, claims)?;
    if place == "nt" && dge.is_some() && liable && allowed != Some(true) {
        return nt_unallowed();
    }
    let deduction = steps.last().expect("group_deduction").value.clone();
    let (more, shares) = match dge {
        Some(dge) if place != "wa" => claim_steps(place, f, dge, &deduction, order)?,
        _ => (Vec::new(), Vec::new()),
    };
    steps.extend(more);
    if let Some(dge) = dge.filter(|_| place == "tas" && liable) {
        steps.extend(tas_top(f, dge, &steps)?);
    }
    Ok((steps, rates, shares, liable))
}

/// Refuses a group whose members paid wages for only part of the year.
fn part_year(args: &PayrollTaxGroupArgs, act: &Act) -> Result<()> {
    if flag(args.employed_all_year, "employed_all_year")? {
        return Ok(());
    }
    let mut why = String::from(
        "A group whose members paid wages for only part of the year is not worked: each Act scales the group's threshold by the days on which a member paid wages (C / FY)",
    );
    if !act.nil.is_empty() {
        why.push_str(&format!(", and C is not an input ({} Sch 1 cl 7)", act.name));
    }
    refuse("payroll_tax_group.part_year", why + ".")
}

pub fn payroll_tax_group(args: &PayrollTaxGroupArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let (place, act) = act(args.jurisdiction.text())?;
    part_year(args, act)?;
    let allowed = nt_allowed(place, args.annual_calculation_allowed)?;
    let (f, index, employer) = gather(args)?;
    let excluded = named(&index, args.exclusions, "exclusions")?;
    let (members, mut groups) = group_of(&f, employer, &excluded, (place, act))?;
    let stated_dges = named(
        &index,
        args.designated_group_employers,
        "designated_group_employers",
    )?;
    let dge = designated(&f, &members, &stated_dges, &mut groups)?;
    let order = excess_order(
        place,
        (&f, &index),
        args.excess_deduction_order,
        &members,
        dge,
    )?;
    if dge.is_none() && place == "sa" {
        return refuse(
            "payroll_tax_group.no_dge_unsettled",
            "A South Australian group with no designated group employer is not worked: Sch 1 of the Payroll Tax Act 2009 (SA) has no Part for one.",
        );
    }
    let (steps, rates, shares, liable) =
        precise(|| worked_steps((place, act), &f, &members, dge, &order, allowed))?;
    let ids = |list: &[usize]| {
        list.iter()
            .map(|&p| f.persons[p].id.clone())
            .collect::<Vec<_>>()
    };
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["group"] = json!(ids(&members));
    out["excluded"] = json!(ids(&excluded));
    out["designated_group_employer"] = dge.map_or(Value::Null, |p| json!(f.persons[p].id));
    out["bases"] = Value::Array(bases(&f, &groups, &members, act));
    if place == "qld" {
        out["excess_shares"] = Value::Array(shares);
    }
    if !act.nil.is_empty() {
        out["members_liable"] = json!(liable);
    }
    Ok(out)
}

/// The Queensland order of members to share an excess deduction (s 46).
fn excess_order(
    place: &str,
    (f, index): (&Facts, &Index),
    arg: Option<&Arg>,
    members: &[usize],
    dge: Option<usize>,
) -> Result<Vec<usize>> {
    let order = match arg {
        None => return Ok(Vec::new()),
        Some(Arg::Json(Value::Array(ids))) => ids
            .iter()
            .map(|id| index.of(id))
            .collect::<Result<Vec<_>>>()?,
        Some(_) => Vec::from([usize::MAX]),
    };
    let fits = place == "qld"
        && dge.is_some()
        && order
            .iter()
            .all(|&p| members.contains(&p) && Some(p) != dge && !f.persons[p].taxable.is_zero())
        && (1..order.len()).all(|i| !order[..i].contains(&order[i]));
    if !fits {
        return refuse(
            "payroll_tax_group.excess_order_invalid",
            "excess_deduction_order applies in Queensland only, with a designated group employer, and must name other members of its group with Queensland taxable wages (an entitled group member has an annual liability, s 44), each once (Payroll Tax Act 1971 (Qld) s 46).",
        );
    }
    Ok(order)
}
