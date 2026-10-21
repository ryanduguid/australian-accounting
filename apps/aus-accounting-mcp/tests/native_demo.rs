//! Compare the complete native proof with the checked Python proof.
use serde_json::{Value, json};
use std::process::{Command, Stdio};

const HEADER: &str = "$ cargo run --locked --bin aus-accounting-mcp-demo\n";

fn expected_payload() -> Value {
    let (_, body) = include_str!("../docs/quick-proof.txt")
        .split_once('\n')
        .unwrap();
    let mut payload: Value = serde_json::from_str(body).unwrap();
    let result = &mut payload["calls"][1]["result"];
    // The same exact native identities accepted for the server in NATIVE.md.
    assert_eq!(result["engine_version"], "0.3.2");
    result["engine_version"] = json!("1.0.0");
    let uri = &mut result["source"]["rate_table_uris"][0]["uri"];
    assert_eq!(*uri, "div7aloan/data/benchmark_rates.csv");
    *uri = json!("data/benchmark_rates.csv");
    payload.sort_all_objects();
    payload
}

#[test]
fn complete_native_demo_preserves_original_proof() {
    let output = Command::new(env!("CARGO_BIN_EXE_aus-accounting-mcp-demo"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let body = stdout.strip_prefix(HEADER).unwrap();
    let actual: Value = serde_json::from_str(body).unwrap();
    let expected = expected_payload();
    assert_eq!(actual, expected);
    assert_eq!(
        stdout,
        format!(
            "{HEADER}{}\n",
            serde_json::to_string_pretty(&expected).unwrap()
        )
    );
    let fixture = &actual["calls"][0]["result"];
    assert_eq!(fixture["synthetic"], true);
    assert_eq!(fixture["not_a_lodgment"], true);
    let loan = &actual["calls"][1]["result"];
    assert_eq!(loan["gate"]["verdict"], "COMPLYING");
    assert_eq!(loan["minimum_yearly_repayment"]["verdict"], "MYR_MET");
    assert_eq!(
        loan["minimum_yearly_repayment"]["myr_required"],
        "108770.00"
    );
}
