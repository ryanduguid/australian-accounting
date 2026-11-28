// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The client against snapshots of odd shapes and replies the provider
//! should never send. Where the retired Python client had the same code,
//! the expected outcome is what its `LodgeitClient.invoke` gave under
//! CPython 3.15.0 on Linux with the transport replaced by the same 200
//! reply; the OpenAPI and depreciation-range checks are the port's own.

use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::rc::Rc;

use lodgeit_calculator_adapter::client::{Client, Outcome, Status};
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract::{self, Contract};
use lodgeit_calculator_adapter::errors::CONTRACT;
use lodgeit_calculator_adapter::pyjson::{Json, PyStr};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::transport::{Connection, Dial};

const RANGE: &str = "urn:sbrm:calculator:depreciation:range";
const UNSCOPED: &str = "urn:sbrm:period:depreciation:unscoped";
const GOOD: &str = r#"{"manifest": {"m": 1}, "advisory": {"notes": ["n"]}, "a": "1.5"}"#;

fn pts(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

/// One 200 reply with this body for each dial, in order; each request line
/// is recorded.
struct Replies {
    bodies: Vec<Vec<u8>>,
    sent: Vec<Rc<RefCell<Vec<u8>>>>,
}

struct Scripted {
    input: io::Cursor<Vec<u8>>,
    sent: Rc<RefCell<Vec<u8>>>,
}

impl Read for Scripted {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl Write for Scripted {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sent.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Dial for Replies {
    fn dial(&mut self, _: &str, _: u16, _: bool, _: f64) -> io::Result<Box<dyn Connection>> {
        let body = self.bodies.remove(0);
        let mut input = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        input.extend_from_slice(&body);
        let sent = Rc::default();
        self.sent.push(Rc::clone(&sent));
        Ok(Box::new(Scripted {
            input: io::Cursor::new(input),
            sent,
        }))
    }
}

impl Replies {
    fn new(bodies: &[&[u8]]) -> Replies {
        Replies {
            bodies: bodies.iter().map(|body| body.to_vec()).collect(),
            sent: Vec::new(),
        }
    }

    /// The request line of each request sent.
    fn requests(&self) -> Vec<String> {
        self.sent
            .iter()
            .map(|sent| {
                let sent = String::from_utf8(sent.borrow().clone()).unwrap();
                sent.lines().next().unwrap().to_owned()
            })
            .collect()
    }
}

fn config() -> AdapterConfig {
    let mut config = AdapterConfig {
        enabled: true,
        max_attempts: 1.into(),
        ..AdapterConfig::default()
    };
    config.base_url = Some(format!("https://{}", config.allowed_hosts[0]));
    config.checked().unwrap()
}

fn shipped() -> Contract {
    contract::load("lodgeit-calculators", None).unwrap()
}

/// The script's snapshot: one calculator `c` and these response rules.
fn snapshot(calc: Json, rules: Json) -> Contract {
    Contract {
        calculators: Json::obj([("c", calc)]),
        response_contract: rules,
        openapi_input_schemas: None,
        ..shipped()
    }
}

fn known() -> Json {
    Json::obj([(
        "known_top_level_fields",
        Json::Array(["manifest", "advisory", "a"].map(Json::str).to_vec()),
    )])
}

/// `{"supported_periods": ["p1"], "route": "/v1/calculators/x", **fields}`.
fn calc(fields: Vec<(&str, Json)>) -> Json {
    let mut all = vec![
        ("supported_periods", Json::Array(vec![Json::str("p1")])),
        ("route", Json::str("/v1/calculators/x")),
    ];
    all.extend(fields);
    Json::Object(
        all.into_iter()
            .map(|(key, value)| (PyStr::Text(key.into()), value))
            .collect(),
    )
}

fn invoke_on(
    contract: &Contract,
    calc_uri: &str,
    period: &str,
    body: &str,
) -> Result<Outcome, Exc> {
    let config = config();
    let mut replies = Replies::new(&[body.as_bytes()]);
    let mut sleep = |_: f64| Ok(());
    Client {
        config: &config,
        contract,
        dial: &mut replies,
        sleep: &mut sleep,
    }
    .invoke(&pts(calc_uri), &pts(period), &Json::obj([]))
}

fn invoke(calc: Json, rules: Json, body: &str) -> Result<Outcome, Exc> {
    invoke_on(&snapshot(calc, rules), "c", "p1", body)
}

fn raised(result: Result<Outcome, Exc>) -> (&'static str, String) {
    let error = result.expect_err("the call raised nothing");
    (error.class, text(&error.text))
}

fn findings(outcome: &Outcome) -> Vec<String> {
    outcome.findings.iter().map(|f| text(f)).collect()
}

fn ints(values: &[i64]) -> Json {
    Json::Array(values.iter().map(|&n| Json::Int(n.into())).collect())
}

#[test]
fn a_calculator_entry_that_is_not_a_mapping_raises_attribute_error() {
    assert_eq!(
        raised(invoke(Json::Int(5.into()), known(), GOOD)),
        (
            "AttributeError",
            "'int' object has no attribute 'get'".into()
        )
    );
}

#[test]
fn a_route_that_is_not_a_string_raises_on_replace() {
    for (route, name) in [(Json::Int(5.into()), "int"), (ints(&[1]), "list")] {
        let calc = Json::obj([
            ("supported_periods", Json::Array(vec![Json::str("p1")])),
            ("route", route),
        ]);
        assert_eq!(
            raised(invoke(calc, known(), GOOD)),
            (
                "AttributeError",
                format!("'{name}' object has no attribute 'replace'")
            )
        );
    }
}

#[test]
fn required_fields_of_other_iterable_shapes_are_looked_for_as_python_does() {
    for (required, absent) in [
        (ints(&[5]), "5"),
        (Json::obj([("b", Json::Int(1.into()))]), "b"),
        (Json::str("ab"), "b"),
    ] {
        let outcome = invoke(
            calc(vec![("required_decimal_fields", required)]),
            known(),
            GOOD,
        )
        .unwrap();
        assert_eq!(outcome.status, Status::ContractFailure);
        assert_eq!(
            findings(&outcome),
            [format!("required decimal field {absent} is absent")]
        );
        assert!(outcome.values.is_empty());
    }
}

#[test]
fn a_field_list_python_cannot_search_or_unpack_raises_type_error() {
    let not_iterable = || {
        (
            "TypeError",
            "Value after * must be an iterable, not int".to_owned(),
        )
    };
    let cases = [
        (
            vec![
                ("required_decimal_fields", Json::str("ab")),
                ("optional_decimal_fields", ints(&[5])),
            ],
            (
                "TypeError",
                "'in <string>' requires string as left operand, not int".to_owned(),
            ),
        ),
        (
            vec![("required_decimal_fields", Json::Int(5.into()))],
            not_iterable(),
        ),
        (
            vec![("optional_decimal_fields", Json::Int(5.into()))],
            not_iterable(),
        ),
        (
            vec![("required_decimal_fields", Json::Array(vec![ints(&[1])]))],
            (
                "TypeError",
                "cannot use 'list' as a dict key (unhashable type: 'list')".to_owned(),
            ),
        ),
    ];
    for (fields, expected) in cases {
        assert_eq!(raised(invoke(calc(fields), known(), GOOD)), expected);
    }
}

#[test]
fn a_field_named_twice_is_parsed_into_one_value() {
    let fields = vec![
        (
            "required_decimal_fields",
            Json::Array(vec![Json::str("a"), Json::str("a")]),
        ),
        ("optional_decimal_fields", Json::Array(vec![Json::str("a")])),
    ];
    let outcome = invoke(calc(fields), known(), GOOD).unwrap();
    assert_eq!(outcome.status, Status::Computed);
    assert!(outcome.findings.is_empty());
    let values: Vec<(String, String)> = outcome
        .values
        .iter()
        .map(|(name, value)| (text(name), value.to_sci_string()))
        .collect();
    assert_eq!(values, [("a".to_owned(), "1.5".to_owned())]);
}

#[test]
fn advisory_carriers_that_are_not_strings_cannot_be_listed() {
    let rules = Json::obj([("advisory_any_of", ints(&[5]))]);
    assert_eq!(
        raised(invoke(
            calc(Vec::new()),
            rules,
            r#"{"manifest": {"m": 1}, "advisory": {"x": 1}}"#
        )),
        (
            "TypeError",
            "sequence item 0: expected str instance, int found".into()
        )
    );
}

#[test]
fn rules_without_known_fields_note_every_field() {
    let outcome = invoke(calc(Vec::new()), Json::obj([]), GOOD).unwrap();
    assert_eq!(outcome.status, Status::Computed);
    assert_eq!(
        findings(&outcome),
        ["note: response carries unrecorded fields: a, advisory, manifest"]
    );
}

#[test]
fn a_reply_nested_past_the_stack_raises_recursion_error() {
    let (class, message) = raised(invoke(calc(Vec::new()), known(), &"[".repeat(100_000)));
    assert_eq!(class, "RecursionError");
    // CPython 3.15 on Linux: "Stack overflow (used 8156 kB) while decoding a
    // JSON array from a unicode string"; the figure is the stack's size.
    assert!(message.starts_with("Stack overflow (used "), "{message}");
    assert!(
        message.ends_with(" kB) while decoding a JSON array from a unicode string"),
        "{message}"
    );
}

fn range_reply(additions: &str, closing: Option<&str>) -> String {
    let closing = closing.map_or(String::new(), |c| format!(r#", "closing_wdv": "{c}""#));
    format!(
        r#"{{"manifest": {{"m": 1}}, "advisory": {{"notes": ["n"]}}, "opening_wdv": "100.00",
        "cost_additions": "{additions}", "range_dep": "10.00"{closing}}}"#
    )
}

#[test]
fn a_negative_cost_addition_fails_the_depreciation_movement() {
    let outcome = invoke_on(
        &shipped(),
        RANGE,
        UNSCOPED,
        &range_reply("-1.00", Some("89.00")),
    )
    .unwrap();
    assert_eq!(outcome.status, Status::ContractFailure);
    assert_eq!(
        findings(&outcome),
        ["cost additions and depreciation must be non-negative"]
    );
}

#[test]
fn a_movement_missing_a_balance_reports_only_the_absence() {
    let outcome = invoke_on(&shipped(), RANGE, UNSCOPED, &range_reply("0.00", None)).unwrap();
    assert_eq!(outcome.status, Status::ContractFailure);
    assert_eq!(
        findings(&outcome),
        ["required decimal field closing_wdv is absent"]
    );
}

/// What `drift` gave, with its findings as text.
type Drift = Result<(Outcome, Vec<String>), Exc>;

fn drift(contract: &Contract, bodies: &[&[u8]]) -> (Drift, Vec<String>) {
    let config = config();
    let mut replies = Replies::new(bodies);
    let mut sleep = |_: f64| Ok(());
    let result = Client {
        config: &config,
        contract,
        dial: &mut replies,
        sleep: &mut sleep,
    }
    .drift()
    .map(|(outcome, found)| (outcome, found.iter().map(|f| text(f)).collect()));
    (result, replies.requests())
}

#[test]
fn a_snapshot_without_openapi_schemas_never_asks_for_openapi() {
    let contract = Contract {
        openapi_input_schemas: None,
        ..shipped()
    };
    let (result, requests) = drift(&contract, &[b"[]"]);
    let (outcome, found) = result.unwrap();
    assert_eq!(outcome.status, Status::Computed);
    let expected: Vec<String> = contract::compare(&contract, &[])
        .unwrap()
        .iter()
        .map(|f| text(f))
        .collect();
    assert!(!expected.is_empty());
    assert_eq!(found, expected);
    assert_eq!(requests, ["GET /v1/calculators HTTP/1.1"]);
}

#[test]
fn reviewed_openapi_schemas_that_are_not_a_mapping_raise_contract_error() {
    let contract = Contract {
        openapi_input_schemas: Some(Json::Array(Vec::new())),
        ..shipped()
    };
    let (result, requests) = drift(&contract, &[b"[]", b"{}"]);
    let error = result.expect_err("the drift raised nothing");
    assert_eq!(error.class, CONTRACT);
    assert_eq!(
        text(&error.text),
        "reviewed OpenAPI input schemas must be an object"
    );
    assert_eq!(requests[1], "GET /openapi.json HTTP/1.1");
}

#[test]
fn an_openapi_document_with_a_repeated_member_is_not_compared() {
    let (result, requests) = drift(&shipped(), &[b"[]", br#"{"a": 1, "a": 2}"#]);
    let (_, found) = result.unwrap();
    assert_eq!(
        found.last().map(String::as_str),
        Some("live OpenAPI could not be compared (CONTRACT_FAILURE)")
    );
    assert_eq!(requests.len(), 2);
}
