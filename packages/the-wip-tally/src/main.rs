use std::io::{Read, Write};
use the_wip_tally::{api, cli};

fn emit(mut stream: impl Write, text: &str) -> std::io::Result<()> {
    stream.write_all(&cli::output_bytes(text))?;
    stream.flush()
}
fn api_run(command: &str) -> (i32, String, String) {
    let mut bytes = Vec::new();
    // Bound the JSON boundary before allocating the complete input. The engine never opens paths.
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
    let (code, out, err) = if args.len() == 1 && api::COMMANDS.contains(&args[0].as_str()) {
        api_run(&args[0])
    } else if args == ["--api-help"] {
        (
            0,
            format!(
                "JSON stdin commands: {}\nAmounts are decimal strings. See README.md for schemas.\n",
                api::COMMANDS.join(", ")
            ),
            String::new(),
        )
    } else {
        cli::run(&args)
    };
    let out_result = emit(std::io::stdout().lock(), &out);
    let err_result = emit(std::io::stderr().lock(), &err);
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
