// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! One manual pass over the Division 7A and FBT car cases against a live
//! service: one request a case, a pause between requests, and a stop at the
//! first answer that is not `COMPUTED`. Nothing here retries, and nothing
//! calls it except `examples/live_trials.rs`, which a person runs by hand.
use std::path::Path;
use std::time::Duration;

use num_bigint::BigInt;

use super::{Comparison, Evaluation, div7a, fbt};
use crate::client::{Client, Outcome};
use crate::config::AdapterConfig;
use crate::contract;
use crate::dial::{self, Net};
use crate::pyjson::{self, Json, Layout};
use crate::pyops::{R, json_str, pts};

/// The gap between requests. The service runs at its operator's expense.
const PAUSE: Duration = Duration::from_secs(3);

/// The record of one pass. `stopped` names the case the pass stopped at.
pub struct Pass {
    pub records: Vec<Json>,
    pub stopped: Option<String>,
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

/// Run every case once, calling `pause` between requests.
pub fn run(client: &mut Client<'_>, pause: &mut dyn FnMut() -> R<()>) -> R<Pass> {
    let mut pass = Pass {
        records: Vec::new(),
        stopped: None,
    };
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
        let outcome = client.invoke(&pts(div7a::CALC_URI), &pts(&case.period_uri), &body)?;
        let comparison = div7a::evaluate_outcome(&case, &verdict, &values, &reasons, &outcome)?;
        pass.records
            .push(sent(vec![("div7a", comparison)], &outcome, body));
        if !outcome.computed() {
            pass.stopped = Some(case.case_id);
            return Ok(pass);
        }
        pause()?;
    }
    for case in fbt::load_cases()? {
        let body = case.benefit.request()?;
        let outcome = client.invoke(&pts(fbt::CALC_URI), &pts(&case.benefit.period_uri), &body)?;
        let mut comparisons = vec![(
            "fbt-gross-up",
            fbt::evaluate_outcome(&case.benefit, &outcome)?,
        )];
        if outcome.computed() {
            comparisons.insert(
                0,
                ("fbt-car-formula", fbt::compare_formula(&case, &outcome)?),
            );
        }
        pass.records.push(sent(comparisons, &outcome, body));
        if !outcome.computed() {
            pass.stopped = Some(case.benefit.case_id);
            return Ok(pass);
        }
        pause()?;
    }
    Ok(pass)
}

/// The written record of a pass, with its money as exact digits.
pub fn record(pass: Pass, base_url: &str, snapshot: Json) -> String {
    let stopped_at = pass.stopped.as_deref().map_or(Json::Null, Json::str);
    let record = Json::obj([
        ("base_url", Json::str(base_url)),
        ("contract_snapshot", snapshot),
        ("synthetic_input", Json::Bool(true)),
        ("stopped_at", stopped_at),
        ("records", Json::Array(pass.records)),
    ]);
    pyjson::dumps(&record, Layout::Indented) + "\n"
}

/// One pass over the real network with no retries, its record written to
/// `out`. Returns the case the pass stopped at, or an error's text.
pub fn run_to_file(base_url: &str, out: &Path) -> Result<Option<String>, String> {
    let attempt = || -> R<(Pass, Json)> {
        let config = AdapterConfig {
            enabled: true,
            base_url: Some(base_url.to_owned()),
            max_attempts: BigInt::from(1),
            ..AdapterConfig::default()
        }
        .checked()?;
        let contract = contract::load("lodgeit-calculators", None)?;
        let mut net = Net;
        let mut sleep = dial::sleep;
        let mut client = Client {
            config: &config,
            contract: &contract,
            dial: &mut net,
            sleep: &mut sleep,
        };
        let mut pause = || {
            std::thread::sleep(PAUSE);
            Ok(())
        };
        Ok((run(&mut client, &mut pause)?, contract.snapshot_id.clone()))
    };
    let (pass, snapshot) = attempt().map_err(|error| {
        let text: String = error
            .text
            .iter()
            .copied()
            .filter_map(char::from_u32)
            .collect();
        format!("{}: {text}", error.class)
    })?;
    let stopped = pass.stopped.clone();
    std::fs::write(out, record(pass, base_url, snapshot))
        .map_err(|error| format!("{}: {error}", out.display()))?;
    Ok(stopped)
}
