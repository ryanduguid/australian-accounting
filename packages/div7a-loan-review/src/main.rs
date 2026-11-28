// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use div7a_loan_review::cli;
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
    let shutdown = if cfg!(unix) { 120 } else { 1 };
    if let Err(e) = &out {
        if cfg!(unix) {
            let _ = std::io::stderr().write_all(unflushed("stdout", e).as_bytes());
        }
        return shutdown;
    }
    if err.is_err() {
        return shutdown;
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
    fn a_stream_that_fails_sets_pythons_shutdown_status() {
        let failed = || Err(Error::from(ErrorKind::BrokenPipe));
        let want = if cfg!(unix) { 120 } else { 1 };
        assert_eq!(finish(0, failed(), Ok(())), want);
        assert_eq!(finish(2, Ok(()), failed()), want);
        assert_eq!(finish(2, Ok(()), Ok(())), 2);
    }

    #[cfg(unix)]
    #[test]
    fn an_unflushed_stdout_is_reported_with_pythons_exception_class() {
        for (errno, class, text) in [
            (libc::EPIPE, "BrokenPipeError", "Broken pipe"),
            (
                libc::ECONNRESET,
                "ConnectionResetError",
                "Connection reset by peer",
            ),
            (
                libc::EAGAIN,
                "BlockingIOError",
                "Resource temporarily unavailable",
            ),
            (libc::ENOSPC, "OSError", "No space left on device"),
        ] {
            assert_eq!(
                unflushed("stdout", &Error::from_raw_os_error(errno)),
                format!(
                    "Exception ignored while flushing sys.stdout:\n{class}: [Errno {errno}] {text}\n"
                )
            );
        }
    }
}
