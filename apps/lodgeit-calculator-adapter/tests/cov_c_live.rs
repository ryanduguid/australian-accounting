//! The manual live pass's command line and FBT stage, through an in-memory
//! `Dial` that answers by request target: nothing is dialled, so the real
//! settings (https, the known hosts, one attempt) apply unchanged. The
//! retired Python adapter had no live pass, so the expected values are the
//! module's documented contract and the packaged cases (7 sendable Division
//! 7A cases, 2 never sent, then 5 FBT car cases).
use std::cell::RefCell;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use lodgeit_calculator_adapter::client::Client;
use lodgeit_calculator_adapter::contract;
use lodgeit_calculator_adapter::dial;
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::pyvalue::points_lossy;
use lodgeit_calculator_adapter::transport::{Connection, Dial};
use lodgeit_calculator_adapter::trials::{fbt, live};

const HOST: &str = "fbt-calculator-api-qkp3j5bjnq-ts.a.run.app";

/// A provider answer every packaged case computes against.
fn answer() -> Vec<u8> {
    let manifest = Json::obj([
        ("calculator", Json::str("fixture")),
        ("period", Json::str("fixture")),
        ("rate_table_uris", Json::Array(Vec::new())),
        ("engine", Json::str("fixture")),
        ("citation", Json::str("fixture")),
    ]);
    let advisory = Json::obj([
        ("figure_type", Json::str("fixture")),
        ("notes", Json::Array(vec![Json::str("Not advice.")])),
    ]);
    let body = Json::obj([
        ("benchmark_rate", Json::str("0.0837")),
        ("statutory_myr", Json::str("19450.97")),
        ("total_repayments", Json::str("22000.00")),
        ("shortfall", Json::str("0.00")),
        ("taxable_value", Json::str("8000.00")),
        ("gross_up_factor", Json::str("1.8868")),
        ("grossed_up_taxable_value", Json::str("15094.40")),
        ("fbt_payable", Json::str("7094.37")),
        ("manifest", manifest),
        ("advisory", advisory),
    ]);
    pyjson::dumps(&body, Layout::Canonical).into_bytes()
}

fn response(status: u16, body: &[u8]) -> Vec<u8> {
    let head = format!(
        "HTTP/1.0 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    [head.as_bytes(), body].concat()
}

/// One connection: the reply is chosen from the request line once the
/// request has been written.
struct Exchange {
    written: Vec<u8>,
    reply: Option<io::Cursor<Vec<u8>>>,
    fbt_status: u16,
    targets: Rc<RefCell<Vec<String>>>,
}

impl Read for Exchange {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.reply.is_none() {
            let text = String::from_utf8_lossy(&self.written);
            let target = text.split(' ').nth(1).unwrap_or_default().to_owned();
            let status = if target.contains(fbt::CALC_URI) {
                self.fbt_status
            } else {
                200
            };
            self.targets.borrow_mut().push(target);
            self.reply = Some(io::Cursor::new(response(status, &answer())));
        }
        self.reply.as_mut().expect("chosen above").read(buf)
    }
}

impl Write for Exchange {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Answers Division 7A requests 200 and FBT requests `fbt_status`,
/// recording each dial and each request target.
struct Provider {
    fbt_status: u16,
    dials: Vec<(String, u16, bool)>,
    targets: Rc<RefCell<Vec<String>>>,
}

impl Provider {
    fn new(fbt_status: u16) -> Provider {
        Provider {
            fbt_status,
            dials: Vec::new(),
            targets: Rc::default(),
        }
    }
}

impl Dial for Provider {
    fn dial(
        &mut self,
        host: &str,
        port: u16,
        tls: bool,
        _: f64,
    ) -> io::Result<Box<dyn Connection>> {
        self.dials.push((host.to_owned(), port, tls));
        Ok(Box::new(Exchange {
            written: Vec::new(),
            reply: None,
            fbt_status: self.fbt_status,
            targets: Rc::clone(&self.targets),
        }))
    }
}

/// A path for a record, with nothing at it.
fn record_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lodgeit-cov-live-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a temporary directory");
    dir.join("record.json")
}

