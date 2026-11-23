// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `trustee-assessment`: who is assessed on each share of a resident trust's net
//! income (ITAA 1936 ss 97, 98, 99, 99A) and the trustee's tax at the Income Tax
//! Rates Act 1986 rates. Added after the Python port.
use crate::api::{array, is_absent, text};
use crate::decimal::{Decimal, Rounding};
use crate::engine::d;
use crate::figures::{
    Result, amount, cents, fact, least, money, out_of_scope, refuse, share_out, year,
};
use serde_json::{Value, json};

const TRUST_OUT_OF_SCOPE: [(&str, &str, &str); 5] = [
    (
        "capital_gain_or_franked_distribution",
        "net income that includes a net capital gain or franked distribution is split under Division 6E",
        "Div 6E ITAA 1936; Subdivs 115-C, 207-B ITAA 1997",
    ),
    (
        "special_disability_trust",
        "a special disability trust is assessed with modifications",
        "s 95AB ITAA 1936",
    ),
    (
        "managed_investment_trust",
        "a managed investment trust or AMIT is taxed under its own rules",
        "Divs 275, 276 ITAA 1997",
    ),
    (
        "public_trading_or_corporate_unit_trust",
        "a public trading trust or corporate unit trust is taxed like a company",
        "Divs 6B, 6C ITAA 1936",
    ),
    (
        "uncontrolled_partnership_income",
        "net income derived from uncontrolled partnership income is taxed under s 94",
        "s 94 ITAA 1936",
    ),
];

const BENEFICIARY_OUT_OF_SCOPE: [(&str, &str, &str); 3] = [
    (
        "deemed_entitlement_s95a2",
        "a beneficiary deemed presently entitled through a vested and indefeasible interest is assessed under s 98(2), which is not modelled",
        "ss 95A(2), 98(2) ITAA 1936",
    ),
    (
        "farm_management_deposit_owner",
        "a beneficiary who owns a farm management deposit made in the year is treated differently",
        "s 97A ITAA 1936",
    ),
    (
        "exempt_non_resident_entity",
        "an exempt non-resident entity beneficiary is assessed under s 97(3)",
        "s 97(3) ITAA 1936",
    ),
];

