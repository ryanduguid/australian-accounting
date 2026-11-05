// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! One manual pass over the Division 7A and FBT car cases against a live
//! service: one request a case, a pause between requests, and a stop at the
//! first answer that is not `COMPUTED`. Nothing here retries, and nothing
//! calls it except `examples/live_trials.rs`, which a person runs by hand.
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use num_bigint::BigInt;

use super::{Comparison, Evaluation, div7a, fbt};
use crate::client::{Client, Outcome};
use crate::config::AdapterConfig;
use crate::contract;
use crate::dial::{self, Net};
use crate::pyjson::{self, Json, Layout};
use crate::pyops::{Exc, R, json_str, pts};
use crate::pyvalue::points_lossy;

/// The gap between requests. The service runs at its operator's expense.
const PAUSE: Duration = Duration::from_secs(3);

/// The record of one pass. `stopped` names the case the pass stopped at and
/// `error` what ended it early.
#[derive(Default)]
pub struct Pass {
    pub records: Vec<Json>,
    pub stopped: Option<String>,
    pub error: Option<String>,
}

fn sent(comparisons: Vec<(&str, Comparison)>, outcome: &Outcome, body: Json) -> Json {
    Json::obj([
        ("sent", Json::Bool(true)),
        ("request", body),
        (
            "http_status",
            outcome
                .http_status
                .map_or(Json::Null, |status| Json::Int(status.into())),
        ),
        ("observed_at", Json::str(&outcome.observed_at)),
        ("provider_result", outcome.result.clone()),
        (
            "findings",
            Json::Array(outcome.findings.iter().cloned().map(json_str).collect()),
        ),
        (
            "comparisons",
            Json::Array(
                comparisons
                    .into_iter()
                    .map(|(trial, comparison)| {
                        Json::obj([
                            ("trial", Json::str(trial)),
                            ("result", comparison.to_json()),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

/// One request. The attempt is saved before it goes out and the answer before
/// it is compared, so a failure at either point leaves what was sent. Returns
/// whether the answer was computed.
fn exchange(
    pass: &mut Pass,
    save: &mut dyn FnMut(&Pass) -> R<()>,
    body: Json,
    invoke: impl FnOnce(&Json) -> R<Outcome>,
    compare: impl FnOnce(&Outcome) -> R<Vec<(&'static str, Comparison)>>,
) -> R<bool> {
    let at = pass.records.len();
    pass.records.push(Json::obj([
        ("attempted", Json::Bool(true)),
        ("request", body.clone()),
    ]));
    save(pass)?;
    let outcome = invoke(&body)?;
    pass.records[at] = sent(Vec::new(), &outcome, body.clone());
    save(pass)?;
    let comparisons = compare(&outcome)?;
    pass.records[at] = sent(comparisons, &outcome, body);
    save(pass)?;
    Ok(outcome.computed())
}

/// Run every case once, calling `pause` between requests and `save` as the
/// record grows. `pass` holds what was sent even if this returns an error.
pub fn run(
    client: &mut Client<'_>,
    pause: &mut dyn FnMut() -> R<()>,
    pass: &mut Pass,
    save: &mut dyn FnMut(&Pass) -> R<()>,
) -> R<()> {
    for case in div7a::load_cases(None)? {
        let unsent = div7a::evaluate(&case, None)?;
        if unsent.evaluation != Evaluation::NotRun {
            pass.records.push(Json::obj([
                ("sent", Json::Bool(false)),
                ("comparison", unsent.to_json()),
            ]));
            continue;
        }
        let (verdict, values, reasons) = div7a::run_local(&case)?;
        let body = div7a::build_request(&case);
        let computed = exchange(
            pass,
            save,
            body,
            |body| client.invoke(&pts(div7a::CALC_URI), &pts(&case.period_uri), body),
            |outcome| {
                let comparison =
                    div7a::evaluate_outcome(&case, &verdict, &values, &reasons, outcome)?;
                Ok(vec![("div7a", comparison)])
            },
        )?;
        if !computed {
            pass.stopped = Some(case.case_id);
            return Ok(());
        }
        pause()?;
    }
    for case in fbt::load_cases()? {
        let body = case.benefit.request()?;
        let computed = exchange(
            pass,
            save,
            body,
            |body| client.invoke(&pts(fbt::CALC_URI), &pts(&case.benefit.period_uri), body),
            |outcome| {
                let mut comparisons = vec![(
                    "fbt-gross-up",
                    fbt::evaluate_outcome(&case.benefit, outcome)?,
                )];
                if outcome.computed() {
                    comparisons.insert(
                        0,
                        ("fbt-car-formula", fbt::compare_formula(&case, outcome)?),
                    );
                }
                Ok(comparisons)
            },
        )?;
        if !computed {
            pass.stopped = Some(case.benefit.case_id);
            return Ok(());
        }
        pause()?;
    }
    Ok(())
}

/// The written record of a pass, with its money as exact digits.
pub fn record(pass: &Pass, base_url: &str, snapshot: &Json) -> String {
    let stopped_at = pass.stopped.as_deref().map_or(Json::Null, Json::str);
    let error = pass.error.as_deref().map_or(Json::Null, Json::str);
    let record = Json::obj([
        ("base_url", Json::str(base_url)),
        ("contract_snapshot", snapshot.clone()),
        ("synthetic_input", Json::Bool(true)),
        ("stopped_at", stopped_at),
        ("error", error),
        ("records", Json::Array(pass.records.clone())),
    ]);
    pyjson::dumps(&record, Layout::Indented) + "\n"
}

/// Rewrite the record through a temporary file, so an interrupted pass leaves
/// the last complete copy.
fn keep(pass: &Pass, base_url: &str, snapshot: &Json, out: &Path) -> std::io::Result<()> {
    let partial = out.with_extension("partial");
    std::fs::write(&partial, record(pass, base_url, snapshot))?;
    std::fs::rename(&partial, out)
}

/// The real path's settings: enabled for this pass, with no retries.
pub fn pass_config(base_url: &str) -> AdapterConfig {
    AdapterConfig {
        enabled: true,
        base_url: Some(base_url.to_owned()),
        max_attempts: BigInt::from(1),
        ..AdapterConfig::default()
    }
}

/// One pass over the real network, its record kept in `out` as it grows. `out`
/// must not yet exist: a pass that cannot keep its record never starts. Returns
/// the case the pass stopped at, or an error's text.
pub fn run_to_file(
    config: AdapterConfig,
    out: &Path,
    pause: &mut dyn FnMut() -> R<()>,
) -> Result<Option<String>, String> {
    let shown = |error: std::io::Error| format!("{}: {error}", out.display());
    File::create_new(out).map_err(shown)?;
    let base_url = config.base_url.clone().unwrap_or_default();
    let mut pass = Pass::default();
    let mut snapshot = Json::Null;
    let result = (|| {
        let config = config.checked()?;
        let contract = contract::load("lodgeit-calculators", None)?;
        snapshot = contract.snapshot_id.clone();
        let mut net = Net;
        let mut sleep = dial::sleep;
        let mut client = Client {
            config: &config,
            contract: &contract,
            dial: &mut net,
            sleep: &mut sleep,
        };
        let mut save = |pass: &Pass| {
            keep(pass, &base_url, &contract.snapshot_id, out)
                .map_err(|error| Exc::msg("OSError", &error.to_string()))
        };
        run(&mut client, pause, &mut pass, &mut save)
    })();
    pass.error = result
        .err()
        .map(|error| format!("{}: {}", error.class, points_lossy(&error.text)));
    let (stopped, error) = (pass.stopped.clone(), pass.error.clone());
    keep(&pass, &base_url, &snapshot, out).map_err(shown)?;
    error.map_or(Ok(stopped), Err)
}

/// The example's command line. Without `--enable-network`, `--base-url` and
/// `--out` it exits 2, having sent nothing and made no file.
pub fn cli(args: &[String]) -> u8 {
    let value = |flag: &str| {
        args.iter()
            .position(|arg| arg == flag)
            .and_then(|at| args.get(at + 1))
    };
    let (Some(base_url), Some(out), true) = (
        value("--base-url"),
        value("--out"),
        args.iter().any(|arg| arg == "--enable-network"),
    ) else {
        eprintln!("usage: live_trials --enable-network --base-url URL --out FILE");
        return 2;
    };
    let mut pause = || {
        std::thread::sleep(PAUSE);
        Ok(())
    };
    match run_to_file(pass_config(base_url), Path::new(out), &mut pause) {
        Ok(None) => 0,
        Ok(Some(case)) => {
            eprintln!("stopped at {case}; read {out}");
            1
        }
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}
