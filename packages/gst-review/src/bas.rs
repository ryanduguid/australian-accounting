//! BAS label attribution and whole-dollar comparison of supplied ledger facts.
use crate::decimal::{Decimal, Rounding};
use crate::{
    Result, VERSION, array, categories, codes, constants, date, field, money, optional_date,
    pytext, repr,
};
use chrono::NaiveDate;
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

const LABELS: [&str; 7] = ["G1", "G2", "G3", "G10", "G11", "1A", "1B"];

pub struct Period {
    pub label: String,
    pub start: NaiveDate,
    pub end: NaiveDate,
}
impl Period {
    pub fn holds(&self, day: NaiveDate) -> bool {
        self.start <= day && day <= self.end
    }
}
struct Line<'a> {
    id: &'a str,
    side: &'a str,
    code: &'a str,
    gross: Decimal,
    gst: Decimal,
    invoice: NaiveDate,
    payment: Option<NaiveDate>,
}
struct Tally {
    totals: [Decimal; 7],
    contributors: [Vec<String>; 7],
    attributed: bool,
    kept_out: Vec<String>,
    timing: Vec<String>,
}
impl Tally {
    fn new() -> Self {
        Self {
            totals: std::array::from_fn(|_| Decimal::from_int(0)),
            contributors: std::array::from_fn(|_| Vec::new()),
            attributed: false,
            kept_out: Vec::new(),
            timing: Vec::new(),
        }
    }
}
fn arithmetic<T>(result: std::result::Result<T, crate::decimal::Signal>) -> Result<T> {
    result.map_err(|e| e.name().to_string())
}

