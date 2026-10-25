//! `evidence.py`: local, digest-bound evidence of one remote call.
//!
//! `calculation` is the deterministic content and `calculation_sha256` its
//! digest, which a consumer pins; `observation` changes between identical
//! calls and is excluded. An evidence file is untrusted input, so `verify`
//! reports what is wrong with it and fails only where Python's did.
use sha2::{Digest, Sha256};

use crate::client::Outcome;
use crate::py;
use crate::pyencode::{self, Options};
use crate::pyjson::{self, DecodeError, Json};
use crate::pyops::{Exc, R, S, get_attr, json_str, pts};
use crate::pyos::{OsError, Style};
use crate::pypath::PyPath;
use crate::pyvalue::{py_eq, repr, str_points, type_name, utf8_strict_error};

pub const SCHEMA: &str = "lodgeit-calculation-evidence/1";
pub const MAX_LABEL: usize = 120;
/// `str(Status.COMPUTED)`.
const COMPUTED: &str = "COMPUTED";

/// The deepest `repr` check each finding's formatting allows (see
/// [`pyencode::repr_fits`]): the schema's `!r` and the recorded digest's
/// `str()`, which sits 3 frames deeper on Windows. Linux's are the encoder
/// budget plus their offsets from it (see `pyencode`).
#[cfg(windows)]
const SCHEMA_REPR_LIMIT: usize = 1_996_800;
#[cfg(windows)]
const RECORDED_REPR_LIMIT: usize = 1_996_128;
#[cfg(not(windows))]
const SCHEMA_REPR_LIMIT: usize = 8_345_552;
#[cfg(not(windows))]
const RECORDED_REPR_LIMIT: usize = 8_345_472;

fn formatted<T>(value: &Json, limit: usize, text: impl FnOnce() -> T) -> R<T> {
    if pyencode::repr_fits(value, pyjson::shifted(limit)) {
        Ok(text())
    } else {
        Err(pyencode::repr_recursion_error())
    }
}

/// `_canonical(payload)`: sorted keys, no spaces, UTF-8, no NaN.
pub fn canonical(payload: &Json) -> R<Vec<u8>> {
    let text = pyencode::dumps(payload, &Options::CANONICAL)?;
    if let Some(message) = utf8_strict_error(&text) {
        return Err(Exc::msg("UnicodeEncodeError", &message));
    }
    Ok(text
        .iter()
        .filter_map(|&cp| char::from_u32(cp))
        .collect::<String>()
        .into_bytes())
}

/// `digest_of(record)` for a record whose calculation block is present.
pub fn digest_of(record: &Json) -> R<String> {
    let bytes = canonical(record.at("calculation"))?;
    Ok(Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// `_check_label(label)`: a slug of at most 120 characters, or the ValueError's message.
pub fn check_label(label: Option<&Json>) -> Result<(), S> {
    let Some(Json::Str(text)) = label else {
        return Err(pts(&format!(
            "label must be a string, got {}",
            type_name(label.unwrap_or(&Json::Null))
        )));
    };
    let points = text.code_points();
    if points.len() > MAX_LABEL {
        return Err(pts(&format!("label is longer than {MAX_LABEL} characters")));
    }
    // `[a-z0-9]+(?:-[a-z0-9]+)*`, matched in full.
    let slug = text.as_str().is_some_and(|s| {
        s.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
    });
    if !slug {
        return Err(crate::p!(
            "label ",
            repr(&Json::Str(text.clone())),
            " is not a slug. A consumer keys a digest on it, so it is lower-case letters, \
             digits and single hyphens."
        ));
    }
    Ok(())
}

/// `_block(calculation, name, findings)`: one named block, or nothing with a finding.
fn block<'a>(calculation: &'a Json, name: &str, findings: &mut Vec<S>) -> Option<&'a Json> {
    match calculation.get(name) {
        None | Some(Json::Null) => {
            findings.push(pts(&format!("the calculation block has no {name} block")));
            None
        }
        Some(value @ Json::Object(_)) => Some(value),
        Some(other) => {
            findings.push(pts(&format!(
                "{name} is {}, not an object",
                type_name(other)
            )));
            None
        }
    }
}

fn is_str(value: Option<&Json>, text: &str) -> bool {
    value.is_some_and(|v| v.as_str() == Some(text))
}

