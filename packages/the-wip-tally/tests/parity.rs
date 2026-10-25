use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use the_wip_tally::csv_io::{self, Failure};
use the_wip_tally::decimal::{Calc, Decimal};
use the_wip_tally::engine::ContractInput;
use the_wip_tally::money::{self, AmountError};
use the_wip_tally::{api, cli, job_to_cash, py::PyPath, report};

fn failure<T>(r: Result<T, Failure>, ok: impl Fn(T) -> Value) -> Value {
    match r {
        Ok(v) => json!({ "ok": ok(v) }),
        Err(Failure::Message(m)) => json!({ "error": m }),
        Err(Failure::Decimal(s)) => json!({ "error": format!("decimal.{}", s.name()) }),
    }
}

/// A `ContractInput` as the harness encodes the dataclass.
fn contract_json(c: &ContractInput) -> Value {
    let d = |v: &Decimal| json!(v.to_string());
    let o = |v: &Option<Decimal>| v.as_ref().map_or(Value::Null, |v| json!(v.to_string()));
    json!({
        "contract_id": c.contract_id, "line_number": c.line_number.parse::<i64>().unwrap(), "customer": c.customer,
        "description": c.description, "original_contract_sum": d(&c.original_contract_sum),
        "approved_variations": d(&c.approved_variations),
        "unapproved_variations_estimate": d(&c.unapproved_variations_estimate),
        "constraint_include_ratio": d(&c.constraint_include_ratio),
        "costs_incurred": d(&c.costs_incurred),
        "inefficiency_rework_wastage": d(&c.inefficiency_rework_wastage),
        "uninstalled_materials": d(&c.uninstalled_materials),
        "estimated_cost_to_complete": d(&c.estimated_cost_to_complete),
        "certified_billings": d(&c.certified_billings),
        "uncertified_claims": d(&c.uncertified_claims),
        "retention_withheld": d(&c.retention_withheld),
        "retention_classification": c.retention_classification,
        "committed_outstanding": d(&c.committed_outstanding),
        "outcome_reasonably_measurable": c.outcome_reasonably_measurable,
        "recoverable_costs": o(&c.recoverable_costs), "progress_method": c.progress_method,
        "output_percent": o(&c.output_percent),
        "prior_transaction_price": o(&c.prior_transaction_price),
        "prior_estimated_cost_at_completion": o(&c.prior_estimated_cost_at_completion),
        "prior_costs_incurred": o(&c.prior_costs_incurred),
        "prior_estimated_cost_to_complete": o(&c.prior_estimated_cost_to_complete),
        "prior_revenue_to_date": o(&c.prior_revenue_to_date), "gst_rate": d(&c.gst_rate),
        "assets_used_carrying": o(&c.assets_used_carrying),
    })
}

