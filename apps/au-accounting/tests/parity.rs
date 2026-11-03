//! Holds `au-accounting ENGINE` to each engine's own command, byte for byte.
//!
//! Every input in each engine's frozen corpora (tests/data/python-*.json), plus
//! edge-case JSON the corpora rarely carry, runs through the engine's standalone
//! binary and through `au-accounting`, each in its own scratch folder; exit
//! statuses, stdout, stderr and (for The WIP Tally's file runs) the files left
//! behind must match, including the line and column of JSON input errors.
//!
//! The comparisons need the native commands, so they are ignored by default:
//!
//!     AU_ACCOUNTING_PARITY_BIN=BIN_DIR cargo test --locked --test parity -- --ignored
//!
//! BIN_DIR holds au-accounting and the 8 engine commands.
//! AU_ACCOUNTING_PARITY_CAP=N samples at most N corpus rows per engine and kind
//! (default: every row).

use serde_json::{Map, Value, json};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const EXE: &str = std::env::consts::EXE_SUFFIX;
/// (subcommand, crate directory under packages/, standalone command)
const ENGINES: [(&str, &str, &str); 8] = [
    (
        "benchmarks",
        "ato-benchmark-compare",
        "ato-benchmark-compare",
    ),
    ("div7a", "div7a-loan-review", "div7a-loan-review"),
    ("exchequer", "the-exchequer-tally", "the-exchequer-tally"),
    ("gst-review", "gst-review", "gst-review"),
    ("payday-super", "payday-super-checker", "payday-super-check"),
    ("solomons", "solomons-sword", "solomons-sword"),
    (
        "tax",
        "australian-tax-calculators",
        "australian-tax-calculators",
    ),
    ("wip", "the-wip-tally", "wip-tally"),
];
const NUMBERS: [&str; 22] = [
    "0",
    "-0",
    "1",
    "-1",
    "1.0",
    "1e2",
    "1E2",
    "0.1",
    "2.5e-3",
    "-0.0",
    "18446744073709551615",
    "18446744073709551616",
    "-9223372036854775808",
    "-9223372036854775809",
    "123456789012345678901234567890",
    "1e308",
    "1e309",
    "-1e400",
    "4.9e-324",
    "1e-400",
    "0.30000000000000004",
    "51.248178375505404",
];

