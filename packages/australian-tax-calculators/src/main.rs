use australian_tax_calculators::api;
use std::io::{Read, Write};

/// One JSON stdin command: (exit code, stdout, stderr).
fn run(command: &str) -> (i32, String, String) {
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    let commands = api::commands();
    let (code, out, err) = match args.as_slice() {
        [command] if commands.contains(&command.as_str()) => run(command),
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