/// FNV-1a 64 of LF-normalised bytes: how the corpus stores outputs over its size limits.
fn fnv(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h = (h ^ u64::from(*b)).wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn lf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

fn matches(actual: &[u8], expected: &Value) -> bool {
    match expected {
        Value::String(hex) => lf(actual) == lf(&unhex(hex)),
        digest => {
            let actual = lf(actual);
            digest["len"].as_u64() == Some(actual.len() as u64)
                && digest["fnv"] == json!(fnv(&actual))
        }
    }
}

fn amount<T>(r: Result<T, AmountError>, ok: impl Fn(T) -> Value) -> Value {
    match r {
        Ok(v) => json!({ "ok": ok(v) }),
        Err(AmountError::Message(m)) => json!({ "error": m }),
        Err(AmountError::Decimal(s)) => json!({ "error": format!("decimal.{}", s.name()) }),
    }
}

fn calc<T>(r: Calc<T>, ok: impl Fn(T) -> Value) -> Value {
    match r {
        Ok(v) => json!({ "ok": ok(v) }),
        Err(s) => json!({ "error": format!("decimal.{}", s.name()) }),
    }
}

fn dec(v: &Value) -> Decimal {
    Decimal::parse(v["value"].as_str().unwrap()).unwrap()
}

/// Run one frozen API or library case in-process.
fn evaluate(command: &str, input: &Value) -> Value {
    let raw = || input["raw"].as_str().unwrap();
    let where_ = || input["where"].as_str().unwrap();
    let text = |d: Decimal| json!(d.to_string());
    match command {
        "lib:parse_amount" => amount(money::parse_amount(raw(), where_()), text),
        "lib:parse_ratio" => amount(money::parse_ratio(raw(), where_()), text),
        "lib:parse_bool" => amount(money::parse_bool(raw(), where_()), |b| json!(b)),
        "lib:as_money" => calc(money::as_money(&dec(input)), text),
        "lib:as_points" => calc(money::as_points(&dec(input)), text),
        "lib:money" => calc(money::money(&dec(input)), |s| json!(s)),
        "lib:percent" => calc(money::percent(&dec(input)), |s| json!(s)),
        "lib:points" => calc(money::points(&dec(input)), |s| json!(s)),
        "lib:guard" => json!({ "ok": report::guard(input["value"].as_str().unwrap()) }),
        "lib:md_cell" => json!({ "ok": report::md_cell(input["value"].as_str().unwrap()) }),
        "lib:read_contracts" => {
            let mapping: csv_io::Mapping = csv_io::CANONICAL_FIELDS
                .iter()
                .map(|f| (*f, input["mapping"][*f].as_str().unwrap().to_string()))
                .collect();
            let bytes = unhex(input["data"].as_str().unwrap());
            failure(
                csv_io::read_contracts("contracts.csv", &bytes, &mapping),
                |contracts| json!(contracts.iter().map(contract_json).collect::<Vec<_>>()),
            )
        }
        "lib:load_mapping" => {
            let folder = scratch();
            let path = folder.join("mapping.json");
            std::fs::write(&path, unhex(input["data"].as_str().unwrap())).unwrap();
            let shown = PyPath::new(&path.to_string_lossy());
            let result = failure(csv_io::load_mapping(Some(&shown)), |mapping| {
                Value::Object(
                    mapping
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), json!(v)))
                        .collect(),
                )
            });
            // Python ran it as `Path("mapping.json")` from a temporary folder.
            let text = result
                .to_string()
                .replace(&shown.display().replace('\\', "\\\\"), "mapping.json");
            serde_json::from_str(&text).unwrap()
        }
        "lib:schedule_as_at" => {
            let folder = scratch();
            let path = folder.join("s.csv");
            if let Some(hex) = input["data"].as_str() {
                std::fs::write(&path, unhex(hex)).unwrap();
            }
            json!({ "ok": cli::schedule_as_at(&PyPath::new(&path.to_string_lossy())) })
        }
        _ => match api::evaluate(command, input) {
            Ok(v) => json!({ "ok": v }),
            Err(e) => json!({ "error": e }),
        },
    }
}

fn replay(prefixes: &[&str]) {
    let rows: Value = serde_json::from_str(include_str!("data/python-api.json")).unwrap();
    let mut n = 0;
    for (index, row) in rows.as_array().unwrap().iter().enumerate() {
        let command = row["command"].as_str().unwrap();
        if !prefixes.iter().any(|p| command.starts_with(p)) {
            continue;
        }
        let mut expected = row["expected"].clone();
        // This frozen case supplied integer-scale billings. The corrected
        // right-to-invoice result, like every money result, has cent scale.
        if command == "measure" && index == 4642 {
            assert_eq!(row["test"], "test_unused_output_progress_is_not_validated");
            assert_eq!(row["input"]["certified_billings"], "200");
            assert_eq!(expected["ok"]["revenue_to_date"], "200");
            expected["ok"]["revenue_to_date"] = json!("200.00");
        }
        let actual = evaluate(command, &row["input"]);
        if expected.get("schema").is_some() {
            // A JSON number where the schema takes a decimal string: the exact
            // refusal the harness checked names a field holding a non-string.
            let want = expected["error"].as_str().expect("a frozen schema refusal");
            let got = actual["error"].as_str();
            assert_eq!(got, Some(want), "{}: {actual}", row["test"]);
        } else if expected.get("fnv").is_some() {
            let text = serde_json::to_string(&actual).unwrap();
            assert!(matches(text.as_bytes(), &expected), "{}", row["test"]);
        } else {
            assert_eq!(
                actual, expected,
                "{}: {command} {}",
                row["test"], row["input"]
            );
        }
        n += 1;
    }
    assert!(n > 0);
}

#[test]
fn measure_positions_flags_and_refusals() {
    replay(&["measure"]);
}

#[test]
fn money_parsing_and_formatting() {
    replay(&[
        "parse-money",
        "lib:parse",
        "lib:as_",
        "lib:money",
        "lib:percent",
        "lib:points",
    ]);
}

#[test]
fn spreadsheet_and_markdown_escaping() {
    replay(&["lib:guard", "lib:md_cell"]);
}

