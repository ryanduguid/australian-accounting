//! `cli.py`: the `lodgeit-adapter` command line. Nothing here reaches the
//! network without `--enable-network`.
//!
//! Exit codes: 0 the command did what it was asked, 1 a refusal or a
//! failure, 2 a usage error. A refusal is not a crash and not a result.
use std::ffi::OsString;
use std::io::Write;

use crate::argparse::{Action, Exit, Namespace, Parser, Value};
use crate::client::{self, Client};
use crate::config::{self, AdapterConfig};
use crate::contract::{self, Contract};
use crate::decimal::Decimal;
use crate::dial::Net;
use crate::errors::ADAPTER_ERROR;
use crate::evidence::{self, ReadError};
use crate::p;
use crate::py;
use crate::pyencode::{self, Options};
use crate::pyjson::{self, DecodeError, Json, Numbers};
use crate::pyops::{Exc, R, S, get_attr, iterate, json_str, pts, py_sorted, strip};
use crate::pyos::{OsError, Style};
use crate::pypath::PyPath;
use crate::pyvalue::{self, repr, str_points, type_name};
use crate::transport::Dial;

const DESCRIPTION: &str =
    "The command line. Nothing here reaches the network without --enable-network.";

/// `_shared_options(root=...)`: the options every subcommand accepts, typed
/// before or after the command word. Only the root copy owns a default; the
/// subcommand copies default to `SUPPRESS`, so a value typed before the
/// command word survives the subparser.
fn shared(parser: Parser, root: bool) -> Parser {
    let own = |action: Action| {
        if root {
            action
        } else {
            action.default(Value::Suppress)
        }
    };
    let contract = Action::option("--contract").help("reviewed snapshot name");
    parser
        .arg(own(
            contract.default(Value::Str("lodgeit-calculators".into()))
        ))
        .arg(own(Action::flag("--enable-network").help(
            "permit this one run to contact the configured service",
        )))
        .arg(own(
            Action::option("--base-url").help("the service base URL; must be on the allowlist")
        ))
        .arg(own(Action::flag("--allow-loopback").help(
            "permit a loopback base URL, for a local stub in tests",
        )))
        .arg(own(
            Action::option("--evidence-out").help("write an evidence file for the call")
        ))
        .arg(own(Action::flag("--not-synthetic").help(
            "record that the input was not fabricated; the flag travels into the evidence",
        )))
}

pub fn build_parser() -> Parser {
    let sub = |p| shared(p, false);
    shared(
        Parser::new("lodgeit-adapter").description(DESCRIPTION),
        true,
    )
    .subcommands("command")
    .sub(
        "contract",
        Some("print the reviewed snapshot, offline"),
        sub,
    )
    .sub("discover", Some("fetch the live calculator listing"), sub)
    .sub(
        "drift",
        Some("compare the live listing with the reviewed snapshot"),
        sub,
    )
    .sub("invoke", Some("call one calculator once"), |p| {
        shared(p, false)
            .arg(Action::option("--calculator").required())
            .arg(Action::option("--period").required())
            .arg(
                Action::option("--body")
                    .required()
                    .help("JSON request body"),
            )
            .arg(
                Action::option("--label")
                    .default(Value::Str("manual-invocation".into()))
                    .help("slug naming this calculation; the consuming pack keys a digest on it"),
            )
    })
    .sub("verify", Some("check an evidence file, offline"), |p| {
        shared(p, false).arg(Action::option("--evidence").required())
    })
}

/// Text-mode stdout: `surrogateescape`, and on Windows each `\n` written as
/// `\r\n` before encoding, so an encoding error counts the added `\r`s.
/// A failed write is kept for the flush at shutdown, where Python reports
/// it; the traceback Python prints when its buffer fills first is not
/// reproduced.
struct Out<W> {
    w: W,
    failed: Option<std::io::Error>,
}

impl<W: Write> Out<W> {
    fn print(&mut self, line: &[u32]) -> R<()> {
        let bytes = pyvalue::encode_print(&line_ends(line))
            .map_err(|m| Exc::msg("UnicodeEncodeError", &m))?;
        if self.failed.is_none()
            && let Err(error) = self.w.write_all(&[&bytes[..], &newlines(b"\n")].concat())
        {
            self.failed = Some(error);
        }
        Ok(())
    }