/// The rate a trustee's assessment uses.
#[derive(Clone, Copy)]
enum Rate {
    /// Sch 7 Part I, with or without the tax-free threshold.
    Resident(bool),
    /// Sch 10 Part I cl 2 with the s 14 limitation.
    Section14,
    /// Sch 7 Part II.
    NonResident,
    Flat(&'static str),
}

/// Who is assessed on a share: the beneficiary, or the trustee at a rate.
struct Assessment {
    provision: &'static str,
    trustee: Option<(Rate, &'static str)>,
}

const fn to_beneficiary(provision: &'static str) -> Assessment {
    Assessment {
        provision,
        trustee: None,
    }
}
const fn to_trustee(provision: &'static str, rate: Rate, basis: &'static str) -> Assessment {
    Assessment {
        provision,
        trustee: Some((rate, basis)),
    }
}

fn stated(b: &Value, key: &str, what: &str, law: &str) -> Result<bool> {
    fact(b, key)?.ok_or_else(|| refuse(format!("{what} is not stated ({key})"), law))
}

fn individual(b: &Value, resident: bool) -> Result<Assessment> {
    if stated(
        b,
        "under_18",
        "whether the beneficiary is under 18",
        "Div 6AA ITAA 1936",
    )? {
        return Err(refuse(
            "a beneficiary under 18 at the end of the year may be taxed at Division 6AA rates",
            "Div 6AA ITAA 1936; Income Tax Rates Act 1986 s 13, Sch 12",
        ));
    }
    let law = "ss 97(1), 98(1) ITAA 1936";
    let disabled = stated(b, "under_legal_disability", "legal disability", law)?;
    Ok(match (resident, disabled) {
        (true, false) => to_beneficiary("s 97(1)(a)(i)"),
        (true, true) => to_trustee(
            "s 98(1)(a)",
            Rate::Resident(true),
            "Sch 10 Part I cl 1(a): resident scale with the tax-free threshold",
        ),
        (false, true) => to_trustee(
            "s 98(1)(b)",
            Rate::NonResident,
            "Sch 10 Part II: non-resident scale",
        ),
        (false, false) => to_trustee(
            "s 98(2A), (3)(a)",
            Rate::NonResident,
            "Sch 10 Part II: non-resident scale",
        ),
    })
}

fn company(b: &Value, resident: bool) -> Result<Assessment> {
    if resident {
        return Ok(to_beneficiary("s 97(1)(a)(i)"));
    }
    let law = "Income Tax Rates Act 1986 s 28(a)";
    Ok(
        if stated(b, "base_rate_entity", "base rate entity status", law)? {
            to_trustee(
                "s 98(2A), (3)(b)",
                Rate::Flat("0.25"),
                "Rates Act ss 28(a)(i), 23(2)(a): 25%",
            )
        } else {
            to_trustee(
                "s 98(2A), (3)(b)",
                Rate::Flat("0.30"),
                "Rates Act ss 28(a)(ii), 23(2)(b): 30%",
            )
        },
    )
}

fn assessment(
    b: &Value,
    kind: &str,
    residency: &str,
    australian: Option<bool>,
) -> Result<Assessment> {
    out_of_scope(b, &BENEFICIARY_OUT_OF_SCOPE)?;
    let resident = match residency {
        "resident" => true,
        "non_resident" if australian == Some(true) => false,
        "non_resident" => {
            return Err(refuse(
                "a non-resident beneficiary's share is limited to net income from Australian sources; state all_net_income_from_australian_sources true",
                "ss 98(1)(b), 98(2A)(d), 99A(4A)(c) ITAA 1936",
            ));
        }
        "part_year" => {
            return Err(refuse(
                "a beneficiary resident for part of the year is assessed on parts attributable to periods",
                "ss 97(1)(a), 98(1) ITAA 1936",
            ));
        }
        _ => return Err("residency must be resident, non_resident or part_year".into()),
    };
    if kind != "individual" && fact(b, "under_legal_disability")? == Some(true) {
        return Err(refuse(
            "only an individual beneficiary can be stated under a legal disability here",
            "ss 95B, 98(1) ITAA 1936",
        ));
    }
    match (kind, resident) {
        ("individual", _) => individual(b, resident),
        ("company", _) => company(b, resident),
        ("trustee", true) => Ok(to_beneficiary("s 97(1)(a)(i)")),
        ("trustee", false) => Ok(to_trustee(
            "s 98(4)",
            Rate::Flat("0.45"),
            "Rates Act s 28(b): the maximum Sch 7 Part II rate, 45%",
        )),
        _ => Err("kind must be individual, company or trustee".into()),
    }
}

/// The stated s 99A(2) kind of trust estate, if any.
fn section_99a_2_kind(v: &Value) -> Result<Option<&str>> {
    if is_absent(v, "section_99a_2_kind") {
        return Ok(None);
    }
    match text(v, "section_99a_2_kind")? {
        k @ ("will_or_intestacy" | "bankrupt_property" | "bankruptcy_part_xi" | "section_102ag_2_c") => Ok(Some(k)),
        _ => Err("section_99a_2_kind must be will_or_intestacy, bankrupt_property, bankruptcy_part_xi, section_102ag_2_c or null".into()),
    }
}

/// The assessment of the part to which no beneficiary is presently entitled.
fn trustee_portion(v: &Value, whole: bool) -> Result<Assessment> {
    let (a99a, a99) = if whole {
        ("s 99A(4)", "s 99(2)")
    } else {
        ("s 99A(4A)", "s 99(3)")
    };
    let special = to_trustee(a99a, Rate::Flat("0.45"), "Rates Act s 12(9): 45%");
    let Some(kind) = section_99a_2_kind(v)? else {
        return Ok(special);
    };
    let law = "s 99A(2) ITAA 1936";
    if !stated(
        v,
        "commissioner_opinion_99a_unreasonable",
        "the Commissioner's opinion",
        law,
    )? {
        return Ok(special);
    }
    let law = "Income Tax Rates Act 1986 Sch 10 Part I cl 1(b)";
    if kind == "will_or_intestacy"
        && stated(
            v,
            "deceased_estate_less_than_3_years",
            "whether the deceased died less than 3 years before the end of the year",
            law,
        )?
    {
        return Ok(to_trustee(
            a99,
            Rate::Resident(true),
            "Sch 10 Part I cl 1(b): resident scale with the tax-free threshold",
        ));
    }
    Ok(to_trustee(
        a99,
        Rate::Section14,
        "Sch 10 Part I cl 2: resident scale without the tax-free threshold, limited by s 14",
    ))
}

fn first_rate(fy: i64) -> Decimal {
    d(if fy == 2027 { "0.15" } else { "0.16" })
}

/// Tax at marginal rates; `bands` holds each band's lower bound and rate, ascending.
fn progressive(x: &Decimal, bands: &[(Decimal, Decimal)]) -> Result<Decimal> {
    let mut tax = d("0");
    for (i, (lower, rate)) in bands.iter().enumerate() {
        if !x.gt(lower)? {
            break;
        }
        let top = match bands.get(i + 1) {
            Some((next, _)) => least(x.clone(), next.clone())?,
            None => x.clone(),
        };
        tax = tax.add(&top.sub(lower)?.mul(rate)?)?;
    }
    Ok(tax)
}

fn resident_bands(fy: i64, threshold: bool) -> Vec<(Decimal, Decimal)> {
    let start = if threshold { "18200" } else { "0" };
    vec![
        (d(start), first_rate(fy)),
        (d("45000"), d("0.30")),
        (d("135000"), d("0.37")),
        (d("190000"), d("0.45")),
    ]
}

/// The s 14(3) net income phase-out limit: $416 x 50% / (50% - lowest Sch 7 Part I rate), rounded down.
fn phase_out(fy: i64) -> Result<Decimal> {
    let limit = d("208").div(&d("0.5").sub(&first_rate(fy))?)?;
    let whole = limit.quantize(0, Rounding::HalfUp)?;
    Ok(if whole.gt(&limit)? {
        whole.sub(&d("1"))?
    } else {
        whole
    })
}

fn tax(fy: i64, x: &Decimal, rate: Rate) -> Result<Decimal> {
    let t = match rate {
        Rate::Resident(threshold) => progressive(x, &resident_bands(fy, threshold))?,
        Rate::NonResident => progressive(
            x,
            &[
                (d("0"), d("0.30")),
                (d("135000"), d("0.37")),
                (d("190000"), d("0.45")),
            ],
        )?,
        Rate::Flat(r) => x.mul(&d(r))?,
        Rate::Section14 if !x.gt(&d("416"))? => d("0"),
        Rate::Section14 => {
            let scale = progressive(x, &resident_bands(fy, false))?;
            if x.gt(&phase_out(fy)?)? {
                scale
            } else {
                least(scale, x.sub(&d("416"))?.mul(&d("0.5"))?)?
            }
        }
    };
    cents(&t)
}

struct Row {
    name: String,
    kind: String,
    residency: String,
    entitlement: Decimal,
    assessment: Assessment,
}

fn row(b: &Value, australian: Option<bool>) -> Result<Row> {
    let kind = text(b, "kind")?;
    let residency = text(b, "residency")?;
    Ok(Row {
        name: text(b, "name")?.into(),
        assessment: assessment(b, kind, residency, australian)?,
        kind: kind.into(),
        residency: residency.into(),
        entitlement: money(b, "entitlement")?,
    })
}

/// Division 6 percentages as fractions, the trustee's last (s 95(1)).
fn weights(rows: &[Row], income: &Decimal) -> Result<Vec<Decimal>> {
    let total = rows.iter().try_fold(d("0"), |t, r| t.add(&r.entitlement))?;
    if total.gt(income)? {
        return Err(format!(
            "beneficiaries' entitlements total {total}, more than the income of the trust estate"
        ));
    }
    let mut w = Vec::with_capacity(rows.len() + 1);
    for r in rows {
        // Nil trust income: every beneficiary has 0% and the trustee 100%.
        w.push(if income.is_zero() {
            d("0")
        } else {
            r.entitlement.div(income)?
        });
    }
    let taken = w.iter().try_fold(d("0"), |t, x| t.add(x))?;
    w.push(d("1").sub(&taken)?);
    Ok(w)
}

/// One share's line and the trustee's tax on it; `None` is a nil share nobody is assessed on.
fn line(
    fy: i64,
    a: Option<&Assessment>,
    share: &Decimal,
    weight: &Decimal,
) -> Result<(Value, Decimal)> {
    let (provision, assessed, rate, t) = match a {
        None => (Value::Null, Value::Null, Value::Null, d("0")),
        Some(a) => {
            let provision = json!(format!("{} ITAA 1936", a.provision));
            match a.trustee {
                Some((rate, basis)) => (
                    provision,
                    json!("trustee"),
                    json!(basis),
                    tax(fy, share, rate)?,
                ),
                None => (provision, json!("beneficiary"), Value::Null, d("0")),
            }
        }
    };
    let pct = weight.mul(&d("100"))?.quantize(-4, Rounding::HalfUp)?;
    let l = json!({"division_6_percentage": pct.to_string(), "share_of_net_income": amount(share),
        "assessed_to": assessed, "provision": provision, "rate": rate, "trustee_tax": amount(&t)});
    Ok((l, t))
}

const STEPS: [&str; 6] = [
    "Division 6 percentage: each beneficiary's present entitlement as a share of the income of the trust estate; the trustee has the rest, or 100% when that income is nil (s 95(1)).",
    "Each share of net income is net income times the Division 6 percentage, rounded half up to the cent; cents left over go to the largest shares first.",
    "A resident beneficiary not under a legal disability, or a resident trustee beneficiary, is assessed on the share itself (s 97(1); s 95B).",
    "The trustee is assessed on the share of an adult beneficiary under a legal disability (s 98(1)), a non-resident individual or company beneficiary (s 98(2A), (3)), or a trustee beneficiary whose trustee is non-resident (s 98(4)).",
    "The part to which no beneficiary is presently entitled is taxed at 45% under s 99A, or under s 99 only where the trust is a s 99A(2) kind and the Commissioner's opinion that s 99A would be unreasonable is stated.",
    "Tax is income tax at the Income Tax Rates Act 1986 rates, rounded half up to the cent. Medicare levy, tax offsets and credits are not computed.",
];

fn resident_trust(v: &Value) -> Result<()> {
    match text(v, "trust_residency")? {
        "resident" => out_of_scope(v, &TRUST_OUT_OF_SCOPE),
        "non_resident" => Err(refuse(
            "a non-resident trust estate is not modelled",
            "ss 99(4), (5), 99A(4B), (4C) ITAA 1936",
        )),
        _ => Err("trust_residency must be resident or non_resident".into()),
    }
}

/// Each beneficiary's line, and the trustee's tax on their shares.
fn beneficiary_lines(
    fy: i64,
    rows: &[Row],
    shares: &[Decimal],
    w: &[Decimal],
) -> Result<(Vec<Value>, Decimal)> {
    let mut total = d("0");
    let mut out = Vec::with_capacity(rows.len());
    for (i, r) in rows.iter().enumerate() {
        let (mut l, t) = line(fy, Some(&r.assessment), &shares[i], &w[i])?;
        total = total.add(&t)?;
        l["name"] = json!(r.name);
        l["kind"] = json!(r.kind);
        l["residency"] = json!(r.residency);
        l["entitlement"] = amount(&r.entitlement);
        out.push(l);
    }
    Ok((out, total))
}

pub fn evaluate(v: &Value) -> Result<Value> {
    let fy = year(v)?;
    resident_trust(v)?;
    let net = money(v, "net_income")?;
    let income = money(v, "income_of_trust_estate")?;
    let australian = fact(v, "all_net_income_from_australian_sources")?;
    let rows = array(v, "beneficiaries")?
        .iter()
        .map(|b| row(b, australian))
        .collect::<Result<Vec<_>>>()?;
    let w = weights(&rows, &income)?;
    let shares = share_out(&net, &w)?;
    let n = rows.len();
    let (out, mut total) = beneficiary_lines(fy, &rows, &shares, &w)?;
    // s 99 and s 99A reach only a part no beneficiary is assessed on; a nil part needs neither.
    let portion = if shares[n].is_zero() {
        None
    } else {
        Some(trustee_portion(v, shares[n].equals(&net)?)?)
    };
    let (rest, t) = line(fy, portion.as_ref(), &shares[n], &w[n])?;
    total = total.add(&t)?;
    Ok(
        json!({"financial_year": fy, "trust_residency": "resident", "net_income": amount(&net),
        "income_of_trust_estate": amount(&income), "beneficiaries": out,
        "no_beneficiary_presently_entitled": rest, "total_trustee_tax": amount(&total),
        "not_computed": "Medicare levy, tax offsets and credits", "steps": STEPS}),
    )
}
