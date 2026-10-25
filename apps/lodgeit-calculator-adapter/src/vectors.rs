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

/// Every path under the working directory, bar the package's own code: the
/// Python runner's `created()`.
fn created() -> std::collections::BTreeSet<std::path::PathBuf> {
    let skip = std::path::Path::new(".").join("pkg").join("lodgeitadapter");
    let mut found = std::collections::BTreeSet::new();
    let mut pending = vec![std::path::PathBuf::from(".")];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = dir.join(entry.file_name());
            if entry.file_type().is_ok_and(|t| t.is_dir()) && path != skip {
                pending.push(path.clone());
            }
            if path != skip {
                found.insert(path);
            }
        }
    }
    found
}

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
        "remote" => {
            // The CLI over real sockets to the harness's loopback server,
            // with the case's environment and each backoff recorded, not slept.
            let env: Vec<(String, String)> = match field("env") {
                Json::Object(pairs) => pairs
                    .iter()
                    .map(|(k, v)| (k.as_str().unwrap_or_default().to_owned(), text(v)))
                    .collect(),
                _ => Vec::new(),
            };
            let lookup = |name: &str| {
                env.iter()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| std::ffi::OsString::from(v))
            };
            let argv: Vec<String> = match field("argv") {
                Json::Array(items) => items.iter().map(text).collect(),
                _ => Vec::new(),
            };
            let mut sleeps = Vec::new();
            let mut sleep = |seconds: f64| {
                sleeps.push(Json::str(&crate::py::float_repr(seconds)));
                Ok(())
            };
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            let code = crate::cli::run_io(
                &argv,
                &mut stdout,
                &mut stderr,
                &mut crate::cli::Io {
                    env: &lookup,
                    dial: &mut crate::dial::Net,
                    sleep: &mut sleep,
                },
            );
            Json::Array(vec![
                Json::Int(code.into()),
                Json::str(&recorded(&stdout, 65536)),
                Json::str(&recorded(&collapsed(&stderr), 1 << 20)),
                Json::Array(sleeps),
            ])
        }
        "cli" => {
            // Each file is hex bytes, or null for a directory; all are
            // relative to the working directory and removed afterwards.
            let files = match field("files") {
                Json::Object(pairs) => pairs.clone(),
                _ => Vec::new(),
            };
            let before = created();
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
            // As the Python runner does: whatever the case wrote, such as an
            // evidence file, must not reach the next case.
            let mut extra: Vec<_> = created().difference(&before).cloned().collect();
            extra.sort_by_key(|path| std::cmp::Reverse(path.as_os_str().len()));
            for path in extra {
                let _ = if path.is_dir() {
                    std::fs::remove_dir(&path)
                } else {
                    std::fs::remove_file(&path)
                };
            }
            Json::Array(vec![
                Json::Int(code.into()),
                Json::str(&recorded(&stdout, 65536)),
                Json::str(&recorded(&collapsed(&stderr), 1 << 20)),
            ])
        }
        "transport" => transport_case(case)?,
        "core" => core_case(case)?,
        "div7a" => div7a_case(case)?,
        "settimeout" => {
            // `s.settimeout(t); s.gettimeout()`, with t given by its bits.
            let bits = text(field("bits")).parse().expect("u64 bits");
            match crate::dial::socket_timeout(f64::from_bits(bits)) {
                Ok(timeout) => {
                    let ns = timeout.as_nanos();
                    // _PyTime_AsSecondsDouble
                    Json::Float(if ns % 1_000_000_000 == 0 {
                        (ns / 1_000_000_000) as f64
                    } else {
                        ns as f64 / 1e9
                    })
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Json::Float(0.0),
                Err(error) => {
                    return Err(match error.into_inner().map(|e| e.downcast()) {
                        Some(Ok(raised)) => {
                            let crate::transport::Raised(exc) = *raised;
                            exc
                        }
                        Some(Err(other)) => Exc::msg("ValueError", &other.to_string()),
                        None => unreachable!("socket_timeout's errors carry a message"),
                    });
                }
            }
        }
        other => unreachable!("unknown op {other}"),
    })
}