    /// The flush at shutdown: None, or the exit status 120 after its report.
    fn finish(mut self, stderr: &mut dyn Write) -> Option<i32> {
        let error = match self.failed.take() {
            Some(error) => error,
            None => self.w.flush().err()?,
        };
        Some(flush_failed(stderr, &error))
    }
}

/// Python's report of a stdout that could not be flushed at exit, whose
/// status then becomes 120 whatever the command returned.
fn flush_failed(stderr: &mut dyn Write, error: &std::io::Error) -> i32 {
    let error = OsError::new(error, None, Style::Errno);
    let report = format!(
        "Exception ignored while flushing sys.stdout:\n{}: {}\n",
        error.class(),
        error.text
    );
    let _ = stderr.write_all(&newlines(report.as_bytes()));
    120
}

fn line_ends(text: &[u32]) -> S {
    let mut out = Vec::with_capacity(text.len());
    for &cp in text {
        if cfg!(windows) && cp == 10 {
            out.push(13);
        }
        out.push(cp);
    }
    out
}

fn newlines(bytes: &[u8]) -> Vec<u8> {
    if !cfg!(windows) {
        return bytes.to_vec();
    }
    let mut out = Vec::with_capacity(bytes.len());
    for &b in bytes {
        if b == b'\n' {
            out.push(b'\r');
        }
        out.push(b);
    }
    out
}

/// `print(text, file=sys.stderr)`: `backslashreplace` shows a lone surrogate
/// as an escape.
fn eprint(stderr: &mut dyn Write, text: &[u32]) {
    let mut line: String = text
        .iter()
        .map(|&cp| char::from_u32(cp).map_or_else(|| format!("\\u{cp:04x}"), String::from))
        .collect();
    line.push('\n');
    let _ = stderr.write_all(&newlines(line.as_bytes()));
}

fn text(ns: &Namespace, key: &str) -> String {
    match ns.get(key) {
        Some(Value::Str(s)) => s.clone(),
        _ => String::new(),
    }
}

pub fn main(argv: Vec<OsString>) -> i32 {
    let mut stderr = std::io::stderr();
    // Python keeps undecodable argument bytes as surrogates and can open a
    // path made of them; this port cannot carry them, so it refuses such an
    // argument rather than read or write a file under another name.
    let Some(args) = argv
        .iter()
        .map(|a| a.to_str().map(str::to_owned))
        .collect::<Option<Vec<String>>>()
    else {
        eprint(
            &mut stderr,
            &pts("lodgeit-adapter: an argument is not valid Unicode"),
        );
        return 2;
    };
    run_with(&args, &mut std::io::stdout().lock(), &mut stderr)
}

/// What a run reaches beyond its arguments and streams: the environment,
/// the network and `time.sleep`.
pub struct Io<'a> {
    pub env: &'a dyn Fn(&str) -> Option<OsString>,
    pub dial: &'a mut dyn Dial,
    pub sleep: &'a mut dyn FnMut(f64) -> R<()>,
}

/// `main(argv)` writing to the given streams; returns the exit status.
pub fn run_with(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let env = |name: &str| std::env::var_os(name);
    let mut sleep = crate::dial::sleep;
    let mut io = Io {
        env: &env,
        dial: &mut Net,
        sleep: &mut sleep,
    };
    run_io(args, stdout, stderr, &mut io)
}

/// `run_with` with the environment, dialler and sleep given.
pub fn run_io(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write, io: &mut Io) -> i32 {
    let ns = match build_parser().parse(args) {
        Ok(ns) => ns,
        Err(Exit::Help(help)) => {
            return match stdout
                .write_all(&newlines(help.as_bytes()))
                .and_then(|()| stdout.flush())
            {
                Ok(()) => 0,
                Err(error) => flush_failed(stderr, &error),
            };
        }
        Err(Exit::Error(usage)) => {
            let _ = stderr.write_all(&newlines(usage.as_bytes()));
            return 2;
        }
    };
    let mut out = Out {
        w: stdout,
        failed: None,
    };
    let result = run(&ns, &mut out, stderr, io);
    let code = result.unwrap_or_else(|error| {
        let _ = stderr.write_all(&newlines(error.traceback().as_bytes()));
        1
    });
    out.finish(stderr).unwrap_or(code)
}