fn edges() -> Vec<String> {
    let mut edges: Vec<String> = [
        "{}",
        "[]",
        "null",
        "",
        "{",
        r#"{"b":1,"a":2}"#,
        r#"{"a":1,"a":2}"#,
    ]
    .map(String::from)
    .into();
    edges.extend(NUMBERS.map(String::from));
    edges.extend(NUMBERS.map(|n| format!(r#"{{"value":{n},"amount":{n},"kind":"money"}}"#)));
    edges.extend(
        NUMBERS[..6]
            .iter()
            .map(|n| format!(r#"{{"z":{{"y":{n},"x":[{n}]}},"a":{n}}}"#)),
    );
    edges.extend(
        [
            r#"{"$serde_json::private::Number":"2026"}"#,
            r#"{"financial_year":{"$serde_json::private::Number":"2026"}}"#,
            r#"{"financial_year":{"$serde_json::private::Number":2026}}"#,
            r#"{"financial_year":{"$serde_json::private::Number":"bad","extra":true}}"#,
            r#"{"financial_year":2026,"ignored":{"$serde_json::private::Number":"bad"}}"#,
            r#"{"a":1e400,"a":2}"#,
            r#"{"12[{":"\\\"[123{","a":[null,true,false,-1.2e+30,{"n":2}]}"#,
        ]
        .map(String::from),
    );
    for depth in [127, 128] {
        edges.push(format!("{}0{}", "[".repeat(depth), "]".repeat(depth)));
    }
    edges
}

fn read_json(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn array(value: Value) -> Vec<Value> {
    match value {
        Value::Array(rows) => rows,
        other => panic!("expected a list of corpus rows, got {other:.80}"),
    }
}

fn corpus(package: &str) -> Vec<Value> {
    let folder = Path::new(ROOT).join("packages").join(package);
    if package == "gst-review" {
        return array(read_json(
            &folder.join("tests/fixtures/python-contracts.json"),
        ));
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(folder.join("tests/data"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            name.starts_with("python-") && name.ends_with(".json")
        })
        .collect();
    paths.sort();
    let mut rows = Vec::new();
    for path in paths {
        rows.extend(match read_json(&path) {
            Value::Object(mut data) => array(data.remove("cases").unwrap_or(Value::Null)),
            data => array(data),
        });
    }
    rows
}

fn sample(rows: Vec<&Value>, cap: usize) -> Vec<&Value> {
    if cap == 0 || rows.len() <= cap {
        return rows;
    }
    (0..cap).map(|i| rows[i * rows.len() / cap]).collect()
}

/// `json.dumps(value)`: ASCII only, with ", " and ": " separators. Numbers keep
/// their text, which is Python's own in the corpora Python wrote.
fn py_dumps(value: &Value) -> String {
    let mut out = String::new();
    write_py(value, &mut out);
    out
}

fn write_py(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(text) => write_py_str(text, out),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_py(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            out.push('{');
            for (i, (key, item)) in map.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_py_str(key, out);
                out.push_str(": ");
                write_py(item, out);
            }
            out.push('}');
        }
    }
}

fn write_py_str(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            ' '..='~' => out.push(c),
            _ => {
                for unit in c.encode_utf16(&mut [0; 2]) {
                    write!(out, "\\u{unit:04x}").unwrap();
                }
            }
        }
    }
    out.push('"');
}

/// A row's environment changes: a null value removes the variable.
type Env = Vec<(String, Option<String>)>;

fn environment(row: &Value) -> Env {
    let Some(Value::Object(env)) = row.get("env") else {
        return Env::new();
    };
    env.iter()
        .map(|(key, value)| match value {
            Value::Null => (key.clone(), None),
            Value::String(text) => (key.clone(), Some(text.clone())),
            other => panic!("env {key} must be a string or null, not {other}"),
        })
        .collect()
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap().to_owned())
        .collect()
}

/// A scratch folder, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "au-accounting-parity-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir(&path).unwrap();
        Scratch(path)
    }

    /// The folder as named and as `os.path.realpath` resolves it.
    fn forms(&self) -> (String, String) {
        let real = std::fs::canonicalize(&self.0).unwrap();
        let real = real.to_string_lossy();
        // Windows canonical paths carry the verbatim prefix, which realpath drops.
        let real = real.strip_prefix(r"\\?\").unwrap_or(&real).to_owned();
        (self.0.to_string_lossy().into_owned(), real)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn replace(data: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        if data[i..].starts_with(from) {
            out.extend_from_slice(to);
            i += from.len();
        } else {
            out.push(data[i]);
            i += 1;
        }
    }
    out
}

/// The scratch folder, as named, resolved or JSON-escaped, becomes `<TMP>`.
fn scrub(data: &[u8], folder: &str, real: &str) -> Vec<u8> {
    let mut forms: Vec<String> = [folder, real]
        .iter()
        .flat_map(|form| [form.to_string(), form.replace('\\', "\\\\")])
        .collect();
    // macOS /var/... aliases /private/var/...; replacing the shorter path first
    // would leave /private<TMP> and create a false difference between commands.
    forms.sort_by_key(|form| std::cmp::Reverse(form.len()));
    let mut data = data.to_vec();
    for form in forms {
        data = replace(&data, form.as_bytes(), b"<TMP>");
    }
    data
}

type Outcome = (ExitStatus, Vec<u8>, Vec<u8>);

fn run(argv: &[PathBuf], args: &[String], stdin: &[u8], folder: &Scratch, env: &Env) -> Outcome {
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .args(args)
        .current_dir(&folder.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in env {
        match value {
            Some(value) => command.env(key, value),
            None => command.env_remove(key),
        };
    }
    let mut child = command
        .spawn()
        .unwrap_or_else(|e| panic!("{}: {e}", argv[0].display()));
    let mut pipe = child.stdin.take().unwrap();
    let output = std::thread::scope(|scope| {
        // A command can exit before it reads all of its input.
        scope.spawn(move || pipe.write_all(stdin));
        child.wait_with_output().unwrap()
    });
    let (named, real) = folder.forms();
    (
        output.status,
        scrub(&output.stdout, &named, &real),
        scrub(&output.stderr, &named, &real),
    )
}

fn show(outcome: &Outcome, limit: usize) -> String {
    let text = format!(
        "{}, stdout {:?}, stderr {:?}",
        outcome.0,
        String::from_utf8_lossy(&outcome.1),
        String::from_utf8_lossy(&outcome.2)
    );
    text.chars().take(limit).collect()
}

type Files = Vec<(String, Vec<u8>)>;

fn files_under(root: &Path, folder: &Path, files: &mut Files) {
    for entry in std::fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files_under(root, &path, files);
        } else if path.is_file() {
            let relative = path.strip_prefix(root).unwrap();
            let name = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            files.push((name, std::fs::read(&path).unwrap()));
        }
    }
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

/// One of The WIP Tally's corpus runs: files written, then its steps in order.
fn file_run(argv: &[PathBuf], row: &Value) -> (Vec<Outcome>, Files) {
    let folder = Scratch::new();
    for (relative, data) in row["files"].as_object().unwrap() {
        let path = folder.0.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, unhex(data.as_str().unwrap())).unwrap();
    }
    let env = environment(row);
    let results = row["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|step| run(argv, &strings(step), b"", &folder, &env))
        .collect();
    let mut files = Files::new();
    files_under(&folder.0, &folder.0, &mut files);
    files.sort();
    (results, files)
}

struct Report {
    name: String,
    cases: usize,
    failures: Vec<String>,
}

struct Case {
    args: Vec<String>,
    stdin: Vec<u8>,
    env: Env,
    label: String,
}

fn commands(standalone: &[PathBuf]) -> Vec<String> {
    let help = Command::new(&standalone[0])
        .arg("--api-help")
        .output()
        .unwrap()
        .stdout;
    let help = String::from_utf8(help).unwrap();
    let first = help.lines().next().unwrap_or_default();
    let first = first.strip_prefix("JSON stdin commands: ").unwrap_or(first);
    first.split(", ").map(String::from).collect()
}

fn cases(rows: &[Value], standalone: &[PathBuf], cap: usize) -> Vec<Case> {
    let label = |row: &Value| row.get("test").unwrap_or(&Value::Null).to_string();
    let kind = |key: &str| {
        sample(
            rows.iter().filter(|row| row.get(key).is_some()).collect(),
            cap,
        )
    };
    let mut cases: Vec<Case> = kind("command")
        .into_iter()
        .map(|row| Case {
            args: vec![row["command"].as_str().unwrap().to_owned()],
            stdin: py_dumps(&row["input"]).into_bytes(),
            env: environment(row),
            label: label(row),
        })
        .collect();
    cases.extend(kind("args").into_iter().map(|row| Case {
        args: strings(&row["args"]),
        stdin: Vec::new(),
        env: environment(row),
        label: label(row),
    }));
    let edges = edges();
    for command in commands(standalone) {
        cases.extend(edges.iter().map(|text| Case {
            args: vec![command.clone()],
            stdin: text.clone().into_bytes(),
            env: Env::new(),
            label: format!("edge {text:?}"),
        }));
    }
    // Routing an engine's own main() adds outside its library must reach
    // au-accounting too; derive and verify are such commands on some branches.
    let routing: [(&[&str], &[u8]); 8] = [
        (&["--api-help"], b""),
        (&["--help"], b""),
        (&["--version"], b""),
        (&[], b""),
        (&["--nope"], b""),
        (&["derive"], b"{}"),
        (&["verify"], b"{}"),
        (&["verify", "missing.json"], b""),
    ];
    cases.extend(routing.iter().map(|(args, stdin)| Case {
        args: args.iter().map(|arg| arg.to_string()).collect(),
        stdin: stdin.to_vec(),
        env: Env::new(),
        label: "routing".into(),
    }));
    cases
}

fn argvs(bin: &Path, name: &str, command: &str) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let standalone = vec![bin.join(format!("{command}{EXE}"))];
    let umbrella = vec![bin.join(format!("au-accounting{EXE}")), PathBuf::from(name)];
    (standalone, umbrella)
}

fn check((name, package, command): (&str, &str, &str), bin: &Path, cap: usize) -> Report {
    let (standalone, umbrella) = argvs(bin, name, command);
    let rows = corpus(package);
    let cases = cases(&rows, &standalone, cap);
    let files = sample(
        rows.iter()
            .filter(|row| row.get("steps").is_some())
            .collect(),
        cap,
    );
    let mut failures = Vec::new();
    let (a, b) = (Scratch::new(), Scratch::new());
    for case in &cases {
        let left = run(&standalone, &case.args, &case.stdin, &a, &case.env);
        let right = run(&umbrella, &case.args, &case.stdin, &b, &case.env);
        if left != right {
            failures.push(format!(
                "{name} {:?} ({}):\n  own: {}\n  au-accounting: {}",
                case.args,
                case.label,
                show(&left, 300),
                show(&right, 300)
            ));
        }
    }
    for row in &files {
        if file_run(&standalone, row) != file_run(&umbrella, row) {
            failures.push(format!(
                "{name} file run {} ({})",
                row["steps"],
                row.get("test").unwrap_or(&Value::Null)
            ));
        }
    }
    Report {
        name: name.into(),
        cases: cases.len() + files.len(),
        failures,
    }
}

fn derivation_input(name: &str) -> Value {
    if name == "div7a" {
        json!({"facts": {"year_of_income": "2026-27", "year_loan_made": "2023-24",
            "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
            "remaining_term_years": "5", "payments_applied_during_the_year": "25556.00",
            "gate_result": {"verdict": "COMPLYING", "benchmark_year_used": "2023-24",
                "reasons": [], "caveats": []}}})
    } else {
        json!({"qe_day": "2026-07-09", "first_to_fund": true})
    }
}

/// Successful and rejected saved derivations use the standalone CLI contract.
fn derivation_check(name: &str, bin: &Path) -> Report {
    let command = ENGINES.iter().find(|engine| engine.0 == name).unwrap().2;
    let (standalone, umbrella) = argvs(bin, name, command);
    let input = derivation_input(name);
    let (a, b) = (Scratch::new(), Scratch::new());
    let raw = py_dumps(&input).into_bytes();
    let derived = run(&standalone, &["derive".into()], &raw, &a, &Env::new());
    if !derived.0.success() {
        return Report {
            name: format!("{name} derivations"),
            cases: 1,
            failures: vec![format!("standalone derive failed: {}", show(&derived, 500))],
        };
    }
    let saved: Value = serde_json::from_slice(&derived.1).unwrap();
    let mut edited_step = saved.clone();
    edited_step["steps"][0]["value"] = "edited".into();
    let mut edited_result = saved.clone();
    edited_result["result"] = Value::Null;
    let mut edited_identity = saved.clone();
    edited_identity["engine"] = "wrong".into();
    let mut extra_step = saved;
    extra_step["steps"]
        .as_array_mut()
        .unwrap()
        .push(Value::Null);
    let mut padded = raw.clone();
    padded.resize(raw.len() + (8 << 20) + 1, b' ');
    let mut cases: Vec<(Vec<&str>, Vec<u8>)> = vec![
        (vec!["derive"], raw.clone()),
        (vec!["verify"], derived.1.clone()),
        (vec!["verify", "-"], derived.1.clone()),
        (vec!["verify", "saved.json"], Vec::new()),
        (vec!["verify", "missing.json"], Vec::new()),
        (vec!["derive"], b"{".to_vec()),
        (vec!["verify"], b"{".to_vec()),
        (vec!["verify"], b"{}".to_vec()),
        (vec!["derive"], padded),
    ];
    for value in [edited_step, edited_result, edited_identity, extra_step] {
        cases.push((vec!["verify"], py_dumps(&value).into_bytes()));
    }
    if name == "div7a" {
        let facts = input["facts"].as_object().unwrap();
        let mut mapping: Map<String, Value> = facts
            .iter()
            .filter(|(key, _)| !["year_of_income", "gate_result"].contains(&key.as_str()))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        mapping.insert("loan_id".into(), json!(9_007_199_254_740_993_u64));
        let exact = py_dumps(&json!({"year_of_income": facts["year_of_income"],
            "gate_result": facts["gate_result"], "mapping": mapping}))
        .into_bytes();
        let native = run(&standalone, &["derive".into()], &exact, &a, &Env::new());
        assert!(native.0.success(), "{}", show(&native, 500));
        cases.push((vec!["derive"], exact));
        cases.push((vec!["verify"], native.1));
    }
    for folder in [&a, &b] {
        std::fs::write(folder.0.join("saved.json"), &derived.1).unwrap();
    }
    let mut failures = Vec::new();
    for (args, stdin) in &cases {
        let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        let left = run(&standalone, &args, stdin, &a, &Env::new());
        let right = run(&umbrella, &args, stdin, &b, &Env::new());
        if left != right {
            failures.push(format!(
                "{name} {args:?}: own={}, umbrella={}",
                show(&left, 500),
                show(&right, 500)
            ));
        }
    }
    let alias = [umbrella[0].clone(), PathBuf::from(command)];
    if derived != run(&alias, &["derive".into()], &raw, &b, &Env::new()) {
        failures.push(format!("{command} alias differs"));
    }
    Report {
        name: format!("{name} derivations"),
        cases: cases.len() + 1,
        failures,
    }
}

fn bin_dir() -> PathBuf {
    let bin = std::env::var_os("AU_ACCOUNTING_PARITY_BIN")
        .expect("AU_ACCOUNTING_PARITY_BIN names the folder of the native commands");
    std::path::absolute(bin).unwrap()
}

fn assert_reports(reports: &[Report]) {
    let mut summary = String::new();
    for report in reports {
        let differences = report.failures.len();
        writeln!(
            summary,
            "{}: {} cases, {differences} differences",
            report.name, report.cases
        )
        .unwrap();
        for failure in report.failures.iter().take(10) {
            writeln!(summary, "{failure}").unwrap();
        }
    }
    println!("{summary}");
    assert!(
        reports.iter().all(|report| report.failures.is_empty()),
        "{summary}"
    );
}

#[test]
#[ignore = "needs AU_ACCOUNTING_PARITY_BIN, a folder of au-accounting and the engine commands"]
fn au_accounting_matches_each_engine() {
    let bin = &bin_dir();
    let cap = std::env::var("AU_ACCOUNTING_PARITY_CAP").map_or(0, |cap| cap.parse().unwrap());
    let reports: Vec<Report> = std::thread::scope(|scope| {
        let running: Vec<_> = ENGINES
            .into_iter()
            .map(|engine| scope.spawn(move || check(engine, bin, cap)))
            .collect();
        running
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect()
    });
    assert_reports(&reports);
}

#[test]
#[ignore = "needs AU_ACCOUNTING_PARITY_BIN, a folder of au-accounting and the engine commands"]
fn au_accounting_matches_saved_derivations() {
    let bin = bin_dir();
    assert_reports(&["div7a", "payday-super"].map(|name| derivation_check(name, &bin)));
}

// Temporary path aliases must not hide real command-output differences.

#[test]
fn macos_alias_and_canonical_path_are_both_removed() {
    let folder = "/var/folders/test/scratch";
    let canonical = format!("/private{folder}");
    assert_eq!(
        scrub(
            format!("{folder}/report.csv {canonical}/report.csv").as_bytes(),
            folder,
            &canonical
        ),
        b"<TMP>/report.csv <TMP>/report.csv"
    );
    assert_eq!(
        scrub(
            format!("error: file not found: {canonical}/report.csv").as_bytes(),
            folder,
            &canonical
        ),
        b"error: file not found: <TMP>/report.csv"
    );
}

#[test]
fn windows_escaped_paths_and_other_errors_are_preserved() {
    let folder = r"C:\Temp\scratch";
    let escaped = folder.replace('\\', r"\\");
    assert_eq!(
        scrub(
            format!("{folder} {escaped} /private/unrelated wrong value").as_bytes(),
            folder,
            folder
        ),
        b"<TMP> <TMP> /private/unrelated wrong value"
    );
}

#[test]
#[should_panic(expected = "env TZ must be a string or null")]
fn a_corpus_environment_value_that_is_not_text_is_refused() {
    environment(&json!({"env": {"HOME": null, "TZ": 10}}));
}
