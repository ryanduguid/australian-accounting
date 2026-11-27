// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use australian_tax_calculators::{api, engine};
use serde_json::{Value, json};
use std::io::{Read, Write};

/// One JSON command read from `input`: (exit code, stdout, stderr). A refusal
/// always keeps its `error:` line on stderr; `refusal_json` adds its code on stdout.
fn run(command: &str, refusal_json: bool, input: &mut impl Read) -> (i32, String, String) {
    let mut bytes = Vec::new();
    // Bound the JSON boundary before allocating the complete input.
    let result = if let Err(e) = input.take((8 << 20) + 1).read_to_end(&mut bytes) {
        Err(api::Refusal::new("input.unreadable", e.to_string()))
    } else if bytes.len() > 8 << 20 {
        Err(api::Refusal::new(
            "input.too_large",
            "JSON input exceeds 8 MiB",
        ))
    } else {
        match serde_json::from_slice::<Value>(&bytes) {
            Err(e) => Err(api::Refusal::new(
                "input.invalid_json",
                format!("invalid JSON: {e}"),
            )),
            Ok(v) => match api::repeated_key(&bytes) {
                Some(key) => Err(api::Refusal::new(
                    "input.invalid_json",
                    format!("invalid JSON: duplicate key {key:?}"),
                )),
                None => api::evaluate_coded(command, &v),
            },
        }
    };
    match result {
        Ok(v) => (0, format!("{v}\n"), String::new()),
        Err(api::Refusal { code, message }) => {
            let out = if refusal_json {
                let refusal = json!({"ok": false, "code": code, "message": message});
                format!("{refusal}\n")
            } else {
                String::new()
            };
            (2, out, format!("error: {message}\n"))
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let commands = api::commands();
    let (code, out, err) = match args.as_slice() {
        [command] if commands.contains(&command.as_str()) => {
            run(command, false, &mut std::io::stdin())
        }
        [command, flag] if flag == "--refusal-json" && commands.contains(&command.as_str()) => {
            run(command, true, &mut std::io::stdin())
        }
        [command, rest @ ..] if command == "contract-register" => {
            api::contract_register(rest, &mut std::io::stdin())
        }
        [flag] if flag == "--version" => (
            0,
            format!("australian-tax-calculators {}\n", engine::VERSION),
            String::new(),
        ),
        [flag] if flag == "--api-help" => (0, api::cli_help(), String::new()),
        _ => (
            2,
            String::new(),
            format!(
                "usage: australian-tax-calculators COMMAND < input.json\n       australian-tax-calculators contract-register FILE\ncommands: {}\n",
                commands.join(", ")
            ),
        ),
    };
    let code = emit(
        code,
        [&out, &err],
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    std::process::exit(code);
}

/// Writes `out` and `err` with the platform's line endings: `code`, or 1 when
/// either write fails.
fn emit(code: i32, [out, err]: [&str; 2], stdout: &mut impl Write, stderr: &mut impl Write) -> i32 {
    let newline = if cfg!(windows) { "\r\n" } else { "\n" };
    let out_ok = stdout.write_all(out.replace('\n', newline).as_bytes());
    let err_ok = stderr.write_all(err.replace('\n', newline).as_bytes());
    if out_ok.is_err() || err_ok.is_err() {
        1
    } else {
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reader whose every read fails, as a closed pipe does.
    struct Broken;

    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("pipe closed"))
        }
    }

    #[test]
    fn an_unreadable_input_is_refused_with_its_error() {
        let command = api::commands()[0];
        let (code, out, err) = run(command, false, &mut Broken);
        assert_eq!(
            (code, out.as_str(), err.as_str()),
            (2, "", "error: pipe closed\n")
        );
        let (_, out, _) = run(command, true, &mut Broken);
        let refusal: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(refusal["code"], "input.unreadable");
    }

    #[test]
    fn output_takes_native_line_endings_and_a_failed_write_exits_1() {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(emit(2, ["a\n", "b\n"], &mut out, &mut err), 2);
        let newline = if cfg!(windows) { "\r\n" } else { "\n" };
        assert_eq!(out, format!("a{newline}").into_bytes());
        assert_eq!(err, format!("b{newline}").into_bytes());
        // A full buffer refuses every byte, as a closed pipe does.
        let full = || -> &'static mut [u8] { &mut [] };
        assert_eq!(emit(0, ["a\n", ""], &mut full(), &mut Vec::new()), 1);
        assert_eq!(emit(0, ["", "b\n"], &mut Vec::new(), &mut full()), 1);
    }
}
