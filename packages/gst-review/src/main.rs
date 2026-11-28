// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use gst_review::{api, cli};
use std::io::{Read, Write};

fn run_api(command: &str) -> cli::Outcome {
    run_api_from(command, std::io::stdin())
}
fn run_api_from(command: &str, reader: impl Read) -> cli::Outcome {
    let mut bytes = Vec::new();
    if let Err(e) = reader.take((8 << 20) + 1).read_to_end(&mut bytes) {
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
        Err(e) => (2, String::new(), format!("error: {e}\n")),
    }
}
fn exit_code(code: i32, out_result: std::io::Result<()>, err_result: std::io::Result<()>) -> i32 {
    if out_result.is_err() || err_result.is_err() {
        1
    } else {
        code
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
    std::process::exit(exit_code(code, out_result, err_result));
}

#[cfg(test)]
mod tests {
    use super::{exit_code, run_api_from};
    use std::io::{self, ErrorKind, Read};

    struct FailingReader;
    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    #[test]
    fn stdin_read_failure_returns_exit_two_with_error_message() {
        let outcome = run_api_from("review-coding", FailingReader);
        assert_eq!(outcome, (2, String::new(), "error: read failed\n".into()));
    }

    #[test]
    fn stream_write_failure_returns_exit_one() {
        let fail = || Err(io::Error::from(ErrorKind::BrokenPipe));
        assert_eq!(exit_code(0, fail(), Ok(())), 1);
        assert_eq!(exit_code(2, Ok(()), fail()), 1);
        assert_eq!(exit_code(2, Ok(()), Ok(())), 2);
    }
}
