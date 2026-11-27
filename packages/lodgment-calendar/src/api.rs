// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! JSON API for the native engine. All dates use canonical YYYY-MM-DD strings.
use crate::{CHECKED, Payment, Rule, VERSION, data, fbt, holidays, returns, statements};
use serde_json::{Map, Value, json};

mod rows;

pub const COMMANDS: &[&str] = &["due-dates"];

pub fn evaluate(command: &str, value: &Value) -> Result<Value, String> {
    if command != "due-dates" {
        return Err(format!(
            "unknown JSON command: {}",
            repr(&Value::String(command.into()))
        ));
    }
    due_dates(value)
}

fn repr(v: &Value) -> String {
    match v {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::String(s) => quoted(s),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(repr).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(map) => format!(
            "{{{}}}",
            map.iter()
                .map(|(k, v)| format!("{}: {}", repr(&Value::String(k.clone())), repr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Number(number) => number_text(number),
    }
}

fn quoted(text: &str) -> String {
    let quote = if text.contains('\'') && !text.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::from(quote);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if !holidays::printable_char(c) => out.push_str(&escaped(c)),
            _ => out.push(c),
        }
    }
    out.push(quote);
    out
}

fn escaped(c: char) -> String {
    let n = c as u32;
    if n <= 0xff {
        format!("\\x{n:02x}")
    } else if n <= 0xffff {
        format!("\\u{n:04x}")
    } else {
        format!("\\U{n:08x}")
    }
}

fn number_text(number: &serde_json::Number) -> String {
    let text = number.to_string();
    if !text.contains(['.', 'e', 'E']) {
        return text;
    }
    // JSON floating literals correspond to the old Python float
    // boundary; integers retain their arbitrary-precision text.
    let Some(number) = number.as_f64() else {
        return if text.starts_with('-') {
            "-inf".into()
        } else {
            "inf".into()
        };
    };
    let shown = format!("{number:?}");
    let Some((base, exponent)) = shown.split_once('e') else {
        return shown;
    };
    let exponent: i32 = exponent.parse().expect("float exponent");
    format!("{base}e{exponent:+03}")
}

fn choice<'a>(request: &'a Value, name: &str, allowed: &[&str]) -> Result<&'a str, String> {
    request[name]
        .as_str()
        .filter(|s| allowed.contains(s))
        .ok_or_else(|| {
            format!(
                "{name} must be one of {}; got {}",
                allowed.join(", "),
                repr(&request[name])
            )
        })
}

fn due_dates(v: &Value) -> Result<Value, String> {
    let request = parse_request(v)?;
    let cal = holidays::calendar(&v["extra_holidays"], &v["holidays_complete_until"])?;
    let rows = obligation_rows(&request, &cal);
    Ok(due_document(v, &request, &rows, &cal))
}

struct Request<'a> {
    obligation: &'a str,
    entity: &'a str,
    channel: &'a str,
    year: &'a str,
    cycle: Option<&'a str>,
    facts: Value,
}

fn parse_request(v: &Value) -> Result<Request<'_>, String> {
    known_fields(v)?;
    let obligation = choice(
        v,
        "obligation",
        &["bas", "ias", "income_tax_return", "fbt_return"],
    )?;
    let entity = choice(
        v,
        "entity_type",
        &["individual", "trust", "partnership", "company", "smsf"],
    )?;
    let channel = choice(v, "channel", &["self", "tax_agent", "bas_agent"])?;
    let year = financial_year(v)?;
    check_cycle(v, obligation, channel)?;
    Ok(Request {
        obligation,
        entity,
        channel,
        year,
        cycle: v["reporting_cycle"].as_str(),
        facts: read_facts(&v["facts"])?,
    })
}

fn known_fields(v: &Value) -> Result<(), String> {
    let object = v.as_object().ok_or("request must be one JSON object")?;
    for key in object.keys() {
        if ![
            "obligation",
            "entity_type",
            "financial_year",
            "channel",
            "reporting_cycle",
            "facts",
            "extra_holidays",
            "holidays_complete_until",
        ]
        .contains(&key.as_str())
        {
            return Err(format!(
                "unknown request field: {}",
                repr(&Value::String(key.clone()))
            ));
        }
    }
    Ok(())
}

fn financial_year(v: &Value) -> Result<&str, String> {
    v["financial_year"]
        .as_str()
        .filter(|s| {
            s.len() == 7
                && s.as_bytes()[4] == b'-'
                && s.bytes()
                    .enumerate()
                    .all(|(i, c)| i == 4 || c.is_ascii_digit())
        })
        .filter(|s| {
            (s[..4].parse::<u32>().expect("digits") + 1) % 100
                == s[5..].parse::<u32>().expect("digits")
        })
        .ok_or_else(|| {
            format!(
                "financial_year must look like 2026-27; got {}",
                repr(&v["financial_year"])
            )
        })
}

fn check_cycle(v: &Value, obligation: &str, channel: &str) -> Result<(), String> {
    if ["income_tax_return", "fbt_return"].contains(&obligation) {
        if !v["reporting_cycle"].is_null() {
            return Err("reporting_cycle applies only to bas and ias".into());
        }
        if obligation == "income_tax_return" && channel == "bas_agent" {
            return Err("a BAS agent cannot lodge an income tax return; use tax_agent".into());
        }
        return Ok(());
    }
    if !matches!(v["reporting_cycle"].as_str(), Some("monthly" | "quarterly")) {
        return Err(format!(
            "reporting_cycle must be one of monthly, quarterly for {obligation}; got {}",
            repr(&v["reporting_cycle"])
        ));
    }
    Ok(())
}

