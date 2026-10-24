//! `cli.py`: the `lodgeit-adapter` command line. Nothing here reaches the
//! network without `--enable-network`.
//!
//! Exit codes: 0 the command did what it was asked, 1 a refusal or a
//! failure, 2 a usage error. `contract` and `verify` are ported; `discover`,
//! `drift` and `invoke` arrive with the client (gate B2).
use std::ffi::OsString;
use std::io::Write;

use crate::argparse::{Action, Exit, Namespace, Parser, Value};
use crate::contract;
use crate::errors::ADAPTER_ERROR;
use crate::evidence::{self, ReadError};
use crate::p;
use crate::pyencode::{self, Options};
use crate::pyjson::Json;
use crate::pyops::{Exc, R, S, iterate, pts, py_sorted};
use crate::pypath::PyPath;
use crate::pyvalue;

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
struct Out<W> {
    w: W,
}

impl<W: Write> Out<W> {
    fn print(&mut self, line: &[u32]) -> R<()> {
        let bytes = pyvalue::encode_print(&line_ends(line))
            .map_err(|m| Exc::msg("UnicodeEncodeError", &m))?;
        let _ = self.w.write_all(&bytes);
        let _ = self.w.write_all(&newlines(b"\n"));
        Ok(())
    }
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
    let args: Vec<String> = argv
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    run_with(&args, &mut std::io::stdout().lock(), &mut std::io::stderr())
}

/// `main(argv)` writing to the given streams; returns the exit status.
pub fn run_with(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let ns = match build_parser().parse(args) {
        Ok(ns) => ns,
        Err(Exit::Help(help)) => {
            let _ = stdout.write_all(&newlines(help.as_bytes()));
            let _ = stdout.flush();
            return 0;
        }
        Err(Exit::Error(usage)) => {
            let _ = stderr.write_all(&newlines(usage.as_bytes()));
            return 2;
        }
    };
    let mut out = Out { w: stdout };
    let result = run(&ns, &mut out, stderr);
    let _ = out.w.flush();
    result.unwrap_or_else(|error| {
        let _ = stderr.write_all(&newlines(error.traceback().as_bytes()));
        1
    })
}

fn run(ns: &Namespace, out: &mut Out<&mut dyn Write>, stderr: &mut dyn Write) -> R<i32> {
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
        command => {
            eprint(
                stderr,
                &pts(&format!(
                    "lodgeit-adapter: {command} is not ported yet; it arrives with the client"
                )),
            );
            Ok(1)
        }
    }
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
