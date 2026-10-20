//! JSON numbers as `json.loads` read them from literal bytes. The differential
//! harness serialises a Python value first, which would normalise `1.2300` or
//! overflow `1e400` before the binary saw it, so these run on raw input here.
//! Every expected value is Python 0.2.2's answer for the same literal.

use ato_benchmark_compare::api::evaluate;
use serde_json::Value;

fn run(command: &str, input: &str) -> Result<Value, String> {
    evaluate(command, &serde_json::from_str(input).expect("valid JSON"))
}

#[test]
fn an_overflowing_float_is_infinity() {
    assert_eq!(run("guard", r#"{"value":1e400}"#).unwrap(), "inf");
    assert_eq!(run("guard", r#"{"value":-1e400}"#).unwrap(), "'-inf");
    assert_eq!(
        run("parse-amount", r#"{"raw":1e400}"#).unwrap_err(),
        "amount: inf is not an amount"
    );
    assert_eq!(
        run("guard", r#"{"value":1.0000000000000001}"#).unwrap(),
        "1.0"
    );
}

#[test]
fn a_passed_through_float_takes_pythons_value_and_spelling() {
    for (literal, want) in [
        ("1.0000000000000001", "1.0"),
        ("1e16", "1e+16"),
        ("1.00", "1.0"),
        ("2", "2"),
    ] {
        let input = format!(
            r#"{{"dataset":{{"year":"2023-24"}},"industry":"Bakeries and hot bread shops","totals":{{"turnover":"850000","other_income":"0"}},"output":"to_dict","unreviewed":{literal}}}"#
        );
        let payload = run("compare-figures", &input).unwrap();
        assert_eq!(
            payload["unreviewed_accounts"].to_string(),
            want,
            "{literal}"
        );
    }
}
