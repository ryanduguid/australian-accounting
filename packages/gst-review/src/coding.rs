use crate::decimal::Decimal;
use crate::{
    Result, VERSION, array, categories, codes, constants, date, field, money, optional_date,
    pytext, repr, truth,
};
use chrono::NaiveDate;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub struct Registration {
    pub registered_from: Option<NaiveDate>,
    pub registered_to: Option<NaiveDate>,
}
impl Registration {
    pub fn covers(&self, day: NaiveDate) -> bool {
        self.registered_from.is_some_and(|start| day >= start)
            && self.registered_to.is_none_or(|end| day <= end)
    }
}
struct BankLine<'a> {
    raw: &'a Value,
    id: &'a str,
    entity: &'a str,
    date: NaiveDate,
    amount: Decimal,
}

pub fn review(input: &Value) -> Result<Value> {
    crate::decimal::with_precision(40, || review_inner(input))
}
fn review_inner(input: &Value) -> Result<Value> {
    let window_value = input.get("match_window_days").cloned().unwrap_or(json!(3));
    let window = window_value
        .as_i64()
        .filter(|n| (0..=31).contains(n))
        .ok_or("match_window_days must be an integer from 0 to 31.")?;
    let codes = codes(input, "gst_codes", "coding", "DEFAULT_GST_CODES")?;
    categories(
        codes,
        &["taxable", "gst_free", "input_taxed", "outside_gst"],
        "GST treatments",
    )?;
    let registrations = input
        .get("registrations")
        .and_then(Value::as_object)
        .ok_or("registrations must be an object")?;
    let mut registered = BTreeMap::new();
    for (entity, r) in registrations {
        if !r.is_object() || r.get("registered_from").is_none() {
            return Err(format!(
                "Registration {} must be an object with registered_from.",
                pytext::repr(entity)
            ));
        }
        let start = field(r, "registered_from");
        let end = field(r, "registered_to");
        if !optional_date(start) || !optional_date(end) {
            return Err(format!(
                "Registration dates for {} must be dates or None.",
                pytext::repr(entity)
            ));
        }
        let start = date(start);
        let end = date(end);
        if end.is_some_and(|end| start.is_none_or(|start| end < start)) {
            return Err(format!(
                "Registration for {} ends before it starts.",
                pytext::repr(entity)
            ));
        }
        registered.insert(
            entity.as_str(),
            Registration {
                registered_from: start,
                registered_to: end,
            },
        );
    }
    let mut lines = Vec::new();
    let mut seen = BTreeSet::new();
    for raw in array(input, "lines")? {
        let id = field(raw, "id");
        let Some(id_text) = id.as_str().filter(|s| !s.is_empty() && seen.insert(*s)) else {
            return Err(format!(
                "Line ids must be unique non-empty strings: {}.",
                repr(id)
            ));
        };
        let entity = field(raw, "entity");
        let entity_text = entity
            .as_str()
            .filter(|s| registered.contains_key(s))
            .ok_or_else(|| format!("No registration facts for entity {}.", repr(entity)))?;
        let day =
            date(field(raw, "date")).ok_or_else(|| format!("Line {} needs a date.", repr(id)))?;
        let amount = money(field(raw, "amount")).ok_or_else(|| {
            format!(
                "Line {} amount must be a finite Decimal, at most 2 decimal places and 1e12.",
                repr(id)
            )
        })?;
        let confirmed = raw.get("confirmed").unwrap_or(&Value::Bool(false));
        let supplier = field(raw, "supplier_registered");
        if !confirmed.is_boolean() || !(supplier.is_boolean() || supplier.is_null()) {
            return Err(format!("Line {} flags must be booleans.", repr(id)));
        }
        if confirmed == &Value::Bool(true)
            && !(truth(field(raw, "account")) && truth(field(raw, "gst_code")))
        {
            return Err(format!(
                "Line {} is confirmed without an account and GST code.",
                repr(id)
            ));
        }
        lines.push(BankLine {
            raw,
            id: id_text,
            entity: entity_text,
            date: day,
            amount,
        });
    }
    let counterparts = matching(&lines, window)?;
    let states = constants()["coding"]["STATES"].as_array().unwrap();
    let mut counts: Map<String, Value> = states
        .iter()
        .map(|s| (s.as_str().unwrap().into(), json!(0)))
        .collect();
    let mut reviewed = Vec::new();
    for line in &lines {
        let counterpart = counterparts.get(line.id);
        let (state, reason, sections) = state(line, &registered[line.entity], codes, counterpart)?;
        let count = counts[state].as_u64().unwrap();
        counts.insert(state.into(), json!(count + 1));
        reviewed.push(json!({"id":line.id,"entity":line.entity,"date":line.date.to_string(),"amount":line.amount.to_sci_string(),"state":state,"reason":reason,"counterpart":counterpart,"sections":sections}));
    }
    let entities: BTreeSet<_> = lines.iter().map(|l| l.entity).collect();
    let registrations:Map<String,Value>=entities.into_iter().map(|entity|{
        let r=&registered[entity];
        (entity.into(),json!({"registered_from":r.registered_from.map(|d|d.to_string()),"registered_to":r.registered_to.map(|d|d.to_string())}))
    }).collect();
    Ok(
        json!({"ok":true,"engine":"gst-review","engine_version":VERSION,"review":"gst_coding","lines":reviewed,"counts":counts,
        "facts_used":{"registrations":registrations,"match_window_days":window,"gst_codes":codes},
        "not_decided":constants()["coding"]["NOT_DECIDED"],"sources":[constants()["coding"]["GST_ACT"]],"source_checked":constants()["coding"]["SOURCE_CHECKED"],
        "warnings":["Registration dates, coding and supplier status are the caller's facts and are not independently verified.","Review aid only, not advice, an assessment or a lodgment. Obtain human review."]}),
    )
}
fn matching(lines: &[BankLine<'_>], window: i64) -> Result<BTreeMap<String, String>> {
    let mut by_amount: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, line) in lines.iter().enumerate() {
        by_amount
            .entry(line.amount.fixed(2, false))
            .or_default()
            .push(i);
    }
    let mut matched = BTreeMap::new();
    for line in lines {
        if matched.contains_key(line.id) || line.amount.is_zero() {
            continue;
        }
        let key = line
            .amount
            .neg()
            .map_err(|e| e.name().to_string())?
            .fixed(2, false);
        let mut nearest = None;
        for &i in by_amount.get(&key).into_iter().flatten() {
            let other = &lines[i];
            let gap = (other.date - line.date).num_days().abs();
            if !matched.contains_key(other.id)
                && other.entity != line.entity
                && gap <= window
                && nearest.is_none_or(|(_, distance)| gap < distance)
            {
                nearest = Some((i, gap));
            }
        }
        if let Some((i, _)) = nearest {
            matched.insert(line.id.into(), lines[i].id.into());
            matched.insert(lines[i].id.into(), line.id.into());
        }
    }
    Ok(matched)
}
fn state(
    line: &BankLine<'_>,
    registration: &Registration,
    codes: &Map<String, Value>,
    counterpart: Option<&String>,
) -> Result<(&'static str, String, Vec<&'static str>)> {
    let stored = field(line.raw, "gst_code");
    let code = if stored.is_null() {
        field(line.raw, "suggested_gst_code")
    } else {
        stored
    };
    if !code.is_null() && !code.as_str().is_some_and(|s| codes.contains_key(s)) {
        return Ok((
            "unknown_gst_code",
            format!(
                "GST code {} is not in the code table, so its treatment is unknown.",
                repr(code)
            ),
            vec![],
        ));
    }
    if code.as_str().is_some_and(|s| codes[s] == "taxable") {
        if !registration.covers(line.date) {
            return Ok((
                "gst_differs_from_registration",
                format!(
                    "Coded {}, which charges or claims GST, but {} is not registered for GST on {}. A taxable supply and a creditable acquisition each need the entity registered or required to be registered.",
                    repr(code),
                    line.entity,
                    line.date
                ),
                vec!["GST Act s 9-5(d)", "GST Act s 11-5(d)"],
            ));
        }
        if line
            .amount
            .lt(&Decimal::from_int(0))
            .map_err(|e| e.name().to_string())?
            && field(line.raw, "supplier_registered") == &Value::Bool(false)
        {
            return Ok((
                "gst_differs_from_registration",
                format!(
                    "Coded {}, which claims GST, but the supplier is recorded as not registered. A supply by an entity that is neither registered nor required to be registered is not a taxable supply, so no input tax credit arises.",
                    repr(code)
                ),
                vec!["GST Act s 9-5(d)", "GST Act s 11-5(b)"],
            ));
        }
    }
    if let Some(counterpart) = counterpart {
        return Ok((
            "between_entities",
            format!(
                "Opposite amount on line {} of another entity within the match window: likely a transfer between entities, not income or expense.",
                pytext::repr(counterpart)
            ),
            vec![],
        ));
    }
    if field(line.raw, "confirmed") == &Value::Bool(true) {
        return Ok(("coded", "Coding confirmed.".into(), vec![]));
    }
    if [
        "account",
        "gst_code",
        "suggested_account",
        "suggested_gst_code",
    ]
    .iter()
    .any(|key| truth(field(line.raw, key)))
    {
        return Ok((
            "suggested",
            "Coding proposed but not confirmed.".into(),
            vec![],
        ));
    }
    Ok((
        "nothing_suggested",
        "No account or GST code proposed.".into(),
        vec![],
    ))
}