#[test]
fn contract_files_mappings_and_schedule_dates() {
    replay(&[
        "lib:read_contracts",
        "lib:load_mapping",
        "lib:schedule_as_at",
    ]);
}

static FOLDERS: AtomicUsize = AtomicUsize::new(0);

fn scratch() -> PathBuf {
    let n = FOLDERS.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("wip-parity-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The corpus was frozen on Windows. Elsewhere Python prints a relative path
/// argument with `/` (also inside `repr`) and reports the one Win32 error the
/// corpus holds with the C library's text, as the Rust binary does there.
fn native_text(expected: &Value, args: &[&str]) -> Value {
    let Value::String(hex) = expected else {
        return expected.clone();
    };
    if cfg!(windows) {
        return expected.clone();
    }
    let mut text = String::from_utf8(unhex(hex)).unwrap().replace(
        "[WinError 183] Cannot create a file when that file already exists",
        "[Errno 17] File exists",
    );
    for arg in args.iter().filter(|a| a.contains('/')) {
        let windows = arg.replace('/', "\\");
        text = text.replace(&windows.replace('\\', "\\\\"), arg);
        text = text.replace(&windows, arg);
        if let Some((folder, _)) = arg.rsplit_once('/') {
            text = text.replace(&folder.replace('/', "\\"), folder);
        }
    }
    json!(text.bytes().map(|b| format!("{b:02x}")).collect::<String>())
}

fn snapshot(folder: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    for entry in std::fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            snapshot(&path, base, out);
        } else {
            let rel = path
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, std::fs::read(&path).unwrap()));
        }
    }
}

#[test]
fn original_cli_runs_with_their_files() {
    let rows: Value = serde_json::from_str(include_str!("data/python-cli.json")).unwrap();
    for row in rows.as_array().unwrap() {
        let folder = scratch();
        for (rel, data) in row["files"].as_object().unwrap() {
            let path = folder.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, unhex(data.as_str().unwrap())).unwrap();
        }
        let steps = row["steps"].as_array().unwrap();
        let expected = &row["expected"];
        for (n, (step, want)) in steps
            .iter()
            .zip(expected["results"].as_array().unwrap())
            .enumerate()
        {
            let schedule = folder.join("s.csv");
            if n == 1 && row["tamper"] == json!(true) && schedule.exists() {
                let bytes = std::fs::read(&schedule).unwrap();
                let text = String::from_utf8(bytes)
                    .unwrap()
                    .replacen("0.00", "0.01", 1);
                std::fs::write(&schedule, text).unwrap();
            }
            let mut args: Vec<&str> = step
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_str().unwrap())
                .collect();
            // Completed schedule snapshots used the capture day's default date,
            // including schedules with review flags (exit 2). Replay that date
            // explicitly; the clock default is checked separately below, with
            // every snapshot byte kept intact. Leave parser-error cases alone.
            if args.first() == Some(&"schedule")
                && !args
                    .iter()
                    .any(|a| *a == "--as-at" || a.starts_with("--as-at="))
                && want["stdout"]
                    .as_str()
                    .is_some_and(|hex| unhex(hex).starts_with(b"TheWIPTally: "))
            {
                args.extend(["--as-at", "2026-10-20"]);
            }
            let mut command = Command::new(env!("CARGO_BIN_EXE_wip-tally"));
            command
                .args(&args)
                .current_dir(&folder)
                .env("PYTHON_COLORS", "0")
                .env("COLUMNS", "80")
                .env_remove("FORCE_COLOR")
                .env_remove("NO_COLOR");
            for (key, value) in row["env"].as_object().unwrap() {
                if key == "FORCE_COLOR" {
                    command.env_remove("PYTHON_COLORS");
                }
                command.env(key, value.as_str().unwrap());
            }
            let run = command.output().unwrap();
            assert_eq!(
                run.status.code().unwrap(),
                want["code"].as_i64().unwrap() as i32,
                "{}: {args:?}",
                row["test"]
            );
            assert!(
                matches(&run.stdout, &native_text(&want["stdout"], &args)),
                "stdout {}: {args:?}",
                row["test"]
            );
            assert!(
                matches(&run.stderr, &native_text(&want["stderr"], &args)),
                "stderr {}: {args:?}",
                row["test"]
            );
        }
        let mut files = Vec::new();
        snapshot(&folder, &folder, &mut files);
        files.sort();
        let mut want: Vec<(String, Vec<u8>)> = expected["files"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), unhex(v.as_str().unwrap())))
            .collect();
        want.sort();
        assert_eq!(files, want, "files {}: {:?}", row["test"], steps);
        let _ = std::fs::remove_dir_all(&folder);
    }
}

