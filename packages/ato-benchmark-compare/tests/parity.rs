//! Replays every case the differential harness froze: the calls and command
//! runs the Python 0.2.2 tests made, plus generated ones, with the files each
//! names. Results are compared as text, key order included, and every file a
//! run wrote is checked against the fingerprint Python's run left.

use ato_benchmark_compare::api;
use serde_json::{Map, Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The corpus with its shared strings put back in place.
fn corpus() -> Vec<Value> {
    let raw: Value =
        serde_json::from_str(include_str!("data/python-cases.json")).expect("corpus parses");
    let mut strings: Vec<String> = Vec::new();
    for entry in raw["strings"].as_array().unwrap() {
        let middle = entry["middle"].as_str().unwrap();
        let text = match entry["base"].as_u64() {
            None => middle.to_string(),
            Some(base) => {
                let base: Vec<char> = strings[base as usize].chars().collect();
                let prefix = entry["prefix"].as_u64().unwrap() as usize;
                let suffix = entry["suffix"].as_u64().unwrap() as usize;
                let head: String = base[..prefix].iter().collect();
                let tail: String = base[base.len() - suffix..].iter().collect();
                format!("{head}{middle}{tail}")
            }
        };
        strings.push(text);
    }
    fn resolve(value: &Value, strings: &[String]) -> Value {
        match value {
            Value::Object(o) if o.len() == 1 && o.contains_key("$string") => {
                json!(strings[o["$string"].as_u64().unwrap() as usize])
            }
            Value::Object(o) => Value::Object(
                o.iter()
                    .map(|(k, v)| (k.clone(), resolve(v, strings)))
                    .collect::<Map<_, _>>(),
            ),
            Value::Array(items) => {
                Value::Array(items.iter().map(|v| resolve(v, strings)).collect())
            }
            other => other.clone(),
        }
    }
    raw["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| resolve(c, &strings))
        .collect()
}

fn fnv(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h = (h ^ u64::from(*b)).wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

fn decode_b64(text: &str) -> Vec<u8> {
    let value = |c: u8| -> u32 {
        match c {
            b'A'..=b'Z' => u32::from(c - b'A'),
            b'a'..=b'z' => u32::from(c - b'a') + 26,
            b'0'..=b'9' => u32::from(c - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("base64 digit {c}"),
        }
    };
    let digits: Vec<u8> = text.bytes().filter(|b| *b != b'=').collect();
    let mut out = Vec::with_capacity(digits.len() * 3 / 4);
    for chunk in digits.chunks(4) {
        let mut acc = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            acc |= value(*c) << (18 - 6 * i);
        }
        out.extend_from_slice(&acc.to_be_bytes()[1..chunk.len()]);
    }
    out
}

/// A frozen file: base64, or run-length segments.
fn file_bytes(value: &Value) -> Vec<u8> {
    match value {
        Value::String(text) => decode_b64(text),
        Value::Object(o) => {
            let mut out = Vec::new();
            for segment in o["segments"].as_array().unwrap() {
                match segment[0].as_str().unwrap() {
                    "bytes" => out.extend(decode_b64(segment[1].as_str().unwrap())),
                    _ => out.extend(std::iter::repeat_n(
                        segment[1].as_u64().unwrap() as u8,
                        segment[2].as_u64().unwrap() as usize,
                    )),
                }
            }
            out
        }
        _ => panic!("file encoding"),
    }
}

/// Windows captured the corpus; a path Python printed with backslashes
/// prints with slashes on Unix. API results are compared as JSON text, where
/// the backslash is itself escaped.
fn native(text: &str) -> String {
    if cfg!(windows) {
        return text.to_string();
    }
    let mut text = text.to_string();
    for dir in ["data", "sub"] {
        text = text
            .replace(&format!("{dir}\\\\"), &format!("{dir}/"))
            .replace(&format!("{dir}\\"), &format!("{dir}/"));
    }
    text
}

fn matches_text(actual: &str, expected: &Value) -> bool {
    match expected {
        Value::String(text) => actual == native(text),
        Value::Object(o) => {
            o.get("fnv").and_then(Value::as_str) == Some(fnv(actual.as_bytes()).as_str())
                && o.get("len").and_then(Value::as_u64) == Some(actual.len() as u64)
        }
        _ => false,
    }
}

static SCRATCH: AtomicUsize = AtomicUsize::new(0);

fn scratch(case: &Value) -> PathBuf {
    let n = SCRATCH.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("ato-parity-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (name, data) in case["files"].as_object().unwrap() {
        let path = dir.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, file_bytes(data)).unwrap();
    }
    dir
}

/// Every file under `dir`, as `/`-separated relative names.
fn files_under(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(folder) = stack.pop() {
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path
                    .strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push(rel);
            }
        }
    }
    out.sort();
    out
}

