//! The differential harness's vector runner: one JSON case per input line,
//! one JSON result per output line, as the Python runner writes them for the
//! same cases. Built only with the `differential` feature.
use std::io::{BufRead, Write};

use crate::config::{AdapterConfig, from_environment};
use crate::contract;
use crate::decimal::Decimal;
use crate::pyjson::{self, DecodeError, Json, Numbers};
use crate::pyops::{Exc, R, json_str, pts};
use crate::pypath::PyPath;
use crate::pyvalue::{dumps_line, repr};
use crate::wire;

fn text(value: &Json) -> String {
    value.as_str().unwrap_or_default().to_owned()
}

fn decoded(text: &str, numbers: Numbers) -> R<Json> {
    pyjson::loads_numbers(text, numbers).map_err(|error| match error {
        DecodeError::Invalid(m) if m.starts_with("Exceeds the limit") => {
            Exc::new("ValueError", pts(&m))
        }
        DecodeError::Invalid(m) => Exc::new("json.decoder.JSONDecodeError", pts(&m)),
        DecodeError::Recursion(kind) => {
            Exc::new("RecursionError", pts(&pyjson::recursion_message(kind)))
        }
        DecodeError::Hook(m) => Exc::new("ValueError", pts(&m)),
        DecodeError::Duplicate => unreachable!("no duplicate hook"),
    })
}

fn strs(values: impl IntoIterator<Item = String>) -> Json {
    Json::Array(values.into_iter().map(|s| Json::str(&s)).collect())
}

fn wire_error(error: wire::WireError) -> Exc {
    Exc::new("lodgeitadapter.decimals.DecimalWireError", error.0)
}

fn run_case(case: &Json) -> R<Json> {
    let field = |name: &str| case.get(name).unwrap_or(&Json::Null);
    let loaded = || {
        contract::load(
            &text(field("name")),
            Some(&PyPath::new(&text(field("dir")))),
        )
    };
    Ok(match text(field("op")).as_str() {
        "check_url" => {
            let mut config = AdapterConfig {
                allow_loopback: field("loopback").truthy(),
                ..AdapterConfig::default()
            };
            if let Json::Array(hosts) = field("hosts") {
                config.allowed_hosts = hosts.iter().map(text).collect();
            }
            config.check_url(&text(field("url")), field("route").truthy())?;
            Json::Null
        }
        "segments" => strs(crate::config::segments(&text(field("path")))),
        "env" => {
            let Json::Object(pairs) = field("env") else {
                unreachable!("env is an object")
            };
            let env = |name: &str| {
                pairs
                    .iter()
                    .find(|(k, _)| k.as_str() == Some(name))
                    .map(|(_, v)| text(v))
            };
            let c = from_environment(env)?;
            Json::Array(vec![
                Json::Bool(c.enabled),
                c.base_url.as_deref().map_or(Json::Null, Json::str),
                strs(c.allowed_hosts.clone()),
                Json::Bool(c.allow_loopback),
                Json::str(&crate::py::float_repr(c.read_timeout)),
                Json::str(&c.max_response_bytes.to_string()),
                Json::str(&c.max_attempts.to_string()),
            ])
        }
        "parse_decimal" => {
            let value = decoded(&text(field("json")), Numbers::Native)?;
            let parsed =
                wire::parse_decimal(&value, &pts(&text(field("field")))).map_err(wire_error)?;
            Json::str(&parsed.to_sci_string())
        }
        "wire" => {
            let body = decoded(&text(field("json")), Numbers::DecimalFloats)?;
            json_str(wire::dumps(&body, &pts(&text(field("field")))).map_err(wire_error)?)
        }
        "body" => Json::str(&repr(&decoded(
            &text(field("json")),
            Numbers::DecimalFloats,
        )?)),
        "response" => Json::str(&repr(&decoded(&text(field("json")), Numbers::Text)?)),
        "writable" => {
            let value = Decimal::parse_exact(&text(field("decimal"))).expect("a decimal literal");
            wire::check_writable(&value, &pts("f")).map_err(wire_error)?;
            Json::str(&wire::format_number(&value))
        }
        "load" => {
            let c = loaded()?;
            Json::Array(vec![
                Json::str(&repr(&c.provider)),
                Json::str(&repr(&c.snapshot_id)),
                Json::str(&repr(&c.read_at)),
                Json::str(&repr(&c.read_by)),
                Json::str(&repr(&c.calculators)),
                Json::str(&repr(&c.response_contract)),
                strs(c.notes.iter().map(repr)),
                Json::str(&c.sha256),
                Json::str(&c.path.display()),
            ])
        }
        "compare" => {
            let c = loaded()?;
            let Json::Array(live) = decoded(&text(field("live")), Numbers::Native)? else {
                unreachable!("live listings are arrays")
            };
            Json::Array(
                contract::compare(&c, &live)?
                    .into_iter()
                    .map(json_str)
                    .collect(),
            )
        }
        "check_period" => {
            let c = loaded()?;
            c.check_period(&pts(&text(field("calc"))), &pts(&text(field("period"))))?;
            Json::Null
        }
        "calculator" => {
            let c = loaded()?;
            Json::str(&repr(c.calculator(&pts(&text(field("calc"))))?))
        }
        other => unreachable!("unknown op {other}"),
    })
}

/// Read cases from `input` and write each result to `output`.
pub fn run(input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
    for line in input.lines() {
        let case = pyjson::loads_standard(&line?).expect("a case is JSON");
        let result = match run_case(&case) {
            Ok(value) => Json::obj([("ok", value)]),
            Err(error) => Json::obj([(
                "error",
                Json::Array(vec![Json::str(error.class), json_str(error.text)]),
            )]),
        };
        let line: String = dumps_line(&result, true)
            .into_iter()
            .filter_map(char::from_u32)
            .collect();
        writeln!(output, "{line}")?;
    }
    Ok(())
}
