// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `pcg_2021_4`: the ATO's risk zone for one individual professional
//! practitioner's (IPP's) profit allocation under PCG 2021/4, from stated
//! totals (the scorer) or from each recipient's firm income, with the tax
//! paragraph 85 counts worked out on the year's resident scale or company
//! rate (the allocation). It mirrors the PCG 2021/4 risk scorer and
//! allocation calculator on ryanduguid.github.io (`assets/pcg-2021-4.mjs`).
//! Citations are to PCG 2021/4 as updated on 19 June 2024, read on
//! 24 November 2026; docs/calculation-evidence.md#pcg-20214 lists the rest.

use serde_json::{Map, Value, json};

use super::deferred_tax::current_rate;
use super::lump_sum_arrears::{scale, scale_tax};
use super::trade_in::when;
use super::{
    Arg, Result, d, flag, int, money, precise, refuse, result, scope, signed_money, stated,
};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "pcg_2021_4";

/// The IPP and up to 5 associates, as the site's allocation form takes them.
const MAX_RECIPIENTS: usize = 6;
/// The site's bound on each amount and on the recipients' firm income together.
const LIMIT: &str = "1000000000";
const RECIPIENT_KEYS: [&str; 4] = ["kind", "firm_income", "firm_deductions", "other_income"];

/// The inputs the worksheet takes only in one mode or with factor 3, in
/// [`PcgArgs::optional`]: `entitlement`, `returned`, `tax`, `remuneration`,
/// `benchmark` and `recipients`.
pub const PCG_OPTIONAL: [&str; 6] = [
    "entitlement",
    "returned",
    "tax",
    "remuneration",
    "benchmark",
    "recipients",
];

/// The facts as the command passes them; see the README for each.
pub struct PcgArgs<'a> {
    pub gateway_1: &'a Arg,
    pub gateway_2: &'a Arg,
    pub factor_3: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 6],
}

/// Table 1 (paragraph 76): each factor's bands for scores 1 to 5, as the
/// percentage and whether the band includes it; anything lower scores 6.
const BANDS: [[(i64, bool); 5]; 3] = [
    [
        (90, false),
        (75, false),
        (60, false),
        (50, true),
        (25, false),
    ],
    [
        (40, false),
        (35, false),
        (30, true),
        (25, false),
        (20, false),
    ],
    [
        (200, false),
        (150, false),
        (100, false),
        (90, false),
        (70, false),
    ],
];

const FACTOR_1: &str = "PCG 2021/4 paragraphs 76, 82 and 83 (Table 1, risk assessment factor 1): the profit entitlement returned in the IPP's hands over the income the IPP and associated entities are entitled to from the whole of firm group";
const FACTOR_2: &str = "PCG 2021/4 paragraphs 76, 84 to 86 and footnote 6 (Table 1, risk assessment factor 2): total tax paid by the IPP and associated entities on firm income over that firm income, with no levy on taxable income";
const FACTOR_3: &str = "PCG 2021/4 paragraphs 76, 100 and 101 (Table 1, risk assessment factor 3): the IPP's remuneration over the commercial benchmark for its services";
const ZONE: &str = "PCG 2021/4 paragraphs 77 to 80 (Table 2): green to 7, amber at 8 and red from 9 on 2 factors; green to 10, amber at 11 and 12 and red from 13 on all 3";
const WHOLE: &str = "PCG 2021/4 paragraph 81: an IPP that returns 100% of the profit is low risk, with no other factor assessed";
const PARAGRAPH_85: &str = "PCG 2021/4 paragraph 85 and examples 4 and 5: the larger of (a) the tax on firm-related amounts alone and (b) the year's tax less the tax with them disregarded, on the resident scale (Income Tax Rates Act 1986 Sch 7) or the company rate (s 23(2)), with no offset or levy";

/// The score for `part` over `whole`, compared exactly with each band.
fn score(part: &Decimal, whole: &Decimal, factor: usize) -> Result<i64> {
    let scaled = part.mul(&int(100))?;
    for (i, (percent, inclusive)) in BANDS[factor].iter().enumerate() {
        let bound = whole.mul(&int(*percent))?;
        let reached = if *inclusive {
            !scaled.lt(&bound)?
        } else {
            scaled.gt(&bound)?
        };
        if reached {
            return Ok(i as i64 + 1);
        }
    }
    Ok(6)
}

