//! `trials::div7a::load_cases` on case files that differ from the packaged
//! one in a single field. Where the port reads a field as Python's
//! `Decimal(value)` or `json.loads(text)` does, the expected value or error is
//! CPython 3.15's for the same input.
use lodgeit_calculator_adapter::pyjson::{self, Json};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::pyvalue::dumps_line;
use lodgeit_calculator_adapter::trials::div7a::{Case, load_cases};

const CASES: &str = include_str!("../fixtures/div7a-cases.json");

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

/// The packaged file with the first case's `name` set to `value`.
fn with_field(name: &str, value: Json) -> Result<Vec<Case>, Exc> {
    let mut file = pyjson::loads(CASES).expect("the packaged cases");
    let Json::Object(top) = &mut file else {
        panic!("an object")
    };
    let (_, cases) = top
        .iter_mut()
        .find(|(k, _)| k.as_str() == Some("cases"))
        .unwrap();
    let Json::Array(cases) = cases else {
        panic!("a list")
    };
    let Json::Object(first) = &mut cases[0] else {
        panic!("an object")
    };
    first
        .iter_mut()
        .find(|(k, _)| k.as_str() == Some(name))
        .expect(name)
        .1 = value;
    load_cases(Some(&text(&dumps_line(&file, false))))
}

fn refusal(result: Result<Vec<Case>, Exc>) -> (String, String) {
    let e = result.unwrap_err();
    (e.class.to_owned(), text(&e.text))
}

fn json(text: &str) -> Json {
    pyjson::loads(text).unwrap()
}

#[test]
fn an_amount_reads_as_python_decimal_reads_a_bool_a_float_or_a_str() {
    for (given, expected) in [
        ("100000", "100000"),
        ("true", "1"),
        ("1.5", "1.5"),
        ("\"2.50\"", "2.50"),
    ] {
        let cases = with_field("amalgamated_base", json(given)).unwrap();
        assert_eq!(
            cases[0].amalgamated_base.to_sci_string(),
            expected,
            "Decimal({given})"
        );
    }
}

#[test]
fn an_amount_python_decimal_refuses_is_refused() {
    assert_eq!(
        refusal(with_field("amalgamated_base", json("\"x\""))),
        (
            "decimal.InvalidOperation".into(),
            "[<class 'decimal.ConversionSyntax'>]".into()
        )
    );
    assert_eq!(
        refusal(with_field("amalgamated_base", Json::Null)),
        (
            "TypeError".into(),
            "conversion from NoneType to Decimal is not supported".into()
        )
    );
}

/// `Decimal(list)` is CPython's `(sign, digits, exponent)` form. Each expected
/// value is CPython 3.15.0's `str(Decimal(json.loads(given)))`, or its
/// exception as `Class: text`, on Linux and Windows alike.
const LIST_AMOUNTS: [(&str, &str); 23] = [
    (
        "[1, 2]",
        "ValueError: argument must be a sequence of length 3",
    ),
    ("[]", "ValueError: argument must be a sequence of length 3"),
    (
        "[2, [1], 0]",
        "ValueError: sign must be an integer with the value 0 or 1",
    ),
    (
        "[\"0\", [1], 0]",
        "ValueError: sign must be an integer with the value 0 or 1",
    ),
    ("[true, [1, 5], -1]", "-1.5"),
    (
        "[18446744073709551616, [1], 0]",
        "OverflowError: Python int too large to convert to C long",
    ),
    (
        "[0, [1, 2], \"x\"]",
        "ValueError: string argument in the third position must be 'F', 'n' or 'N'",
    ),
    ("[0, [1, 2], \"F\"]", "Infinity"),
    ("[1, [5], \"F\"]", "-Infinity"),
    ("[0, [1, 2], \"n\"]", "NaN12"),
    ("[1, [], \"N\"]", "-sNaN"),
    ("[0, [1], 1.5]", "ValueError: exponent must be an integer"),
    (
        "[0, [1], 18446744073709551616]",
        "OverflowError: Python int too large to convert to C ssize_t",
    ),
    (
        "[0, \"12\", 0]",
        "ValueError: coefficient must be a tuple of digits",
    ),
    ("[0, [], 3]", "0E+3"),
    ("[0, [2, 5], -2]", "0.25"),
    (
        "[0, [1, 10], 0]",
        "ValueError: coefficient must be a tuple of digits",
    ),
    (
        "[0, [1, -1], 0]",
        "ValueError: coefficient must be a tuple of digits",
    ),
    (
        "[0, [1, \"2\"], 0]",
        "ValueError: coefficient must be a tuple of digits",
    ),
    (
        "[0, [18446744073709551616], 0]",
        "OverflowError: Python int too large to convert to C long",
    ),
    (
        "[0, [1, 10], \"F\"]",
        "ValueError: coefficient must be a tuple of digits",
    ),
    ("[0, [0, 0, 7], 0]", "7"),
    (
        "{}",
        "TypeError: conversion from dict to Decimal is not supported",
    ),
];