fn args(out: &Path) -> Vec<String> {
    [
        "--enable-network",
        "--base-url",
        &format!("https://{HOST}"),
        "--out",
        &out.display().to_string(),
    ]
    .map(str::to_owned)
    .to_vec()
}

fn written(out: &Path) -> Json {
    pyjson::loads_standard(&fs::read_to_string(out).expect("a record")).expect("JSON")
}

fn first_fbt_case() -> String {
    fbt::load_cases().expect("the packaged cases load")[0]
        .benefit
        .case_id
        .clone()
}

#[test]
fn a_pass_where_every_answer_is_computed_exits_zero_with_its_record_kept() {
    let out = record_path("clean");
    let mut provider = Provider::new(200);
    let mut pauses = 0;
    let code = live::cli_through(&args(&out), &mut provider, &mut || {
        pauses += 1;
        Ok(())
    });
    assert_eq!(code, 0);
    // One TLS dial to the configured host a request, one pause after each.
    assert_eq!(provider.dials, vec![(HOST.to_owned(), 443, true); 12]);
    assert_eq!(pauses, 12);
    let record = written(&out);
    assert_eq!(record.at("stopped_at"), &Json::Null);
    assert_eq!(record.at("error"), &Json::Null);
    let Json::Array(records) = record.at("records") else {
        panic!("records is a list")
    };
    assert_eq!(records.len(), 14);
}

#[test]
fn an_fbt_answer_that_is_not_computed_stops_the_pass_at_that_case_and_exits_one() {
    let out = record_path("fbt-stop");
    let mut provider = Provider::new(503);
    let mut pauses = 0;
    let code = live::cli_through(&args(&out), &mut provider, &mut || {
        pauses += 1;
        Ok(())
    });
    assert_eq!(code, 1);
    // 7 Division 7A requests, each followed by a pause, then the first FBT
    // request, after which nothing more is sent.
    let targets = provider.targets.borrow();
    assert_eq!(targets.len(), 8);
    assert!(targets[7].contains(fbt::CALC_URI), "{}", targets[7]);
    assert_eq!(pauses, 7);
    let record = written(&out);
    assert_eq!(record.at("stopped_at"), &Json::str(&first_fbt_case()));
    let Json::Array(records) = record.at("records") else {
        panic!("records is a list")
    };
    // 7 sent and 2 unsent Division 7A records, then the stopped FBT one,
    // which carries only its gross-up comparison.
    assert_eq!(records.len(), 10);
    let Json::Array(comparisons) = records[9].at("comparisons") else {
        panic!("comparisons is a list")
    };
    assert_eq!(comparisons.len(), 1);
    assert_eq!(comparisons[0].at("trial"), &Json::str("fbt-gross-up"));
}

#[test]
fn a_record_that_cannot_be_saved_in_the_fbt_stage_ends_the_pass_with_that_error() {
    let config = live::pass_config(&format!("https://{HOST}"))
        .checked()
        .expect("the real settings");
    let contract = contract::load("lodgeit-calculators", None).expect("the snapshot loads");
    let mut provider = Provider::new(200);
    let mut sleep = dial::sleep;
    let mut client = Client {
        config: &config,
        contract: &contract,
        dial: &mut provider,
        sleep: &mut sleep,
    };
    let mut pass = live::Pass::default();
    // Saving fails from the first FBT attempt on (record 10).
    let mut save = |pass: &live::Pass| {
        if pass.records.len() > 9 {
            Err(Exc {
                class: "OSError",
                text: "fixture: disk full".chars().map(u32::from).collect(),
                notes: Vec::new(),
            })
        } else {
            Ok(())
        }
    };
    let error =
        live::run(&mut client, &mut || Ok(()), &mut pass, &mut save).expect_err("the save fails");
    assert_eq!(error.class, "OSError");
    assert_eq!(points_lossy(&error.text), "fixture: disk full");
    // The attempt was recorded before the save that failed, and never sent.
    assert_eq!(pass.records.len(), 10);
    assert_eq!(pass.records[9].at("attempted"), &Json::Bool(true));
    assert_eq!(provider.dials.len(), 7);
}

#[test]
fn the_real_pause_waits_three_seconds() {
    let started = Instant::now();
    live::pause().expect("a sleep");
    assert!(started.elapsed().as_secs_f64() >= 3.0);
}
