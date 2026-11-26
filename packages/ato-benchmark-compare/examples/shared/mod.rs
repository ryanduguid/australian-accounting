// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Code the `build_dataset` and `build_other_benchmarks` examples share: their
//! argparse-compatible command line, JSON output and process plumbing, plus the
//! harness that replays the recorded answers of the Python scripts they replace.

use std::collections::HashMap;
use std::io::Write;
use std::process::ExitCode;

use ato_benchmark_compare::cli::{decode_os, output_bytes};
use ato_benchmark_compare::py::repr_str;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// What a run printed and the exit code it ends with.
pub struct Outcome {
    pub code: u8,
    pub stdout: String,
    pub stderr: String,
}

impl Outcome {
    pub fn fail(code: u8, stderr: String) -> Self {
        Self {
            code,
            stdout: String::new(),
            stderr,
        }
    }

    /// Writes both streams (CRLF on Windows, as CPython's text mode does) and exits.
    pub fn emit(self) -> ExitCode {
        let _ = std::io::stdout().write_all(&output_bytes(&self.stdout));
        let _ = std::io::stderr().write_all(&output_bytes(&self.stderr));
        ExitCode::from(self.code)
    }
}

/// The command-line arguments, decoded as CPython decodes `sys.argv`, so a path
/// that is not valid UTF-8 still reaches the builder. They name local files only.
pub fn os_args() -> Vec<String> {
    let raw = std::env::args_os(); // nosemgrep: rust.lang.security.args-os.args-os
    raw.skip(1).map(|a| decode_os(&a)).collect()
}

/// A program's flags, each taking one value, in argparse's declaration order.
pub struct Spec {
    pub prog: &'static str,
    pub usage: &'static str,
    /// `(flag, required)`; an optional flag may repeat and collects every value.
    pub flags: &'static [(&'static str, bool)],
}

pub type Values = HashMap<&'static str, Vec<String>>;

/// argparse's `^-\d+$|^-\d*\.\d+$`: a value, as no flag here looks like a number.
fn is_negative_number(arg: &str) -> bool {
    let body = &arg[1..];
    let (int, frac) = body
        .split_once('.')
        .map_or((body, None), |(i, f)| (i, Some(f)));
    let digits = |s: &str| s.chars().all(|c| c.is_ascii_digit());
    match frac {
        None => !int.is_empty() && digits(int),
        Some(f) => !f.is_empty() && digits(f) && digits(int),
    }
}

enum Token {
    Value(String),
    Help,
    HelpWithValue(String),
    Flag(&'static str, Option<String>),
    Unknown(String),
}

impl Spec {
    fn error(&self, msg: &str) -> String {
        format!("{}\n{}: error: {msg}\n", self.usage, self.prog)
    }

    /// argparse's `_parse_optional`: an exact flag, `--flag=value`, or a unique
    /// prefix of a long flag; anything else starting with `-` is unrecognised.
    fn classify(&self, arg: &str) -> Result<Token, String> {
        if !arg.starts_with('-') || arg.len() == 1 {
            return Ok(Token::Value(arg.to_string()));
        }
        let (head, inline) = match arg.split_once('=') {
            Some((h, v)) => (h, Some(v.to_string())),
            None => (arg, None),
        };
        let names = || {
            ["-h", "--help"]
                .into_iter()
                .chain(self.flags.iter().map(|f| f.0))
        };
        if let Some(name) = names().find(|n| *n == arg || (inline.is_some() && *n == head)) {
            return Ok(self.token(name, inline));
        }
        if head.starts_with("--") {
            let matches: Vec<&str> = names().filter(|n| n.starts_with(head)).collect();
            match matches.as_slice() {
                [name] => return Ok(self.token(name, inline)),
                [] => {}
                many => {
                    return Err(self.error(&format!(
                        "ambiguous option: {arg} could match {}",
                        many.join(", ")
                    )));
                }
            }
        }
        if arg.starts_with("-h") && !arg.starts_with("--") {
            // `-hx`: argparse runs `-h` first, which prints help and exits.
            return Ok(Token::Help);
        }
        if arg.contains(' ') || is_negative_number(arg) {
            return Ok(Token::Value(arg.to_string()));
        }
        Ok(Token::Unknown(arg.to_string()))
    }

    fn token(&self, name: &str, inline: Option<String>) -> Token {
        match self.flags.iter().find(|f| f.0 == name) {
            Some(flag) => Token::Flag(flag.0, inline),
            None => inline.map_or(Token::Help, Token::HelpWithValue),
        }
    }

    /// Parses `args` as argparse would: `Ok(None)` asks for help; `Err` is the
    /// usage and error text argparse prints before exiting with status 2.
    pub fn parse(&self, args: &[String]) -> Result<Option<Values>, String> {
        let mut tokens = Vec::with_capacity(args.len());
        let mut rest = args.iter();
        for arg in rest.by_ref() {
            if arg == "--" {
                break;
            }
            tokens.push(self.classify(arg)?);
        }
        tokens.extend(rest.map(|a| Token::Value(a.clone())));
        let (mut values, mut extras) = (Values::new(), Vec::new());
        let mut tokens = tokens.into_iter().peekable();
        while let Some(token) = tokens.next() {
            match token {
                Token::Help => return Ok(None),
                Token::HelpWithValue(v) => {
                    let msg = format!(
                        "argument -h/--help: ignored explicit argument {}",
                        repr_str(&v)
                    );
                    return Err(self.error(&msg));
                }
                Token::Value(v) | Token::Unknown(v) => extras.push(v),
                Token::Flag(name, inline) => {
                    let value = match (inline, tokens.peek()) {
                        (Some(v), _) => v,
                        (None, Some(Token::Value(_))) => match tokens.next() {
                            Some(Token::Value(v)) => v,
                            _ => unreachable!("peeked a value"),
                        },
                        _ => {
                            return Err(
                                self.error(&format!("argument {name}: expected one argument"))
                            );
                        }
                    };
                    values.entry(name).or_default().push(value);
                }
            }
        }
        self.finish(values, &extras)
    }

