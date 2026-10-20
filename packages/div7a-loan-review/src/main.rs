use div7a_loan_review::{api, cli, derivation};
use serde_json::Value;
use std::io::{Read, Write};

/// One JSON object from stdin or, for `verify FILE`, a file, bounded at
/// 8 MiB before the complete input is allocated.
fn read_json(path: Option<&str>) -> Result<Value, String> {
    let mut bytes = Vec::new();
    let limit = (8 << 20) + 1;
    match path {
        None | Some("-") => std::io::stdin().take(limit).read_to_end(&mut bytes),
        Some(path) => std::fs::File::open(path)
            .and_then(|f| f.take(limit).read_to_end(&mut bytes))
            .map_err(|e| std::io::Error::new(e.kind(), format!("{path}: {e}"))),
    }
    .map_err(|e| e.to_string())?;
    if bytes.len() > 8 << 20 {
        return Err("JSON input exceeds 8 MiB".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid JSON: {e}"))
}

/// One JSON stdin command: (exit code, stdout, stderr).
fn run_api(command: &str) -> (i32, String, String) {
    match read_json(None).and_then(|v| api::evaluate(command, &v)) {
        Ok(v) => (0, format!("{v}\n"), String::new()),
        Err(m) => (2, String::new(), format!("error: {m}\n")),
    }
}

/// `derive` writes the derivation; `verify` exits 1 when a step differs.
fn run_derivation(command: &str, path: Option<&str>) -> (i32, String, String) {
    let result = read_json(path).and_then(|v| {
        if command == "derive" {
            derivation::derive(&v)
        } else {
            derivation::verify(&v)
        }
    });
    match result {
        Ok(v) => (
            i32::from(v["verified"] == false),
            format!("{}\n", serde_json::to_string_pretty(&v).expect("JSON")),
            String::new(),
        ),
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
        [command] if command == "derive" || command == "verify" => run_derivation(command, None),
        [command, path] if command == "verify" => run_derivation(command, Some(path)),
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
    let mut stdout = std::io::stdout().lock();
    let out_result = stdout
        .write_all(&cli::output_bytes(&out))
        .and_then(|()| stdout.flush());
    let err_result = std::io::stderr().write_all(&cli::output_bytes(&err));
    std::process::exit(finish(code, out_result, err_result));
}

/// The exit status once both streams are written. Python reported a stream
/// it could not flush at shutdown, on stderr, and exited 120; on Windows the
/// binary exits 1 without that message.
fn finish(code: i32, out: std::io::Result<()>, err: std::io::Result<()>) -> i32 {
    if let Err(e) = &out {
        if cfg!(unix) {
            let _ = std::io::stderr().write_all(unflushed("stdout", e).as_bytes());
            return 120;
        }
        return 1;
    }
    if err.is_err() {
        return if cfg!(unix) { 120 } else { 1 };
    }
    code
}

/// "Exception ignored while flushing sys.stdout:" and the OSError, as Python printed them.
fn unflushed(stream: &str, e: &std::io::Error) -> String {
    let code = e.raw_os_error().unwrap_or(0);
    let class = match e.kind() {
        std::io::ErrorKind::BrokenPipe => "BrokenPipeError",
        std::io::ErrorKind::ConnectionReset => "ConnectionResetError",
        std::io::ErrorKind::WouldBlock => "BlockingIOError",
        _ => "OSError",
    };
    let text = e.to_string();
    let text = text
        .rfind(" (os error ")
        .map_or(text.as_str(), |i| &text[..i]);
    format!("Exception ignored while flushing sys.{stream}:\n{class}: [Errno {code}] {text}\n")
}
