//! README: the server "makes no network calls once installed". This watches the original
//! `tests/test_no_network.py` calculations over stdio.
//!
//! It follows the engines' `tests/no_network.rs`. Linux runs the server under
//! `strace -f`, so any network syscall or child process fails the test; the one
//! exemption is `socketpair(AF_UNIX, ...)`, which the async runtime may use to wake
//! itself and which reaches nothing outside the process. That proves this run, not
//! unexercised paths. CI installs strace; elsewhere the test skips if it is absent.
//!
//! Windows reads the built binary for the name of any network library or
//! process-launching call, which an import must store in plain bytes. That covers
//! every linked path, but cannot see a library loaded by name at run time; the
//! elevated Windows locality proof in the task evidence covers that.

use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

const BINARY: &str = env!("CARGO_BIN_EXE_aus-accounting-mcp");

/// The calculations and figures of the original `tests/test_no_network.py`, plus an
/// ABN check.
fn calls() -> Vec<(&'static str, Value, &'static str, Value)> {
    vec![
        (
            "review_div7a_loan",
            json!({"year_of_income":"2026-27","year_loan_made":"2025-26","written_agreement":true,"terms_in_place_before_lodgment_day":true,"maximum_term_years":"7","secured_by_registered_mortgage_over_real_property":false,"interest_rate_for_years_after_year_loan_made":"0.0837","amalgamated_loan_unpaid_at_end_of_previous_year":"100000.00","remaining_term_years":"1","payments_applied_during_the_year":"108770.00","loan_id":"synthetic-loan-1"}),
            "/minimum_yearly_repayment/verdict",
            json!("MYR_MET"),
        ),
        (
            "get_lodgment_due_dates",
            json!({"obligation":"bas","entity_type":"company","financial_year":"2026-27","channel":"bas_agent","reporting_cycle":"quarterly","facts":{"consolidated_head_company":false,"lodges_electronically":true,"previous_statement_lodged_electronically":true,"includes_monthly_payg_obligation":false}}),
            "/obligations/1/due_date",
            json!("2027-03-02"),
        ),
        (
            "calculate_tax_worksheet",
            json!({"facts":{"kind":"gst","amount":"1100","gst_inclusive":true,"scope_confirmed":true,"year":"2025-26"}}),
            "/amounts/gst",
            json!("100.00"),
        ),
        (
            "calc_payday_super_deadline",
            json!({"qe_day":"2026-08-06","sg_amount":"800.00","remitted":"2026-08-07","as_at":"2026-08-21"}),
            "/result/verdict",
            json!("AT_RISK"),
        ),
        (
            "get_ato_benchmarks",
            json!({"industry":"Bakeries and hot bread shops","turnover":"850000.00","other_income":"0","cost_of_sales":"270000.00"}),
            "/ratios",
            Value::Null,
        ),
        (
            "review_gst_coding",
            json!({"lines":[{"id":"1","entity":"Synthetic Alder Pty Ltd","date":"2026-08-03","amount":"-110.00","account":"Repairs","gst_code":"GST","confirmed":true,"supplier_registered":false}],"registrations":[{"entity":"Synthetic Alder Pty Ltd","registered_from":"2024-07-01"}]}),
            "/lines/0/state",
            json!("gst_differs_from_registration"),
        ),
        (
            "validate_abn",
            json!({"abn":"51 824 753 556"}),
            "/result",
            json!("CHECK_DIGIT_VALID"),
        ),
    ]
}

fn calculate(mut command: Command) {
    let mut child = command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = vec![
        json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"no-network","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}),
    ];
    for (id, (name, arguments, _, _)) in calls().into_iter().enumerate() {
        input.push(json!({"jsonrpc":"2.0","id":id + 1,"method":"tools/call","params":{"name":name,"arguments":arguments}}));
    }
    let mut stdin = child.stdin.take().unwrap();
    for message in input {
        writeln!(stdin, "{message}").unwrap();
    }
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    let replies: Vec<Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for (id, (name, _, pointer, figure)) in calls().into_iter().enumerate() {
        let reply = replies.iter().find(|r| r["id"] == id + 1).unwrap();
        assert_eq!(reply["result"]["isError"], false, "{name}: {reply}");
        let found = &reply["result"]["structuredContent"].pointer(pointer);
        if figure.is_null() {
            // Python asserts only that the ratios are non-empty.
            assert!(
                found.is_some_and(|v| v.as_array().is_some_and(|a| !a.is_empty())),
                "{name}: {reply}"
            );
        } else {
            assert_eq!(*found, Some(&figure), "{name}: {reply}");
        }
    }
}

#[test]
fn real_tool_calls_give_their_figures() {
    calculate(Command::new(BINARY));
}

#[cfg(target_os = "linux")]
#[test]
fn real_tool_calls_make_no_network_syscall() {
    if Command::new("strace").arg("-V").output().is_err() {
        assert!(std::env::var_os("CI").is_none(), "CI must install strace");
        eprintln!("skipped: strace is not installed");
        return;
    }
    let log = concat!(env!("CARGO_TARGET_TMPDIR"), "/no_network.strace");
    let mut command = Command::new("strace");
    command.args([
        "-f",
        "-qq",
        "-e",
        "trace=%network,execve,execveat,fork,vfork",
        "-o",
        log,
        BINARY,
    ]);
    calculate(command);
    // The server's own execve shows the tracer was attached; anything else is a
    // network syscall or a child process.
    let trace = std::fs::read_to_string(log).unwrap();
    let calls: Vec<&str> = trace
        .lines()
        .filter(|line| !line.contains(" socketpair(AF_UNIX,"))
        .collect();
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
