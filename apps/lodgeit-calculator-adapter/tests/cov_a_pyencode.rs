// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The C encoder port: non-finite floats and Decimals under each option set,
//! the repr stack check, and the RecursionErrors that replace an encoding
//! error deep in a value. Depths follow the module's measured stack costs:
//! 144 bytes a list and 256 a dict on Linux, 304 and 448 (plus 16 under a
//! dict) on Windows.
use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::pyencode::{Options, dumps, repr_fits};
use lodgeit_calculator_adapter::pyjson::Json;
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::pyvalue::points_lossy;

const USED_KB: u32 = if cfg!(windows) { 1952 } else { 8156 };

fn written(value: &Json, options: &Options) -> String {
    points_lossy(&dumps(value, options).expect("encodes"))
}

fn failure(value: &Json, options: &Options) -> (String, String) {
    let error: Exc = dumps(value, options).expect_err("refused");
    (error.class.to_owned(), points_lossy(&error.text))
}

/// Runs `f` on a thread with room for the encoder's recursion.
fn deep<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(f)
        .expect("spawn")
        .join()
        .expect("no panic")
}

fn nested_arrays(levels: usize, inner: Json) -> Json {
    (1..levels).fold(Json::Array(vec![inner]), |v, _| Json::Array(vec![v]))
}

#[test]
fn printed_output_writes_non_finite_floats_and_decimals_as_strings() {
    let value = Json::Array(vec![
        Json::Float(f64::NAN),
        Json::Float(f64::INFINITY),
        Json::Float(f64::NEG_INFINITY),
        Json::Dec(Decimal::parse("1.0").expect("valid")),
    ]);
    assert_eq!(
        written(&value, &Options::PRINTED),
        "[\n  NaN,\n  Infinity,\n  -Infinity,\n  \"1.0\"\n]"
    );
}

#[test]
fn a_decimal_without_default_str_is_a_type_error() {
    let value = Json::Dec(Decimal::parse("1.0").expect("valid"));
    assert_eq!(
        failure(&value, &Options::INDENTED),
        (
            "TypeError".to_owned(),
            "Object of type Decimal is not JSON serializable".to_owned()
        )
    );
}

#[test]
fn repr_fits_checks_every_nested_container_against_the_limit() {
    let list_in_list = Json::Array(vec![Json::Array(vec![])]);
    assert!(!repr_fits(&list_in_list, 0));
    assert!(repr_fits(&list_in_list, 1000));
    let dict = Json::obj([("a", Json::Array(vec![]))]);
    assert!(!repr_fits(&dict, 100));
    assert!(repr_fits(&dict, 1000));
    assert!(repr_fits(&Json::Object(vec![]), 0));
}

#[test]
fn nesting_past_the_encoders_budget_is_a_recursion_error() {
    let (class, text) = deep(|| failure(&nested_arrays(60_000, Json::Null), &Options::CANONICAL));
    assert_eq!(class, "RecursionError");
    assert_eq!(
        text,
        format!("Stack overflow (used {USED_KB} kB) while encoding a JSON object")
    );
}

#[test]
fn a_refused_nan_near_the_budget_fails_its_repr_instead() {
    // The NaN's message repr is checked 784 bytes (592 on Windows) deeper than
    // the lists around it, which still fit.
    let levels = if cfg!(windows) { 6_565 } else { 57_950 };
    let (class, text) = deep(move || {
        failure(
            &nested_arrays(levels, Json::Float(f64::NAN)),
            &Options::CANONICAL,
        )
    });
    assert_eq!(class, "RecursionError");
    assert_eq!(
        text,
        format!("Stack overflow (used {USED_KB} kB) while getting the repr of an object")
    );
    // Shallow, the same NaN is refused by name.
    assert_eq!(
        failure(
            &Json::Array(vec![Json::Float(f64::NAN)]),
            &Options::CANONICAL
        )
        .1,
        "Out of range float values are not JSON compliant: nan"
    );
}

#[test]
fn a_dict_note_near_the_budget_fails_its_repr_instead() {
    let levels = if cfg!(windows) { 4_301 } else { 32_598 };
    let (class, text) = deep(move || {
        let value = (1..levels).fold(Json::obj([("k", Json::Float(f64::NAN))]), |v, _| {
            Json::obj([("k", v)])
        });
        failure(&value, &Options::CANONICAL)
    });
    assert_eq!(class, "RecursionError");
    assert_eq!(
        text,
        format!("Stack overflow (used {USED_KB} kB) while getting the repr of an object")
    );
}
