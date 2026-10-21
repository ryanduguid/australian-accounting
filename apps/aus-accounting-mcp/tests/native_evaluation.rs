//! Complete output oracles captured from the unchanged Python evaluation tool.
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/native/evaluation.json")).unwrap()
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "native-evaluation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aus-accounting-mcp-evaluate"))
        .args(args)
        .stdin(Stdio::null())
        .env_remove("AUS_ACCOUNTING_LIBRARY_ROOT")
        .env_remove("AUS_ACCOUNTING_CORPUS_ROOT")
        .env_remove("AUS_ACCOUNTING_RULINGS_ROOT")
        .output()
        .unwrap()
}
#[test]
fn complete_scoring_matches_original_integrity_and_typed_value_oracles() {
    let fixture = fixture();
    let temp = Temp::new();
    for case in fixture["cases"].as_array().unwrap() {
        fs::write(
            temp.0.join("record.json"),
            case["recording"].as_str().unwrap(),
        )
        .unwrap();
        let record = temp.0.join("record.json");
        let reference = temp.0.join("questions.xml");
        let mut args = Vec::new();
        if let Some(xml) = case["reference"].as_str() {
            fs::write(&reference, xml).unwrap();
            args.extend(["--reference", reference.to_str().unwrap()]);
        }
        args.extend(["score", record.to_str().unwrap()]);
        if let Some(digest) = case["digest"].as_str() {
            args.extend(["--reference-sha256", digest]);
        }
        let output = run(&args);
        let label = case["label"].as_str().unwrap();
        if label == "internal-entity" {
            // Explicit custom-reference limit: native rejects all DTDs before
            // expansion, whereas the original accepts small internal entities.
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8(output.stderr).unwrap().contains("DTD"));
            continue;
        }
        assert_eq!(
            output.status.code().unwrap() as i64,
            case["exit"].as_i64().unwrap(),
            "{label}: {:?}",
            output.stderr
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            case["stdout"].as_str().unwrap(),
            "{label}"
        );
        let error = String::from_utf8(output.stderr).unwrap();
        if let Some(fragment) = case["diagnostic_contains"].as_str() {
            // Parser wording/locations are implementation-specific; refusal and
            // empty stdout must still match. Semantic validation text is exact.
            assert!(error.contains(fragment), "{label}: {error}");
        } else {
            assert_eq!(error, case["stderr"].as_str().unwrap(), "{label}");
        }
    }
}
#[test]
fn questions_and_complete_actual_stdio_context_match_original() {
    let fixture = fixture();
    let questions = run(&["questions"]);
    assert!(questions.status.success());
    assert!(questions.stderr.is_empty());
    assert_eq!(
        String::from_utf8(questions.stdout).unwrap(),
        fixture["questions"].as_str().unwrap()
    );
    let context = run(&["context"]);
    assert!(context.status.success(), "{:?}", context.stderr);
    assert!(context.stderr.is_empty());
    let context = String::from_utf8(context.stdout).unwrap();
    let marker = "# Preloaded scope resource\n\naus-accounting://scope\n\n";
    let (definitions, scope) = context.split_once(marker).unwrap();
    let (expected_definitions, expected_scope) = fixture["context"]
        .as_str()
        .unwrap()
        .split_once(marker)
        .unwrap();
    assert_eq!(
        definitions, expected_definitions,
        "full instructions, tools and schemas"
    );
    // Resource JSON formatting is incidental; every decoded field is compared.
    let actual: Value = serde_json::from_str(scope).unwrap();
    let mut expected: Value = serde_json::from_str(expected_scope).unwrap();
    // Existing reviewed native study-loan provenance, as qualified for scope
    // resources and recorded in NATIVE.md. No calculation field is adapted.
    let worksheet = &mut expected["calculation_worksheets"]["study_loan_repayment"];
    for (pointer, before, after) in [
        (
            "/source",
            "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-support-loans-rates-and-repayment-thresholds",
            "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates",
        ),
        (
            "/sources/0",
            "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-support-loans-rates-and-repayment-thresholds",
            "https://www.ato.gov.au/tax-rates-and-codes/study-and-training-loan-minimum-repayment-income-and-rates",
        ),
        ("/source_checked", "2026-09-27", "2026-10-20"),
        ("/example_evidence/checked", "2026-09-27", "2026-10-20"),
        ("/example_evidence/reviewed", "2026-06-30", "2026-10-13"),
        (
            "/example_evidence/document",
            "ATO study and training loan repayment thresholds and rates",
            "ATO study and training loan minimum repayment income and rates",
        ),
        (
            "/source_passage",
            "Study and training loan repayment thresholds and rates: tables 1 and 2, repayment income, examples 1 to 3; Gazette C2026G00249 for the 2026-27 minimum repayment income and replacement indexable amount",
            "Study and training loan minimum repayment income and rates: tables 1 and 2, repayment income, examples 1 to 3; Gazette C2026G00249 for the 2026-27 minimum repayment income and replacement indexable amount",
        ),
    ] {
        let value = worksheet.pointer_mut(pointer).unwrap();
        assert_eq!(value, before);
        *value = Value::String(after.into());
    }
    assert_eq!(actual, expected);
}

