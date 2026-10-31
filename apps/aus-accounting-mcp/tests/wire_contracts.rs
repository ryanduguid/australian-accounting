//! Frozen original JSON-RPC exchanges, replayed through the actual stdio binary.
use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
struct SessionDirectory(PathBuf);
impl SessionDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "native-mcp-wire-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("new owned session directory");
        Self(path)
    }
}
impl Drop for SessionDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Server {
    child: Child,
    reader: Option<thread::JoinHandle<()>>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.child.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            reader.join().expect("stdio reader joined");
        }
    }
}
fn canonical(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for value in object.values_mut() {
                canonical(value);
            }
            if (object.get("type").is_some_and(|v| v == "text")
                || object
                    .get("mimeType")
                    .is_some_and(|v| v == "application/json"))
                && let Some(Value::String(text)) = object.get("text")
                && let Ok(mut json) = serde_json::from_str(text)
            {
                canonical(&mut json);
                object.insert("text".into(), json);
            }
        }
        Value::Array(items) => {
            for item in items {
                canonical(item);
            }
        }
        _ => {}
    }
}
fn difference(a: &Value, b: &Value, path: &str) -> Option<String> {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return Some(format!(
                    "{path}: keys {:?} != {:?}",
                    a.keys().collect::<Vec<_>>(),
                    b.keys().collect::<Vec<_>>()
                ));
            }
            for (key, value) in a {
                let Some(expected) = b.get(key) else {
                    return Some(format!("{path}/{key}: unexpected key"));
                };
                if let Some(found) = difference(value, expected, &format!("{path}/{key}")) {
                    return Some(found);
                }
            }
            None
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Some(format!("{path}: lengths {} != {}", a.len(), b.len()));
            }
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                if let Some(found) = difference(a, b, &format!("{path}/{i}")) {
                    return Some(found);
                }
            }
            None
        }
        _ if a == b => None,
        _ => Some(format!(
            "{path}: {:?} != {:?}",
            a.to_string().chars().take(150).collect::<String>(),
            b.to_string().chars().take(150).collect::<String>()
        )),
    }
}
#[test]
fn original_protocol_sessions_over_stdio() {
    let sessions: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/wire-sessions.json")).unwrap();
    assert_eq!(sessions.len(), 8);
    assert_eq!(replay(sessions), 84);
}
#[test]
fn original_lifecycle_sessions_over_stdio() {
    let sessions: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/lifecycle-sessions.json")).unwrap();
    assert_eq!(sessions.len(), 6);
    assert_eq!(replay(sessions), 18);
}
#[test]
fn original_lifecycle_boundaries_over_stdio() {
    let sessions: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/lifecycle-edges.json")).unwrap();
    assert_eq!(sessions.len(), 7);
    assert_eq!(replay(sessions), 1044);
}
#[test]
fn original_legacy_openers_over_stdio() {
    let sessions: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/legacy-openers.json")).unwrap();
    assert_eq!(sessions.len(), 9);
    assert_eq!(replay(sessions), 18);
}
#[test]
fn original_large_pack_pagination_and_scope_over_stdio() {
    let sessions: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/wire-coverage.json")).unwrap();
    assert_eq!(sessions.len(), 4);
    assert_eq!(replay(sessions), 11);
}

