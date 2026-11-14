// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The reference `npm test` compares the built Worker with, byte for byte:
//! this crate's own replies, computed natively from the engines in `packages/`,
//! to the 32 conformance cases and the served tools' other calls, in both
//! protocol eras. This test fails when `fixtures/reference.json` differs from
//! the replies at this commit; record it again with
//!   RECORD_REFERENCE=1 cargo test --locked --test reference
use aus_accounting_connector::{COMMIT, Headers, MODERN, SERVE_UNTIL, respond};
use chrono::NaiveDate;
use serde_json::{Value, json};
use std::{collections::HashMap, fs};

const LEGACY: &str = "2025-11-25";
const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reference.json");
const CASES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../apps/aus-accounting-mcp/conformance/cases.json"
);

struct Sent(HashMap<&'static str, String>);

impl Headers for Sent {
    fn get(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
}

/// The calls compared: the conformance cases, the no-argument tool and the data
/// freshness report for each served tool, for all of them and on a date.
fn requests() -> Vec<Value> {
    let cases: Value = serde_json::from_str(&fs::read_to_string(CASES).unwrap()).unwrap();
    let mut calls: Vec<Value> = cases["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| json!({"name":case["id"],"tool":case["tool"],"arguments":case["arguments"]}))
        .collect();
    calls.push(json!({"name":"refuse_div7a","tool":"refuse_div7a","arguments":{}}));
    for tool in [
        "calc_payday_super_deadline",
        "get_div7a_benchmark_rate",
        "review_div7a_loan",
        "calculate_tax_worksheet",
    ] {
        calls.push(
            json!({"name":format!("freshness_{tool}"),"tool":"get_data_freshness",
            "arguments":{"tool":tool}}),
        );
    }
    calls.push(json!({"name":"freshness_all","tool":"get_data_freshness","arguments":{}}));
    calls.push(json!({"name":"freshness_as_at","tool":"get_data_freshness",
        "arguments":{"tool":"get_div7a_benchmark_rate","as_at":"2026-11-14"}}));
    calls
}

/// The text of `result` in the reply to one request, sent as a conforming client
/// sends it in `era`, on a day the release still serves. `serverInfo` names the
/// commit as `reference`, so the file does not depend on the build's commit.
fn result(era: &str, id: usize, method: &str, mut params: Value) -> String {
    let mut sent = HashMap::from([("content-type", "application/json".to_owned())]);
    sent.insert("mcp-protocol-version", era.to_owned());
    if era == MODERN {
        params["_meta"] = json!({"io.modelcontextprotocol/protocolVersion":MODERN,
            "io.modelcontextprotocol/clientCapabilities":{}});
        sent.insert("mcp-method", method.to_owned());
        if let Some(name) = params["name"].as_str() {
            sent.insert("mcp-name", name.to_owned());
        }
    }
    let body = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
    let served = NaiveDate::parse_from_str(SERVE_UNTIL, "%Y-%m-%d")
        .unwrap()
        .pred_opt();
    let reply = respond(
        body.to_string().as_bytes(),
        &Sent(sent),
        &served.unwrap().to_string(),
    );
    assert_eq!(reply.status, 200, "{body}");
    let text = reply.body.unwrap();
    let head = format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":");
    let result = text.strip_prefix(&head).and_then(|t| t.strip_suffix('}'));
    let version = format!("\"version\":\"{}+", env!("CARGO_PKG_VERSION"));
    let release = format!("{version}{COMMIT}\"");
    result
        .unwrap()
        .replace(&release, &format!("{version}reference\""))
}

fn recorded() -> String {
    let mut fixture = json!({"source":{
        "replies":"this crate's handler, run natively with the engines in packages/ at the same commit",
        "record":"RECORD_REFERENCE=1 cargo test --locked --test reference"}});
    for era in [MODERN, LEGACY] {
        let calls: Vec<Value> = requests()
            .into_iter()
            .enumerate()
            .map(|(id, mut call)| {
                let params = json!({"name":call["tool"],"arguments":call["arguments"]});
                call["result"] = json!(result(era, id, "tools/call", params));
                call
            })
            .collect();
        let tools = result(era, calls.len(), "tools/list", json!({}));
        fixture[era] = json!({"tools":tools,"calls":calls});
    }
    serde_json::to_string_pretty(&fixture).unwrap() + "\n"
}

#[test]
fn the_reference_is_this_commits_replies() {
    let rebuilt = recorded();
    if std::env::var_os("RECORD_REFERENCE").is_some() {
        fs::write(FIXTURE, &rebuilt).unwrap();
    }
    let committed = fs::read_to_string(FIXTURE).unwrap();
    assert!(
        committed == rebuilt,
        "tests/fixtures/reference.json is stale: from tools/aus-accounting-connector, run RECORD_REFERENCE=1 cargo test --locked --test reference"
    );
}