/// `part` as a percentage of `whole`, to 0.01, half up.
fn percent(part: &Decimal, whole: &Decimal) -> Result<Decimal> {
    precise(|| {
        Ok(part
            .mul(&int(100))?
            .div(whole)?
            .quantize(-2, Rounding::HalfUp)?)
    })
}

/// Table 2's zone and risk level for an aggregate on 2 or 3 factors.
fn zone(aggregate: i64, factors: usize) -> (&'static str, &'static str) {
    let (green, amber) = if factors == 3 { (10, 12) } else { (7, 8) };
    if aggregate <= green {
        ("green", "low")
    } else if aggregate <= amber {
        ("amber", "moderate")
    } else {
        ("red", "high")
    }
}

/// The zone, risk level and steps, written to `out`.
fn conclude(out: &mut Map<String, Value>, (zone, level): (&str, &str), steps: Vec<Value>) {
    out.insert("zone".into(), json!(zone));
    out.insert("risk_level".into(), json!(level));
    out.insert("steps".into(), json!(steps));
}

/// An amount within the site's bound.
fn bounded(amount: Decimal, name: &str) -> Result<Decimal> {
    if amount.gt(&d(LIMIT))? {
        return refuse(
            "input.amount_invalid",
            format!("{name} must be at most 1,000,000,000."),
        );
    }
    Ok(amount)
}

fn amount(value: &Arg, name: &str) -> Result<Decimal> {
    bounded(money(value)?, name)
}

/// The totals the factors are scored on.
struct Totals {
    entitlement: Decimal,
    returned: Decimal,
    tax: Decimal,
    /// The IPP's remuneration and the benchmark, with factor 3.
    benchmark: Option<(Decimal, Decimal)>,
}

/// One factor's working, as the result shows it.
fn factor(
    id: &str,
    authority: &str,
    (part, whole): (&Decimal, &Decimal),
    figure: (&Decimal, i64),
) -> Value {
    json!({
        "id": id,
        "authority": authority,
        "working": format!("{part} / {whole} x 100 = {}%: score {}", figure.0, figure.1),
    })
}

/// Factor 3 (paragraphs 100 and 101): the share of the benchmark and its
/// score, its working added to `steps`.
fn third_factor(
    (remuneration, benchmark): &(Decimal, Decimal),
    steps: &mut Vec<Value>,
) -> Result<(Value, i64)> {
    if benchmark.is_zero() {
        return refuse(
            "pcg_2021_4.benchmark_nil",
            "benchmark must be above nil to score factor 3.",
        );
    }
    let shown = percent(remuneration, benchmark)?;
    let scored = score(remuneration, benchmark, 2)?;
    let working = (remuneration, benchmark);
    steps.push(factor("factor_3", FACTOR_3, working, (&shown, scored)));
    Ok((json!(shown.to_string()), scored))
}

/// Paragraphs 76 to 81 on the totals: the percentages, scores and zone.
fn assess(t: &Totals, out: &mut Map<String, Value>) -> Result<()> {
    let returned = percent(&t.returned, &t.entitlement)?;
    if t.returned.equals(&t.entitlement)? {
        out.insert("percentages".into(), json!({"returned": returned.to_string(), "effective_tax_rate": null, "benchmark_share": null}));
        out.insert(
            "scores".into(),
            json!({"factor_1": 1, "factor_2": null, "factor_3": null, "aggregate": null}),
        );
        let step = json!({"id": "factor_1", "authority": WHOLE, "working": "100% returned: score 1, no other factor"});
        conclude(out, ("green", "low"), vec![step]);
        return Ok(());
    }
    let first = score(&t.returned, &t.entitlement, 0)?;
    let mut steps = vec![factor(
        "factor_1",
        FACTOR_1,
        (&t.returned, &t.entitlement),
        (&returned, first),
    )];
    let rate = percent(&t.tax, &t.entitlement)?;
    let second = score(&t.tax, &t.entitlement, 1)?;
    steps.push(factor(
        "factor_2",
        FACTOR_2,
        (&t.tax, &t.entitlement),
        (&rate, second),
    ));
    let (share, third) = match &t.benchmark {
        Some(pair) => {
            let (share, scored) = third_factor(pair, &mut steps)?;
            (share, Some(scored))
        }
        None => (Value::Null, None),
    };
    let aggregate = first + second + third.unwrap_or(0);
    let factors = if third.is_some() { 3 } else { 2 };
    let rated = zone(aggregate, factors);
    let working = format!("aggregate {aggregate} on {factors} factors: {}", rated.0);
    steps.push(json!({"id": "zone", "authority": ZONE, "working": working}));
    out.insert("percentages".into(), json!({"returned": returned.to_string(), "effective_tax_rate": rate.to_string(), "benchmark_share": share}));
    out.insert(
        "scores".into(),
        json!({"factor_1": first, "factor_2": second, "factor_3": third, "aggregate": aggregate}),
    );
    conclude(out, rated, steps);
    Ok(())
}

