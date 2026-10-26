//! `tests/test_evidence_flags.py`: evidence flags keep their stated boolean
//! meaning across writing and reading.
use std::path::PathBuf;

use num_bigint::BigInt;

use lodgeit_calculator_adapter::cli;
use lodgeit_calculator_adapter::client::{self, Outcome, Status};
use lodgeit_calculator_adapter::evidence;
use lodgeit_calculator_adapter::pyjson::{Json, PyStr};
use lodgeit_calculator_adapter::pypath::PyPath;

fn pts(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

/// `INVALID_FLAGS`: `"false", "true", 0, 1, 0.0, 1.0, None, [], {}`.
fn invalid_flags() -> Vec<Json> {
    vec![
        Json::str("false"),
        Json::str("true"),
        Json::Int(BigInt::from(0)),
        Json::Int(BigInt::from(1)),
        Json::Float(0.0),
        Json::Float(1.0),
        Json::Null,
        Json::Array(Vec::new()),
        Json::Object(Vec::new()),
    ]
}

fn outcome(status: Status) -> Outcome {
    Outcome {
        status,
        calculator: None,
        period: None,
        request_body: None,
        request_json: None,
        raw_response: Json::Null,
        http_status: None,
        result: Json::Null,
        values: Vec::new(),
        manifest: Json::Null,
        advisory: Json::Null,
        upstream_refusal_class: Json::Null,
        findings: Vec::new(),
        observed_at: client::now(),
        elapsed_ms: None,
        contract_snapshot: Json::Null,
        contract_sha256: String::new(),
        base_url: None,
    }
}

fn computed_outcome() -> Outcome {
    Outcome {
        calculator: Some(pts("urn:example:calculator")),
        period: Some(pts("urn:example:period:2026-07")),
        manifest: Json::obj([("calculator", Json::str("urn:example:calculator"))]),
        advisory: Json::obj([(
            "notes",
            Json::Array(vec![Json::str("Fabricated evidence test.")]),
        )]),
        ..outcome(Status::Computed)
    }
}

fn build(outcome: &Outcome, synthetic: bool) -> Json {
    evidence::build(outcome, "fabricated", synthetic, &Json::Null).expect("a record builds")
}

fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lodgeit-evidence-flags-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test folder");
    dir
}

fn member_mut<'a>(value: &'a mut Json, key: &str) -> &'a mut Json {
    let Json::Object(pairs) = value else {
        panic!("not an object");
    };
    &mut pairs
        .iter_mut()
        .find(|(k, _)| k.as_str() == Some(key))
        .unwrap_or_else(|| panic!("no member {key}"))
        .1
}

fn set(value: &mut Json, key: &str, item: Json) {
    let Json::Object(pairs) = value else {
        panic!("not an object");
    };
    match pairs.iter_mut().find(|(k, _)| k.as_str() == Some(key)) {
        Some(pair) => pair.1 = item,
        None => pairs.push((PyStr::Text(key.to_owned()), item)),
    }
}

fn remove(value: &mut Json, key: &str) {
    let Json::Object(pairs) = value else {
        panic!("not an object");
    };
    pairs.retain(|(k, _)| k.as_str() != Some(key));
}

/// The object holding the last part of a dotted field under `calculation`,
/// and that last part.
fn target<'a, 'f>(record: &'a mut Json, field: &'f str) -> (&'a mut Json, &'f str) {
    let mut node = member_mut(record, "calculation");
    let parts: Vec<&str> = field.split('.').collect();
    for part in &parts[..parts.len() - 1] {
        node = member_mut(node, part);
    }
    (node, parts[parts.len() - 1])
}

fn redigest(record: &mut Json) {
    let digest = evidence::digest_of(record).expect("a digest");
    set(record, "calculation_sha256", Json::str(&digest));
}

fn findings(record: &Json) -> Vec<String> {
    evidence::verify(record)
        .expect("verify runs")
        .iter()
        .map(|f| text(f))
        .collect()
}