fn run(
    ns: &Namespace,
    out: &mut Out<&mut dyn Write>,
    stderr: &mut dyn Write,
    io: &mut Io,
) -> R<i32> {
    let contract = match contract::load(&text(ns, "contract"), None) {
        Ok(contract) => contract,
        Err(error) if error.is(ADAPTER_ERROR) => {
            eprint(stderr, &p!("lodgeit-adapter: ", error.text));
            return Ok(1);
        }
        Err(error) => return Err(error),
    };
    match text(ns, "command").as_str() {
        "contract" => {
            let record = Json::obj([
                ("snapshot_id", contract.snapshot_id.clone()),
                ("read_at", contract.read_at.clone()),
                ("read_by", contract.read_by.clone()),
                ("sha256", Json::str(&contract.sha256)),
                (
                    "calculators",
                    Json::Array(py_sorted(iterate(&contract.calculators)?)?),
                ),
                ("notes", Json::Array(contract.notes.clone())),
            ]);
            out.print(&pyencode::dumps(&record, &Options::INDENTED)?)?;
            Ok(0)
        }
        "verify" => verify(&PyPath::new(&text(ns, "evidence")), out, stderr),
        _ => remote(ns, &contract, out, stderr, io),
    }
}

fn flag(ns: &Namespace, key: &str) -> bool {
    matches!(ns.get(key), Some(Value::Bool(true)))
}

/// `discover`, `drift` and `invoke`: the config from the environment and
/// the flags (its retry backoff the default), one call, its outcome printed
/// and, if asked, its evidence written.
fn remote(
    ns: &Namespace,
    contract: &Contract,
    out: &mut Out<&mut dyn Write>,
    stderr: &mut dyn Write,
    io: &mut Io,
) -> R<i32> {
    let base = config::from_os_environment(io.env)?;
    let base_url = match ns.get("base_url") {
        Some(Value::Str(url)) if !url.is_empty() => Some(url.clone()),
        _ => base.base_url,
    };
    let config = AdapterConfig {
        enabled: flag(ns, "enable_network") || base.enabled,
        base_url,
        allowed_hosts: base.allowed_hosts,
        allow_loopback: flag(ns, "allow_loopback") || base.allow_loopback,
        read_timeout: base.read_timeout,
        max_response_bytes: base.max_response_bytes,
        max_attempts: base.max_attempts,
        ..AdapterConfig::default()
    }
    .checked()?;
    let mut client = Client {
        config: &config,
        contract,
        dial: &mut *io.dial,
        sleep: &mut *io.sleep,
    };
    let command = text(ns, "command");
    let outcome = match command.as_str() {
        "discover" => client.discover()?,
        "drift" => {
            let (outcome, findings) = client.drift()?;
            if outcome.computed() {
                for finding in &findings {
                    out.print(&p!("drift: ", finding))?;
                }
                if findings.is_empty() {
                    out.print(&p!(
                        "drift: live discovery agrees with snapshot ",
                        contract.snapshot()
                    ))?;
                }
                out.print(&pts(
                    "drift: a difference is for a person to review. Nothing here updates a \
                     snapshot.",
                ))?;
                return Ok(i32::from(!findings.is_empty()));
            }
            outcome
        }
        _ => {
            let path = PyPath::new(&text(ns, "body"));
            let body = match invoke_body(ns, contract, &path)? {
                Ok(body) => body,
                Err(message) => {
                    eprint(
                        stderr,
                        &p!(
                            "lodgeit-adapter: ",
                            path.display(),
                            " could not be read: ",
                            message
                        ),
                    );
                    return Ok(2);
                }
            };
            client.invoke(
                &pts(&text(ns, "calculator")),
                &pts(&text(ns, "period")),
                &body,
            )?
        }
    };

    let record = Json::obj([
        ("status", Json::str(outcome.status.as_str())),
        (
            "http_status",
            outcome
                .http_status
                .map_or(Json::Null, |s| Json::Int(s.into())),
        ),
        (
            "findings",
            Json::Array(outcome.findings.iter().cloned().map(json_str).collect()),
        ),
        (
            "result",
            if outcome.computed() {
                outcome.result.clone()
            } else {
                Json::Null
            },
        ),
    ]);
    out.print(&pyencode::dumps(&record, &Options::PRINTED)?)?;

    if let Some(Value::Str(target)) = ns.get("evidence_out") {
        let label = match ns.get("label") {
            Some(Value::Str(label)) => label.clone(),
            _ => command,
        };
        let record = match evidence::build(
            &outcome,
            &label,
            !flag(ns, "not_synthetic"),
            &contract.response_contract,
        ) {
            Ok(record) => record,
            Err(error) if error.is("ValueError") => {
                // A record with a label the consumer refuses is worse than
                // no record, because it looks like evidence until read.
                eprint(
                    stderr,
                    &p!("lodgeit-adapter: no evidence written: ", error.text),
                );
                return Ok(2);
            }
            Err(error) => return Err(error),
        };
        let path = PyPath::new(target);
        evidence::write(&record, &path)?;
        out.print(&pts(&format!(
            "evidence written to {} (calculation_sha256 {})",
            path.display(),
            record
                .get("calculation_sha256")
                .and_then(Json::as_str)
                .unwrap_or_default()
        )))?;
    }

    if outcome.computed() {
        return Ok(0);
    }
    eprint(
        stderr,
        &pts(&format!(
            "lodgeit-adapter: {}. This is not a result and must not be read as one.",
            outcome.status.as_str()
        )),
    );
    Ok(1)
}