#[test]
fn unknown_arguments_and_expected_input_errors_leave_the_session_usable() {
    use serde_json::json;
    let original: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/wire-sessions.json")).unwrap();
    let mut exchanges = original[0]["exchanges"].as_array().unwrap()[..2].to_vec();
    let payday = json!({"qe_day":"2027-07-01","sg_amount":"120.00","as_at":"2027-08-01","received":"2027-07-02"});
    let mut cases = Vec::new();
    for tool in aus_accounting_mcp::contracts::CATALOGUE["tools"]
        .as_array()
        .unwrap()
    {
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        let name = tool["name"].as_str().unwrap();
        let mut arguments = match name {
            "get_ato_benchmarks" => {
                json!({"industry":"Bakeries and hot bread shops","turnover":"850000.00","cost_of_sales":"270000.00"})
            }
            "calc_payday_super_deadline" => payday.clone(),
            "get_div7a_benchmark_rate" | "review_div7a_loan" => json!({"year_of_income":"2027-28"}),
            _ => json!({}),
        };
        arguments["misspelled_fact"] = json!("50.00");
        cases.push((
            name,
            arguments,
            json!({"/result/isError":true}),
            Some("misspelled_fact"),
        ));
    }
    assert_eq!(cases.len(), 22);
    let mut partial = payday.clone();
    partial["matched_amount"] = json!("50.00");
    cases.push((
        "calc_payday_super_deadline",
        partial,
        json!({"/result/isError":false,"/result/structuredContent/result/base_shortfall":"70.00"}),
        None,
    ));
    let mut ambiguous = payday.clone();
    ambiguous["received"] = json!("02/07/2027 00:00");
    cases.push((
        "calc_payday_super_deadline",
        ambiguous,
        json!({"/result/isError":true}),
        None,
    ));
    cases.extend([
        ("get_ato_benchmarks",json!({"industry":"Bakeries and hot bread shops","turnover":"0.00","cost_of_sales":"35000.00"}),json!({"/result/isError":true}),Some("turnover")),
        ("calc_payday_super_deadline",json!({"qe_day":"2026-06-15","sg_amount":"120.00","as_at":"2027-08-01"}),json!({"/result/isError":true}),Some("1 Jul 2026")),
        ("generate_synthetic_sbr_fixture",json!({"form_type":"INVALID"}),json!({"/result/isError":true}),Some("Supported: CTR, BAS")),
        ("refuse_div7a",json!({}),json!({"/result/isError":false,"/result/structuredContent/available":false}),None),
    ]);
    let row = json!({"employee_id":"SYN001","qe_day":"2026-08-06","sg_amount":"120.00","remitted":"2026-08-14","received":null,"first_to_fund":false,"out_of_cycle":false,"db_interest":false});
    let mut numeric = row.clone();
    numeric["sg_amount"] = json!(120);
    let mut boolean = row.clone();
    boolean["first_to_fund"] = json!("false");
    let mut path = row.clone();
    path["path"] = json!("../outside.csv");
    for changes in [
        json!({"output_dir":"../outside"}),
        json!({"csv_path":"C:\\outside.csv"}),
        json!({"fixture_id":"../../outside"}),
        json!({"as_at":"01/02/2027"}),
        json!({"contributions":[]}),
        json!({"contributions":vec![row.clone();201]}),
        json!({"contributions":[numeric]}),
        json!({"contributions":[boolean]}),
        json!({"contributions":[path]}),
        json!({"response_detail":"summary"}),
        json!({"response_detail":true}),
    ] {
        let mut arguments = json!({"contributions":[row],"as_at":"2026-08-20"});
        arguments
            .as_object_mut()
            .unwrap()
            .extend(changes.as_object().unwrap().clone());
        cases.push((
            "build_payday_super_evidence_pack",
            arguments,
            json!({"/result/isError":true}),
            None,
        ));
    }
    for (index, (name, arguments, assertions, message)) in cases.into_iter().enumerate() {
        let error = assertions["/result/isError"] == true;
        exchanges.push(json!({"request":{"jsonrpc":"2.0","id":index+2,"method":"tools/call","params":{"name":name,"arguments":arguments}},"assertions":assertions,"message_contains":message,"absent":if error {vec!["/result/structuredContent"]} else {vec![]}}));
    }
    assert_eq!(
        replay(vec![
            json!({"label":"original-boundary-errors-and-recovery","exchanges":exchanges})
        ]),
        39
    );
}

