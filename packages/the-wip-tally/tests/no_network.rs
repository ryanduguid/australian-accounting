//! README: the engines 'contact no service'. This watches a real calculation.
//!
//! Linux runs it under `strace -f`, so any network syscall, from any library or
//! child process, fails the test. That proves this one run, not unexercised
//! paths. CI installs strace; elsewhere the test skips with a message if absent.
//!
//! Windows has no such tool in CI, so the test reads the built binary for the
//! name of any network library or process-launching call, which an import must
//! store in plain bytes. That covers every linked path, but cannot see a library
//! loaded by name at run time. Linux cannot use this check: a debug binary lists
//! every socket call std has, used or not. macOS only checks the figure.

use std::io::Write;
use std::process::{Command, Stdio};

const BINARY: &str = env!("CARGO_BIN_EXE_wip-tally");
const ARGUMENTS: &[&str] = &["measure"];
const INPUT: &str = r#"{"approved_variations": "0.00", "certified_billings": "450.00",
    "committed_outstanding": "380.00", "constraint_include_ratio": "0", "contract_id": "MAP-1",
    "costs_incurred": "400.00", "customer": "", "description": "",
    "estimated_cost_to_complete": "400.00", "gst_rate": "0.10",
    "inefficiency_rework_wastage": "0.00", "line_number": 2, "original_contract_sum": "1000.00",
    "outcome_reasonably_measurable": true, "progress_method": "cost_to_cost",
    "retention_classification": "review", "retention_withheld": "0.00",
    "unapproved_variations_estimate": "0.00", "uncertified_claims": "0.00",
    "uninstalled_materials": "0.00"}"#;
const EXIT_CODE: i32 = 0;
const FIGURE: &str = r#""revenue_to_date":"500.00""#;

// Everything below is the same in every engine; tests/test_shared_blocks.py pins it.

fn calculate(mut command: Command) {
    let mut child = command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(INPUT.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(EXIT_CODE), "{stderr}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(FIGURE), "{FIGURE} not in: {stdout}");
}

#[test]
fn a_real_calculation_gives_its_figure() {
    let mut command = Command::new(BINARY);
    command.args(ARGUMENTS);
    calculate(command);
}

#[cfg(target_os = "linux")]
#[test]
fn a_real_calculation_makes_no_network_syscall() {
    if Command::new("strace").arg("-V").output().is_err() {
        assert!(std::env::var_os("CI").is_none(), "CI must install strace");
        eprintln!("skipped: strace is not installed");
        return;
    }
    let log = concat!(env!("CARGO_TARGET_TMPDIR"), "/no_network.strace");
    let mut command = Command::new("strace");
    command
        .args([
            "-f",
            "-qq",
            "-e",
            "trace=%network,execve",
            "-o",
            log,
            BINARY,
        ])
        .args(ARGUMENTS);
    calculate(command);
    // The engine's own execve shows the tracer was attached; anything more is a
    // network syscall or a child process.
    let trace = std::fs::read_to_string(log).unwrap();
    let calls: Vec<&str> = trace.lines().collect();
    assert_eq!(calls.len(), 1, "unexpected syscalls:\n{trace}");
    assert!(
        calls[0].contains("execve(") && calls[0].ends_with("= 0"),
        "{trace}"
    );
}

#[cfg(windows)]
#[test]
fn the_binary_imports_no_network_or_process_call() {
    let binary = std::fs::read(BINARY).unwrap().to_ascii_lowercase();
    let names = |name: &str| binary.windows(name.len()).any(|w| w == name.as_bytes());
    assert!(names("kernel32"), "the import table is not readable");
    for name in [
        "ws2_32",
        "wsock32",
        "mswsock",
        "winhttp",
        "wininet",
        "dnsapi",
        "iphlpapi",
        "urlmon",
        "createprocess",
        "shellexecute",
    ] {
        assert!(!names(name), "{BINARY} names {name}");
    }
}
