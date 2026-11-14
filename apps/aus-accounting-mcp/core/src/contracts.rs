// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Frozen discovery is metadata, never a replacement calculation or dataset.
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::LazyLock;

pub static CATALOGUE: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/catalogue.v1.json"))
        .expect("reviewed embedded catalogue must be JSON")
});

pub fn tool(name: &str) -> Option<&'static Value> {
    CATALOGUE["tools"]
        .as_array()?
        .iter()
        .find(|t| t["name"] == name)
}

pub fn check_arguments(name: &str, args: &Value) -> Result<(), String> {
    let spec = tool(name).ok_or_else(|| format!("Unknown tool: {name}"))?;
    let properties = spec["inputSchema"]["properties"].as_object().unwrap();
    let args = args.as_object().ok_or("arguments must be an object")?;
    let mut unknown: Vec<_> = args
        .keys()
        .filter(|k| !properties.contains_key(*k))
        .cloned()
        .collect();
    unknown.sort();
    if !unknown.is_empty() {
        return Err(format!(
            "{name}: unsupported arguments: {}",
            unknown.join(", ")
        ));
    }
    Ok(())
}

/// Output models permit additional engine fields but validate declared values.
pub fn validate_output(name: &str, result: &Value) -> Result<(), String> {
    static VALIDATORS: LazyLock<HashMap<String, jsonschema::Validator>> = LazyLock::new(|| {
        CATALOGUE["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| {
                let validator = jsonschema::draft202012::options()
                    .should_validate_formats(true)
                    .build(&tool["outputSchema"])
                    .expect("reviewed embedded output schema");
                (tool["name"].as_str().unwrap().to_owned(), validator)
            })
            .collect()
    });
    let validator = VALIDATORS.get(name).ok_or("Unknown tool")?;
    validator
        .validate(result)
        .map_err(|e| format!("Invalid output for {name}: {e}"))
}

pub fn success(result: Value) -> Value {
    json!({"content":[{"type":"text","text":serde_json::to_string_pretty(&result).unwrap()}],
           "structuredContent":result,"isError":false,"resultType":"complete"})
}

pub fn error(message: String) -> Value {
    json!({"content":[{"type":"text","text":message}],"isError":true,"resultType":"complete"})
}