/// The checks paragraph 85's totals must pass before they are scored.
fn checked(t: &Totals) -> Result<()> {
    if t.entitlement.is_zero() {
        return refuse(
            "pcg_2021_4.entitlement_nil",
            "The profit entitlement (the firm income of the IPP and associates) must be above nil.",
        );
    }
    if t.returned.gt(&t.entitlement)? {
        return refuse(
            "pcg_2021_4.returned_above_entitlement",
            "returned cannot exceed entitlement.",
        );
    }
    if t.tax.gt(&t.entitlement)? {
        return refuse(
            "pcg_2021_4.tax_above_entitlement",
            "tax cannot exceed entitlement, the firm income it is paid on.",
        );
    }
    Ok(())
}

/// Who receives firm income, and the rate its tax is worked at.
#[derive(Clone, Copy)]
enum Kind {
    Individual,
    Company { base_rate_entity: bool },
}

struct Recipient {
    kind: Kind,
    firm: Decimal,
    deductions: Decimal,
    other: Decimal,
}

fn bad_recipients<T>() -> Result<T> {
    refuse(
        "pcg_2021_4.recipients_invalid",
        format!(
            "recipients must be a list of 1 to {MAX_RECIPIENTS} objects, the IPP first, each with kind, firm_income, firm_deductions and other_income."
        ),
    )
}

fn kind(value: &Value, item: &str, first: bool) -> Result<Kind> {
    let kind = match value.as_str() {
        Some("individual") => Kind::Individual,
        Some("base_rate_company") => Kind::Company {
            base_rate_entity: true,
        },
        Some("company") => Kind::Company {
            base_rate_entity: false,
        },
        _ => {
            return refuse(
                "pcg_2021_4.recipient_kind_unsupported",
                format!(
                    "{item}: kind must be individual (an Australian resident aged 18 or over all year), base_rate_company or company; work out the tax of any other recipient, such as a trustee, a non-resident, a minor or a super fund, yourself and give the totals as entitlement, returned and tax."
                ),
            );
        }
    };
    if first && !matches!(kind, Kind::Individual) {
        return refuse(
            "pcg_2021_4.recipients_invalid",
            "The first recipient is the IPP, an individual.",
        );
    }
    Ok(kind)
}

/// `other_income`: other income less other deductions, which may be a loss.
fn other_income(fields: &Map<String, Value>, item: &str) -> Result<Decimal> {
    let parsed = match stated::required(fields, item, "other_income")? {
        Value::String(text) if text.len() <= 32 => Decimal::parse_input(text).ok(),
        _ => None,
    };
    let Some(Ok(other)) = parsed.map(|v| signed_money(&Arg::Decimal(v))) else {
        return refuse(
            "input.signed_amount_invalid",
            format!(
                "{item}: other_income must be a decimal string of at most 2dp, which may be negative."
            ),
        );
    };
    if other.gt(&d(LIMIT))? || other.lt(&d(LIMIT).neg()?)? {
        return refuse(
            "input.signed_amount_invalid",
            format!("{item}: other_income must be from -1,000,000,000 to 1,000,000,000."),
        );
    }
    Ok(other)
}

