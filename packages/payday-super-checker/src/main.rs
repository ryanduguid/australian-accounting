use payday_super_checker::cli;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| cli::decode_os(&a))
        .collect();
    let (code, out, err) = cli::dispatch(&args);
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