/// The `invoke` body: `_decimalise(load_body(args.body.read_text()), ...)`,
/// or the message of the `OSError`, `JSONDecodeError` or `ValueError` the
/// CLI reports. `load_body` keeps each float literal as a Decimal.
fn invoke_body(ns: &Namespace, contract: &Contract, path: &PyPath) -> R<Result<Json, S>> {
    let empty = Json::Object(Vec::new());
    let calculator = json_str(pts(&text(ns, "calculator")));
    let recorded = client::dict_get(&contract.calculators, &calculator)?.unwrap_or(&empty);
    let text = match py::read_text(&path.fs()) {
        Ok(text) => text,
        Err(py::ReadError::Os(error)) => {
            return Ok(Err(pts(&OsError::new(
                &error,
                Some(&path.display()),
                Style::Errno,
            )
            .text)));
        }
        Err(py::ReadError::Decode(error)) => return Ok(Err(pts(&error.to_string()))),
    };
    let body = match pyjson::loads_numbers(&text, Numbers::DecimalFloats) {
        Ok(body) => body,
        Err(DecodeError::Invalid(message)) => return Ok(Err(pts(&message))),
        Err(DecodeError::Recursion(kind)) => {
            return Err(Exc::msg("RecursionError", &pyjson::recursion_message(kind)));
        }
        Err(other) => unreachable!("load_body has no hooks: {other:?}"),
    };
    let none = Json::Array(Vec::new());
    let fields = get_attr(recorded, "request_number_fields")?.unwrap_or(&none);
    decimalise(body, fields)
}

/// `_decimalise(body, number_fields)`: Decimals rebuilt at the dotted paths
/// the snapshot records as JSON numbers, and nowhere else. `[]` walks every
/// element of a list.
fn decimalise(body: Json, number_fields: &Json) -> R<Result<Json, S>> {
    if !matches!(body, Json::Object(_)) {
        return Ok(Err(pts("a request body must be a JSON object")));
    }
    deepcopy_fits(&body)?;
    let mut converted = body;
    for path in iterate(number_fields)? {
        let Json::Str(path) = path else {
            return Err(Exc::msg(
                "AttributeError",
                &format!("'{}' object has no attribute 'split'", type_name(&path)),
            ));
        };
        let path = path.code_points();
        let parts: Vec<S> = path
            .split(|&cp| cp == u32::from('.'))
            .map(<[u32]>::to_vec)
            .collect();
        if let Err(message) = set_decimal(&mut converted, &parts, &path)? {
            return Ok(Err(message));
        }
    }
    Ok(Ok(converted))
}

/// `node[name]` of a dict node.
fn member<'a>(node: &'a mut Json, name: &[u32]) -> Option<&'a mut Json> {
    let Json::Object(pairs) = node else {
        return None;
    };
    pairs
        .iter_mut()
        .find(|(k, _)| k.code_points() == name)
        .map(|(_, v)| v)
}