fn recipients(given: &Arg) -> Result<Vec<Recipient>> {
    let Arg::Json(Value::Array(items)) = given else {
        return bad_recipients();
    };
    if items.is_empty() || items.len() > MAX_RECIPIENTS {
        return bad_recipients();
    }
    let mut out = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let name = if i == 0 {
            "IPP".to_string()
        } else {
            format!("associate {i}")
        };
        let Value::Object(fields) = item else {
            return bad_recipients();
        };
        stated::known_keys(fields, &name, "a recipient", &RECIPIENT_KEYS)?;
        let kind = kind(stated::required(fields, &name, "kind")?, &name, i == 0)?;
        let firm = bounded(
            stated::amount(fields, &name, "firm_income")?,
            &format!("{name}: firm_income"),
        )?;
        let deductions = stated::amount(fields, &name, "firm_deductions")?;
        let deductions = bounded(deductions, &format!("{name}: firm_deductions"))?;
        let other = other_income(fields, &name)?;
        out.push(Recipient {
            kind,
            firm,
            deductions,
            other,
        });
    }
    Ok(out)
}

/// The tax on `income`, nil when it is not above nil, to the cent.
fn tax_on(kind: Kind, income: &Decimal, year: &str) -> Result<Decimal> {
    if !income.gt(&int(0))? {
        return Ok(int(0));
    }
    let exact = match kind {
        Kind::Individual => scale_tax(scale(year).expect("a supported year"), income, "x")?.0,
        Kind::Company { base_rate_entity } => {
            income.mul(&d(current_rate(year, base_rate_entity)))?
        }
    };
    Ok(exact.quantize(-2, Rounding::HalfUp)?)
}

/// Paragraph 85: the tax counted for one recipient and the limb that gives it.
fn counted(r: &Recipient, year: &str) -> Result<(Decimal, &'static str)> {
    let net = r.firm.sub(&r.deductions)?;
    let alone = tax_on(r.kind, &net, year)?;
    let with = tax_on(r.kind, &net.add(&r.other)?, year)?;
    let difference = with.sub(&tax_on(r.kind, &r.other, year)?)?;
    Ok(if difference.gt(&alone)? {
        (difference, "85(b)")
    } else {
        (alone, "85(a)")
    })
}

/// The allocation's totals, with each recipient's tax and limb in `out`.
fn allocate(
    list: &[Recipient],
    year: &str,
    benchmark: Option<Decimal>,
    out: &mut Map<String, Value>,
) -> Result<Totals> {
    let (mut firm, mut tax) = (int(0), int(0));
    let mut shown = Vec::new();
    for r in list {
        let (counted, rule) = precise(|| counted(r, year))?;
        firm = firm.add(&r.firm)?;
        tax = tax.add(&counted)?;
        shown.push(json!({"tax": counted.quantize(-2, Rounding::HalfUp)?.to_string(), "rule": rule, "authority": PARAGRAPH_85}));
    }
    if firm.gt(&d(LIMIT))? {
        return refuse(
            "input.amount_invalid",
            "The recipients' firm income together must be at most 1,000,000,000.",
        );
    }
    out.insert("recipients".into(), json!(shown));
    let remuneration = list[0].firm.clone();
    Ok(Totals {
        entitlement: firm,
        returned: remuneration.clone(),
        tax,
        benchmark: benchmark.map(|b| (remuneration, b)),
    })
}

/// The facts scored: the allocation's recipients, or the scorer's
/// entitlement, returned and tax.
enum Mode {
    Allocation(Vec<Recipient>),
    Scorer(Box<[Decimal; 3]>),
}

