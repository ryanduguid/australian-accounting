//! JSON numbers as `json.loads` read them from literal bytes; the differential
//! harness serialises Python values first, which would hide these spellings.
//! Expected messages are Python 0.2.0's `gst` on the same literals.

use australian_tax_calculators::api::evaluate;

#[test]
fn a_period_prints_as_pythons_repr_of_the_decoded_value() {
    for (year, shown) in [
        ("1e400", "inf"),
        ("-1e400", "-inf"),
        ("2025.5", "2025.5"),
        ("1.0000000000000001", "1.0"),
        ("-0.0", "-0.0"),
        (r#"[1e400,{"b":1,"a":2}]"#, "[inf, {'b': 1, 'a': 2}]"),
    ] {
        let input = format!(
            r#"{{"amount":"1","gst_inclusive":true,"scope_confirmed":true,"year":{year}}}"#
        );
        let err = evaluate("gst", &serde_json::from_str(&input).unwrap()).unwrap_err();
        assert_eq!(
            err,
            format!("Unsupported period {shown}; supported: 2025-26."),
            "{year}"
        );
    }
}
