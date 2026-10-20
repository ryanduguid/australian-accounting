//! Trace one fabricated contract through the existing WIP engine to cash inputs.
//!
//! The Rust form of `examples/job_to_cash.py`: the same files and stdout bytes.
//! Run it with `cargo run --locked --example job_to_cash -- --output <new folder>`
//! and optionally `--extra-cost-to-complete <amount>`.

use std::io::Write;

use the_wip_tally::csv_io::Failure;
use the_wip_tally::decimal::Decimal;
use the_wip_tally::job_to_cash::{run, text_mode};
use the_wip_tally::py::PyPath;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut output = None;
    let mut extra = String::from("0");
    let mut i = 0;
    while i < args.len() {
        let (flag, inline) = match args[i].split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f.to_string(), Some(v.to_string())),
            _ => (args[i].clone(), None),
        };
        let value = match inline {
            Some(v) => v,
            None => {
                i += 1;
                args.get(i).cloned().unwrap_or_default()
            }
        };
        match flag.as_str() {
            "--output" => output = Some(value),
            "--extra-cost-to-complete" => extra = value,
            other => {
                eprintln!("error: unrecognized argument {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    let Some(output) = output else {
        eprintln!("error: the following arguments are required: --output");
        std::process::exit(2);
    };
    let Ok(extra) = Decimal::parse(&extra) else {
        eprintln!("error: decimal.InvalidOperation");
        std::process::exit(1);
    };
    match run(&PyPath::new(&output), &extra) {
        Ok(json) => {
            let text = text_mode(&format!("{json}\n"));
            let _ = std::io::stdout().write_all(text.as_bytes());
        }
        Err(Failure::Message(message)) => {
            eprintln!("error: {message}");
            std::process::exit(1);
        }
        Err(Failure::Decimal(signal)) => {
            eprintln!("error: decimal.{}", signal.name());
            std::process::exit(1);
        }
    }
}