/// Factor 3's inputs: off, the benchmark (the allocation takes the IPP's
/// firm income as its remuneration), the scorer's remuneration and
/// benchmark, or the input factor 3 needs that was left out.
enum Factor3 {
    Off,
    Benchmark(Decimal),
    Scorer(Decimal, Decimal),
    Missing(&'static str),
}

/// The mode's inputs, and the benchmark with factor 3. With factor 3 the
/// benchmark (and the scorer's remuneration) may be left out only where the
/// IPP returns all of the entitlement, as paragraph 81 then scores nothing
/// else: the input left out is named, and refused once the totals show it is
/// needed.
fn mode(args: &PcgArgs, factor_3: bool) -> Result<(Mode, Factor3)> {
    let [entitlement, returned, tax, remuneration, benchmark, list] = args.optional;
    let benchmark = with_factor_3(factor_3, benchmark, "benchmark")?;
    match (list, [entitlement, returned, tax]) {
        (Some(list), [None, None, None]) => {
            let why =
                "with entitlement, returned and tax; the allocation uses the IPP's firm income";
            when(false, remuneration, "remuneration", why)?;
            let third = third(factor_3, benchmark.map(Factor3::Benchmark), "benchmark");
            Ok((Mode::Allocation(recipients(list)?), third))
        }
        (None, [Some(e), Some(r), Some(t)]) => {
            let paid = with_factor_3(factor_3, remuneration, "remuneration")?;
            let totals = [
                amount(e, "entitlement")?,
                amount(r, "returned")?,
                amount(t, "tax")?,
            ];
            let third = match (benchmark, paid) {
                (Some(b), Some(p)) => Factor3::Scorer(p, b),
                (None, _) => third(factor_3, None, "benchmark"),
                (Some(_), None) => third(factor_3, None, "remuneration"),
            };
            Ok((Mode::Scorer(Box::new(totals)), third))
        }
        (Some(_), _) => refuse(
            "input.field_unknown",
            "Give recipients or entitlement, returned and tax, not both.",
        ),
        (None, _) => refuse(
            "input.missing_field",
            "Give recipients, or all of entitlement, returned and tax.",
        ),
    }
}

/// Factor 3's inputs when given; otherwise off, or, with factor 3, `key`
/// as the input left out.
fn third(factor_3: bool, given: Option<Factor3>, key: &'static str) -> Factor3 {
    match given {
        Some(given) => given,
        None if factor_3 => Factor3::Missing(key),
        None => Factor3::Off,
    }
}

/// An input taken only with factor 3: refused without it, read with it.
fn with_factor_3(factor_3: bool, given: Option<&Arg>, key: &str) -> Result<Option<Decimal>> {
    let need = factor_3 && given.is_some();
    let given = when(need, given, key, "when factor_3 is true")?;
    given.map(|v| amount(v, key)).transpose()
}

/// The totals the mode gives, with the amounts the result shows.
fn totals<'a>(
    mode: Mode,
    year: &str,
    third: &Factor3,
    extra: &mut Map<String, Value>,
) -> Result<(Totals, Vec<(&'a str, Decimal)>)> {
    let mut amounts = Vec::new();
    let totals = match mode {
        Mode::Allocation(list) => {
            let benchmark = match third {
                Factor3::Benchmark(b) => Some(b.clone()),
                _ => None,
            };
            let t = allocate(&list, year, benchmark, extra)?;
            amounts.push(("firm_income", t.entitlement.clone()));
            amounts.push(("tax", t.tax.clone()));
            t
        }
        Mode::Scorer(given) => {
            let [entitlement, returned, tax] = *given;
            amounts.extend([
                ("entitlement", entitlement.clone()),
                ("returned", returned.clone()),
                ("tax", tax.clone()),
            ]);
            let benchmark = match third {
                Factor3::Scorer(paid, b) => {
                    amounts.push(("remuneration", paid.clone()));
                    Some((paid.clone(), b.clone()))
                }
                _ => None,
            };
            Totals {
                entitlement,
                returned,
                tax,
                benchmark,
            }
        }
    };
    if let Some((_, b)) = &totals.benchmark {
        amounts.push(("benchmark", b.clone()));
    }
    Ok((totals, amounts))
}

pub fn pcg_2021_4(args: &PcgArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    for (n, gateway) in [(1, args.gateway_1), (2, args.gateway_2)] {
        if !flag(gateway, &format!("gateway_{n}"))? {
            return refuse(
                "pcg_2021_4.gateway_failed",
                format!(
                    "The arrangement fails Gateway {n}, so PCG 2021/4 gives it no risk score (paragraphs 10 to 12)."
                ),
            );
        }
    }
    let factor_3 = flag(args.factor_3, "factor_3")?;
    let (mode, third) = mode(args, factor_3)?;
    let mut extra = Map::new();
    let (totals, amounts) = totals(mode, &year, &third, &mut extra)?;
    checked(&totals)?;
    if let Factor3::Missing(key) = third
        && !totals.returned.equals(&totals.entitlement)?
    {
        return refuse(
            "input.missing_field",
            format!(
                "{key} is required when factor_3 is true, unless the IPP returns all of the entitlement (paragraph 81)."
            ),
        );
    }
    assess(&totals, &mut extra)?;
    let mut out = result(KIND, &year, amounts, Vec::new())?;
    for (key, value) in extra {
        out[key] = value;
    }
    Ok(out)
}