/// `_json_safe(value)`: each Decimal as its exact digits, so the record is
/// plain JSON.
fn json_safe(value: &Json) -> Json {
    match value {
        Json::Dec(d) => Json::str(&d.fixed_format()),
        Json::Object(pairs) => Json::Object(
            pairs
                .iter()
                .map(|(k, v)| (k.clone(), json_safe(v)))
                .collect(),
        ),
        Json::Array(items) => Json::Array(items.iter().map(json_safe).collect()),
        other => other.clone(),
    }
}

fn opt_str(value: Option<&S>) -> Json {
    value.map_or(Json::Null, |s| json_str(s.clone()))
}

fn opt_int(value: Option<u64>) -> Json {
    value.map_or(Json::Null, |n| Json::Int(n.into()))
}

/// `build(outcome, label=..., synthetic=..., response_contract=...)`: an
/// evidence record of one outcome, with no engine, local result or notes.
/// A label that is not a slug, and a calculation block that cannot be
/// canonicalised, are `ValueError`s.
pub fn build(outcome: &Outcome, label: &str, synthetic: bool, response_contract: &Json) -> R<Json> {
    check_label(Some(&Json::str(label))).map_err(|text| Exc::new("ValueError", text))?;
    // What the reviewed contract required of a 200, so that verify() asks
    // the client's question. A discovery call names no calculator.
    let requires = |name: &str| -> R<bool> {
        if !outcome.calculator.as_ref().is_some_and(|c| !c.is_empty()) {
            return Ok(false);
        }
        if !response_contract.truthy() {
            return Ok(true);
        }
        Ok(get_attr(response_contract, name)?.is_none_or(Json::truthy))
    };
    let provider = Json::obj([
        ("name", Json::str("lodgeit-labs")),
        ("contract_snapshot", outcome.contract_snapshot.clone()),
        ("contract_sha256", Json::str(&outcome.contract_sha256)),
        (
            "requires_manifest",
            Json::Bool(requires("requires_manifest")?),
        ),
        (
            "requires_advisory",
            Json::Bool(requires("requires_advisory")?),
        ),
    ]);
    let values = crate::trials::dict(&outcome.values);
    let mut values: Vec<&(S, crate::decimal::Decimal)> = values.iter().collect();
    values.sort_by(|a, b| a.0.cmp(&b.0));
    let calculation = Json::obj([
        ("schema", Json::str(SCHEMA)),
        ("label", Json::str(label)),
        ("synthetic_input", Json::Bool(synthetic)),
        ("provider", provider),
        (
            "call",
            Json::obj([
                ("calculator", opt_str(outcome.calculator.as_ref())),
                ("period", opt_str(outcome.period.as_ref())),
                ("status", Json::str(outcome.status.as_str())),
                ("http_status", opt_int(outcome.http_status.map(u64::from))),
            ]),
        ),
        (
            "input",
            outcome.request_body.as_ref().map_or(Json::Null, json_safe),
        ),
        ("input_json", opt_str(outcome.request_json.as_ref())),
        (
            "upstream",
            Json::obj([
                ("response", outcome.raw_response.clone()),
                ("manifest", outcome.manifest.clone()),
                ("advisory", outcome.advisory.clone()),
                ("refusal_class", outcome.upstream_refusal_class.clone()),
            ]),
        ),
        (
            "normalised",
            Json::obj([
                (
                    "values",
                    Json::Object(
                        values
                            .into_iter()
                            .map(|(k, v)| {
                                (
                                    crate::pyjson::PyStr::from_code_points(k.clone()),
                                    Json::str(&v.fixed_format()),
                                )
                            })
                            .collect(),
                    ),
                ),
                (
                    "note",
                    Json::str(
                        "Parsed from the response's own decimal strings. The response above is \
                         unaltered.",
                    ),
                ),
            ]),
        ),
        (
            "validation",
            Json::obj([
                (
                    "findings",
                    Json::Array(outcome.findings.iter().cloned().map(json_str).collect()),
                ),
                ("accepted", Json::Bool(outcome.computed())),
            ]),
        ),
        ("engine", Json::Null),
        ("local_result", Json::Null),
        ("notes", Json::Array(Vec::new())),
        (
            "boundary",
            Json::Array(vec![
                Json::str(
                    "Calculated on supplied facts. Not advice, not a lodgement and not a review \
                     sign-off.",
                ),
                Json::str(
                    "A digest proves the bytes are the bytes. It proves nothing about whether \
                     the figure is right.",
                ),
            ]),
        ),
    ]);
    let digest: String = Sha256::digest(canonical(&calculation)?)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(Json::obj([
        ("schema", Json::str(SCHEMA)),
        ("calculation_sha256", Json::str(&digest)),
        ("calculation", calculation),
        (
            "observation",
            Json::obj([
                ("captured_at", Json::str(&outcome.observed_at)),
                ("elapsed_ms", opt_int(outcome.elapsed_ms)),
                (
                    "base_url",
                    outcome.base_url.as_deref().map_or(Json::Null, Json::str),
                ),
                (
                    "note",
                    Json::str(
                        "Observation metadata is excluded from calculation_sha256 on purpose: \
                         it changes between two identical calls.",
                    ),
                ),
            ]),
        ),
    ]))
}

