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

#[test]
fn a_non_finite_unreviewed_count_prints_as_python_printed_it() {
    for (literal, shown) in [("1e400", "inf"), (r#"{"n":-1e400}"#, "{'n': -inf}")] {
        let input = format!(
            r#"{{"dataset":{{"year":"2023-24"}},"industry":"Bakeries and hot bread shops","totals":{{"turnover":"850000","other_income":"0"}},"output":"text","unreviewed":{literal}}}"#
        );
        let text = run("compare-figures", &input).unwrap();
        let want = format!("Review outstanding\n  {shown} account(s) still carry");
        assert!(text.as_str().unwrap().contains(&want), "{literal}");
    }
}

#[test]
fn json_integers_stop_at_pythons_4300_digit_limit() {
    let at = format!(r#"{{"value":1{}}}"#, "0".repeat(4299));
    assert_eq!(run("guard", &at).unwrap(), format!("1{}", "0".repeat(4299)));
    let past = format!(r#"{{"value":-1{}}}"#, "0".repeat(4300));
    assert_eq!(
        run("guard", &past).unwrap_err(),
        "ValueError: Exceeds the limit (4300 digits) for integer string conversion: value has \
         4301 digits; use sys.set_int_max_str_digits() to increase the limit"
    );
}
