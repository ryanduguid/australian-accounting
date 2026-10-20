//! Compare periods using the existing benchmark engine and reviewed mappings.
//!
//! Use --previous-expenses-negative and --current-expenses-negative for snapshots
//! whose expense rows are negative. Each flag applies to that snapshot, including
//! its counterfactual calculation; signs are never guessed from account names.

use ato_benchmark_compare::Failure;
use ato_benchmark_compare::dataset::{DataDir, load};
use ato_benchmark_compare::decimal::Decimal;
use ato_benchmark_compare::mapping::{Mapping, read_mapping, route};
use ato_benchmark_compare::pnl::read;
use ato_benchmark_compare::py::PyPath;
use ato_benchmark_compare::ratios::compute;
use ato_benchmark_compare::report::{compare, to_evidenced_dict};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const USAGE: &str = "usage: benchmark_history --previous-pnl PATH --current-pnl PATH \
--previous-mapping PATH --current-mapping PATH --previous-period LABEL \
--current-period LABEL --industry NAME [--previous-year YEAR] [--current-year YEAR] \
[--previous-expenses-negative] [--current-expenses-negative]";

/// The inputs of one history comparison.
#[derive(Clone, Debug, Default)]
pub struct Request {
    pub previous_pnl: PathBuf,
    pub current_pnl: PathBuf,
    pub previous_mapping: PathBuf,
    pub current_mapping: PathBuf,
    pub previous_period: String,
    pub current_period: String,
    pub industry: String,
    pub previous_year: Option<String>,
    pub current_year: Option<String>,
    pub previous_expenses_negative: bool,
    pub current_expenses_negative: bool,
}

fn path_of(path: &Path) -> PyPath {
    PyPath::new(&path.to_string_lossy())
}

fn single(
    pnl_path: &Path,
    mapping: &Mapping,
    year: Option<&str>,
    industry: &str,
    expenses_negative: bool,
) -> Result<Value, Failure> {
    let source = read(&path_of(pnl_path), None)?;
    let routed = route(&source.rows, mapping, expenses_negative)?;
    let figures = compute(&routed.totals, &routed.supplied_buckets, None)?;
    let data = load(year, &DataDir::Shipped)?;
    let result = compare(
        &data,
        data.get(industry)?,
        &figures,
        Some(&routed.supplied_buckets),
    )?;
    to_evidenced_dict(&result, &routed.supplied_buckets, json!(routed.unreviewed))
        .map(Value::Object)
}