/// `_set_decimal(node, parts, path)`: walk one dotted path and convert the
/// leaf. A missing path is not an error; a present leaf that is not a
/// decimal is.
fn set_decimal(node: &mut Json, parts: &[S], path: &[u32]) -> R<Result<(), S>> {
    let Some((head, rest)) = parts.split_first() else {
        return Err(Exc::msg("IndexError", "list index out of range"));
    };
    if let Some(name) = head.strip_suffix(&pts("[]")[..]) {
        let Some(Json::Array(items)) = member(node, name) else {
            return Ok(Ok(()));
        };
        for (index, item) in items.iter_mut().enumerate() {
            let at = p!(path, format!("[{index}]"));
            if let Err(message) = set_decimal(item, rest, &at)? {
                return Ok(Err(message));
            }
        }
        return Ok(Ok(()));
    }
    let Some(value) = member(node, head) else {
        return Ok(Ok(()));
    };
    if !rest.is_empty() {
        return set_decimal(value, rest, path);
    }
    let refused = |value: &Json| Ok(Err(p!(path, ": ", repr(value), " is not a number")));
    match value {
        Json::Dec(_) => Ok(Ok(())),
        Json::Bool(_) | Json::Null => refused(value),
        other => match Decimal::parse_points(strip(&str_points(other))) {
            Ok(decimal) => {
                *other = Json::Dec(decimal);
                Ok(Ok(()))
            }
            Err(_) => refused(other),
        },
    }
}

/// How deep `_decimalise`'s `copy.deepcopy` nests before Python's recursion
/// limit of 1,000 frames: 2 frames per list or dict, the body itself
/// included, 3 above it from an installed CLI, and leaves return without a
/// frame of their own that counts. Measured with CPython 3.15 on Windows and
/// Linux (`artifacts/.../tools/b2_measure_deep.py`): 498 levels copy and 499
/// raise, whatever the containers and leaves.
const DEEPCOPY_DEPTH: usize = 499;

/// The `RecursionError` `copy.deepcopy` raises for a body nested
/// [`DEEPCOPY_DEPTH`] lists and dicts deep.
fn deepcopy_fits(body: &Json) -> R<()> {
    let mut pending = vec![(body, 1)];
    while let Some((value, depth)) = pending.pop() {
        let children: Vec<&Json> = match value {
            Json::Array(items) => items.iter().collect(),
            Json::Object(pairs) => pairs.iter().map(|(_, v)| v).collect(),
            _ => continue,
        };
        if depth >= DEEPCOPY_DEPTH {
            return Err(Exc::msg(
                "RecursionError",
                "maximum recursion depth exceeded",
            ));
        }
        pending.extend(children.into_iter().map(|child| (child, depth + 1)));
    }
    Ok(())
}

