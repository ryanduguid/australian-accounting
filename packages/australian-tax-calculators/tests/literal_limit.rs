//! Python's `json.loads` refuses an integer token of more than 4,300 digits.

#[test]
fn json_integers_stop_at_pythons_4300_digit_limit() {
    let input = format!(
        r#"{{"amount":"1","gst_inclusive":true,"scope_confirmed":true,"year":1{}}}"#,
        "0".repeat(4300)
    );
    let value: serde_json::Value = serde_json::from_str(&input).unwrap();
    assert_eq!(
        australian_tax_calculators::api::evaluate("gst", &value).unwrap_err(),
        concat!(
            "ValueError: Exceeds the limit (4300 digits) for integer string conversion: ",
            "value has 4301 digits; use sys.set_int_max_str_digits() to increase the limit"
        )
    );
}