#[test]
fn schedule_defaults_to_the_current_local_date() {
    let rows: Value = serde_json::from_str(include_str!("data/python-cli.json")).unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["test"] == "test_stated_measurability_produces_the_schedule")
        .unwrap();
    let folder = scratch();
    for (rel, data) in row["files"].as_object().unwrap() {
        let path = folder.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, unhex(data.as_str().unwrap())).unwrap();
    }
    let before = chrono::Local::now().date_naive();
    let run = Command::new(env!("CARGO_BIN_EXE_wip-tally"))
        .args(["schedule", "contracts.csv", "-o", "wip-schedule.csv"])
        .current_dir(&folder)
        .output()
        .unwrap();
    let after = chrono::Local::now().date_naive();
    assert!(run.status.success());
    assert!(run.stderr.is_empty());
    let stdout = lf(&run.stdout);
    let schedule = lf(&std::fs::read(folder.join("wip-schedule.csv")).unwrap());
    let expected_stdout = String::from_utf8(unhex(
        row["expected"]["results"][0]["stdout"].as_str().unwrap(),
    ))
    .unwrap();
    let expected_schedule = String::from_utf8(unhex(
        row["expected"]["files"]["wip-schedule.csv"]
            .as_str()
            .unwrap(),
    ))
    .unwrap();
    assert!([before, after].iter().any(|date| {
        stdout
            == lf(expected_stdout
                .replace("2026-10-20", &date.to_string())
                .as_bytes())
            && schedule
                == lf(expected_schedule
                    .replace("2026-10-20", &date.to_string())
                    .as_bytes())
    }));
    std::fs::remove_dir_all(folder).unwrap();
}

fn summary(json: &str) -> Value {
    serde_json::from_str(json).unwrap()
}

fn d(value: &Value) -> Decimal {
    Decimal::parse(value.as_str().unwrap()).unwrap()
}

#[test]
fn profit_does_not_fund_cash_and_uncertified_claims_are_excluded() {
    let folder = scratch();
    let base = PyPath::new(&folder.join("base").to_string_lossy());
    let result = summary(&job_to_cash::run(&base, &Decimal::parse("0").unwrap()).unwrap());
    for (key, want) in [
        ("gross_profit_at_completion", "200000"),
        ("revenue_to_date", "500000"),
        ("contract_asset", "50000"),
        ("cash_to_date", "-140000"),
        ("retention_gross", "49500"),
        ("collectible_certified_balance", "145500"),
    ] {
        assert!(
            d(&result[key])
                .equals(&Decimal::parse(want).unwrap())
                .unwrap(),
            "{key}"
        );
    }
    let cash = std::fs::read_to_string(folder.join("base/cash-assumptions.csv")).unwrap();
    let rows: Vec<Vec<&str>> = cash
        .lines()
        .skip(1)
        .map(|l| l.split(',').collect())
        .collect();
    let ids: Vec<&str> = rows.iter().map(|r| r[0]).collect();
    assert_eq!(ids, ["CERTIFIED-RECEIVABLE", "RETENTION", "REMAINING-COST"]);
    let receipts = rows.iter().fold(Decimal::parse("0").unwrap(), |sum, r| {
        sum.add(&Decimal::parse(r[2]).unwrap()).unwrap()
    });
    assert!(receipts.equals(&Decimal::parse("195000").unwrap()).unwrap());
}

#[test]
fn cost_change_flows_through_margin_and_cash() {
    let folder = scratch();
    let path = |name: &str| PyPath::new(&folder.join(name).to_string_lossy());
    let base = summary(&job_to_cash::run(&path("base"), &Decimal::parse("0").unwrap()).unwrap());
    let changed =
        summary(&job_to_cash::run(&path("changed"), &Decimal::parse("100000").unwrap()).unwrap());
    let diff = |key: &str| d(&base[key]).sub(&d(&changed[key])).unwrap();
    assert!(
        diff("gross_profit_at_completion")
            .equals(&Decimal::parse("100000").unwrap())
            .unwrap()
    );
    assert!(
        diff("future_cost_cash")
            .equals(&Decimal::parse("-110000").unwrap())
            .unwrap()
    );
    // An existing folder is refused rather than overwritten.
    assert!(job_to_cash::run(&path("base"), &Decimal::parse("0").unwrap()).is_err());
}

