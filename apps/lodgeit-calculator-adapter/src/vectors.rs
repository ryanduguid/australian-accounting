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
        "cli" => {
            // Each file is hex bytes, or null for a directory; all are
            // relative to the working directory and removed afterwards.
            let files = match field("files") {
                Json::Object(pairs) => pairs.clone(),
                _ => Vec::new(),
            };
            for (path, content) in &files {
                let path = std::path::Path::new(path.as_str().expect("an ASCII path"));
                match content {
                    Json::Null => std::fs::create_dir_all(path),
                    hex => std::fs::write(path, unhex(&text(hex))),
                }
                .expect("write a case file");
            }
            let argv: Vec<String> = match field("argv") {
                Json::Array(items) => items.iter().map(text).collect(),
                _ => Vec::new(),
            };
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            let code = crate::cli::run_with(&argv, &mut stdout, &mut stderr);
            for (path, content) in &files {
                let path = std::path::Path::new(path.as_str().expect("an ASCII path"));
                let _ = match content {
                    Json::Null => std::fs::remove_dir_all(path),
                    _ => std::fs::remove_file(path),
                };
            }
            Json::Array(vec![
                Json::Int(code.into()),
                Json::str(&recorded(&stdout, 65536)),
                Json::str(&recorded(&collapsed(&stderr), 1 << 20)),
            ])
        }
        other => unreachable!("unknown op {other}"),
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Runs of identical lines as one line marked ` [x N]`, as the harness
/// records stderr: a Linux traceback can carry 65,000 notes.
fn collapsed(bytes: &[u8]) -> Vec<u8> {
    let lines: Vec<&[u8]> = bytes.split_inclusive(|&b| b == b'\n').collect();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let run = lines[i..]
            .iter()
            .take_while(|&&other| other == line)
            .count();
        if run > 1 {
            let end = line
                .iter()
                .rev()
                .take_while(|&&b| b == b'\n' || b == b'\r')
                .count();
            out.extend_from_slice(&line[..line.len() - end]);
            out.extend_from_slice(format!(" [x {run}]").as_bytes());
            out.extend_from_slice(&line[line.len() - end..]);
        } else {
            out.extend_from_slice(line);
        }
        i += run;
    }
    out
}

/// Output as the harness records it: hex, or its SHA-256 and length past `large`.
fn recorded(bytes: &[u8], large: usize) -> String {
    use sha2::{Digest, Sha256};
    if bytes.len() > large {
        format!("sha256:{}:{}", hex(&Sha256::digest(bytes)), bytes.len())
    } else {
        hex(bytes)
    }
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
        .collect()
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