fn read_facts(supplied: &Value) -> Result<Value, String> {
    if !supplied.is_null() && !supplied.is_object() {
        return Err("facts must be an object or null".into());
    }
    let names = data()["fact_names"]
        .as_array()
        .expect("recorded fact names");
    let mut facts = Map::new();
    for name in names {
        let name = name.as_str().expect("fact name");
        let value = &supplied[name];
        if !value.is_null() && !value.is_boolean() {
            return Err(format!(
                "facts.{name} must be true, false or null; got {}",
                repr(value)
            ));
        }
        facts.insert(name.into(), value.clone());
    }
    if let Some(supplied) = supplied.as_object() {
        for key in supplied.keys() {
            if !facts.contains_key(key) {
                return Err(format!(
                    "unknown fact: {}",
                    repr(&Value::String(key.clone()))
                ));
            }
        }
    }
    Ok(Value::Object(facts))
}

fn family_name(obligation: &str, entity: &str, channel: &str) -> String {
    format!(
        "{}-{}",
        match obligation {
            "income_tax_return" => entity.to_uppercase(),
            "fbt_return" => "FBT".into(),
            _ => obligation.to_uppercase(),
        },
        if channel == "self" { "SELF" } else { "AGENT" }
    )
}

fn obligation_rows(request: &Request<'_>, cal: &holidays::Calendar) -> Vec<Value> {
    let family = family_name(request.obligation, request.entity, request.channel);
    let recorded = if request.obligation == "income_tax_return" {
        "2025-26"
    } else {
        "2026-27"
    };
    if request.year != recorded {
        return unrecorded(request, recorded, cal);
    }
    match request.obligation {
        "fbt_return" => fbt_rows(request, &family, cal),
        "income_tax_return" => income_rows(request, &family, cal),
        _ => statement_rows(request, &family, cal),
    }
}

fn unrecorded(request: &Request<'_>, recorded: &str, cal: &holidays::Calendar) -> Vec<Value> {
    let year = request.year;
    let mut rule = Rule::new(
        "YEAR-NOT-RECORDED",
        format!(
            "This release records the 2026-27 lodgment program, which covers {recorded} for this obligation; {year} is not recorded."
        ),
        "about",
        None,
    )
    .unresolved(&format!(
        "rules for {year}: this release records only {recorded} for this obligation"
    ));
    rule.payment = Payment::Mode("unknown".into());
    let label = match request.obligation {
        "income_tax_return" => format!("Income tax return {year}"),
        "fbt_return" => format!("FBT return, FBT year {year}"),
        _ => format!("{} {year}", request.obligation.to_uppercase()),
    };
    rows::make_rows(
        &label,
        year,
        Some(rule),
        &[],
        "",
        &rows::Pending {
            payment: "unknown",
            source: "about",
        },
        cal,
    )
}

fn fbt_rows(request: &Request<'_>, family: &str, cal: &holidays::Calendar) -> Vec<Value> {
    let (rule, missing) = fbt::select(request.channel, &request.facts);
    rows::make_rows(
        &format!("FBT return, FBT year {}", request.year),
        request.year,
        rule,
        &missing,
        family,
        &rows::Pending {
            payment: "unknown",
            source: "fbt_agent",
        },
        cal,
    )
}

fn income_rows(request: &Request<'_>, family: &str, cal: &holidays::Calendar) -> Vec<Value> {
    let label = format!("Income tax return {}", request.year);
    let (rule, missing, concessions) =
        returns::select(request.entity, request.channel, &request.facts);
    let mut rows = rows::make_rows(
        &label,
        request.year,
        rule,
        &missing,
        family,
        &rows::Pending {
            payment: if request.entity == "partnership" {
                "none"
            } else {
                "unknown"
            },
            source: "about",
        },
        cal,
    );
    for (rule, missing) in concessions {
        rows.push(rows::row(
            &label,
            request.year,
            "concession",
            &rule,
            &missing,
            None,
            cal,
        ));
    }
    rows
}

fn statement_rows(request: &Request<'_>, family: &str, cal: &holidays::Calendar) -> Vec<Value> {
    let mut rows = vec![];
    for (period, label, rule, missing) in statements::select(
        request.obligation,
        request.entity,
        request.channel,
        request.cycle.expect("validated cycle"),
        &request.facts,
    ) {
        rows.extend(rows::make_rows(
            &label,
            &period,
            rule,
            &missing,
            family,
            &rows::Pending {
                payment: "unknown",
                source: "about",
            },
            cal,
        ));
    }
    rows
}

fn due_document(
    v: &Value,
    request: &Request<'_>,
    rows: &[Value],
    cal: &holidays::Calendar,
) -> Value {
    let determined = rows.iter().filter(|r| r["status"] == "DETERMINED").count();
    json!({"ok":true,"engine":"lodgment-calendar","engine_version":VERSION,"calendar":"lodgment_due_dates","program_year":"2026-27",
        "request":{"obligation":request.obligation,"entity_type":request.entity,"financial_year":request.year,"channel":request.channel,"reporting_cycle":v["reporting_cycle"]},
        "facts_used":request.facts,"counts":{"DETERMINED":determined,"UNKNOWN":rows.len()-determined},"obligations":rows,
        "holiday_calendar":holidays::description(cal,&v["holidays_complete_until"]),"not_decided":data()["not_decided"],"warnings":data()["warnings"],"source_checked":CHECKED})
}