#[test]
fn float_amounts_are_refused_not_converted() {
    // The Python API raised TypeError for a float ratio; a JSON number is refused.
    let mut contract = json!({"contract_id": "J", "line_number": 2, "customer": "",
        "description": "", "retention_classification": "review",
        "progress_method": "cost_to_cost", "outcome_reasonably_measurable": true});
    for field in [
        "original_contract_sum",
        "approved_variations",
        "unapproved_variations_estimate",
        "costs_incurred",
        "inefficiency_rework_wastage",
        "uninstalled_materials",
        "estimated_cost_to_complete",
        "certified_billings",
        "uncertified_claims",
        "retention_withheld",
        "committed_outstanding",
        "gst_rate",
    ] {
        contract[field] = json!("0");
    }
    contract["constraint_include_ratio"] = json!(0.5);
    assert_eq!(
        api::evaluate("measure", &contract).unwrap_err(),
        "constraint_include_ratio must be a string"
    );
    // The same ratio as a string reaches the engine, which refuses nil progress EAC.
    contract["constraint_include_ratio"] = json!("0.5");
    assert!(
        api::evaluate("measure", &contract)
            .unwrap_err()
            .contains("cost-to-cost progress is undefined")
    );
}

/// argparse prints the version through the help formatter, so a narrow
/// terminal wraps it (Python 3.15 at COLUMNS=10 printed the same layout).
#[test]
fn the_version_wraps_like_help_text() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wip-tally"))
        .arg("--version")
        .env("COLUMNS", "10")
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap().replace("\r\n", "\n");
    assert_eq!(text, "wip-tally\n1.0.0\n");
}

#[test]
fn as_at_help_distinguishes_command_defaults() {
    for (command, expected) in [
        (
            "schedule",
            "--as-at AS_AT reporting date YYYY-MM-DD (default: today)",
        ),
        (
            "review-pack",
            "--as-at AS_AT reporting date YYYY-MM-DD (default: schedule date, or today if unavailable)",
        ),
    ] {
        for help in ["-h", "--help"] {
            let out = Command::new(env!("CARGO_BIN_EXE_wip-tally"))
                .args([command, help])
                .env("PYTHON_COLORS", "0")
                .env("COLUMNS", "80")
                .env_remove("FORCE_COLOR")
                .env_remove("NO_COLOR")
                .output()
                .unwrap();
            assert!(out.status.success(), "{command} {help}: {:?}", out.stderr);
            assert!(out.stderr.is_empty(), "{command} {help}: {:?}", out.stderr);
            let text = String::from_utf8(out.stdout).unwrap();
            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            assert!(text.contains(expected), "{command} {help}: {text}");
        }
    }
}

/// `where` as `json.loads` read it from literal bytes: the harness serialises a
/// Python value first, which would normalise `1.2300` before the binary saw it.
/// Expected strings are Python 0.2.0's `parse_money` on the same literals.
#[test]
fn where_prints_as_python_printed_the_decoded_value() {
    let cases = [
        (r#"{"raw":"1","where":[]}"#, Ok("1.00")),
        (r#"{"raw":null,"where":[]}"#, Err("[]: no amount given")),
        (
            r#"{"raw":null,"where":1.2300}"#,
            Err("1.23: no amount given"),
        ),
        (
            r#"{"raw":null,"where":1e3}"#,
            Err("1000.0: no amount given"),
        ),
        (
            r#"{"raw":null,"where":1.0000000000000001}"#,
            Err("1.0: no amount given"),
        ),
        (r#"{"raw":null,"where":1e400}"#, Err("inf: no amount given")),
        (
            r#"{"raw":null,"where":-1e400}"#,
            Err("-inf: no amount given"),
        ),
        (
            r#"{"raw":null,"where":[1e400,-1e400]}"#,
            Err("[inf, -inf]: no amount given"),
        ),
        (
            r#"{"raw":null,"where":[1,"a",true,null,[2.5]]}"#,
            Err("[1, 'a', True, None, [2.5]]: no amount given"),
        ),
    ];
    for (input, want) in cases {
        let value: Value = serde_json::from_str(input).unwrap();
        let got = the_wip_tally::api::evaluate("parse-money", &value);
        match want {
            Ok(text) => assert_eq!(got.unwrap(), serde_json::json!(text), "{input}"),
            Err(text) => assert_eq!(got.unwrap_err(), text, "{input}"),
        }
    }
}