fn digest(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

fn ratio_values(payload: &Value) -> Map<String, Value> {
    payload["ratios"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| {
            (
                row["ratio"].as_str().unwrap_or_default().to_string(),
                row["value"].clone(),
            )
        })
        .collect()
}

fn difference(a: &Value, b: &Value) -> Result<String, Failure> {
    let parse = |v: &Value| Decimal::parse(v.as_str().unwrap_or_default());
    Ok(parse(a)?.sub(&parse(b)?)?.to_sci_string())
}

/// The comparison of two periods, with the mapping effect kept apart from the
/// movement under the previous mapping.
pub fn history(request: &Request) -> Result<Value, String> {
    let r = request;
    if r.previous_period.trim().is_empty()
        || r.current_period.trim().is_empty()
        || r.previous_period == r.current_period
    {
        return Err("Supply distinct, explicit period labels".into());
    }
    let paths = [
        &r.previous_pnl,
        &r.current_pnl,
        &r.previous_mapping,
        &r.current_mapping,
    ];
    let hashes = paths
        .iter()
        .map(|p| digest(p))
        .collect::<Result<Vec<_>, _>>()?;
    let text = |f: Failure| f.to_string();
    let old_map = read_mapping(&path_of(&r.previous_mapping)).map_err(text)?;
    let new_map = read_mapping(&path_of(&r.current_mapping)).map_err(text)?;
    let (previous_year, current_year) = (r.previous_year.as_deref(), r.current_year.as_deref());
    let previous = single(
        &r.previous_pnl,
        &old_map,
        previous_year,
        &r.industry,
        r.previous_expenses_negative,
    )
    .map_err(text)?;
    let current = single(
        &r.current_pnl,
        &new_map,
        current_year,
        &r.industry,
        r.current_expenses_negative,
    )
    .map_err(text)?;

    let find = |map: &Mapping, key: &str| {
        map.iter()
            .find(|(k, _)| k == key)
            .map(|(_, row)| row.clone())
    };
    let mut keys: Vec<&String> = old_map
        .iter()
        .chain(new_map.iter())
        .map(|(k, _)| k)
        .collect();
    keys.sort();
    keys.dedup();
    let mut mapping_changes = Vec::new();
    for key in keys {
        let (old, new) = (find(&old_map, key), find(&new_map, key));
        if old.as_ref().map(|o| &o.bucket) != new.as_ref().map(|n| &n.bucket) {
            let account = new.as_ref().or(old.as_ref()).unwrap().account.clone();
            mapping_changes.push(json!({
                "account": account,
                "previous_bucket": old.map(|o| o.bucket),
                "current_bucket": new.map(|n| n.bucket),
            }));
        }
    }

    // The current ledger under the previous mapping. A mapping or ratio refusal
    // makes it unavailable; a trapped decimal signal still stops the comparison.
    let (mut constant, mut limitation) = match single(
        &r.current_pnl,
        &old_map,
        previous_year,
        &r.industry,
        r.current_expenses_negative,
    ) {
        Ok(value) => (Some(value), None),
        Err(Failure::Value(message)) => (
            None,
            Some(format!(
                "Current figures cannot be compared under the previous mapping: {message}"
            )),
        ),
        Err(other) => return Err(other.to_string()),
    };
    let unreviewed = |v: &Value| v["unreviewed_accounts"].as_u64().unwrap_or(0) > 0;
    if unreviewed(&previous) || unreviewed(&current) || constant.as_ref().is_some_and(unreviewed) {
        constant = None;
        limitation =
            Some("Suggested mappings remain; a performance/mapping split is withheld.".into());
    }

    let old_ratios = ratio_values(&previous);
    let same_ratios = constant.as_ref().map(ratio_values).unwrap_or_default();
    let mut bridge = Vec::new();
    for row in current["ratios"].as_array().into_iter().flatten() {
        let key = row["ratio"].as_str().unwrap_or_default();
        let old = old_ratios.get(key).cloned().unwrap_or(Value::Null);
        let same = same_ratios.get(key).cloned().unwrap_or(Value::Null);
        let now = row["value"].clone();
        let available = !old.is_null() && !same.is_null() && !now.is_null();
        let (movement, effect) = if available {
            (
                json!(difference(&same, &old).map_err(text)?),
                json!(difference(&now, &same).map_err(text)?),
            )
        } else {
            (Value::Null, Value::Null)
        };
        bridge.push(json!({
            "ratio": key,
            "previous": old,
            "current": now,
            "current_under_previous_mapping": same,
            "movement_under_previous_mapping": movement,
            "mapping_effect": effect,
            "comparison": if available { "DECOMPOSED" } else { "NOT_COMPARABLE" },
        }));
    }
    let basis_changes: Vec<&str> = [
        "benchmark_year",
        "business_type",
        "turnover_basis",
        "turnover_band",
        "source",
    ]
    .into_iter()
    .filter(|field| previous.get(*field) != current.get(*field))
    .collect();
    if hashes
        != paths
            .iter()
            .map(|p| digest(p))
            .collect::<Result<Vec<_>, _>>()?
    {
        return Err("An input changed during the comparison; rerun from stable files".into());
    }
    let names = [
        "previous_pnl",
        "current_pnl",
        "previous_mapping",
        "current_mapping",
    ];
    Ok(json!({
        "previous_period": r.previous_period,
        "current_period": r.current_period,
        "source_sha256": names.iter().zip(&hashes).map(|(n, h)| (n.to_string(), json!(h))).collect::<Map<_, _>>(),
        "previous": previous,
        "current": current,
        "mapping_changes": mapping_changes,
        "expense_signs": {
            "previous_negative": r.previous_expenses_negative,
            "current_negative": r.current_expenses_negative,
        },
        "basis_changes": basis_changes,
        "ratio_bridge": bridge,
        "limitation": limitation,
        "scope": "Existing benchmark definitions and datasets only. Ratio differences are proportions, not percentage points. No cause inferred from ledger movement.",
    }))
}

/// Indented JSON with non-ASCII characters escaped, as Python's json.dumps(indent=2).
fn ascii_json(value: &Value) -> String {
    let mut out = String::new();
    for c in serde_json::to_string_pretty(value).unwrap().chars() {
        if c.is_ascii() {
            out.push(c);
        } else {
            let mut units = [0u16; 2];
            for unit in c.encode_utf16(&mut units) {
                out.push_str(&format!("\\u{unit:04x}"));
            }
        }
    }
    out
}

fn parse(args: &[String]) -> Result<Request, String> {
    let mut r = Request::default();
    let mut seen = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        match flag {
            "--previous-expenses-negative" => r.previous_expenses_negative = true,
            "--current-expenses-negative" => r.current_expenses_negative = true,
            _ => {
                let value = args
                    .get(i + 1)
                    .ok_or(format!("{flag} needs a value"))?
                    .clone();
                match flag {
                    "--previous-pnl" => r.previous_pnl = value.into(),
                    "--current-pnl" => r.current_pnl = value.into(),
                    "--previous-mapping" => r.previous_mapping = value.into(),
                    "--current-mapping" => r.current_mapping = value.into(),
                    "--previous-period" => r.previous_period = value,
                    "--current-period" => r.current_period = value,
                    "--industry" => r.industry = value,
                    "--previous-year" => r.previous_year = Some(value),
                    "--current-year" => r.current_year = Some(value),
                    _ => return Err(format!("unrecognised argument {flag}")),
                }
                seen.push(flag);
                i += 1;
            }
        }
        i += 1;
    }
    let required = [
        "--previous-pnl",
        "--current-pnl",
        "--previous-mapping",
        "--current-mapping",
        "--previous-period",
        "--current-period",
        "--industry",
    ];
    match required.iter().find(|name| !seen.contains(name)) {
        Some(name) => Err(format!("{name} is required")),
        None => Ok(r),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return;
    }
    let request = parse(&args).unwrap_or_else(|message| {
        eprintln!("{USAGE}\nerror: {message}");
        std::process::exit(2);
    });
    match history(&request) {
        Ok(result) => println!("{}", ascii_json(&result)),
        Err(message) => {
            eprintln!("error: {message}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ato_benchmark_compare::mapping::{account_key, is_expense};

    fn example(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(name)
    }

    fn bakery(previous_mapping: PathBuf, current_mapping: PathBuf) -> Request {
        Request {
            previous_pnl: example("bakery-pnl.csv"),
            current_pnl: example("bakery-pnl.csv"),
            previous_mapping,
            current_mapping,
            previous_period: "FY2025".into(),
            current_period: "FY2026".into(),
            industry: "Bakeries and hot bread shops".into(),
            ..Request::default()
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ato-history-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn bridge<'a>(result: &'a Value, ratio: &str) -> &'a Value {
        result["ratio_bridge"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ratio"] == ratio)
            .unwrap()
    }

    #[test]
    fn reclassification_is_separate_from_unchanged_business() {
        let dir = scratch("reclassified");
        let original = std::fs::read_to_string(example("bakery-mapping.csv")).unwrap();
        assert!(original.contains(",salary_wages,"));
        let changed = original
            .lines()
            .map(|line| {
                if line.contains(",salary_wages,") {
                    let fields: Vec<&str> = line.splitn(3, ',').collect();
                    let rest = fields[2].rsplitn(2, ',').last().unwrap();
                    format!("{},other_expense,{rest},Fabricated reclassification to demonstrate comparison basis", fields[0])
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let mapping = dir.join("changed.csv");
        std::fs::write(&mapping, changed + "\n").unwrap();
        let result = history(&bakery(example("bakery-mapping.csv"), mapping)).unwrap();
        assert!(!result["mapping_changes"].as_array().unwrap().is_empty());
        for row in result["ratio_bridge"].as_array().unwrap() {
            if row["comparison"] == "DECOMPOSED" {
                let movement =
                    Decimal::parse(row["movement_under_previous_mapping"].as_str().unwrap())
                        .unwrap();
                assert!(movement.is_zero(), "{row}");
            }
        }
        // An omitted labour bucket is unknown, never an invented zero improvement.
        let labour = bridge(&result, "labour_to_turnover");
        assert_eq!(labour["comparison"], "NOT_COMPARABLE");
        assert!(labour["current"].is_null());
    }

    #[test]
    fn unsupported_industry_gets_no_invented_range() {
        let mut request = bakery(example("bakery-mapping.csv"), example("bakery-mapping.csv"));
        request.industry = "Invented unsupported industry".into();
        let message = history(&request).unwrap_err();
        assert!(
            message.contains("Invented unsupported industry"),
            "{message}"
        );
    }

    #[test]
    fn unavailable_counterfactual_preserves_standalone_results() {
        let dir = scratch("counterfactual");
        let write = |name: &str, text: &str| {
            let path = dir.join(name);
            std::fs::write(&path, text).unwrap();
            path
        };
        let previous = write(
            "previous.csv",
            "Account,Amount\nSales,500000\nNew activity,0\nCosts,100000\n",
        );
        let current = write(
            "current.csv",
            "Account,Amount\nSales,0\nNew activity,600000\nCosts,120000\n",
        );
        let mapping = |bucket: &str| {
            format!(
                "account,bucket,source,amount,note\nSales,turnover,reviewed,0,Fabricated\n\
                 New activity,{bucket},reviewed,0,Fabricated\nCosts,cost_of_sales,reviewed,0,Fabricated\n"
            )
        };
        let old = write("old.csv", &mapping("excluded"));
        let new = write("new.csv", &mapping("turnover"));
        let request = Request {
            previous_pnl: previous,
            current_pnl: current,
            previous_period: "2025".into(),
            current_period: "2026".into(),
            ..bakery(old, new)
        };
        let result = history(&request).unwrap();
        assert!(result["previous"].is_object() && result["current"].is_object());
        assert!(result["limitation"].is_string());
        for row in result["ratio_bridge"].as_array().unwrap() {
            assert_eq!(row["comparison"], "NOT_COMPARABLE");
        }
    }

    #[test]
    fn expense_signs_are_explicit_for_each_snapshot() {
        let dir = scratch("signs");
        let positive = example("bakery-pnl.csv");
        let reviewed = read_mapping(&path_of(&example("bakery-mapping.csv"))).unwrap();
        let mut text = String::from("Account,Amount\r\n");
        for row in read(&path_of(&positive), None).unwrap().rows {
            let key = account_key(&row.account);
            let bucket = &reviewed.iter().find(|(k, _)| *k == key).unwrap().1.bucket;
            let amount = if is_expense(bucket) {
                row.amount.neg().unwrap()
            } else {
                row.amount.clone()
            };
            text.push_str(&format!("{},{}\r\n", row.account, amount.to_sci_string()));
        }
        let negative = dir.join("negative.csv");
        std::fs::write(&negative, text).unwrap();
        let request = Request {
            current_pnl: negative,
            previous_period: "2025".into(),
            current_period: "2026".into(),
            current_expenses_negative: true,
            ..bakery(example("bakery-mapping.csv"), example("bakery-mapping.csv"))
        };
        let result = history(&request).unwrap();
        let cost = bridge(&result, "cost_of_sales_to_turnover");
        assert_eq!(cost["current"], cost["previous"]);
        assert!(!cost["current"].is_null());
        let movement =
            Decimal::parse(cost["movement_under_previous_mapping"].as_str().unwrap()).unwrap();
        assert!(movement.is_zero());
        assert_eq!(
            result["expense_signs"],
            json!({"previous_negative": false, "current_negative": true})
        );
    }

    #[test]
    fn output_escapes_as_python_did() {
        assert_eq!(
            ascii_json(&json!({"a": ["é", "😀"], "b": {}})),
            "{\n  \"a\": [\n    \"\\u00e9\",\n    \"\\ud83d\\ude00\"\n  ],\n  \"b\": {}\n}"
        );
    }
}
