use div7a_loan_review::{api, cli};
use std::io::{Read, Write};

/// One JSON stdin command: (exit code, stdout, stderr).
fn run_api(command: &str) -> (i32, String, String) {
    let mut bytes = Vec::new();
    // Bound the JSON boundary before allocating the complete input.
    if let Err(e) = std::io::stdin().take((8 << 20) + 1).read_to_end(&mut bytes) {
        return (2, String::new(), format!("error: {e}\n"));
    }
    let result = if bytes.len() > 8 << 20 {
        Err("JSON input exceeds 8 MiB".into())
    } else {
        serde_json::from_slice(&bytes)
            .map_err(|e| format!("invalid JSON: {e}"))
            .and_then(|v| api::evaluate(command, &v))
    };
    match result {
        Ok(v) => (0, format!("{v}\n"), String::new()),
        Err(m) => (2, String::new(), format!("error: {m}\n")),
    }
}

fn main() {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| cli::decode_os(&a))
        .collect();
    let (code, out, err) = match args.as_slice() {
        [command] if api::COMMANDS.contains(&command.as_str()) => run_api(command),
        [flag] if flag == "--api-help" => (
            0,
            format!(
                "JSON stdin commands: {}\nSee README.md for their schemas.\n",
                api::COMMANDS.join(", ")
            ),
            String::new(),
        ),
        _ => cli::run(&args),
    };
    let out_ok = std::io::stdout().write_all(&cli::output_bytes(&out));
    let err_ok = std::io::stderr().write_all(&cli::output_bytes(&err));
    std::process::exit(if out_ok.is_err() || err_ok.is_err() {
        1
    } else {
        code
    });
}
