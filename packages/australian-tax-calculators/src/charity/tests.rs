// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `charity-check` against a fabricated register copy; the ABNs are unissued.
use super::*;

const FIXTURE: &str = concat!(
    r#"{"source":"fixture","snapshot_date":"2026-11-22","charities":2,"licence":"Creative Commons Attribution 3.0 Australia"}"#,
    "\n",
    r#"["23000000004","EXAMPLE HEALTH CHARITY",null,[1,14]]"#,
    "\n",
    r#"["61000000002","EXAMPLE SCHOOL","2012-12-03",[2]]"#,
    "\n",
);

fn run(input: Value) -> Result<Value, Refusal> {
    check_with(&parse_register(FIXTURE), &input)
}

fn code(input: Value) -> String {
    run(input).unwrap_err().code.to_string()
}

#[test]
fn an_abn_on_the_copy_gives_its_name_date_and_subtypes_in_table_order() {
    let out =
        run(json!({"abns": ["23 000 000 004", "61000000002"], "as_at": "2026-11-24"})).unwrap();
    let first = &out["charities"][0];
    assert_eq!(first["on_register_snapshot"], true);
    assert_eq!(first["abn"], "23000000004");
    assert_eq!(first["legal_name"], "EXAMPLE HEALTH CHARITY");
    assert_eq!(first["registration_date"], Value::Null);
    assert_eq!(
        first["subtypes"],
        json!([{"item": 1, "subtype": "advancing health"}, {"item": 14, "subtype": "public benevolent institution"}])
    );
    assert_eq!(first["snapshot_date"], "2026-11-22");
    assert_eq!(out["charities"][1]["registration_date"], "2012-12-03");
    assert_eq!(out["charities"][1]["index"], 1);
    assert_eq!(out["data"]["answers_until"], "2026-12-23");
    assert_eq!(out["data"]["charities"], 2);
}

#[test]
fn an_abn_not_on_the_copy_is_not_called_unregistered() {
    let out = run(json!({"abns": ["42000000003"], "as_at": "2026-11-22"})).unwrap();
    let result = &out["charities"][0];
    assert_eq!(result["on_register_snapshot"], false);
    let finding = result["finding"].as_str().unwrap();
    assert!(finding.contains("does not establish that the entity is not a registered charity"));
    assert_eq!(result["limits"], json!(LIMITS));
    let both = run(json!({"abns": ["42000000003", "23000000004"], "as_at": "2026-11-22"})).unwrap();
    for result in both["charities"].as_array().unwrap() {
        let finding = result["finding"].as_str().unwrap().to_lowercase();
        for word in ["eligible", "clear", "exempt", "deductible"] {
            assert!(!finding.contains(word), "{word}: {finding}");
        }
    }
}

#[test]
fn the_copy_answers_from_its_date_to_31_days_after() {
    for day in ["2026-11-22", "2026-12-23"] {
        assert!(
            run(json!({"abns": ["42000000003"], "as_at": day})).is_ok(),
            "{day}"
        );
    }
    assert_eq!(
        code(json!({"abns": ["42000000003"], "as_at": "2026-12-24"})),
        "charity_check.snapshot_stale"
    );
    assert_eq!(
        code(json!({"abns": ["42000000003"], "as_at": "2026-11-21"})),
        "charity_check.as_at_before_snapshot"
    );
}

#[test]
fn inputs_it_cannot_read_are_refused() {
    let many: Vec<&str> = vec!["42000000003"; MAX_ABNS + 1];
    let cases = [
        (
            json!({"abns": [], "as_at": "2026-11-22"}),
            "charity_check.abns_invalid",
        ),
        (
            json!({"abns": many, "as_at": "2026-11-22"}),
            "charity_check.abns_invalid",
        ),
        (
            json!({"abns": "42000000003", "as_at": "2026-11-22"}),
            "charity_check.abns_invalid",
        ),
        (
            json!({"abns": ["42000000004"], "as_at": "2026-11-22"}),
            "input.abn_invalid",
        ),
        (
            json!({"abns": [42000000003_u64], "as_at": "2026-11-22"}),
            "input.abn_invalid",
        ),
        (json!({"as_at": "2026-11-22"}), "input.missing_field"),
        (json!({"abns": ["42000000003"]}), "input.missing_field"),
        (
            json!({"abns": ["42000000003"], "as_at": "22/11/2026"}),
            "input.date_invalid",
        ),
        (
            json!({"abns": ["42000000003"], "as_at": "2026-11-22", "names": []}),
            "input.field_unknown",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(code(input.clone()), expected, "{input}");
    }
    let refused =
        run(json!({"abns": ["42000000003", "42000000004"], "as_at": "2026-11-22"})).unwrap_err();
    assert!(
        refused.message.starts_with("abns[1] "),
        "{}",
        refused.message
    );
}

#[test]
#[should_panic(expected = "charity count")]
fn a_copy_whose_count_disagrees_with_its_lines_does_not_load() {
    parse_register(&FIXTURE.replace("\"charities\":2", "\"charities\":3"));
}

#[test]
#[should_panic(expected = "sorted")]
fn an_unsorted_copy_does_not_load() {
    let swapped = FIXTURE
        .replace("23000000004", "99999999999")
        .replace("61000000002", "23000000004")
        .replace("99999999999", "61000000002");
    parse_register(&swapped);
}