fn verify(path: &PyPath, out: &mut Out<&mut dyn Write>, stderr: &mut dyn Write) -> R<i32> {
    let record = match evidence::read(path) {
        Ok(record) => record,
        Err(ReadError::Reported(message)) => {
            eprint(
                stderr,
                &pts(&format!(
                    "lodgeit-adapter: {} could not be read: {message}",
                    path.display()
                )),
            );
            return Ok(1);
        }
        Err(ReadError::Uncaught(error)) => return Err(error),
    };
    let findings = evidence::verify(&record)?;
    for finding in &findings {
        out.print(&p!("evidence: ", finding))?;
    }
    if !findings.is_empty() {
        return Ok(1);
    }
    out.print(&pts(&format!(
        "evidence: {} verifies against its own digest",
        path.display()
    )))?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::tests::{Replies, reply};

    fn text(points: &[u32]) -> String {
        points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
    }

    fn body(text: &str) -> Json {
        pyjson::loads_numbers(text, Numbers::DecimalFloats).unwrap()
    }

    fn decimalised(text: &str, fields: &str) -> Result<String, String> {
        decimalise(body(text), &pyjson::loads_standard(fields).unwrap())
            .unwrap()
            .map(|b| repr(&b))
            .map_err(|m| self::text(&m))
    }

    #[test]
    fn decimalise_converts_only_the_recorded_paths() {
        let fields = r#"["repayments[].amount", "asset.cost", "ref"]"#;
        assert_eq!(
            decimalised(
                r#"{"repayments": [{"amount": 5}, {"amount": " 1.50 "}], "asset": {}, "abn": "0012"}"#,
                fields
            ),
            Ok("{'repayments': [{'amount': Decimal('5')}, {'amount': Decimal('1.50')}], 'asset': {}, \
                'abn': '0012'}"
                .into())
        );
        assert_eq!(
            decimalised(
                r#"{"repayments": [{"amount": 1}, {"amount": "x"}]}"#,
                fields
            ),
            Err("repayments[].amount[1]: 'x' is not a number".into())
        );
        assert_eq!(
            decimalised(r#"{"ref": null}"#, fields),
            Err("ref: None is not a number".into())
        );
        assert_eq!(
            decimalised("[]", fields),
            Err("a request body must be a JSON object".into())
        );
        // `a[]` as the last part walks into each element with no parts left.
        let error = decimalise(
            body(r#"{"a": [1]}"#),
            &pyjson::loads_standard(r#"["a[]"]"#).unwrap(),
        )
        .unwrap_err();
        assert_eq!(error.class, "IndexError");
    }

    #[test]
    fn a_body_nested_past_the_recursion_limit_is_refused_as_deepcopy_refuses_it() {
        // b2_measure_deep.py: {"a": 497 lists around 1} copies; one more raises.
        let nested =
            |depth: usize| format!("{{\"a\": {}1{}}}", "[".repeat(depth), "]".repeat(depth));
        assert!(decimalised(&nested(497), "[]").is_ok());
        let error = decimalise(body(&nested(498)), &Json::Array(Vec::new())).unwrap_err();
        assert_eq!(
            (error.class, text(&error.text).as_str()),
            ("RecursionError", "maximum recursion depth exceeded")
        );
    }

    #[test]
    fn invoke_writes_evidence_that_verifies() {
        let dir = std::env::temp_dir().join(format!("lodgeit-b2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let body_path = dir.join("b.json");
        std::fs::write(
            &body_path,
            r#"{"asset": {"cost": 120000.00}, "reference": "007"}"#,
        )
        .unwrap();
        let evidence = dir.join("ev").join("o.json");
        let good = r#"{"wdv_at": "96000.00", "period_dep_at": "24000.00", "manifest": {"a": 1}, "advisory": {"notes": ["Not advice."]}}"#;
        let args: Vec<String> = [
            "invoke",
            "--enable-network",
            "--allow-loopback",
            "--base-url",
            "http://127.0.0.1:9",
            "--calculator",
            "urn:sbrm:calculator:depreciation:at",
            "--period",
            "urn:sbrm:period:depreciation:unscoped",
            "--body",
            body_path.to_str().unwrap(),
            "--evidence-out",
            evidence.to_str().unwrap(),
        ]
        .map(String::from)
        .to_vec();
        let env = |_: &str| None;
        let mut sleep = |_: f64| Ok(());
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let code = run_io(
            &args,
            &mut stdout,
            &mut stderr,
            &mut Io {
                env: &env,
                dial: &mut Replies(vec![reply("200 OK", "", good)]),
                sleep: &mut sleep,
            },
        );
        let stdout = String::from_utf8(stdout).unwrap();
        assert_eq!(code, 0, "{stdout}{}", String::from_utf8_lossy(&stderr));
        assert!(stdout.contains("\"status\": \"COMPUTED\""));
        assert!(stdout.contains("evidence written to "));
        let record = std::fs::read_to_string(&evidence).unwrap();
        assert!(record.contains("\"input_json\": \"{\\\"asset\\\":{\\\"cost\\\":120000.00},\\\"reference\\\":\\\"007\\\"}\""));
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let verified = run_with(
            &[
                "verify".into(),
                "--evidence".into(),
                evidence.to_str().unwrap().into(),
            ],
            &mut out,
            &mut err,
        );
        assert_eq!(verified, 0, "{}", String::from_utf8_lossy(&out));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A stdout that refuses every write, as `/dev/full` does.
    struct Full;

    impl Write for Full {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(28))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_stdout_that_cannot_be_written_exits_120_as_pythons_flush_does() {
        // Checked against CPython 3.15 on Linux with stdout on /dev/full.
        for args in [["contract"], ["--help"]] {
            let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            let mut stderr = Vec::new();
            assert_eq!(run_with(&args, &mut Full, &mut stderr), 120);
            let stderr = String::from_utf8(stderr).unwrap();
            assert!(stderr.starts_with("Exception ignored while flushing sys.stdout:"));
        }
    }
}