pub fn reconcile(input: &Value) -> Result<Value> {
    crate::decimal::with_precision(40, || reconcile_inner(input))
}
fn reconcile_inner(input: &Value) -> Result<Value> {
    let basis = field(input, "basis")
        .as_str()
        .filter(|s| ["cash", "accruals"].contains(s))
        .ok_or("basis must be 'cash' or 'accruals'.")?;
    let includes = input
        .get("g1_includes_gst")
        .unwrap_or(&Value::Bool(true))
        .as_bool()
        .ok_or("g1_includes_gst must be a boolean.")?;
    let codes = codes(input, "bas_codes", "bas", "DEFAULT_BAS_CODES")?;
    categories(
        codes,
        &[
            "taxable",
            "capital",
            "gst_free",
            "capital_gst_free",
            "export",
            "input_taxed",
            "excluded",
        ],
        "BAS categories",
    )?;
    let raw_periods = array(input, "periods")?;
    let mut seen = BTreeSet::new();
    for raw in raw_periods {
        if !field(raw, "label")
            .as_str()
            .is_some_and(|s| !s.is_empty() && seen.insert(s))
        {
            return Err("Period labels must be unique non-empty strings.".into());
        }
    }
    let mut periods = Vec::new();
    for raw in raw_periods {
        let label = raw["label"].as_str().unwrap();
        let dates = date(field(raw, "start")).zip(date(field(raw, "end")));
        let (start, end) = dates.filter(|(start, end)| start <= end).ok_or_else(|| {
            format!(
                "Period {} needs a start on or before its end.",
                pytext::repr(label)
            )
        })?;
        periods.push(Period {
            label: label.into(),
            start,
            end,
        });
    }
    let mut ordered: Vec<_> = periods.iter().collect();
    ordered.sort_by_key(|p| p.start);
    for pair in ordered.windows(2) {
        if pair[0].end >= pair[1].start {
            return Err(format!(
                "Period {} overlaps {}.",
                pytext::repr(&pair[1].label),
                pytext::repr(&pair[0].label)
            ));
        }
    }
    let lodged = input
        .get("lodged")
        .and_then(Value::as_object)
        .ok_or("lodged must be an object")?;
    for (label, figures) in lodged {
        if !seen.contains(label.as_str()) {
            return Err(format!(
                "Lodged figures for unknown period {}.",
                pytext::repr(label)
            ));
        }
        let figures = figures
            .as_object()
            .ok_or("lodged period figures must be an object")?;
        for (key, value) in figures {
            if !LABELS.contains(&key.as_str()) {
                return Err(format!(
                    "Unsupported BAS label {} for {}.",
                    pytext::repr(key),
                    pytext::repr(label)
                ));
            }
            if !money(value).is_some_and(|d| {
                !d.lt(&Decimal::from_int(0)).unwrap_or(true)
                    && d.to_integral_value()
                        .and_then(|whole| d.equals(&whole))
                        .unwrap_or(false)
            }) {
                return Err(format!(
                    "Lodged {key} for {} must be whole non-negative dollars as a Decimal.",
                    pytext::repr(label)
                ));
            }
        }
    }
    let mut seen = BTreeSet::new();
    let mut lines = Vec::new();
    for raw in array(input, "lines")? {
        let id = field(raw, "id");
        let Some(id_text) = id.as_str().filter(|s| !s.is_empty() && seen.insert(*s)) else {
            return Err(format!(
                "Line ids must be unique non-empty strings: {}.",
                repr(id)
            ));
        };
        let side = field(raw, "side").as_str();
        let code = field(raw, "gst_code").as_str();
        let (Some(side), Some(code)) = (side.filter(|s| ["sale", "purchase"].contains(s)), code)
        else {
            return Err(format!(
                "Line {} needs a side of 'sale' or 'purchase' and a GST code.",
                repr(id)
            ));
        };
        let invoice = date(field(raw, "invoice_date"));
        let payment = field(raw, "payment_date");
        if invoice.is_none() || !optional_date(payment) {
            return Err(format!(
                "Line {} needs an invoice date and an optional payment date.",
                repr(id)
            ));
        }
        let (Some(gross), Some(gst)) = (money(field(raw, "gross")), money(field(raw, "gst")))
        else {
            return Err(format!(
                "Line {} amounts must be finite Decimals, at most 2 decimal places and 1e12.",
                repr(id)
            ));
        };
        if arithmetic(arithmetic(gst.abs())?.gt(&arithmetic(gross.abs())?))?
            || arithmetic(arithmetic(gst.mul(&gross))?.lt(&Decimal::from_int(0)))?
        {
            return Err(format!(
                "Line {} GST must share the sign of the gross and not exceed it.",
                repr(id)
            ));
        }
        lines.push(Line {
            id: id_text,
            side,
            code,
            gross,
            gst,
            invoice: invoice.unwrap(),
            payment: date(payment),
        });
    }
    let mut tally: Vec<_> = periods.iter().map(|_| Tally::new()).collect();
    let mut unpaid = Vec::new();
    let mut outside = Vec::new();
    let mut unplaced = Vec::new();
    let period_of =
        |day: Option<NaiveDate>| day.and_then(|d| periods.iter().position(|p| p.holds(d)));
    for line in &lines {
        let day = if basis == "cash" {
            line.payment
        } else {
            Some(line.payment.map_or(line.invoice, |p| p.min(line.invoice)))
        };
        let index = period_of(day);
        let invoiced = period_of(Some(line.invoice));
        if let Some(i) = invoiced.filter(|i| Some(*i) != index) {
            tally[i].timing.push(line.id.into());
        }
        if day.is_none() {
            unpaid.push(line.id);
            continue;
        }
        let Some(index) = index else {
            outside.push(line.id);
            continue;
        };
        let row = &mut tally[index];
        row.attributed = true;
        let Some(category) = codes.get(line.code).and_then(Value::as_str) else {
            unplaced.push(format!(
                "{}: GST code {} is not in the code table.",
                pytext::repr(line.id),
                pytext::repr(line.code)
            ));
            continue;
        };
        let Some(mut reached) = reached(line.side, category) else {
            unplaced.push(format!(
                "{}: GST code {} ({category}) cannot be placed on a {}.",
                pytext::repr(line.id),
                pytext::repr(line.code),
                line.side
            ));
            continue;
        };
        let carries_gst = ["taxable", "capital"].contains(&category);
        if !line.gst.is_zero() && !carries_gst {
            unplaced.push(format!(
                "{}: GST of {} on code {}, which carries no GST.",
                pytext::repr(line.id),
                line.gst.to_sci_string(),
                pytext::repr(line.code)
            ));
            continue;
        }
        if carries_gst {
            reached.push(if line.side == "sale" { 5 } else { 6 });
        }
        if reached.is_empty() {
            row.kept_out.push(line.id.into());
        }
        let amount = if includes {
            line.gross.clone()
        } else {
            arithmetic(line.gross.sub(&line.gst))?
        };
        for i in reached {
            row.totals[i] =
                arithmetic(row.totals[i].add(if i >= 5 { &line.gst } else { &amount }))?;
            row.contributors[i].push(line.id.into());
        }
    }
    if !unplaced.is_empty() {
        return Err(format!(
            "Lines that cannot be placed on a BAS label: {}",
            unplaced.join("; ")
        ));
    }
    let mut output = Vec::new();
    let mut not_lodged = Vec::new();
    let mut no_lines = Vec::new();
    for (period, row) in periods.iter().zip(tally) {
        let figures = lodged.get(&period.label).and_then(Value::as_object);
        if row.attributed && figures.is_none() {
            not_lodged.push(&period.label);
        }
        if !row.attributed && figures.is_some() {
            no_lines.push(&period.label);
        }
        let mut labels = Map::new();
        for (i, label) in LABELS.iter().enumerate() {
            let value = figures.and_then(|f| f.get(*label)).and_then(money);
            labels.insert(
                (*label).into(),
                compare(&row.totals[i], value.as_ref(), &row.contributors[i])?,
            );
        }
        output.push(json!({"period":period.label,"start":period.start.to_string(),"end":period.end.to_string(),"lodged":figures.is_some(),"labels":labels,"kept_out":row.kept_out,"timing":row.timing}));
    }
    Ok(
        json!({"ok":true,"engine":"gst-review","engine_version":VERSION,"review":"bas_reconciliation","basis":basis,
        "periods":output,"not_lodged":not_lodged,"no_ledger_lines":no_lines,"outside_periods":outside,"unpaid":unpaid,
        "facts_used":{"basis":basis,"g1_includes_gst":includes,"bas_codes":codes,"rounding":"Ledger totals are rounded down to whole dollars before comparison."},
        "not_decided":constants()["bas"]["NOT_DECIDED"],"sources":[constants()["coding"]["GST_ACT"],constants()["bas"]["ATO_BAS_METHOD"],constants()["bas"]["ATO_BAS_INSTRUCTIONS"]],
        "source_checked":constants()["coding"]["SOURCE_CHECKED"],
        "warnings":["Ledger lines, codes, periods and lodged figures are the caller's facts and are not independently verified.","Review aid only, not advice, an assessment or a lodgment. Obtain human review."]}),
    )
}
fn reached(side: &str, category: &str) -> Option<Vec<usize>> {
    Some(match (side, category) {
        ("sale", "taxable" | "input_taxed") => vec![0],
        ("sale", "gst_free") => vec![0, 2],
        ("sale", "export") => vec![0, 1],
        ("purchase", "taxable" | "gst_free" | "input_taxed") => vec![4],
        ("purchase", "capital" | "capital_gst_free") => vec![3],
        (_, "excluded") => vec![],
        _ => return None,
    })
}
fn compare(ledger: &Decimal, lodged: Option<&Decimal>, lines: &[String]) -> Result<Value> {
    let whole = arithmetic(ledger.to_integral_rounded(Rounding::Down))?;
    let mut row = json!({"ledger":ledger.to_sci_string(),"ledger_whole_dollars":whole.to_sci_string(),"lodged":lodged.map(Decimal::to_sci_string),"difference":null,"lines":[]});
    let status = if arithmetic(ledger.lt(&Decimal::from_int(0)))? {
        row["lines"] = json!(lines);
        "negative_ledger"
    } else if let Some(lodged) = lodged {
        if arithmetic(whole.equals(lodged))? {
            "matches"
        } else {
            row["lines"] = json!(lines);
            row["difference"] = json!(arithmetic(whole.sub(lodged))?.to_sci_string());
            "differs"
        }
    } else {
        "not_reported"
    };
    row["status"] = json!(status);
    Ok(row)
}
