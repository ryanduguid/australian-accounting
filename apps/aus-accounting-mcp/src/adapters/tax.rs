// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use serde_json::{Value, json};

const MONEY: &[&str] = &[
    "amount",
    "taxable_income",
    "other_gains",
    "discount_gains",
    "current_losses",
    "prior_losses",
    "type_one_value",
    "type_two_value",
    "cost",
    "ordinary_time_earnings",
    "qualifying_contributions",
    "earnings",
    "total_super_balance",
    "concessional_contributions",
    "unused_concessional_cap",
    "non_concessional_contributions",
    "account_balance",
    "repayment_income",
    "division_293_income",
    "low_tax_contributions",
];

/// The linked engine's scope words for periods the input schema refuses (b4a70733 and
/// d8f34dc2), each with the published 0.2.0 words for the periods it admits. Remove an
/// entry when the schema opens its periods.
const NARROWED_SCOPE: [(&str, &str, &str); 3] = [
    (
        "payg_withholding",
        "made from 1 July 2024, in the income year given, to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations. The year selects the Schedule 1 coefficients for payments made in it.",
        "made from 1 July 2026 to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established from the payee's declarations.",
    ),
    (
        "fbt",
        "for the year ended 31 March 2026 or 2027, both at the 47% FBT rate and the 2.0802 and 1.8868 gross-up rates. Excludes",
        "for the year ended 31 March 2026. Excludes",
    ),
    (
        "fbt",
        "before any optional rounding down to 5 cents. For 2027 the items follow the FBT return 2026 instructions until the ATO publishes the 2027 return.",
        "before any optional rounding down to 5 cents.",
    ),
];

/// A worksheet's scope as this server serves it: no wider than the periods its input
/// schema admits.
fn served_scope(kind: &str, record: &mut Value) {
    if let Some(text) = record["scope"].as_str() {
        let narrowed = NARROWED_SCOPE
            .iter()
            .filter(|(narrowed, ..)| *narrowed == kind)
            .fold(text.to_owned(), |text, (_, wide, admitted)| {
                text.replace(wide, admitted)
            });
        record["scope"] = json!(narrowed);
    }
}

/// The engine's worksheets and periods this server's input schema can reach. The
/// linked engine can hold kinds, and periods outside a year the schema fixes or bounds,
/// that the published contract does not advertise yet.
pub fn advertised_worksheets() -> Result<Value, String> {
    let mut catalogue =
        australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({}))?;
    let spec =
        crate::contracts::tool("calculate_tax_worksheet").ok_or("calculate_tax_worksheet")?;
    let schema = &spec["inputSchema"];
    let advertised = &schema["properties"]["facts"]["discriminator"]["mapping"];
    let kinds = catalogue.as_object_mut().ok_or("worksheet catalogue")?;
    kinds.retain(|kind, _| advertised.get(kind).is_some());
    for (kind, entry) in kinds {
        let reference = advertised[kind.as_str()].as_str().ok_or("facts mapping")?;
        let variant = reference
            .strip_prefix('#')
            .and_then(|pointer| schema.pointer(pointer))
            .ok_or("facts variant")?;
        served_scope(kind, entry);
        if let Some(periods) = entry["supported_periods"].as_array_mut() {
            periods.retain(|period| {
                period["arguments"].as_object().is_none_or(|arguments| {
                    arguments
                        .iter()
                        .all(|(name, value)| admits(&variant["properties"][name], value))
                })
            });
        }
    }
    Ok(catalogue)
}

/// Whether a published facts property admits a period argument, by the keywords those
/// properties use: const, enum, minimum and maximum.
fn admits(property: &Value, value: &Value) -> bool {
    let number = value.as_f64();
    let bound = |key: &str, holds: fn(f64, f64) -> bool| {
        property[key]
            .as_f64()
            .is_none_or(|limit| number.is_some_and(|number| holds(number, limit)))
    };
    property.get("const").is_none_or(|fixed| fixed == value)
        && property["enum"]
            .as_array()
            .is_none_or(|allowed| allowed.contains(value))
        && bound("minimum", |number, limit| number >= limit)
        && bound("maximum", |number, limit| number <= limit)
}

pub fn calculate(input: &Value) -> Result<Value, String> {
    let mut facts = input["facts"].clone();
    let object = facts.as_object_mut().unwrap();
    let kind = object
        .remove("kind")
        .unwrap()
        .as_str()
        .unwrap()
        .replace('_', "-");
    for (field, value) in object {
        if MONEY.contains(&field.as_str()) {
            *value = json!(super::money::amount(value.as_str().unwrap(), field)?.to_sci_string());
        } else if ["effective_life", "taxable_use"].contains(&field.as_str()) {
            let parsed = div7a_loan_review::decimal::Decimal::parse(value.as_str().unwrap())
                .map_err(|_| "[<class 'decimal.ConversionSyntax'>]".to_string())?;
            *value = json!(parsed.to_sci_string());
        }
    }
    let mut result = australian_tax_calculators::api::evaluate(&kind, &facts)?;
    served_scope(&kind.replace('-', "_"), &mut result);
    if input["file_note"] == true {
        result["file_note"] = json!(file_note(&input["facts"], &result)?);
    }
    Ok(result)
}

