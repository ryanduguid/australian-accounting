// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use australian_tax_calculators::api;
use serde_json::{Value, json};
use std::io::{Read, Write};

/// One JSON stdin command: (exit code, stdout, stderr). A refusal always keeps
/// its `error:` line on stderr; `refusal_json` adds its code on stdout.
fn run(command: &str, refusal_json: bool) -> (i32, String, String) {
    let mut bytes = Vec::new();
    // Bound the JSON boundary before allocating the complete input.
    let result = if let Err(e) = std::io::stdin().take((8 << 20) + 1).read_to_end(&mut bytes) {
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
        [command] if commands.contains(&command.as_str()) => run(command, false),
        [command, flag] if flag == "--refusal-json" && commands.contains(&command.as_str()) => {
            run(command, true)
        }
        [flag] if flag == "--api-help" => (
            0,
            format!(
                "JSON stdin commands: {}\nAmounts are decimal strings. See README.md for schemas.\n",
                commands.join(", ")
            ),
            String::new(),
        ),
        _ => (
            2,
            String::new(),
            format!(
                "usage: australian-tax-calculators COMMAND < input.json\ncommands: {}\n",
                commands.join(", ")
            ),
        ),
    };
    let native = |text: &str| {
        if cfg!(windows) {
            text.replace('\n', "\r\n")
        } else {
            text.to_string()
        }
    };
    let out_ok = std::io::stdout().write_all(native(&out).as_bytes());
    let err_ok = std::io::stderr().write_all(native(&err).as_bytes());
    std::process::exit(if out_ok.is_err() || err_ok.is_err() {
        1
    } else {
        code
    });
}