fn float(value: &Json) -> f64 {
    match value {
        Json::Float(f) => *f,
        Json::Int(i) => i.to_string().parse().expect("a small int"),
        _ => unreachable!("a number"),
    }
}

fn int(value: &Json) -> num_bigint::BigInt {
    match value {
        Json::Int(i) => i.clone(),
        _ => unreachable!("an int"),
    }
}

/// One scripted connection: the reply in segments, then its end, and the
/// byte count after which a send fails.
struct Scripted {
    reply: std::collections::VecDeque<Vec<u8>>,
    end: String,
    send_fail: Option<usize>,
    sent: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
}

impl std::io::Read for Scripted {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::{Error, ErrorKind};
        let Some(segment) = self.reply.front_mut() else {
            return match self.end.as_str() {
                "eof" => Ok(0),
                "reset" => Err(Error::from(ErrorKind::ConnectionReset)),
                _ => Err(Error::from(ErrorKind::TimedOut)),
            };
        };
        let n = buf.len().min(segment.len());
        buf[..n].copy_from_slice(&segment[..n]);
        segment.drain(..n);
        if segment.is_empty() {
            self.reply.pop_front();
        }
        Ok(n)
    }
}

impl std::io::Write for Scripted {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut sent = self.sent.borrow_mut();
        let room = self
            .send_fail
            .map_or(buf.len(), |limit| limit.saturating_sub(sent.len()));
        if room == 0 && !buf.is_empty() {
            return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
        }
        let n = buf.len().min(room);
        sent.extend_from_slice(&buf[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// The dialler the harness scripts: each dial takes the next step, and
/// one past the script is refused.
struct Script {
    steps: Vec<Json>,
    dials: Vec<Json>,
    sent: Vec<std::rc::Rc<std::cell::RefCell<Vec<u8>>>>,
}

impl crate::transport::Dial for Script {
    fn dial(
        &mut self,
        host: &str,
        port: u16,
        tls: bool,
        timeout: f64,
    ) -> std::io::Result<Box<dyn crate::transport::Connection>> {
        use std::io::{Error, ErrorKind};
        self.dials.push(Json::Array(vec![
            Json::str(host),
            Json::Int(port.into()),
            Json::Bool(tls),
            Json::str(&crate::py::float_repr(timeout)),
        ]));
        let sent = std::rc::Rc::default();
        self.sent.push(std::rc::Rc::clone(&sent));
        let Some(step) = self.steps.get(self.dials.len() - 1) else {
            return Err(Error::from(ErrorKind::ConnectionRefused));
        };
        let step_field = |name: &str| step.get(name).unwrap_or(&Json::Null);
        match text(step_field("connect")).as_str() {
            "refused" => return Err(Error::from(ErrorKind::ConnectionRefused)),
            "timeout" => return Err(Error::from(ErrorKind::TimedOut)),
            _ => {}
        }
        let reply = match step_field("reply") {
            Json::Array(parts) => parts.iter().map(|p| unhex(&text(p))).collect(),
            _ => Default::default(),
        };
        let send_fail = match step_field("send_fail") {
            Json::Int(n) => Some(n.to_string().parse().expect("a small int")),
            _ => None,
        };
        Ok(Box::new(Scripted {
            reply,
            end: text(step_field("end")),
            send_fail,
            sent,
        }))
    }
}

/// `transport.request` over a scripted dialler: the dials, the bytes each
/// connection was sent, the backoff sleeps and the result.
fn transport_case(case: &Json) -> R<Json> {
    use crate::transport::{Call, request};
    let field = |name: &str| case.get(name).unwrap_or(&Json::Null);
    let c = field("config");
    let cf = |name: &str| c.get(name).unwrap_or(&Json::Null);
    let pairs = |value: &Json| -> Vec<(String, String)> {
        match value {
            Json::Array(items) => items
                .iter()
                .map(|pair| match pair {
                    Json::Array(kv) => (text(&kv[0]), text(&kv[1])),
                    _ => unreachable!("a pair"),
                })
                .collect(),
            _ => Vec::new(),
        }
    };
    let config = AdapterConfig {
        enabled: cf("enabled").truthy(),
        base_url: cf("base_url").as_str().map(str::to_owned),
        allowed_hosts: match cf("allowed_hosts") {
            Json::Array(hosts) => hosts.iter().map(text).collect(),
            _ => Vec::new(),
        },
        allow_loopback: cf("allow_loopback").truthy(),
        read_timeout: float(cf("read_timeout")),
        max_response_bytes: int(cf("max_response_bytes")),
        max_attempts: int(cf("max_attempts")),
        retry_backoff_seconds: float(cf("retry_backoff_seconds")),
        user_agent: text(cf("user_agent")),
        extra_headers: pairs(cf("extra_headers")),
    }
    .checked()?;
    let body = field("body").as_str().map(unhex);
    let method = text(field("method"));
    let url = text(field("url"));
    let call = Call {
        method: &method,
        url: &url,
        body: body.as_deref(),
        content_type: field("content_type").as_str(),
        retry_safe: field("retry_safe").truthy(),
        bearer_token: field("bearer").as_str(),
    };
    let mut script = Script {
        steps: match field("script") {
            Json::Array(steps) => steps.clone(),
            _ => Vec::new(),
        },
        dials: Vec::new(),
        sent: Vec::new(),
    };
    let mut sleeps = Vec::new();
    // `real_sleep` cases (non-finite backoff only) reach `time.sleep`'s errors.
    let real = field("real_sleep").truthy();
    let result = request(&config, &mut script, &call, &mut |seconds| {
        sleeps.push(Json::str(&crate::py::float_repr(seconds)));
        if real {
            crate::dial::sleep(seconds)
        } else {
            Ok(())
        }
    });
    let result = match result {
        Ok(raw) => Json::obj([(
            "raw",
            Json::Array(vec![
                Json::Int(raw.status.into()),
                Json::Array(
                    raw.headers
                        .iter()
                        .map(|(k, v)| Json::Array(vec![Json::str(k), Json::str(v)]))
                        .collect(),
                ),
                Json::str(&hex(&raw.body)),
                Json::str(&raw.url),
                Json::Int(raw.elapsed_ms.into()),
            ]),
        )]),
        Err(error) => Json::obj([(
            "error",
            Json::Array(vec![Json::str(error.class), json_str(error.text)]),
        )]),
    };
    Ok(Json::obj([
        ("dials", Json::Array(script.dials)),
        (
            "sent",
            Json::Array(
                script
                    .sent
                    .iter()
                    .map(|s| Json::str(&hex(&s.borrow())))
                    .collect(),
            ),
        ),
        ("sleeps", Json::Array(sleeps)),
        ("result", result),
    ]))
}

/// `CoreClient.list_clients` over a scripted dialler: the dials, the bytes
/// each connection was sent and the page or the `CoreError`.
fn core_case(case: &Json) -> R<Json> {
    use crate::core::{CoreClient, CoreConfig, CoreContinuation};
    let field = |name: &str| case.get(name).unwrap_or(&Json::Null);
    let c = field("config");
    let config = CoreConfig::new(
        c.at("enabled").truthy(),
        float(c.at("read_timeout")),
        int(c.at("max_response_bytes"))
            .to_string()
            .parse()
            .expect("a small int"),
    )?;
    let continuation = field("continuation")
        .as_str()
        .map(|value| CoreContinuation {
            value: value.to_owned(),
        });
    let mut script = Script {
        steps: match field("script") {
            Json::Array(steps) => steps.clone(),
            _ => Vec::new(),
        },
        dials: Vec::new(),
        sent: Vec::new(),
    };
    // Past i64 is out of range either way.
    let size = int(field("page_size"))
        .to_string()
        .parse()
        .unwrap_or(i64::MAX);
    let result = match (CoreClient { config }).list_clients_via(
        &mut script,
        &text(field("bearer")),
        size,
        continuation.as_ref(),
    ) {
        Ok(page) => {
            let optional = |value: &Option<String>| value.as_deref().map_or(Json::Null, Json::str);
            let items = page.items.iter().map(|item| {
                Json::Array(vec![
                    Json::str(&item.id),
                    optional(&item.code),
                    Json::Bool(item.archived),
                    Json::Bool(item.private),
                ])
            });
            let token = page.continuation.map(|c| c.value);
            Json::obj([(
                "page",
                Json::Array(vec![Json::Array(items.collect()), optional(&token)]),
            )])
        }
        Err(error) => Json::obj([(
            "core",
            Json::Array(vec![
                Json::str(error.category.as_str()),
                error
                    .http_status
                    .map_or(Json::Null, |s| Json::Int(s.into())),
            ]),
        )]),
    };
    Ok(Json::obj([
        ("dials", Json::Array(script.dials)),
        (
            "sent",
            Json::Array(
                script
                    .sent
                    .iter()
                    .map(|s| Json::str(&hex(&s.borrow())))
                    .collect(),
            ),
        ),
        ("result", result),
    ]))
}

/// The Division 7A trial over a case file: per case, the request body's
/// `repr` and the comparison, against no provider or a fabricated outcome.
fn div7a_case(case: &Json) -> R<Json> {
    use crate::client::{Outcome, Status};
    use crate::trials::div7a;
    let field = |name: &str| case.get(name).unwrap_or(&Json::Null);
    let cases = div7a::load_cases(Some(&text(field("file"))))?;
    let given = field("outcome");
    let outcome = (given != &Json::Null).then(|| Outcome {
        status: [
            Status::Computed,
            Status::UpstreamRefused,
            Status::UpstreamRejected,
            Status::UpstreamNotFound,
            Status::UpstreamThrottled,
            Status::UpstreamUnavailable,
            Status::ContractFailure,
            Status::RefusedToSend,
        ]
        .into_iter()
        .find(|s| s.as_str() == text(given.at("status")))
        .expect("a status"),
        calculator: None,
        period: None,
        request_body: None,
        request_json: None,
        raw_response: Json::Null,
        http_status: None,
        result: given.at("result").clone(),
        values: Vec::new(),
        manifest: Json::Null,
        advisory: Json::Null,
        upstream_refusal_class: given.at("refusal").clone(),
        findings: match given.at("findings") {
            Json::Array(items) => items.iter().map(|f| pts(&text(f))).collect(),
            _ => Vec::new(),
        },
        observed_at: String::new(),
        elapsed_ms: None,
        contract_snapshot: Json::Null,
        contract_sha256: String::new(),
        base_url: None,
    });
    let entries = cases.iter().map(|one| {
        let compared = match &outcome {
            None => div7a::evaluate(one, None),
            Some(outcome) => div7a::run_local(one).and_then(|(verdict, values, reasons)| {
                div7a::evaluate_outcome(one, &verdict, &values, &reasons, outcome)
            }),
        };
        Json::obj([
            ("request", Json::str(&repr(&div7a::build_request(one)))),
            (
                "result",
                match compared {
                    Ok(comparison) => Json::obj([("ok", comparison.to_json())]),
                    Err(error) => Json::obj([(
                        "error",
                        Json::Array(vec![Json::str(error.class), json_str(error.text)]),
                    )]),
                },
            ),
        ])
    });
    Ok(Json::Array(entries.collect()))
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
        output.flush()?;
    }
    Ok(())
}
