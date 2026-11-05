// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The manual pass over the trial cases, against a loopback stub: one
//! request a case, a pause between requests, and a stop at the first answer
//! that is not `COMPUTED`. Nothing here leaves the loopback interface.
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use lodgeit_calculator_adapter::client::Client;
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract;
use lodgeit_calculator_adapter::dial::{self, Net};
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::trials::{Evaluation, fbt, live};
use num_bigint::BigInt;

struct Stub {
    base_url: String,
    requests: Arc<Mutex<usize>>,
}

/// Answers every request with `status` and `body`, counting them.
fn stub(status: u16, body: &Json) -> Stub {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let port = listener.local_addr().expect("a bound address").port();
    let requests = Arc::new(Mutex::new(0));
    let shared = Arc::clone(&requests);
    let reply = pyjson::dumps(body, Layout::Canonical).into_bytes();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            serve(stream, status, &reply, &shared);
        }
    });
    Stub {
        base_url: format!("http://127.0.0.1:{port}"),
        requests,
    }
}

fn serve(stream: TcpStream, status: u16, reply: &[u8], requests: &Mutex<usize>) {
    let mut reader = BufReader::new(stream.try_clone().expect("a second handle"));
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; length];
    if reader.read_exact(&mut body).is_err() {
        return;
    }
    *requests.lock().unwrap() += 1;
    let mut stream = stream;
    let head = format!(
        "HTTP/1.0 {status} Stub\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        reply.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(reply);
    let _ = stream.flush();
}

fn answer() -> Json {
    Json::obj([
        ("benchmark_rate", Json::str("0.0837")),
        ("statutory_myr", Json::str("19450.97")),
        ("total_repayments", Json::str("22000.00")),
        ("shortfall", Json::str("0.00")),
        ("taxable_value", Json::str("8000.00")),
        ("gross_up_factor", Json::str("1.8868")),
        ("grossed_up_taxable_value", Json::str("15094.40")),
        ("fbt_payable", Json::str("7094.37")),
        (
            "manifest",
            Json::obj([
                ("calculator", Json::str("stub")),
                ("period", Json::str("stub")),
                ("rate_table_uris", Json::Array(Vec::new())),
                ("engine", Json::str("stub")),
                ("citation", Json::str("stub")),
            ]),
        ),
        (
            "advisory",
            Json::obj([
                ("figure_type", Json::str("stub")),
                ("notes", Json::Array(vec![Json::str("Not advice.")])),
            ]),
        ),
    ])
}

/// The pass against `stub`, with how many pauses it took.
fn pass(stub: &Stub) -> (live::Pass, usize) {
    let config = AdapterConfig {
        enabled: true,
        base_url: Some(stub.base_url.clone()),
        allow_loopback: true,
        max_attempts: BigInt::from(1),
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a loopback config");
    let contract = contract::load("lodgeit-calculators", None).expect("the snapshot loads");
    let mut net = Net;
    let mut sleep = dial::sleep;
    let mut client = Client {
        config: &config,
        contract: &contract,
        dial: &mut net,
        sleep: &mut sleep,
    };
    let mut pauses = 0;
    let mut result = live::Pass::default();
    let mut pause = || {
        pauses += 1;
        Ok(())
    };
    live::run(&mut client, &mut pause, &mut result).expect("a pass");
    (result, pauses)
}

/// A path for a record, with nothing at it.
fn record_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lodgeit-live-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a temporary directory");
    dir.join("record.json")
}

/// The real path's settings, except that they may reach the stub.
fn to_stub(stub: &Stub) -> AdapterConfig {
    AdapterConfig {
        allow_loopback: true,
        ..live::pass_config(&stub.base_url)
    }
}

fn written(path: &PathBuf) -> Json {
    pyjson::loads_standard(&fs::read_to_string(path).expect("a record")).expect("JSON")
}

#[test]
fn a_clean_pass_sends_each_sendable_case_once_and_pauses_after_each() {
    let stub = stub(200, &answer());
    let (result, pauses) = pass(&stub);
    assert_eq!(result.stopped, None);
    // 7 Division 7A and 5 FBT car requests; D7A-8 and D7A-9 are never sent.
    assert_eq!(*stub.requests.lock().unwrap(), 12);
    assert_eq!(pauses, 12);
    assert_eq!(result.records.len(), 14);
}

#[test]
fn the_record_of_a_pass_is_written_with_its_money_as_exact_digits() {
    let stub = stub(200, &answer());
    let (result, _) = pass(&stub);
    let text = live::record(result, "https://example.invalid", Json::str("snapshot"));
    assert!(
        text.contains("\"amalgamated_base\": \"100000.00\""),
        "{text}"
    );
    let parsed = pyjson::loads_standard(&text).expect("the record is JSON");
    let Some(Json::Array(records)) = parsed.get("records") else {
        panic!("a list of records")
    };
    assert_eq!(records.len(), 14);
}

#[test]
fn the_first_answer_that_is_not_computed_stops_the_pass() {
    let stub = stub(503, &Json::obj([("error", Json::str("unavailable"))]));
    let (result, pauses) = pass(&stub);
    assert_eq!(result.stopped.as_deref(), Some("D7A-1-met-first-real-year"));
    assert_eq!(*stub.requests.lock().unwrap(), 1);
    assert_eq!(pauses, 0);
    assert_eq!(result.records.len(), 1);
}