#[test]
fn an_amount_given_as_a_list_reads_as_python_decimal_reads_a_tuple() {
    for (given, expected) in LIST_AMOUNTS {
        let shown = match with_field("amalgamated_base", json(given)) {
            Ok(cases) => cases[0].amalgamated_base.to_sci_string(),
            Err(e) => format!("{}: {}", e.class, text(&e.text)),
        };
        assert_eq!(shown, expected, "Decimal({given})");
    }
}

/// CPython 3.15.0 on win32 and on Linux: a sign or digit past 2**31 overflows
/// Windows' 32-bit C long, but on a 64-bit long it is only out of range.
#[test]
fn a_list_amount_past_a_32_bit_c_long_overflows_only_on_windows() {
    let sign = "ValueError: sign must be an integer with the value 0 or 1";
    let digit = "ValueError: coefficient must be a tuple of digits";
    let overflow = "OverflowError: Python int too large to convert to C long";
    let past = |in_range: &'static str| if cfg!(windows) { overflow } else { in_range };
    let cases = [
        ("[2147483647, [1], 0]", sign),
        ("[2147483648, [1], 0]", past(sign)),
        ("[-2147483649, [1], 0]", past(sign)),
        ("[0, [2147483647], 0]", digit),
        ("[0, [2147483648], 0]", past(digit)),
        ("[0, [-2147483649], 0]", past(digit)),
    ];
    for (given, expected) in cases {
        let error = with_field("amalgamated_base", json(given))
            .err()
            .expect("refused");
        assert_eq!(
            format!("{}: {}", error.class, text(&error.text)),
            expected,
            "Decimal({given})"
        );
    }
}

#[test]
fn a_field_of_the_wrong_type_names_what_it_should_be() {
    let cases = [
        ("case_id", "1", "TypeError", "a fixture field is not a str"),
        (
            "loan_term_years",
            "\"7\"",
            "TypeError",
            "a fixture field is not an int",
        ),
        (
            "loan_term_years",
            "99999999999999999999",
            "OverflowError",
            "a fixture int is out of range",
        ),
        (
            "is_first_real_myr_year",
            "1",
            "TypeError",
            "is_first_real_myr_year is not a bool",
        ),
        ("repayments", "{}", "TypeError", "repayments is not a list"),
        ("expected", "[]", "AttributeError", "expected has no items"),
    ];
    for (name, value, class, message) in cases {
        assert_eq!(
            refusal(with_field(name, json(value))),
            (class.into(), message.into()),
            "{name} = {value}"
        );
    }
}

#[test]
fn a_case_file_json_loads_refuses_is_refused_with_its_error() {
    assert_eq!(
        refusal(load_cases(Some("x"))),
        (
            "json.decoder.JSONDecodeError".into(),
            "Expecting value: line 1 column 1 (char 0)".into()
        )
    );
    assert_eq!(
        refusal(load_cases(Some(&"1".repeat(5000)))),
        (
            "ValueError".into(),
            "Exceeds the limit (4300 digits) for integer string conversion: value has 5000 \
             digits; use sys.set_int_max_str_digits() to increase the limit"
                .into()
        )
    );
    // CPython's limit is the stack's, so only the class is compared.
    assert_eq!(
        refusal(load_cases(Some(&"[".repeat(1_000_000)))).0,
        "RecursionError"
    );
}
