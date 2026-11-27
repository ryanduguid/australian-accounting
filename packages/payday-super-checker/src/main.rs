// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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
    const FLUSH_FAILED: i32 = if cfg!(unix) { 120 } else { 1 };
    if let Err(e) = &out {
        if cfg!(unix) {
            let _ = std::io::stderr().write_all(unflushed("stdout", e).as_bytes());
        }
        return FLUSH_FAILED;
    }
    if err.is_err() {
        return FLUSH_FAILED;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn a_stream_that_cannot_be_written_fails_the_run() {
        let failed = if cfg!(unix) { 120 } else { 1 };
        let pipe = || Err(Error::from(ErrorKind::BrokenPipe));
        assert_eq!(finish(0, pipe(), Ok(())), failed);
        assert_eq!(finish(2, Ok(()), pipe()), failed);
        assert_eq!(finish(2, Ok(()), Ok(())), 2);
    }

    #[test]
    fn the_unflushed_message_names_the_python_exception_class() {
        let line = |kind| unflushed("stdout", &Error::from(kind));
        assert_eq!(
            line(ErrorKind::BrokenPipe),
            "Exception ignored while flushing sys.stdout:\nBrokenPipeError: [Errno 0] broken pipe\n"
        );
        assert!(line(ErrorKind::ConnectionReset).contains("\nConnectionResetError: [Errno 0] "));
        assert!(line(ErrorKind::WouldBlock).contains("\nBlockingIOError: [Errno 0] "));
        let other = unflushed("stdout", &Error::other("disk gone"));
        assert!(
            other.ends_with("\nOSError: [Errno 0] disk gone\n"),
            "{other}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_os_error_keeps_its_errno() {
        let raw = unflushed("stdout", &Error::from_raw_os_error(32));
        assert!(
            raw.ends_with("BrokenPipeError: [Errno 32] Broken pipe\n"),
            "{raw}"
        );
    }
}
