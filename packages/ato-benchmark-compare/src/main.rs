// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use ato_benchmark_compare::{api, cli};
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
    let mut stdout = std::io::stdout().lock();
    let out_result = stdout
        .write_all(&cli::output_bytes(&out))
        .and_then(|()| stdout.flush());
    let err_result = std::io::stderr().write_all(&cli::output_bytes(&err));
    let unix = cfg!(unix);
    std::process::exit(finish(
        code,
        out_result,
        err_result,
        unix,
        &mut std::io::stderr(),
    ));
}

/// The exit status once both streams are written. Python reported a stream
/// it could not flush at shutdown, on stderr, and exited 120; on Windows
/// (`unix` false) the binary exits 1 without that message.
fn finish(
    code: i32,
    out: std::io::Result<()>,
    err: std::io::Result<()>,
    unix: bool,
    stderr: &mut dyn Write,
) -> i32 {
    if let Err(e) = &out {
        if unix {
            let _ = stderr.write_all(unflushed("stdout", e).as_bytes());
            return 120;
        }
        return 1;
    }
    if err.is_err() {
        return if unix { 120 } else { 1 };
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
    use std::io::Error;

    /// A stderr that refuses every write, as a closed descriptor does.
    struct Closed;

    impl Write for Closed {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(Error::other("closed"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(Error::other("closed"))
        }
    }

    #[test]
    fn a_stream_that_cannot_be_written_sets_the_exit_status() {
        for (unix, failed) in [(true, 120), (false, 1)] {
            let mut stderr = Vec::new();
            assert_eq!(finish(3, Ok(()), Ok(()), unix, &mut stderr), 3);
            assert_eq!(
                finish(0, Ok(()), Err(Error::other("gone")), unix, &mut stderr),
                failed
            );
            assert!(stderr.is_empty());
            assert_eq!(
                finish(0, Err(Error::other("gone")), Ok(()), unix, &mut stderr),
                failed
            );
            // Only Unix reports the unflushed stdout, as Python did.
            let expected = if unix {
                unflushed("stdout", &Error::other("gone"))
            } else {
                String::new()
            };
            assert_eq!(String::from_utf8(stderr).unwrap(), expected);
        }
    }

    #[test]
    fn a_stderr_that_also_fails_still_gives_the_exit_status() {
        assert_eq!(
            finish(0, Err(Error::other("gone")), Ok(()), true, &mut Closed),
            120
        );
        assert!(Closed.flush().is_err());
    }

    #[test]
    fn an_error_without_an_os_code_is_an_oserror_with_errno_0() {
        assert_eq!(
            unflushed("stdout", &Error::other("boom")),
            "Exception ignored while flushing sys.stdout:\nOSError: [Errno 0] boom\n"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn os_errors_take_python_class_names_and_drop_rusts_suffix() {
        let message = |code| unflushed("stdout", &Error::from_raw_os_error(code));
        assert_eq!(
            message(32),
            "Exception ignored while flushing sys.stdout:\nBrokenPipeError: [Errno 32] Broken pipe\n"
        );
        assert!(
            message(104).contains("\nConnectionResetError: [Errno 104] Connection reset by peer\n")
        );
        assert!(
            message(11)
                .contains("\nBlockingIOError: [Errno 11] Resource temporarily unavailable\n")
        );
        assert!(message(28).contains("\nOSError: [Errno 28] No space left on device\n"));
    }
}