/// The folder after a run: the files it wrote or changed, as the corpus
/// fingerprints them, the inputs it left alone, and nothing else.
fn written_ok(dir: &Path, case: &Value) -> Result<(), String> {
    let mut expected: Vec<String> = case["written"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    for name in case["unchanged"].as_array().unwrap() {
        let name = name.as_str().unwrap();
        let data = std::fs::read(dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
        if data != file_bytes(&case["files"][name]) {
            return Err(format!("{name} was changed"));
        }
        expected.push(name.to_string());
    }
    expected.sort();
    let found = files_under(dir);
    if found != expected {
        return Err(format!(
            "files after the run: {found:?}, expected {expected:?}"
        ));
    }
    for (name, want) in case["written"].as_object().unwrap() {
        let data = std::fs::read(dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
        let ok = want["fnv"].as_str() == Some(fnv(&data).as_str())
            && want["len"].as_u64() == Some(data.len() as u64);
        if !ok {
            return Err(format!(
                "{name} differs: {}",
                String::from_utf8_lossy(&data)
            ));
        }
    }
    Ok(())
}

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ato-benchmark-compare"))
}

fn lf(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .expect("UTF-8 output")
        .replace("\r\n", "\n")
}

fn check(case: &Value) -> Option<String> {
    let expected = &case["expected"];
    if case["kind"] == "api" {
        let command = case["command"].as_str().unwrap();
        // A case that names a path runs in its own folder, never this crate's.
        let input = &case["input"];
        let names_a_path = input["data_dir"].is_string()
            || input["dataset"]["data_dir"].is_string()
            || input["path"].is_string();
        let no_files = case["files"].as_object().unwrap().is_empty()
            && case["written"].as_object().unwrap().is_empty()
            && !names_a_path;
        let (actual, written) = if no_files {
            let result = api::evaluate(command, &case["input"]);
            (
                match result {
                    Ok(v) => json!({ "ok": v }),
                    Err(m) => json!({ "error": m }),
                },
                Ok(()),
            )
        } else {
            let dir = scratch(case);
            let mut child = binary()
                .arg(command)
                .current_dir(&dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            child
                .stdin
                .take()
                .unwrap()
                .write_all(case["input"].to_string().as_bytes())
                .unwrap();
            let out = child.wait_with_output().unwrap();
            let written = written_ok(&dir, case);
            let _ = std::fs::remove_dir_all(&dir);
            let value = if out.status.code() == Some(0) {
                json!({"ok": serde_json::from_slice::<Value>(&out.stdout).unwrap()})
            } else {
                assert_eq!(out.status.code(), Some(2), "an API refusal exits 2");
                let err = lf(&out.stderr);
                json!({"error": err.strip_prefix("error: ").unwrap().strip_suffix('\n').unwrap()})
            };
            (value, written)
        };
        let text = native(&actual.to_string());
        let want = native(&expected.to_string());
        let ok = match expected.get("fnv") {
            Some(_) => matches_text(&actual.to_string(), expected),
            None => text == want,
        };
        return match (ok, written) {
            (true, Ok(())) => None,
            (_, written) => Some(format!(
                "{} {command} {}\n  expected {want}\n  actual   {text}\n  files {written:?}",
                case["test"],
                case["input"]
                    .to_string()
                    .chars()
                    .take(300)
                    .collect::<String>()
            )),
        };
    }
    let dir = scratch(case);
    let mut command = binary();
    command
        .args(
            case["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_str().unwrap()),
        )
        .current_dir(&dir)
        .env_remove("FORCE_COLOR")
        .env_remove("NO_COLOR")
        .env("PYTHON_COLORS", "0")
        .env("COLUMNS", "80");
    let env = case["env"].as_object().unwrap();
    if env.contains_key("FORCE_COLOR") && !env.contains_key("PYTHON_COLORS") {
        command.env_remove("PYTHON_COLORS");
    }
    for (key, value) in env {
        command.env(key, value.as_str().unwrap());
    }
    let out = command.output().unwrap();
    let written = written_ok(&dir, case);
    let _ = std::fs::remove_dir_all(&dir);
    let (stdout, stderr) = (lf(&out.stdout), lf(&out.stderr));
    let ok = out.status.code() == expected["code"].as_i64().map(|c| c as i32)
        && matches_text(&stdout, &expected["stdout"])
        && matches_text(&stderr, &expected["stderr"])
        && written.is_ok();
    (!ok).then(|| {
        format!(
            "{} {:?}\n  expected {expected}\n  actual   {:?} {stdout:?} {stderr:?}\n  files {written:?}",
            case["test"],
            case["args"],
            out.status.code()
        )
    })
}

fn replay(filter: impl Fn(&Value) -> bool) -> usize {
    let cases: Vec<Value> = corpus().into_iter().filter(|c| filter(c)).collect();
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
    let failures: Vec<String> = std::thread::scope(|s| {
        let handles: Vec<_> = cases
            .chunks(cases.len().div_ceil(workers).max(1))
            .map(|chunk| s.spawn(move || chunk.iter().filter_map(check).collect::<Vec<_>>()))
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    assert!(
        failures.is_empty(),
        "{} of {} differ:\n{}",
        failures.len(),
        cases.len(),
        failures[..failures.len().min(5)].join("\n")
    );
    cases.len()
}

#[test]
fn python_api_calls_match() {
    let n = replay(|c| c["kind"] == "api");
    assert!(n > 2000, "{n}");
}

#[test]
fn python_command_runs_match() {
    let n = replay(|c| c["kind"] == "cli");
    assert!(n > 800, "{n}");
}

#[test]
fn every_command_has_cases() {
    let cases = corpus();
    for command in api::COMMANDS {
        assert!(
            cases.iter().any(|c| c["command"] == command),
            "{command} has no case"
        );
    }
    for name in ["industries", "show", "buckets", "map", "compare"] {
        assert!(
            cases
                .iter()
                .any(|c| c["kind"] == "cli" && c["args"][0] == name),
            "{name} has no run"
        );
    }
}
