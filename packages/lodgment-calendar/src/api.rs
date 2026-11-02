//! JSON API for the native engine. All dates use canonical YYYY-MM-DD strings.
use crate::{CHECKED, Payment, Rule, VERSION, data, fbt, holidays, returns, statements};
use serde_json::{Map, Value, json};

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
        Value::String(s) => {
            let quote = if s.contains('\'') && !s.contains('"') {
                '"'
            } else {
                '\''
            };
            let mut out = String::from(quote);
            for c in s.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c == quote => {
                        out.push('\\');
                        out.push(c);
                    }
                    c if !holidays::printable_char(c) => {
                        let n = c as u32;
                        out.push_str(&if n <= 0xff {
                            format!("\\x{n:02x}")
                        } else if n <= 0xffff {
                            format!("\\u{n:04x}")
                        } else {
                            format!("\\U{n:08x}")
                        });
                    }
                    _ => out.push(c),
                }
            }
            out.push(quote);
            out
        }
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
        Value::Number(number) => {
            let text = number.to_string();
            if text.contains(['.', 'e', 'E']) {
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
                if let Some((base, exponent)) = shown.split_once('e') {
                    let exponent: i32 = exponent.parse().expect("float exponent");
                    format!("{base}e{exponent:+03}")
                } else {
                    shown
                }
            } else {
                text
            }
        }
    }
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
    let year = v["financial_year"]
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
        })?;
    let cycle = v["reporting_cycle"].as_str();
    if ["income_tax_return", "fbt_return"].contains(&obligation) {
        if !v["reporting_cycle"].is_null() {
            return Err("reporting_cycle applies only to bas and ias".into());
        }
        if obligation == "income_tax_return" && channel == "bas_agent" {
            return Err("a BAS agent cannot lodge an income tax return; use tax_agent".into());
        }
    } else if !matches!(cycle, Some("monthly" | "quarterly")) {
        return Err(format!(
            "reporting_cycle must be one of monthly, quarterly for {obligation}; got {}",
            repr(&v["reporting_cycle"])
        ));
    }
    let supplied = &v["facts"];
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
    let facts = Value::Object(facts);
    let cal = holidays::calendar(&v["extra_holidays"], &v["holidays_complete_until"])?;
    let family = format!(
        "{}-{}",
        match obligation {
            "income_tax_return" => entity.to_uppercase(),
            "fbt_return" => "FBT".into(),
            _ => obligation.to_uppercase(),
        },
        if channel == "self" { "SELF" } else { "AGENT" }
    );
    let recorded = if obligation == "income_tax_return" {
        "2025-26"
    } else {
        "2026-27"
    };
    let mut rows = vec![];
    if year != recorded {
        let mut rule=Rule::new("YEAR-NOT-RECORDED",format!("This release records the 2026-27 lodgment program, which covers {recorded} for this obligation; {year} is not recorded."),"about",None)
            .unresolved(&format!("rules for {year}: this release records only {recorded} for this obligation"));
        rule.payment = Payment::Mode("unknown".into());
        let label = match obligation {
            "income_tax_return" => format!("Income tax return {year}"),
            "fbt_return" => format!("FBT return, FBT year {year}"),
            _ => format!("{} {year}", obligation.to_uppercase()),
        };
        rows.extend(make_rows(
            &label,
            year,
            Some(rule),
            &[],
            "",
            ("unknown", "about"),
            &cal,
        )?);
    } else if obligation == "fbt_return" {
        let (rule, missing) = fbt::select(channel, &facts);
        rows.extend(make_rows(
            &format!("FBT return, FBT year {year}"),
            year,
            rule,
            &missing,
            &family,
            ("unknown", "fbt_agent"),
            &cal,
        )?);
    } else if obligation == "income_tax_return" {
        let label = format!("Income tax return {year}");
        let (rule, missing, concessions) = returns::select(entity, channel, &facts);
        rows.extend(make_rows(
            &label,
            year,
            rule,
            &missing,
            &family,
            (
                if entity == "partnership" {
                    "none"
                } else {
                    "unknown"
                },
                "about",
            ),
            &cal,
        )?);
        for (rule, missing) in concessions {
            rows.push(row(
                &label,
                year,
                "concession",
                &rule,
                &missing,
                None,
                &cal,
            )?);
        }
    } else {
        for (period, label, rule, missing) in statements::select(
            obligation,
            entity,
            channel,
            cycle.expect("validated cycle"),
            &facts,
        ) {
            rows.extend(make_rows(
                &label,
                &period,
                rule,
                &missing,
                &family,
                ("unknown", "about"),
                &cal,
            )?);
        }
    }
    let determined = rows.iter().filter(|r| r["status"] == "DETERMINED").count();
    Ok(
        json!({"ok":true,"engine":"lodgment-calendar","engine_version":VERSION,"calendar":"lodgment_due_dates","program_year":"2026-27",
        "request":{"obligation":obligation,"entity_type":entity,"financial_year":year,"channel":channel,"reporting_cycle":v["reporting_cycle"]},
        "facts_used":facts,"counts":{"DETERMINED":determined,"UNKNOWN":rows.len()-determined},"obligations":rows,
        "holiday_calendar":holidays::description(&cal,&v["holidays_complete_until"]),"not_decided":data()["not_decided"],"warnings":data()["warnings"],"source_checked":CHECKED}),
    )
}

