//! `evidence.py`: local, digest-bound evidence of one remote call.
//!
//! `calculation` is the deterministic content and `calculation_sha256` its
//! digest, which a consumer pins; `observation` changes between identical
//! calls and is excluded. An evidence file is untrusted input, so `verify`
//! reports what is wrong with it and fails only where Python's did.
use sha2::{Digest, Sha256};

use crate::py;
use crate::pyencode::{self, Options};
use crate::pyjson::{self, DecodeError, Json};
use crate::pyops::{Exc, R, S, pts};
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
