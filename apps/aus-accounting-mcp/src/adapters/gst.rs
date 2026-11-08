// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use serde_json::{Map, Value, json};

const DISCLAIMER: &str = "Experimental review aid. Not a BAS, a revised BAS, a lodgement or a GST determination. The operator supplies registration dates, codes, periods, lodged figures and the basis, and none of them is verified. Obtain human review before acting.";
fn date(value: &Value, field: &str) -> Result<(), String> {
    if let Some(day) = value.as_str() {
        super::valid_day(day, field)?;
    }
    Ok(())
}
fn amount(value: &mut Value, field: &str) -> Result<(), String> {
    *value = json!(super::money::amount(value.as_str().unwrap(), field)?.to_sci_string());
    Ok(())
}
pub fn coding(input: &Value) -> Result<Value, String> {
    let mut input = input.clone();
    let mut registrations = Map::new();
    for item in input["registrations"].as_array().unwrap() {
        let entity = item["entity"].as_str().unwrap();
        if registrations.contains_key(entity) {
            return Err(format!(
                "registrations: entity {} appears more than once",
                div7a_loan_review::raw::Raw::text(entity).py_repr()
            ));
        }
        for key in ["registered_from", "registered_to"] {
            date(&item[key], &format!("registrations[{entity}].{key}"))?;
        }
        registrations.insert(entity.into(),json!({"registered_from":item["registered_from"],"registered_to":item["registered_to"]}));
    }
    input["registrations"] = Value::Object(registrations);
    for line in input["lines"].as_array_mut().unwrap() {
        let id = line["id"].as_str().unwrap().to_string();
        date(&line["date"], &format!("lines[{id}].date"))?;
        amount(&mut line["amount"], &format!("lines[{id}].amount"))?;
    }
    let mut result = gst_review::api::evaluate("review-coding", &input)?;
    result["disclaimer"] = json!(DISCLAIMER);
    Ok(result)
}
pub fn bas(input: &Value) -> Result<Value, String> {
    let mut input = input.clone();
    let mut lodged = Map::new();
    for period in input["periods"].as_array().unwrap() {
        let label = period["label"].as_str().unwrap();
        for field in ["start", "end"] {
            date(&period[field], &format!("periods[{label}].{field}"))?;
        }
    }
    for period in input["periods"].as_array().unwrap() {
        let label = period["label"].as_str().unwrap();
        if let Some(figures) = period["lodged"].as_object() {
            let mut amounts = figures.clone();
            for (key, value) in &mut amounts {
                amount(value, &format!("periods[{label}].lodged.{key}"))?;
            }
            lodged.insert(label.into(), Value::Object(amounts));
        }
    }
    input["lodged"] = Value::Object(lodged);
    for line in input["lines"].as_array_mut().unwrap() {
        let id = line["id"].as_str().unwrap().to_string();
        for field in ["gross", "gst"] {
            amount(&mut line[field], &format!("lines[{id}].{field}"))?;
        }
        for field in ["invoice_date", "payment_date"] {
            date(&line[field], &format!("lines[{id}].{field}"))?;
        }
    }
    let mut result = gst_review::api::evaluate("reconcile-bas", &input)?;
    result["disclaimer"] = json!(DISCLAIMER);
    Ok(result)
}