/// `write(record, path)`: the parent folders, then pretty JSON with a
/// trailing newline through `Path.write_text`, which opens the file before
/// it encodes and on Windows writes each `\n` as `\r\n`. Its `OSError` or
/// `UnicodeEncodeError` is raised.
pub fn write(record: &Json, path: &PyPath) -> R<()> {
    crate::pyos::makedirs(&path.parent()).map_err(|e| crate::pyops::os_exc(&e))?;
    let mut text = pyencode::dumps(record, &Options::EVIDENCE)?;
    text.push(u32::from('\n'));
    let os_error = |e: &std::io::Error| {
        crate::pyops::os_exc(&OsError::new(e, Some(&path.display()), Style::Errno))
    };
    let mut file = std::fs::File::create(path.fs()).map_err(|e| os_error(&e))?;
    let mut translated = Vec::with_capacity(text.len());
    for cp in text {
        if cfg!(windows) && cp == u32::from('\n') {
            translated.push(u32::from('\r'));
        }
        translated.push(cp);
    }
    if let Some(message) = utf8_strict_error(&translated) {
        return Err(Exc::msg("UnicodeEncodeError", &message));
    }
    let bytes: String = translated
        .iter()
        .filter_map(|&cp| char::from_u32(cp))
        .collect();
    std::io::Write::write_all(&mut file, bytes.as_bytes()).map_err(|e| os_error(&e))
}

/// `verify(record)`: findings against an evidence record; empty means it
/// hangs together. A RecursionError from canonicalising escapes, as in Python.
pub fn verify(record: &Json) -> R<Vec<S>> {
    let mut findings = Vec::new();
    if !matches!(record, Json::Object(_)) {
        return Ok(vec![pts(&format!(
            "the evidence file is {}, not a JSON object",
            type_name(record)
        ))]);
    }
    if !is_str(record.get("schema"), SCHEMA) {
        let schema = record.get("schema").unwrap_or(&Json::Null);
        findings.push(formatted(schema, SCHEMA_REPR_LIMIT, || {
            crate::p!("unknown evidence schema ", repr(schema))
        })?);
        return Ok(findings);
    }
    let Some(calculation @ Json::Object(_)) = record.get("calculation") else {
        findings.push(pts("no calculation block"));
        return Ok(findings);
    };
    let recorded = record.get("calculation_sha256").unwrap_or(&Json::Null);
    let actual = match digest_of(record) {
        Ok(actual) => actual,
        Err(error) if error.is_any(&["TypeError", "ValueError"]) => {
            findings.push(crate::p!(
                "calculation block cannot be canonicalised: ",
                error.text
            ));
            return Ok(findings);
        }
        Err(error) => return Err(error),
    };
    if !py_eq(recorded, &Json::str(&actual)) {
        findings.push(formatted(recorded, RECORDED_REPR_LIMIT, || {
            crate::p!(
                "calculation_sha256 ",
                str_points(recorded),
                format!(" does not match the calculation block ({actual})")
            )
        })?);
    }
    if !is_str(calculation.get("schema"), SCHEMA) {
        findings.push(pts(
            "the calculation block names a different schema from the record",
        ));
    }
    if let Err(message) = check_label(calculation.get("label")) {
        findings.push(message);
    }
    let empty = Json::Object(Vec::new());
    let call = block(calculation, "call", &mut findings).unwrap_or(&empty);
    let upstream = block(calculation, "upstream", &mut findings).unwrap_or(&empty);
    let provider = block(calculation, "provider", &mut findings).unwrap_or(&empty);
    if calculation
        .get("synthetic_input")
        .is_some_and(|v| !matches!(v, Json::Bool(_)))
    {
        findings.push(pts("synthetic_input must be a boolean"));
    }
    let computed = is_str(call.get("status"), COMPUTED);
    if let Some(validation @ Json::Object(_)) = calculation.get("validation")
        && let Some(accepted) = validation.get("accepted")
    {
        match accepted {
            Json::Bool(false) if computed => {
                findings.push(pts("a COMPUTED result has validation.accepted=false"));
            }
            Json::Bool(_) => {}
            _ => findings.push(pts("validation.accepted must be a boolean")),
        }
    }
    if computed {
        // Records written before the provider block carried these flags all
        // came from a snapshot that requires both, so absent means required.
        let required = |name: &str| provider.get(name).is_none_or(Json::truthy);
        let present = |name: &str| upstream.get(name).is_some_and(Json::truthy);
        if required("requires_manifest") && !present("manifest") {
            findings.push(pts("a computed result with no upstream manifest"));
        }
        if required("requires_advisory") && !present("advisory") {
            findings.push(pts("a computed result with no upstream advisory"));
        }
    }
    Ok(findings)
}