    /// argparse's checks after the last argument: required flags, then extras.
    fn finish(&self, values: Values, extras: &[String]) -> Result<Option<Values>, String> {
        let missing: Vec<&str> = self
            .flags
            .iter()
            .filter(|(name, required)| *required && !values.contains_key(name))
            .map(|f| f.0)
            .collect();
        if !missing.is_empty() {
            return Err(self.error(&format!(
                "the following arguments are required: {}",
                missing.join(", ")
            )));
        }
        if !extras.is_empty() {
            return Err(self.error(&format!("unrecognized arguments: {}", extras.join(" "))));
        }
        Ok(Some(values))
    }
}

/// The last value given for a required flag, as argparse's `store` keeps it.
pub fn last(values: &Values, flag: &str) -> String {
    values[flag].last().cloned().unwrap_or_default()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn indent(out: &mut String, level: usize) {
    out.push('\n');
    out.extend(std::iter::repeat_n(' ', level));
}

fn write_json_indented(value: &Value, level: usize, out: &mut String) {
    match value {
        Value::Array(items) if !items.is_empty() => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                out.push_str(if i > 0 { "," } else { "" });
                indent(out, level + 1);
                write_json_indented(item, level + 1, out);
            }
            indent(out, level);
            out.push(']');
        }
        Value::Object(map) if !map.is_empty() => {
            out.push('{');
            for (i, (k, v)) in map.iter().enumerate() {
                out.push_str(if i > 0 { "," } else { "" });
                indent(out, level + 1);
                out.push_str(&serde_json::to_string(k).expect("JSON key"));
                out.push_str(": ");
                write_json_indented(v, level + 1, out);
            }
            indent(out, level);
            out.push('}');
        }
        other => out.push_str(&serde_json::to_string(other).expect("JSON value")),
    }
}

/// `json.dumps(value, indent=1, ensure_ascii=False) + "\n"`.
pub fn to_json_indented(value: &Value) -> String {
    let mut out = String::new();
    write_json_indented(value, 0, &mut out);
    out.push('\n');
    out
}

/// Replays `tests/data/builders/<tool>-python.json`, the answers the deleted
/// Python script gave on each case's inputs (see the file's `about`).
#[cfg(test)]
pub mod parity {
    use super::Outcome;
    use serde_json::Value;
    use std::fs;
    use std::path::{Path, PathBuf};

    pub fn cases(recorded: &str) -> Vec<Value> {
        let doc: Value = serde_json::from_str(recorded).expect("recorded cases");
        doc["cases"].as_array().expect("cases").clone()
    }

    /// A fresh scratch folder for one case; test-only, holding nothing secret.
    pub fn scratch(tool: &str, case: &str) -> PathBuf {
        let base = std::env::temp_dir(); // nosemgrep: rust.lang.security.temp-dir.temp-dir
        let dir = base.join(format!("{tool}-parity-{}-{case}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("scratch folder");
        dir
    }

    pub fn write_files(dir: &Path, case: &Value) {
        for (rel, text) in case["files"].as_object().into_iter().flatten() {
            let path = dir.join(rel);
            fs::create_dir_all(path.parent().expect("parent")).expect("folder");
            fs::write(path, text.as_str().expect("text").as_bytes()).expect("input");
        }
    }

    pub fn args(dir: &Path, case: &Value) -> Vec<String> {
        let root = dir.to_string_lossy().replace('\\', "/");
        case["args"]
            .as_array()
            .expect("args")
            .iter()
            .map(|a| a.as_str().expect("arg").replace("{dir}", &root))
            .collect()
    }

    fn relative(dir: &Path, text: &str) -> String {
        let root = format!("{}/", dir.to_string_lossy().replace('\\', "/"));
        text.replace('\\', "/").replace(&root, "")
    }

    fn argparse_error(stderr: &str) -> &str {
        stderr
            .lines()
            .find_map(|l| l.split_once(": error: ").map(|p| p.1))
            .unwrap_or_default()
    }

    /// Asserts the port answered as the recorded Python run did.
    pub fn check(dir: &Path, case: &Value, got: &Outcome) {
        let name = case["name"].as_str().expect("name");
        let stdout = relative(dir, &got.stdout);
        let stderr = relative(dir, &got.stderr);
        assert_eq!(
            u64::from(got.code),
            case["exit"],
            "{name}: exit code\n{stderr}"
        );
        match case["check"].as_str().expect("check") {
            "exact" | "raised" => {
                assert_eq!(stdout, case["stdout"], "{name}: stdout");
                assert_eq!(stderr, case["stderr"], "{name}: stderr");
            }
            "argparse" => assert_eq!(argparse_error(&stderr), case["stderr"], "{name}"),
            _ => {}
        }
        for (rel, digest) in case["output_sha256"].as_object().into_iter().flatten() {
            let written = fs::read(dir.join(rel)).expect("output written");
            assert_eq!(super::sha256_hex(&written), *digest, "{name}: {rel}");
        }
        for (rel, text) in case["outputs"].as_object().into_iter().flatten() {
            let written = fs::read_to_string(dir.join(rel)).expect("output written");
            assert_eq!(written, text.as_str().expect("text"), "{name}: {rel}");
        }
        let _ = fs::remove_dir_all(dir);
    }
}