/// A Python f-string's rendering of one supplied fact or result value.
fn shown(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => "None".into(),
        other => other.to_string(),
    }
}

fn table(heading: &str, values: &Value) -> Vec<String> {
    let values = values.as_object().into_iter().flatten();
    let rows: Vec<_> = values
        .map(|(name, value)| format!("| `{name}` | {} |", shown(value)))
        .collect();
    if rows.is_empty() {
        return rows;
    }
    [
        vec![format!("| {heading} | Value |"), "| --- | --- |".into()],
        rows,
        vec![String::new()],
    ]
    .concat()
}

/// Restate one worksheet result as a Markdown file note, as the Python server's
/// render_file_note does. Every figure, provision, date and caveat is the engine's or
/// the operator's; the note adds none.
fn file_note(facts: &Value, result: &Value) -> Result<String, String> {
    let catalogue = australian_tax_calculators::api::evaluate("worksheet-catalogue", &json!({}))?;
    let calculation = result["calculation"].as_str().unwrap_or_default();
    let passage = shown(&catalogue[calculation]["source_passage"]);
    let text = |key: &str| shown(&result[key]);
    let items = |key: &str| -> Vec<String> {
        result[key]
            .as_array()
            .into_iter()
            .flatten()
            .map(shown)
            .collect()
    };
    let mut lines = vec![
        format!("# File note: {calculation} worksheet, {}", text("period")),
        String::new(),
        format!(
            "Drafted from the {} {} result, which it restates without adding any figure.",
            text("engine"),
            text("engine_version")
        ),
        String::new(),
        "## Facts relied on".into(),
        String::new(),
        "Established by the operator and not verified by this server.".into(),
        String::new(),
    ];
    lines.extend(
        facts
            .as_object()
            .into_iter()
            .flatten()
            .map(|(name, value)| format!("- `{name}`: {}", shown(value))),
    );
    lines.extend([String::new(), "## Calculation".into(), String::new()]);
    lines.extend(table("Amount (AUD)", &result["amounts"]));
    lines.extend(table("Rate", &result["rates"]));
    lines.extend([
        "## Assumptions".into(),
        String::new(),
        "The operator confirmed this scope:".into(),
        String::new(),
        text("scope"),
        String::new(),
        "## Provisions and sources".into(),
        String::new(),
        format!("The engine's source check covered: {passage}."),
        String::new(),
    ]);
    let checked = text("source_checked");
    lines.extend(
        items("sources")
            .into_iter()
            .map(|url| format!("- {url} (checked {checked})")),
    );
    lines.extend([String::new(), "## Open issues".into(), String::new()]);
    lines.extend(
        items("warnings")
            .into_iter()
            .map(|warning| format!("- {warning}")),
    );
    lines.push(String::new());
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    fn year(date: &Value) -> u32 {
        date.as_str().unwrap()[..4].parse().unwrap()
    }

    /// The years from `first` on that a text names. Earlier years name legislation or
    /// history, such as ITAA 1997 or assets first held from 10 May 2006, not periods.
    fn named_years(text: &str, first: u32) -> Vec<u32> {
        text.split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|word| word.len() == 4)
            .filter_map(|word| word.parse().ok())
            .filter(|named| *named >= first)
            .collect()
    }

    /// Every year a served worksheet scope names falls in one of the worksheet's
    /// advertised periods, in the scope resource and in each period's result.
    #[test]
    fn every_year_a_scope_names_is_an_advertised_period() {
        let catalogue = super::advertised_worksheets().unwrap();
        let catalogue = catalogue.as_object().unwrap();
        let periods = |entry: &Value| entry["supported_periods"].as_array().unwrap().clone();
        let first = catalogue
            .values()
            .flat_map(periods)
            .map(|period| year(&period["start"]))
            .min()
            .unwrap();
        for (kind, entry) in catalogue {
            let periods = periods(entry);
            let mut scopes = vec![entry["scope"].clone()];
            for period in &periods {
                let mut facts = entry["example"]["facts"].clone();
                let arguments = period["arguments"].as_object().unwrap();
                facts.as_object_mut().unwrap().extend(arguments.clone());
                let result = super::calculate(&json!({ "facts": facts })).unwrap();
                scopes.push(result["scope"].clone());
            }
            for scope in &scopes {
                let scope = scope.as_str().unwrap();
                for named in named_years(scope, first) {
                    assert!(
                        periods.iter().any(|period| {
                            (year(&period["start"])..=year(&period["end"])).contains(&named)
                        }),
                        "{kind} names {named}, outside its advertised periods: {scope}"
                    );
                }
            }
        }
    }
}
