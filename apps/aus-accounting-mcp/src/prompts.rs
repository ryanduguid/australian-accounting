//! Preserve the published guidance, including historical version references.
use serde_json::{Map, Value, json};
pub fn get(name: &str, arguments: Option<&Map<String, Value>>) -> Result<Value, String> {
    let key = match name {
        "compare_ato_benchmarks" => "industry",
        "review_payday_super_contribution" => "as_at",
        "review_div7a_loan_terms" => "year_of_income",
        _ => return Err(format!("Unknown prompt: {name}")),
    };
    if arguments.is_some_and(|a| {
        a.keys().any(|k| k != key) || a.get(key).is_some_and(|v| !v.is_null() && !v.is_string())
    }) {
        return Err(format!("Error rendering prompt {name}"));
    }
    let argument = arguments
        .and_then(|a| a.get(key))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    let templates = &crate::contracts::CATALOGUE["prompt_templates"][name];
    let mut result = templates[if argument.is_some() {
        "with_argument"
    } else {
        "default"
    }]
    .clone();
    result["resultType"] = json!("complete");
    if let Some(argument) = argument {
        let text = result["messages"][0]["content"]["text"]
            .as_str()
            .unwrap()
            .replace("synthetic example", argument);
        result["messages"][0]["content"]["text"] = json!(text);
    }
    Ok(result)
}
