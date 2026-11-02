use ato_benchmark_compare::{
    api,
    dataset::{self, DataDir},
    mapping::BUCKETS,
    py, report,
};
use serde_json::{Value, json};

pub fn list(input: &Value) -> Result<Value, String> {
    let data =
        dataset::load(input["year"].as_str(), &DataDir::Shipped).map_err(|e| e.to_string())?;
    let matches = match input["search"].as_str().filter(|s| !s.is_empty()) {
        Some(search) => data.search(search, false),
        None => data.business_types.iter().collect(),
    };
    let offset = input["offset"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(usize::MAX);
    let limit = input["limit"].as_u64().map_or(usize::MAX, |n| n as usize);
    let page: Vec<_> = matches
        .iter()
        .skip(offset)
        .take(limit)
        .map(|bt| json!({"name":bt.name,"key_ratio":bt.key_ratio}))
        .collect();
    let next = offset.saturating_add(page.len());
    let more = next < matches.len();
    Ok(
        json!({"ok":true,"engine":"ato-benchmark-compare","engine_version":ato_benchmark_compare::VERSION,
        "benchmark_year":data.year,"count":page.len(),"total_count":matches.len(),"offset":input["offset"],
        "has_more":more,"next_offset":if more {json!(next)}else{Value::Null},
        "total_business_types":data.business_types.len(),"industries":page,"source":report::raw_json(&data.source)}),
    )
}

pub fn compare(input: &Value) -> Result<Value, String> {
    let mut totals = serde_json::Map::new();
    totals.insert(
        "turnover".into(),
        json!(
            super::money::amount(input["turnover"].as_str().unwrap(), "turnover")?.to_sci_string()
        ),
    );
    let mut supplied = vec!["turnover".to_string()];
    for (field, _) in BUCKETS {
        if field == "turnover" {
            continue;
        }
        if let Some(amount) = super::money::optional(input.get(field), field)? {
            totals.insert(field.into(), json!(amount.to_sci_string()));
            supplied.push(field.to_string());
        } else {
            totals.insert(field.into(), json!("0"));
        }
    }
    let w1 = super::money::optional(input.get("w1"), "w1")?;
    if !supplied
        .iter()
        .any(|s| !["turnover", "other_income"].contains(&s.as_str()))
    {
        return Err("no expense figures were supplied, so no ATO ratio can be compared. Pass at least one expense bucket as a decimal string; use 0 only when the operator established that the amount is zero.".into());
    }
    if w1.is_some() {
        supplied.push("w1".into());
    }
    let mut result = api::evaluate(
        "compare-figures",
        &json!({"dataset":{"year":input["year"]},"industry":input["industry"],
        "totals":totals,"w1":w1.map(|n|n.to_sci_string()),"output":"evidenced","output_supplied":supplied}),
    )
    .map_err(|error| unmatched(input, error))?;
    result["ok"] = json!(true);
    result["engine"] = json!("ato-benchmark-compare");
    result["engine_version"] = json!(ato_benchmark_compare::VERSION);
    Ok(result)
}

/// The engine's message for an industry no business type matches sends the reader to
/// its command line, which an MCP client cannot run; name the tool that lists them.
fn unmatched(input: &Value, error: String) -> String {
    let industry = input["industry"].as_str().unwrap_or_default();
    match dataset::load(input["year"].as_str(), &DataDir::Shipped) {
        Ok(data) if data.search(industry, false).is_empty() => format!(
            "no ATO business type matches {}. Call list_ato_benchmark_industries with a \
             search word and year {} to find the exact name among all {} of them.",
            py::repr_str(industry),
            py::repr_str(&data.year),
            data.business_types.len()
        ),
        _ => error,
    }
}
