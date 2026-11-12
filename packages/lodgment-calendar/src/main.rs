// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use lodgment_calendar::{api, cli};
use std::io::{Read, Write};
mod json_input;

fn run_api(command: &str) -> cli::Outcome {
    let mut bytes = vec![];
    if let Err(e) = std::io::stdin().take((8 << 20) + 1).read_to_end(&mut bytes) {
        return (2, String::new(), format!("error: {e}\n"));
    }
    let result = if bytes.len() > 8 << 20 {
        Err("JSON input exceeds 8 MiB".into())
    } else {
        json_input::parse(&bytes)
            .map_err(|e| format!("invalid JSON: {e}"))
            .and_then(|v| api::evaluate(command, &v))
    };
    match result {
        Ok(value) => (0, format!("{value}\n"), String::new()),
        Err(e) => (2, String::new(), format!("error: {e}\n")),
    }
}
fn main() {
    let args: Vec<_> = std::env::args_os()
        .skip(1)
        .map(|a| cli::decode_os(&a))
        .collect();
    let (code, out, err) = match args.as_slice() {
        [command] if api::COMMANDS.contains(&command.as_str()) => run_api(command),
        _ => cli::run(&args),
    };
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    let out_result = stdout
        .write_all(out.as_bytes())
        .and_then(|()| stdout.flush());
    let err_result = stderr
        .write_all(err.as_bytes())
        .and_then(|()| stderr.flush());
    std::process::exit(if out_result.is_err() || err_result.is_err() {
        1
    } else {
        code
    });
}