#[test]
fn verify_reports_a_present_non_boolean_flag() {
    for field in ["synthetic_input", "validation.accepted"] {
        for invalid in invalid_flags() {
            let mut record = build(&computed_outcome(), true);
            let (node, last) = target(&mut record, field);
            set(node, last, invalid.clone());
            redigest(&mut record);
            let found = findings(&record);
            assert!(
                found
                    .iter()
                    .any(|f| f.contains(field) && f.contains("bool")),
                "{field} = {invalid:?}: {found:?}"
            );
        }
    }
}

#[test]
fn verify_reports_a_computed_record_the_producer_did_not_accept() {
    let mut record = build(&computed_outcome(), true);
    let (node, last) = target(&mut record, "validation.accepted");
    set(node, last, Json::Bool(false));
    redigest(&mut record);
    let found = findings(&record);
    assert!(found.iter().any(|f| f.contains("accepted")), "{found:?}");
}

#[test]
fn literal_flags_survive_a_file_round_trip() {
    for synthetic in [false, true] {
        let record = build(&computed_outcome(), synthetic);
        let original = record.clone();
        let dir = tmp_dir(&format!("round-trip-{synthetic}"));
        let path = PyPath::new(dir.join("evidence.json").to_str().expect("a UTF-8 path"));
        evidence::write(&record, &path).expect("the record is written");
        let reloaded = evidence::read(&path).unwrap_or_else(|_| panic!("the record reads back"));
        assert_eq!(reloaded, original);
        assert_eq!(
            reloaded.at("calculation").at("synthetic_input"),
            &Json::Bool(synthetic)
        );
        assert!(findings(&reloaded).is_empty());
        assert!(record == reloaded && reloaded == original);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn omitted_legacy_flags_keep_their_verification_result() {
    for field in ["synthetic_input", "validation.accepted", "validation"] {
        let mut record = build(&computed_outcome(), true);
        let (node, last) = target(&mut record, field);
        remove(node, last);
        redigest(&mut record);
        let original = record.clone();
        assert!(findings(&record).is_empty(), "{field}");
        assert_eq!(record, original);
    }
}

#[test]
fn non_computed_outcomes_keep_their_accepted_false_flag() {
    for status in [
        Status::UpstreamRefused,
        Status::UpstreamRejected,
        Status::UpstreamNotFound,
        Status::UpstreamThrottled,
        Status::UpstreamUnavailable,
        Status::ContractFailure,
        Status::RefusedToSend,
    ] {
        let record = build(&outcome(status), true);
        assert_eq!(
            record.at("calculation").at("validation").at("accepted"),
            &Json::Bool(false),
            "{status:?}"
        );
        assert!(findings(&record).is_empty(), "{status:?}");
    }
}

#[test]
fn verify_command_refuses_the_contradictory_record() {
    let mut record = build(&computed_outcome(), true);
    let (node, last) = target(&mut record, "validation.accepted");
    set(node, last, Json::Bool(false));
    redigest(&mut record);
    let dir = tmp_dir("verify-command");
    let path = dir.join("evidence.json");
    let path = path.to_str().expect("a UTF-8 path");
    evidence::write(&record, &PyPath::new(path)).expect("the record is written");
    let args: Vec<String> = ["verify", "--evidence", path].map(str::to_owned).to_vec();
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    assert_eq!(cli::run_with(&args, &mut stdout, &mut stderr), 1);
    let stdout = String::from_utf8(stdout).expect("UTF-8 output");
    assert!(stdout.contains("validation.accepted=false"), "{stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A repeated value name is recorded once, at its first position, with its
/// last value, as Python's dict holds it.
#[test]
fn a_repeated_value_is_recorded_once_with_its_last_figure() {
    use lodgeit_calculator_adapter::decimal::Decimal;
    let dec = |text: &str| Decimal::parse_exact(text).expect("a literal");
    let outcome = Outcome {
        values: vec![
            (pts("b"), dec("1.00")),
            (pts("a"), dec("2.00")),
            (pts("b"), dec("3.00")),
        ],
        ..computed_outcome()
    };
    let record = build(&outcome, true);
    assert_eq!(
        record.at("calculation").at("normalised").at("values"),
        &Json::obj([("a", Json::str("2.00")), ("b", Json::str("3.00"))])
    );
    assert!(evidence::verify(&record).unwrap().is_empty());
}
