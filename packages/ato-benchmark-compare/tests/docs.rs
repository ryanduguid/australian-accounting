//! Keep the documentation honest against the code.

use ato_benchmark_compare::{VERSION, api};
use serde_json::json;

const README: &str = include_str!("../README.md");
const LIMITATIONS: &str = include_str!("../LIMITATIONS.md");
const CHANGELOG: &str = include_str!("../CHANGELOG.md");

#[test]
fn the_register_lists_every_documented_entry() {
    assert!(LIMITATIONS.contains("ABC-1"));
    assert!(LIMITATIONS.contains("ABC-2"));
    // The clause that distinguishes a register from a disclaimer: every entry has one.
    assert_eq!(
        LIMITATIONS.matches("**What stays correct.**").count(),
        LIMITATIONS.matches("\n## ABC-").count()
    );
}

#[test]
fn the_readme_documents_every_json_command() {
    let start = README.find("## JSON API").expect("JSON API section");
    let section = &README[start..];
    let section = &section[..section[3..].find("\n## ").map_or(section.len(), |i| i + 3)];
    for command in api::COMMANDS {
        assert!(
            section.contains(&format!("`{command}`")),
            "{command} is undocumented"
        );
    }
}

#[test]
fn the_changelog_heads_with_this_version() {
    assert!(CHANGELOG.contains(&format!("## {VERSION} (unreleased)")));
    assert!(README.contains(&format!(
        "ATO Benchmark Compare {VERSION} is a standalone Rust binary"
    )));
}

/// ABC-1 exists because the raw serialiser has no supplied-field set: the
/// `to_dict` output ignores `output_supplied`, and the evidenced one reads it.
/// If that changes, the LIMITATIONS.md entry must be revisited.
#[test]
fn the_raw_serialiser_cannot_see_which_buckets_were_supplied() {
    let base = json!({
        "dataset": {"year": "2023-24", "data_dir": null},
        "industry": "Bakeries and hot bread shops",
        "totals": {"turnover": "850000", "other_income": "0", "rent": "40000"},
    });
    let run = |output: &str, supplied: serde_json::Value| {
        let mut input = base.clone();
        input["output"] = json!(output);
        input["output_supplied"] = supplied;
        api::evaluate("compare-figures", &input).unwrap()
    };
    let all = json!(["turnover", "other_income", "rent", "cost_of_sales"]);
    let some = json!(["turnover", "other_income", "rent"]);
    assert_eq!(run("to_dict", all.clone()), run("to_dict", some.clone()));
    assert_ne!(run("evidenced", all), run("evidenced", some));
}
