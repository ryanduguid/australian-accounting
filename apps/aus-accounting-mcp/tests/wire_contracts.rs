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
    let mut replies = 0;
    for session in sessions {
        let directory = SessionDirectory::new();
        let mut command = Command::new(env!("CARGO_BIN_EXE_aus-accounting-mcp"));
        for name in aus_accounting_mcp::resources::RETRIEVAL_FOLDERS {
            command.env_remove(name);
        }
        let mut configured = false;
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
            if let Some(expected) = exchange.get("expected") {
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
                assert_eq!(
                    difference(&actual, expected, ""),
                    None,
                    "{} {}",
                    session["label"],
                    exchange["request"]["method"]
                );
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
            usize::from(configured),
            "server wrote files in its working directory"
        );
    }
    replies
}
