// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Frozen discovery is metadata, never a replacement calculation or dataset.
use serde_json::{Value, json};
use std::sync::{LazyLock, OnceLock};

const WORKSHEET_COUNT_PLACEHOLDER: &str = "{worksheet_count}";

pub static CATALOGUE: LazyLock<Value> = LazyLock::new(|| {
    let mut catalogue: Value = serde_json::from_str(include_str!("../data/catalogue.v1.json"))
        .expect("reviewed embedded catalogue must be JSON");
    let worksheets = catalogue["tools"]
        .as_array_mut()
        .and_then(|tools| tools.iter_mut().find(|t| t["name"] == "calculate_tax_worksheet"))
        .expect("calculate_tax_worksheet tool");
    crate::adapters::tax::open_other_kinds(&mut worksheets["inputSchema"]);
    let kinds: Vec<&str> = crate::adapters::tax::kinds().collect();
    derive_worksheet_counts(&mut catalogue, &kinds);
    catalogue
});

fn derive_worksheet_counts(catalogue: &mut Value, engine_kinds: &[&str]) {
    let tool = catalogue["tools"]
        .as_array_mut()
        .and_then(|tools| {
            tools
                .iter_mut()
                .find(|t| t["name"] == "calculate_tax_worksheet")
        })
        .expect("calculate_tax_worksheet tool");
    let schema = &tool["inputSchema"];
    let typed = schema["properties"]["facts"]["discriminator"]["mapping"]
        .as_object()
        .expect("calculate_tax_worksheet discriminator mapping")
        .keys()
        .map(String::as_str);
    let other = crate::adapters::tax::other_kinds(schema);
    let count = typed
        .chain(other)
        .filter(|kind| engine_kinds.contains(kind))
        .count()
        .to_string();
    let description = tool["description"]
        .as_str()
        .expect("calculate_tax_worksheet description");
    debug_assert!(description.contains(WORKSHEET_COUNT_PLACEHOLDER));
    tool["description"] = Value::String(description.replace(WORKSHEET_COUNT_PLACEHOLDER, &count));
}

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

/// Output models permit additional engine fields but validate declared values. Each
/// tool's validator is compiled on its first call.
pub fn validate_output(name: &str, result: &Value) -> Result<(), String> {
    let tools = CATALOGUE["tools"].as_array().unwrap();
    static VALIDATORS: LazyLock<Vec<OnceLock<jsonschema::Validator>>> =
        LazyLock::new(|| CATALOGUE["tools"].as_array().unwrap().iter().map(|_| OnceLock::new()).collect());
    let index = tools
        .iter()
        .position(|tool| tool["name"] == name)
        .ok_or("Unknown tool")?;
    let validator = VALIDATORS[index].get_or_init(|| {
        jsonschema::draft202012::options()
            .should_validate_formats(true)
            .build(&tools[index]["outputSchema"])
            .expect("reviewed embedded output schema")
    });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worksheet_count_is_derived_from_advertised_catalogue() {
        let served = crate::adapters::tax::advertised_worksheets()
            .unwrap()
            .as_object()
            .unwrap()
            .len();
        let description = tool("calculate_tax_worksheet").unwrap()["description"]
            .as_str()
            .unwrap();
        assert!(description.contains(&format!(
            "Calculate one of {served} bounded tax and super worksheets"
        )));
        assert!(!description.contains("fourteen"));
        assert!(!description.contains(WORKSHEET_COUNT_PLACEHOLDER));
    }

    #[test]
    fn derive_worksheet_counts_tracks_catalogue_changes() {
        let mut mapping = serde_json::Map::new();
        let names: Vec<String> = (0..70).map(|i| format!("kind_{i}")).collect();
        for (i, kind) in names.iter().enumerate() {
            mapping.insert(kind.clone(), json!(format!("#/$defs/Kind{i}")));
        }
        let mut engine_kinds: Vec<&str> = names.iter().map(String::as_str).collect();
        engine_kinds.extend(["unadvertised_kind", "other_kind"]);
        let mut catalogue = json!({
            "tools": [{
                "name": "calculate_tax_worksheet",
                "description": "Calculate one of {worksheet_count} bounded tax and super worksheets.",
                "inputSchema": {
                    "$defs": {"WorksheetFacts": {"properties": {"kind": {"enum": ["other_kind"]}}}},
                    "properties": {
                        "facts": {
                            "discriminator": {
                                "mapping": mapping
                            }
                        }
                    }
                }
            }]
        });
        derive_worksheet_counts(&mut catalogue, &engine_kinds);
        assert_eq!(
            catalogue["tools"][0]["description"],
            "Calculate one of 71 bounded tax and super worksheets."
        );
    }
}