fn make_rows(
    label: &str,
    period: &str,
    rule: Option<Rule>,
    missing: &[String],
    family: &str,
    (pending_payment, pending_source): (&str, &str),
    cal: &holidays::Calendar,
) -> Result<Vec<Value>, String> {
    if rule.is_none() || !missing.is_empty() {
        let meanings: Vec<_> = missing
            .iter()
            .map(|n| data()["fact_meanings"][n].as_str().expect("fact meaning"))
            .collect();
        let pending = Rule::new(
            family,
            format!(
                "The date depends on facts not supplied: {}.",
                meanings.join("; ")
            ),
            pending_source,
            None,
        );
        return Ok(vec![row(
            label,
            period,
            "lodgment",
            &pending,
            missing,
            Some(pending_payment),
            cal,
        )?]);
    }
    let rule = rule.expect("present rule");
    let payment = match &rule.payment {
        Payment::Mode(mode) => mode.as_str(),
        Payment::Separate(_) => "separate",
    };
    let mut result = vec![row(
        label,
        period,
        "lodgment",
        &rule,
        &[],
        Some(payment),
        cal,
    )?];
    if let Payment::Separate(payment) = &rule.payment {
        result.push(row(label, period, "payment", payment, &[], None, cal)?);
    }
    Ok(result)
}

fn row(
    label: &str,
    period: &str,
    kind: &str,
    rule: &Rule,
    missing: &[String],
    payment: Option<&str>,
    cal: &holidays::Calendar,
) -> Result<Value, String> {
    let source = &data()["sources"][&rule.source];
    let mut needed: Vec<String> = missing
        .iter()
        .map(|n| {
            format!(
                "{n}: {}",
                data()["fact_meanings"][n].as_str().expect("fact meaning")
            )
        })
        .collect();
    let mut due = None;
    let mut holiday_roll = Value::Null;
    if let Some(unresolved) = &rule.unresolved {
        needed.push(unresolved.clone());
    } else if let Some(nominal) = rule.nominal.filter(|_| needed.is_empty()) {
        let (rolled, skipped, question) = holidays::roll_day(nominal, cal)?;
        due = rolled;
        if !skipped.is_empty() || due.is_none() {
            holiday_roll = json!({"skipped":skipped,"source_url":data()["holiday_source"]});
        }
        if let Some(question) = question {
            needed.push(question);
        } else if kind == "concession"
            && holiday_roll["skipped"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        {
            needed.push(format!("whether the next-business-day rule extends this concession: {nominal} is not a business day, and the ATO pages call it both a concessional due date and not a lodgment due date"));
            due = None;
        }
    }
    if !needed.is_empty() {
        due = None;
    }
    let mut result = json!({"obligation":label,"period":period,"kind":kind,"status":if needed.is_empty() {"DETERMINED"} else {"UNKNOWN"},
        "due_date":due.map(|d|d.to_string()),"nominal_due_date":rule.nominal.map(|d|d.to_string()),"rule_id":rule.id,"summary":rule.summary,
        "source_url":source[0],"source_provision":source[1],"source_checked":CHECKED,"holiday_roll":holiday_roll,"missing":needed});
    if kind == "lodgment" {
        result["payment"] = json!(payment);
    }
    Ok(result)
}
