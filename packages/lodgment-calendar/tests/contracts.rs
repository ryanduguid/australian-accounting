use lodgment_calendar::{api, holidays};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[test]
fn original_unusual_refusal_values_are_escaped_and_preserved() {
    let cases: Value = serde_json::from_str(include_str!("fixtures/refusals.json")).unwrap();
    for case in cases.as_array().unwrap() {
        assert_eq!(
            api::evaluate("due-dates", &case["request"]).unwrap_err(),
            case["error"].as_str().unwrap(),
            "{}",
            case["request"]
        );
    }
}

fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(map) => serde_json::to_value(
            map.iter()
                .map(|(k, v)| (k, sorted(v)))
                .collect::<BTreeMap<_, _>>(),
        )
        .unwrap(),
        Value::Array(items) => items.iter().map(sorted).collect(),
        _ => value.clone(),
    }
}

#[test]
fn original_targeted_contracts() {
    let fixtures: Value =
        serde_json::from_str(include_str!("fixtures/python-contracts.json")).unwrap();
    for (index, case) in fixtures["targeted"].as_array().unwrap().iter().enumerate() {
        let actual = if case["command"] == "roll" {
            holidays::roll(&case["request"])
        } else {
            api::evaluate(case["command"].as_str().unwrap(), &case["request"])
        };
        if let Some(error) = case["error"].as_str() {
            assert_eq!(actual.unwrap_err(), error, "case {index}");
        } else {
            assert_eq!(actual.unwrap(), case["output"], "case {index}");
        }
    }
}

#[test]
fn original_seeded_and_targeted_fact_calendar_matrix() {
    let fixtures: Value =
        serde_json::from_str(include_str!("fixtures/python-contracts.json")).unwrap();
    assert_eq!(fixtures["matrix"].as_array().unwrap().len(), 8636);
    for (index, case) in fixtures["matrix"].as_array().unwrap().iter().enumerate() {
        let mut request = fixtures["requests"][case[0].as_u64().unwrap() as usize].clone();
        request["facts"] = fixtures["fact_sets"][case[1].as_u64().unwrap() as usize].clone();
        let actual = api::evaluate("due-dates", &request).unwrap();
        let digest: String = Sha256::digest(serde_json::to_vec(&sorted(&actual)).unwrap())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            digest,
            case[2].as_str().unwrap(),
            "matrix {index}: {request}\nactual: {actual}"
        );
    }
}

fn request() -> Value {
    json!({"obligation":"bas","entity_type":"trust","financial_year":"2026-27","channel":"self","reporting_cycle":"quarterly"})
}

#[test]
fn json_boundary_rejects_unknown_fields_and_non_boolean_facts() {
    for bad in [json!([]), json!("yes"), json!(1), json!(true)] {
        let mut r = request();
        r["facts"] = bad;
        assert!(
            api::evaluate("due-dates", &r)
                .unwrap_err()
                .contains("facts must be an object")
        );
    }
    for bad in [json!(0), json!(1), json!("false"), json!([]), json!({})] {
        let mut r = request();
        r["facts"] = json!({"lodges_electronically":bad});
        assert!(
            api::evaluate("due-dates", &r)
                .unwrap_err()
                .starts_with("facts.lodges_electronically must be true, false or null")
        );
    }
    let mut r = request();
    r["typo"] = json!(false);
    assert!(
        api::evaluate("due-dates", &r)
            .unwrap_err()
            .contains("unknown request field")
    );
    let mut r = request();
    r["facts"] = json!({"typo":false});
    assert!(
        api::evaluate("due-dates", &r)
            .unwrap_err()
            .contains("unknown fact")
    );
    assert!(api::evaluate("due-dates", &Value::Null).is_err());
    assert!(api::evaluate("wrong", &request()).is_err());
}

#[test]
fn canonical_real_dates_and_holiday_names_are_required() {
    for day in [
        "2027-7-28",
        "2027-02-29",
        "2027-07-28T00:00:00",
        "0000-01-01",
        "+2027-07-28",
        "２０２７-07-28",
    ] {
        let mut r = request();
        r["extra_holidays"] = json!({day:"Fabricated holiday"});
        assert!(api::evaluate("due-dates", &r).is_err(), "{day}");
        let mut r = request();
        r["holidays_complete_until"] = json!(day);
        assert!(api::evaluate("due-dates", &r).is_err(), "{day}");
    }
    for name in [
        "",
        " ",
        "Synthetic\nHoliday",
        "Synthetic\u{202e}Holiday",
        "Synthetic\u{a0}Holiday",
    ] {
        let mut r = request();
        r["extra_holidays"] = json!({"2027-07-28":name});
        assert!(api::evaluate("due-dates", &r).is_err());
    }
    let mut r = request();
    r["extra_holidays"] = json!({"2027-07-28":"  Fabricated holiday  "});
    let actual = api::evaluate("due-dates", &r).unwrap();
    assert_eq!(
        actual["holiday_calendar"]["caller_holidays"]["2027-07-28"],
        "Fabricated holiday (caller-supplied)"
    );
    assert_eq!(actual["holiday_calendar"]["covers_to"], "2027-06-30");
}

#[test]
fn supplied_facts_never_become_missing_and_unknown_rows_never_gain_a_due_date() {
    for facts in [
        json!({}),
        json!({"lodges_electronically":false}),
        json!({"lodges_electronically":true}),
    ] {
        let mut r = request();
        r["facts"] = facts.clone();
        let output = api::evaluate("due-dates", &r).unwrap();
        for row in output["obligations"].as_array().unwrap() {
            if row["status"] == "UNKNOWN" {
                assert!(row["due_date"].is_null());
                assert!(!row["missing"].as_array().unwrap().is_empty());
            }
            for missing in row["missing"].as_array().unwrap() {
                let name = missing.as_str().unwrap().split(':').next().unwrap();
                assert!(facts[name].is_null());
            }
        }
    }
}