#[test]
fn score_option_forms_and_nested_inputs_are_bounded() {
    let fixture = fixture();
    let temp = Temp::new();
    let record = temp.0.join("record.json");
    let case = &fixture["cases"][0];
    fs::write(&record, case["recording"].as_str().unwrap()).unwrap();
    let path = record.to_str().unwrap();
    let digest = fixture["reference_sha256"].as_str().unwrap();
    for args in [
        vec!["score", "--reference-sha256", digest, path],
        vec!["score", path, &format!("--reference-sha256={digest}")],
    ] {
        let output = run(&args);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            case["stdout"].as_str().unwrap()
        );
    }
    for args in [
        vec!["score", path, "--help"],
        vec!["score", "--reference-sha256", digest, "--help"],
    ] {
        let output = run(&args);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
    }
    for depth in [600, 990, 1_001] {
        let text = format!(
            r#"{{"unknown-rate":{{"calls":[{{"name":"get_div7a_benchmark_rate","arguments":{{"value":{}1{}}}}}],"answer":"UNKNOWN"}}}}"#,
            "[".repeat(depth),
            "]".repeat(depth)
        );
        fs::write(&record, text).unwrap();
        let output = run(&["score", path]);
        if depth < 1_000 {
            assert!(
                output.status.success(),
                "depth {depth}: {:?}",
                output.stderr
            );
            assert!(
                String::from_utf8(output.stdout)
                    .unwrap()
                    .contains("calls differ (arguments, order or count)")
            );
            assert!(output.stderr.is_empty());
        } else {
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .contains("1,000 containers")
            );
        }
    }
    // Refuse entity amplification before parsing/expansion, independently of
    // roxmltree's per-entity recursion guards and regardless of scoring result.
    let reference = temp.0.join("expanded.xml");
    fs::write(&reference, format!("<!DOCTYPE questions [<!ENTITY a '{}'>]><questions><qa_pair id='x'><question>Q</question><calls>[]</calls><answer>{}</answer></qa_pair></questions>", "a".repeat(4_096), "&a;".repeat(4_096))).unwrap();
    fs::write(&record, "{}").unwrap();
    let output = run(&["--reference", reference.to_str().unwrap(), "score", path]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("DTD"));
    // Bad recordings are rejected before an unavailable custom reference.
    fs::write(&record, "null").unwrap();
    let output = run(&[
        "--reference",
        temp.0.join("missing.xml").to_str().unwrap(),
        "score",
        path,
    ]);
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "invalid recording: recording must be an object keyed by question id\n"
    );
}