/// Why `read` failed: an `OSError` or `JSONDecodeError`, which the CLI
/// reports, or anything else, which escaped it as a traceback.
pub enum ReadError {
    Reported(String),
    Uncaught(Exc),
}

/// `read(path)`: `json.loads(Path(path).read_text(encoding="utf-8"))`.
pub fn read(path: &PyPath) -> Result<Json, ReadError> {
    let text = py::read_text(&path.fs()).map_err(|error| match error {
        py::ReadError::Os(error) => {
            ReadError::Reported(OsError::new(&error, Some(&path.display()), Style::Errno).text)
        }
        py::ReadError::Decode(error) => {
            ReadError::Uncaught(Exc::msg("UnicodeDecodeError", &error.to_string()))
        }
    })?;
    pyjson::loads_standard(&text).map_err(|error| match error {
        DecodeError::Invalid(m) if m.starts_with("Exceeds the limit") => {
            ReadError::Uncaught(Exc::msg("ValueError", &m))
        }
        DecodeError::Invalid(m) => ReadError::Reported(m),
        DecodeError::Recursion(kind) => {
            ReadError::Uncaught(Exc::msg("RecursionError", &pyjson::recursion_message(kind)))
        }
        other => unreachable!("json.loads has no hooks here: {other:?}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn findings(text: &str) -> Vec<String> {
        verify(&pyjson::loads_standard(text).unwrap())
            .unwrap()
            .iter()
            .map(|f| f.iter().filter_map(|&c| char::from_u32(c)).collect())
            .collect()
    }

    #[test]
    fn a_record_is_checked_against_its_own_digest() {
        let calculation = r#"{"schema": "lodgeit-calculation-evidence/1", "label": "a-1", "call": {"status": "COMPUTED"}, "upstream": {"manifest": 1, "advisory": 0}, "provider": {}, "validation": {"accepted": false}}"#;
        let digest = digest_of(
            &pyjson::loads_standard(&format!(r#"{{"calculation": {calculation}}}"#)).unwrap(),
        )
        .unwrap();
        let record = format!(
            r#"{{"schema": "lodgeit-calculation-evidence/1", "calculation_sha256": "{digest}", "calculation": {calculation}}}"#
        );
        assert_eq!(
            findings(&record),
            [
                "a COMPUTED result has validation.accepted=false",
                "a computed result with no upstream advisory"
            ]
        );
        assert_eq!(
            findings("[1]"),
            ["the evidence file is list, not a JSON object"]
        );
        assert_eq!(
            findings(r#"{"schema": null}"#),
            ["unknown evidence schema None"]
        );
        let nan = r#"{"schema": "lodgeit-calculation-evidence/1", "calculation": {"x": NaN}}"#;
        assert_eq!(
            findings(nan),
            [
                "calculation block cannot be canonicalised: Out of range float values are not JSON compliant: nan"
            ]
        );
    }

    #[test]
    fn labels_are_slugs() {
        let label = |text: &str| check_label(Some(&pyjson::loads_standard(text).unwrap())).is_ok();
        assert!(label(r#""a-1-b2""#));
        assert!(!label(r#""a--b""#) && !label(r#""A""#) && !label(r#""-a""#) && !label("1"));
        assert!(!label(&format!("\"{}\"", "a".repeat(121))));
    }
}