#[test]
fn a_refusal_stops_the_pass_too() {
    let body = Json::obj([("refusal_class", Json::str("out_of_scope"))]);
    let stub = stub(400, &body);
    let (result, _) = pass(&stub);
    assert_eq!(result.stopped.as_deref(), Some("D7A-1-met-first-real-year"));
    assert_eq!(*stub.requests.lock().unwrap(), 1);
}

#[test]
fn without_enable_network_nothing_is_sent_and_no_file_is_made() {
    let stub = stub(200, &answer());
    let out = record_path("no-flag");
    let path = out.display().to_string();
    let args = |flags: &[&str]| -> Vec<String> {
        let tail = ["--base-url", &stub.base_url, "--out", &path];
        flags.iter().chain(&tail).map(ToString::to_string).collect()
    };
    assert_eq!(live::cli(&args(&[])), 2);
    assert!(!out.exists());
    // The flag is the difference: with it the pass starts, and the real
    // settings refuse the loopback stub before any request.
    assert_eq!(live::cli(&args(&["--enable-network"])), 1);
    assert!(out.exists());
    assert_eq!(*stub.requests.lock().unwrap(), 0);
}

#[test]
fn the_real_path_makes_one_attempt_where_the_default_settings_would_retry() {
    assert!(AdapterConfig::default().max_attempts > BigInt::from(1));
    let stub = stub(503, &Json::obj([("error", Json::str("unavailable"))]));
    let out = record_path("one-attempt");
    let stopped = live::run_to_file(to_stub(&stub), &out, &mut || Ok(()));
    assert_eq!(
        stopped.expect("a pass").as_deref(),
        Some("D7A-1-met-first-real-year")
    );
    assert_eq!(*stub.requests.lock().unwrap(), 1);
    let Some(Json::Array(records)) = written(&out).get("records").cloned() else {
        panic!("a list of records")
    };
    assert_eq!(records.len(), 1);
}

#[test]
fn an_error_part_way_through_keeps_what_was_sent() {
    let stub = stub(200, &answer());
    let out = record_path("error");
    let mut pause = || {
        Err(Exc {
            class: "OSError",
            text: "stub failure".chars().map(u32::from).collect(),
            notes: Vec::new(),
        })
    };
    let result = live::run_to_file(to_stub(&stub), &out, &mut pause);
    assert_eq!(result, Err("OSError: stub failure".to_string()));
    assert_eq!(*stub.requests.lock().unwrap(), 1);
    let record = written(&out);
    let Some(Json::Array(records)) = record.get("records").cloned() else {
        panic!("a list of records")
    };
    assert_eq!(records.len(), 1);
    assert_eq!(
        record.get("error").cloned(),
        Some(Json::str("OSError: stub failure"))
    );
}

#[test]
fn a_pass_that_could_not_keep_its_record_never_starts() {
    let stub = stub(200, &answer());
    let taken = record_path("taken");
    fs::write(&taken, "an earlier record").expect("a file");
    let missing = record_path("missing").with_file_name("no-such-folder/record.json");
    for out in [&taken, &missing] {
        let result = live::run_to_file(to_stub(&stub), out, &mut || Ok(()));
        assert!(result.is_err(), "{}", out.display());
    }
    assert_eq!(*stub.requests.lock().unwrap(), 0);
    assert_eq!(fs::read_to_string(&taken).unwrap(), "an earlier record");
}

#[test]
fn the_local_worksheet_reproduces_each_cases_derived_taxable_value() {
    for case in fbt::load_cases().expect("the packaged cases load") {
        let local = fbt::run_local_statutory(&case.benefit).expect("a worksheet result");
        assert_eq!(
            local[0].1.to_sci_string(),
            case.expected[0].1.to_sci_string(),
            "{}",
            case.benefit.case_id
        );
    }
}

#[test]
fn a_cent_of_formula_difference_is_named() {
    let case = fbt::load_cases()
        .expect("the packaged cases load")
        .remove(1);
    let mut outcome = lodgeit_calculator_adapter::client::Outcome {
        status: lodgeit_calculator_adapter::client::Status::Computed,
        calculator: None,
        period: None,
        request_body: None,
        request_json: None,
        raw_response: Json::Null,
        http_status: Some(200),
        result: Json::Null,
        values: Vec::new(),
        manifest: Json::Null,
        advisory: Json::Null,
        upstream_refusal_class: Json::Null,
        findings: Vec::new(),
        observed_at: String::new(),
        elapsed_ms: None,
        contract_snapshot: Json::Null,
        contract_sha256: String::new(),
        base_url: None,
    };
    let name: Vec<u32> = "taxable_value".chars().map(u32::from).collect();
    let figure = |text: &str| {
        lodgeit_calculator_adapter::decimal::Decimal::parse_exact(text).expect("a literal")
    };
    outcome.values = vec![(name.clone(), figure("4804.38"))];
    let agreed = fbt::compare_formula(&case, &outcome).expect("a comparison");
    assert_eq!(agreed.evaluation, Evaluation::Match);
    outcome.values = vec![(name, figure("4804.40"))];
    let differed = fbt::compare_formula(&case, &outcome).expect("a comparison");
    assert_eq!(differed.evaluation, Evaluation::NumericDifference);
}