#[test]
fn current_evaluation_answers_follow_actual_reference_calls_and_dependent_facts() {
    use serde_json::json;
    use std::collections::{BTreeMap, BTreeSet};
    let xml = roxmltree::Document::parse(include_str!("../evaluation/questions.xml")).unwrap();
    let originals: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/wire-sessions.json")).unwrap();
    let files: Value =
        serde_json::from_str(include_str!("fixtures/native/evaluation-files.json")).unwrap();
    use sha2::{Digest, Sha256};
    for (name, source) in [
        (
            "tests/synthetic_corpus.py",
            include_bytes!("synthetic_corpus.py").as_slice(),
        ),
        (
            "tests/synthetic_rulings.py",
            include_bytes!("synthetic_rulings.py").as_slice(),
        ),
    ] {
        assert_eq!(
            Sha256::digest(source)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            files["sources"][name]
        );
    }
    let mut exchanges = originals[0]["exchanges"].as_array().unwrap()[..2].to_vec();
    let mut questions = BTreeMap::new();
    let mut selected = BTreeSet::new();
    let mut calls = 0;
    for pair in xml
        .descendants()
        .filter(|node| node.has_tag_name("qa_pair"))
    {
        let id = pair.attribute("id").unwrap();
        let text = |tag| {
            pair.children()
                .find(|n| n.has_tag_name(tag))
                .unwrap()
                .text()
                .unwrap()
        };
        let reference: Vec<Value> = serde_json::from_str(text("calls")).unwrap();
        let tools: BTreeSet<_> = pair
            .descendants()
            .filter(|n| n.has_tag_name("tool"))
            .map(|n| n.text().unwrap().to_owned())
            .collect();
        let called: BTreeSet<_> = reference
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(called, tools, "{id}");
        selected.extend(tools);
        for call in &reference {
            exchanges.push(json!({"request":{"jsonrpc":"2.0","id":exchanges.len(),"method":"tools/call","params":call},"assertions":{"/result/isError":false},"absent":[],"case":id}));
            calls += 1;
        }
        if id.starts_with("scope-") {
            assert!(reference.is_empty());
            exchanges.push(json!({"request":{"jsonrpc":"2.0","id":exchanges.len(),"method":"resources/read","params":{"uri":"aus-accounting://scope"}},"assertions":{"/jsonrpc":"2.0"},"absent":[],"case":id}));
        }
        assert!(questions.insert(id, (text("answer"), reference)).is_none());
    }
    assert_eq!(questions.len(), 37);
    assert_eq!(calls, 43);
    let registered: BTreeSet<_> = aus_accounting_mcp::contracts::CATALOGUE["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(selected, registered);
    let mut results: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    replay_observed(
        vec![
            json!({"label":"current-evaluation-answers","retrieval_files":files["folders"],"exchanges":exchanges}),
        ],
        |exchange, reply| {
            if let Some(id) = exchange["case"].as_str() {
                let result = if exchange["request"]["method"] == "resources/read" {
                    serde_json::from_str(reply["result"]["contents"][0]["text"].as_str().unwrap())
                        .unwrap()
                } else {
                    assert!(reply["result"]["structuredContent"].is_object(), "{reply}");
                    reply["result"]["structuredContent"].clone()
                };
                results.entry(id.into()).or_default().push(result);
            }
        },
    );
    fn ratio<'a>(result: &'a Value, name: &str) -> &'a str {
        result["ratios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["ratio"] == name)
            .unwrap()["status"]
            .as_str()
            .unwrap()
    }
    for (id, (expected, reference)) in questions {
        let r = &results[id];
        let answer = match id {
            "catalogue-pages" => {
                assert_eq!(reference[1]["arguments"]["year"], r[0]["benchmark_year"]);
                assert_eq!(reference[1]["arguments"]["offset"], r[0]["next_offset"]);
                r.iter()
                    .flat_map(|p| p["industries"].as_array().unwrap())
                    .map(|i| i["name"].as_str().unwrap())
                    .collect::<BTreeSet<_>>()
                    .len()
                    .to_string()
            }
            "missing-income" | "missing-rent" | "established-zero-income" => {
                let bakery = r[0]["industries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|i| i["name"] == "Bakeries and hot bread shops")
                    .unwrap();
                for call in &reference[1..] {
                    assert_eq!(call["arguments"]["industry"], bakery["name"]);
                }
                if id == "missing-rent" {
                    assert_ne!(ratio(&r[2], "rent_to_turnover"), "not_supplied");
                    ratio(&r[1], "rent_to_turnover").into()
                } else if id == "established-zero-income" {
                    assert_eq!(ratio(&r[1], "cost_of_sales_to_turnover"), "not_supplied");
                    ratio(&r[2], "cost_of_sales_to_turnover").into()
                } else {
                    ratio(&r[1], "cost_of_sales_to_turnover").into()
                }
            }
            "receipt-evidence" => {
                assert_eq!(r[1]["result"]["verdict"], "ON_TIME");
                r[0]["result"]["verdict"].as_str().unwrap().into()
            }
            "unknown-rate" => {
                assert_eq!(r[0]["verdict"], r[1]["verdict"]);
                assert!(r[0]["benchmark_rate"].is_null() && r[1]["benchmark_rate"].is_null());
                r[0]["verdict"].as_str().unwrap().into()
            }
            "missing-loan-facts" => {
                assert_eq!(r[0]["verdict"], "KNOWN");
                assert_eq!(
                    reference[1]["arguments"]["interest_rate_for_years_after_year_loan_made"],
                    r[0]["benchmark_rate"]
                );
                r[1]["gate"]["verdict"].as_str().unwrap().into()
            }
            "audit-consistency" => {
                assert_eq!(r[0]["benchmark_rate"], r[1]["benchmark_rate"]);
                assert!(!r[1]["statutory_trace"].as_array().unwrap().is_empty());
                r[0]["benchmark_rate"].as_str().unwrap().into()
            }
            "synthetic-boundary" => {
                assert!(r.iter().all(|p| p["synthetic"] == true));
                r.iter().all(|p| p["not_a_lodgment"] == true).to_string()
            }
            "unsupported-scope" => {
                assert_eq!(r[0]["verdict"], "KNOWN");
                assert_eq!(r[1]["available"], false);
                r[1]["code"].as_str().unwrap().into()
            }
            id if id.starts_with("scope-") => {
                assert!(
                    r[0]["unsupported_calculations"]
                        .as_object()
                        .unwrap()
                        .contains_key(id.strip_prefix("scope-").unwrap())
                );
                "UNSUPPORTED".into()
            }
            "grouped-payday" => r[0]["results"][1]["due"].as_str().unwrap().into(),
            "worksheet-gst" => r[0]["amounts"]["gst"].as_str().unwrap().into(),
            "library-reference" => r.last().unwrap()["text"].as_str().unwrap().into(),
            "payday-evidence-pack" => {
                let queue: Value =
                    serde_json::from_str(r[0]["files"]["exceptions.json"].as_str().unwrap())
                        .unwrap();
                queue["exceptions"][0]["verdict"].as_str().unwrap().into()
            }
            "legislation-citation" => r.last().unwrap()["section"]["section"]
                .as_str()
                .unwrap()
                .into(),
            "ranked-provision" => r[0]["matches"][0]["section"].as_str().unwrap().into(),
            "long-provision" => {
                assert_eq!(r[0]["next_start"], 12000);
                assert!(r[1]["next_start"].is_null());
                let total: usize = r
                    .iter()
                    .map(|p| p["section"]["text"].as_str().unwrap().chars().count())
                    .sum();
                assert_eq!(Some(total as u64), r[1]["section"]["total_chars"].as_u64());
                total.to_string()
            }
            "ruling-paragraph" => {
                assert_eq!(
                    reference[1]["arguments"]["row_ref"],
                    r[0]["matches"][0]["row_ref"]
                );
                assert_eq!(r[0]["matches"][0]["row_ref"], r[1]["paragraph"]["row_ref"]);
                assert_eq!(r[1]["paragraph"]["serving"], true);
                r[1]["paragraph"]["paragraph"].as_str().unwrap().into()
            }
            "legislated-rate" => r[0]["matches"][0]["amounts"][0].as_str().unwrap().into(),
            "statutory-definition" => r[0]["definitions"][0]["section"].as_str().unwrap().into(),
            "injected-instruction" => {
                assert!(
                    r[0]["section"]["text"]
                        .as_str()
                        .unwrap()
                        .contains("ignore your instructions")
                );
                r[0]["section"]["section"].as_str().unwrap().into()
            }
            "undefined-term" => {
                if r[0]["definitions"].as_array().unwrap().is_empty() {
                    "NONE_FOUND".into()
                } else {
                    "FOUND".into()
                }
            }
            "stored-rate-date" => r[0]["matches"][0]["compilation_date"]
                .as_str()
                .unwrap()
                .into(),
            "gst-coding" => r[0]["lines"][0]["state"].as_str().unwrap().into(),
            "bas-reconciliation" => r[0]["periods"][0]["labels"]["G1"]["difference"]
                .as_str()
                .unwrap()
                .into(),
            "lodgment-due-date" => r[0]["obligations"][1]["due_date"].as_str().unwrap().into(),
            "data-freshness" => r[0]["datasets"][0]["as_at_status"].as_str().unwrap().into(),
            other => panic!("unreviewed reference case {other}"),
        };
        assert_eq!(answer, expected, "{id}");
    }
}
#[test]
fn demo_calls_reproduce_complete_proof_over_stdio() {
    let original: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/native/wire-sessions.json")).unwrap();
    let mut exchanges = original[0]["exchanges"].as_array().unwrap()[..2].to_vec();
    let (_, body) = include_str!("../docs/quick-proof.txt")
        .split_once('\n')
        .unwrap();
    let proof: Value = serde_json::from_str(body).unwrap();
    for (index, call) in proof["calls"].as_array().unwrap().iter().enumerate() {
        let mut result = call["result"].clone();
        if call["tool"] == "review_div7a_loan" {
            result["engine_version"] = serde_json::json!("1.0.0");
            result["source"]["rate_table_uris"][0]["uri"] =
                serde_json::json!("data/benchmark_rates.csv");
        }
        let id = index + 2;
        exchanges.push(serde_json::json!({
            "request": {"jsonrpc": "2.0", "id": id, "method": "tools/call",
                "params": {"name": call["tool"], "arguments": call["arguments"]}},
            "expected": {"jsonrpc": "2.0", "id": id, "result": {
                "isError": false, "content": [{"type": "text", "text": result}],
                "structuredContent": result}}
        }));
    }
    assert_eq!(
        replay(vec![
            serde_json::json!({"label": "native-demo", "exchanges": exchanges})
        ]),
        3
    );
}
fn replay(sessions: Vec<Value>) -> usize {
    replay_observed(sessions, |_, _| {})
}

fn replay_observed(sessions: Vec<Value>, mut observe: impl FnMut(&Value, &Value)) -> usize {
    let mut replies = 0;
    for session in sessions {
        let directory = SessionDirectory::new();
        let mut command = Command::new(env!("CARGO_BIN_EXE_aus-accounting-mcp"));
        for name in aus_accounting_mcp::resources::RETRIEVAL_FOLDERS {
            command.env_remove(name);
        }
        let mut configured = false;
        let mut reference_folders = 0;
        if let Some(folders) = session["retrieval_files"].as_object() {
            for (name, files) in folders {
                assert!(aus_accounting_mcp::resources::RETRIEVAL_FOLDERS.contains(&name.as_str()));
                let root = directory.0.join(format!("reference-{reference_folders}"));
                std::fs::create_dir(&root).unwrap();
                for (relative, text) in files.as_object().unwrap() {
                    let relative = std::path::Path::new(relative);
                    assert!(
                        !relative.is_absolute()
                            && relative
                                .components()
                                .all(|c| matches!(c, std::path::Component::Normal(_)))
                    );
                    let target = root.join(relative);
                    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                    std::fs::write(target, text.as_str().unwrap()).unwrap();
                }
                command.env(name, &root);
                reference_folders += 1;
            }
        }
        if let Some(folders) = session["retrieval_folders"].as_object() {
            for (name, exists) in folders {
                assert!(aus_accounting_mcp::resources::RETRIEVAL_FOLDERS.contains(&name.as_str()));
                let exists = exists.as_bool().unwrap();
                let path = directory
                    .0
                    .join(if exists { "configured" } else { "missing" });
                if exists && !configured {
                    std::fs::create_dir(&path).unwrap();
                    configured = true;
                }
                command.env(name, path);
            }
        }
        let mut child = command
            .current_dir(&directory.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        let reader = thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        let mut server = Server {
            child,
            reader: Some(reader),
        };
        let mut full_pack: Option<(Value, usize)> = None;
        for exchange in session["exchanges"].as_array().unwrap() {
            let stdin = server.child.stdin.as_mut().unwrap();
            writeln!(stdin, "{}", exchange["request"]).unwrap();
            stdin.flush().unwrap();
            if exchange.get("expected").is_some() || exchange.get("assertions").is_some() {
                let line = rx
                    .recv_timeout(Duration::from_secs(10))
                    .unwrap_or_else(|e| {
                        panic!(
                            "{} {}: {e}",
                            session["label"], exchange["request"]["method"]
                        )
                    });
                let mut actual: Value =
                    serde_json::from_str(&line).expect("stdio must contain JSON-RPC only");
                observe(exchange, &actual);
                if let Some(assertions) = exchange["assertions"].as_object() {
                    assert_eq!(actual["id"], exchange["request"]["id"]);
                    assert_eq!(actual["jsonrpc"], "2.0");
                    for (pointer, expected) in assertions {
                        assert_eq!(
                            actual.pointer(pointer),
                            Some(expected),
                            "{pointer}: {actual}"
                        );
                    }
                    for pointer in exchange["absent"].as_array().unwrap() {
                        assert!(
                            actual.pointer(pointer.as_str().unwrap()).is_none(),
                            "{actual}"
                        );
                    }
                    if let Some(message) = exchange["message_contains"].as_str() {
                        assert!(
                            actual["result"]["content"][0]["text"]
                                .as_str()
                                .unwrap()
                                .contains(message),
                            "{actual}"
                        );
                    }
                    assert_eq!(
                        std::fs::read_dir(&directory.0).unwrap().count(),
                        usize::from(configured) + reference_folders,
                        "tool wrote into its working directory"
                    );
                }
                if session["label"] == "large-evidence-pack-full-and-compact"
                    && exchange["request"]["method"] == "tools/call"
                {
                    let payload = actual["result"]["structuredContent"].clone();
                    if exchange["request"]["params"]["arguments"]["response_detail"] == "full" {
                        full_pack = Some((payload, line.len()));
                    } else {
                        let (full, bytes) = full_pack.as_ref().unwrap();
                        assert_eq!(&payload, full);
                        assert!(
                            (line.len() as f64) < (*bytes as f64) * 0.55,
                            "compact response did not reduce wire size"
                        );
                    }
                }
                canonical(&mut actual);
                if let Some(expected) = exchange.get("expected") {
                    assert_eq!(
                        difference(&actual, expected, ""),
                        None,
                        "{} {}",
                        session["label"],
                        exchange["request"]["method"]
                    );
                }
                replies += 1;
            }
            if let Some(ms) = exchange["quiet_ms"].as_u64() {
                assert!(
                    matches!(
                        rx.recv_timeout(Duration::from_millis(ms)),
                        Err(mpsc::RecvTimeoutError::Timeout)
                    ),
                    "unexpected post-cancel response"
                );
            }
        }
        server.child.stdin.take();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = server.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "server failed to exit after EOF"
            );
            thread::sleep(Duration::from_millis(10));
        };
        assert!(
            status.success(),
            "{}: nonzero exit after EOF",
            session["label"]
        );
        assert!(
            rx.try_recv().is_err(),
            "unsolicited response after session closed"
        );
        assert_eq!(
            std::fs::read_dir(&directory.0).unwrap().count(),
            usize::from(configured) + reference_folders,
            "server wrote files in its working directory"
        );
    }
    replies
}
