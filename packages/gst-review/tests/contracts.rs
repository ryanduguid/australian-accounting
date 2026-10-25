use gst_review::{api, bas::Period, coding::Registration, decimal};
use serde_json::{Value, json};

#[test]
fn every_original_behaviour_case_matches_the_python_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/python-contracts.json")).unwrap();
    let tests: std::collections::BTreeSet<_> =
        cases.iter().map(|c| c["test"].as_str().unwrap()).collect();
    assert_eq!(tests.len(), 77);
    for case in cases {
        decimal::with_precision(case["caller_precision"].as_i64().unwrap().into(), || {
            let before = decimal::prec();
            let actual = api::evaluate(case["command"].as_str().unwrap(), &case["input"]);
            if let Some(error) = case.get("error") {
                assert_eq!(
                    actual.unwrap_err(),
                    error.as_str().unwrap(),
                    "{}",
                    case["test"]
                );
            } else {
                let actual = actual.unwrap();
                assert_eq!(actual, case["expected"], "{}", case["test"]);
                if let Some(counts) = actual.get("counts") {
                    assert_eq!(
                        counts
                            .as_object()
                            .unwrap()
                            .keys()
                            .map(String::as_str)
                            .collect::<Vec<_>>(),
                        [
                            "unknown_gst_code",
                            "gst_differs_from_registration",
                            "between_entities",
                            "coded",
                            "suggested",
                            "nothing_suggested"
                        ]
                    );
                }
                if let Some(periods) = actual.get("periods").and_then(Value::as_array) {
                    for period in periods {
                        assert_eq!(
                            period["labels"]
                                .as_object()
                                .unwrap()
                                .keys()
                                .map(String::as_str)
                                .collect::<Vec<_>>(),
                            ["G1", "G2", "G3", "G10", "G11", "1A", "1B"]
                        );
                    }
                }
            }
            assert_eq!(
                decimal::prec(),
                before,
                "caller context changed: {}",
                case["test"]
            );
        });
    }
}

#[test]
fn public_date_helpers_include_both_boundaries() {
    let start = "2026-07-01".parse().unwrap();
    let end = "2026-09-30".parse().unwrap();
    let period = Period {
        label: "Q1".into(),
        start,
        end,
    };
    let registration = Registration {
        registered_from: Some(start),
        registered_to: Some(end),
    };
    for day in [
        start.pred_opt().unwrap(),
        start,
        end,
        end.succ_opt().unwrap(),
    ] {
        let expected = day >= start && day <= end;
        assert_eq!(period.holds(day), expected);
        assert_eq!(registration.covers(day), expected);
    }
    assert!(
        !Registration {
            registered_from: None,
            registered_to: None
        }
        .covers(start)
    );
}

#[test]
fn money_scale_pairing_ties_signed_zero_and_custom_table_order() {
    let input = json!({"registrations":{"A":{"registered_from":null},"B":{"registered_from":null}},"lines":[
        {"id":"out","entity":"A","date":"2026-07-10","amount":"-110.0"},
        {"id":"first","entity":"B","date":"2026-07-09","amount":"110.00"},
        {"id":"second","entity":"B","date":"2026-07-11","amount":"110"},
        {"id":"zero","entity":"A","date":"2026-07-10","amount":"-0.00"},
        {"id":"otherzero","entity":"B","date":"2026-07-10","amount":"0"}]});
    let result = api::evaluate("review-coding", &input).unwrap();
    assert_eq!(result["lines"][0]["counterpart"], "first");
    assert_eq!(result["lines"][2]["counterpart"], Value::Null);
    assert_eq!(result["lines"][3]["counterpart"], Value::Null);
    assert_eq!(result["lines"][3]["amount"], "-0.00");
    let error = api::evaluate(
        "review-coding",
        &json!({"gst_codes":{"a":"z","b":"a'"},"registrations":{},"lines":[]}),
    )
    .unwrap_err();
    assert_eq!(
        error,
        "Unknown GST treatments in the code table: [\"a'\", 'z']."
    );
}

#[test]
fn empty_lodged_period_and_input_period_order_are_retained() {
    let result = api::evaluate(
        "reconcile-bas",
        &json!({"basis":"accruals","lines":[],"periods":[
        {"label":"Q2","start":"2026-10-01","end":"2026-12-31"},
        {"label":"Q1","start":"2026-07-01","end":"2026-09-30"}],"lodged":{"Q2":{}}}),
    )
    .unwrap();
    assert_eq!(result["periods"][0]["period"], "Q2");
    assert_eq!(result["periods"][0]["lodged"], true);
    assert_eq!(result["periods"][1]["lodged"], false);
    assert_eq!(result["no_ledger_lines"], json!(["Q2"]));
}

#[test]
fn malformed_json_types_are_refused_without_panics() {
    for input in [Value::Null, json!([]), json!(false)] {
        assert!(api::evaluate("review-coding", &input).is_err());
    }
    for input in [
        json!({}),
        json!({"lines":null,"registrations":{}}),
        json!({"lines":[],"registrations":false}),
        json!({"lines":[null],"registrations":{}}),
    ] {
        assert!(api::evaluate("review-coding", &input).is_err());
    }
    assert!(api::evaluate("unknown", &json!({})).is_err());
}