#[test]
fn large_nested_values_use_borrowed_tokens_and_bounded_work() {
    let temp = Temp::new();
    let record = temp.0.join("record.json");
    for (depth, bytes, expected_exit) in [
        (800, 1_048_576, 0),
        (995, 1_048_576, 0),
        (600, 2_097_152, 2),
    ] {
        let text = format!(
            r#"{{"unknown-rate":{{"calls":[{{"name":"get_div7a_benchmark_rate","arguments":{{"value":{}"{}"{}}}}}],"answer":"UNKNOWN"}}}}"#,
            "[".repeat(depth),
            "a".repeat(bytes),
            "]".repeat(depth)
        );
        fs::write(&record, text).unwrap();
        let output = run(&["score", record.to_str().unwrap()]);
        assert_eq!(
            output.status.code(),
            Some(expected_exit),
            "depth {depth}: {:?}",
            output.stderr
        );
        if expected_exit == 0 {
            assert!(output.stderr.is_empty());
            assert!(
                String::from_utf8(output.stdout)
                    .unwrap()
                    .contains("calls differ (arguments, order or count)")
            );
        } else {
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .contains("cumulative parsing work")
            );
        }
    }
}

#[test]
fn lenient_questions_keep_first_position_and_last_duplicate_value() {
    let temp = Temp::new();
    let reference = temp.0.join("questions.xml");
    fs::write(&reference, "<questions><qa_pair id='b'><question>First</question></qa_pair><qa_pair id='a'><question>Middle</question></qa_pair><qa_pair id='b'><question>Last</question></qa_pair></questions>").unwrap();
    let output = run(&["--reference", reference.to_str().unwrap(), "questions"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "b: Last\na: Middle\n"
    );
}
#[test]
fn embedded_reference_and_sibling_server_work_outside_checkout() {
    let temp = Temp::new();
    for name in ["aus-accounting-mcp-evaluate", "aus-accounting-mcp"] {
        let source = if name.ends_with("evaluate") {
            env!("CARGO_BIN_EXE_aus-accounting-mcp-evaluate")
        } else {
            env!("CARGO_BIN_EXE_aus-accounting-mcp")
        };
        fs::copy(
            source,
            temp.0
                .join(format!("{name}{}", std::env::consts::EXE_SUFFIX)),
        )
        .unwrap();
    }
    let exe = temp.0.join(format!(
        "aus-accounting-mcp-evaluate{}",
        std::env::consts::EXE_SUFFIX
    ));
    let empty = temp.0.join("empty");
    fs::create_dir(&empty).unwrap();
    for command in ["questions", "context"] {
        let output = Command::new(&exe)
            .arg(command)
            .current_dir(&empty)
            .env("PATH", "")
            .env_remove("AUS_ACCOUNTING_LIBRARY_ROOT")
            .env_remove("AUS_ACCOUNTING_CORPUS_ROOT")
            .env_remove("AUS_ACCOUNTING_RULINGS_ROOT")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        assert!(output.stderr.is_empty());
        assert!(!output.stdout.is_empty());
        assert_eq!(fs::read_dir(&empty).unwrap().count(), 0);
    }
    let output = Command::new(&exe)
        .arg("context")
        .current_dir(&empty)
        .env("PATH", "")
        .env("AUS_ACCOUNTING_LIBRARY_ROOT", &empty)
        .env("AUS_ACCOUNTING_CORPUS_ROOT", &empty)
        .env("AUS_ACCOUNTING_RULINGS_ROOT", &empty)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    let scope: Value = serde_json::from_str(
        text.split_once("# Preloaded scope resource\n\naus-accounting://scope\n\n")
            .unwrap()
            .1,
    )
    .unwrap();
    for name in [
        "AUS_ACCOUNTING_LIBRARY_ROOT",
        "AUS_ACCOUNTING_CORPUS_ROOT",
        "AUS_ACCOUNTING_RULINGS_ROOT",
    ] {
        assert_eq!(scope["retrieval_folders_configured"][name], true);
    }
    fs::remove_file(temp.0.join(format!(
        "aus-accounting-mcp{}",
        std::env::consts::EXE_SUFFIX
    )))
    .unwrap();
    let output = Command::new(&exe)
        .arg("context")
        .current_dir(&empty)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
}
