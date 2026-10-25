use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};
use the_wip_tally::{api, cli, csv_io, money};

const TOO_LARGE: &str = "100000000000000000000000000";
const REASON: &str = "amount is outside the supported range for cents";

#[test]
fn money_range_errors_keep_the_field_context() {
    for raw in [
        TOO_LARGE.to_string(),
        format!("-{TOO_LARGE}"),
        format!("({TOO_LARGE})"),
        format!("{TOO_LARGE} CR"),
        "99999999999999999999999999.995".into(),
    ] {
        assert_eq!(
            money::parse_money(&raw, "estimate").unwrap_err(),
            money::AmountError::Message(format!("estimate: {REASON}")),
            "{raw}"
        );
        assert_eq!(
            api::evaluate("parse-money", &json!({"raw":raw,"where":"estimate"})),
            Err(format!("estimate: {REASON}"))
        );
    }
}

#[test]
fn amounts_at_the_precision_boundary_keep_existing_rounding() {
    for (raw, expected) in [
        (
            "99999999999999999999999999.99",
            "99999999999999999999999999.99",
        ),
        (
            "99999999999999999999999999.994",
            "99999999999999999999999999.99",
        ),
        (
            "12345678901234567890123456.785 CR",
            "-12345678901234567890123456.78",
        ),
        (
            "-12345678901234567890123456.785",
            "-12345678901234567890123456.79",
        ),
        ("0.005", "0.01"),
    ] {
        assert_eq!(
            money::parse_money(raw, "amount").unwrap().to_string(),
            expected
        );
    }
    assert!(
        matches!(money::parse_money("0.004", "amount"), Err(money::AmountError::Message(m)) if m.contains("smaller than half a cent"))
    );
    assert!(
        matches!(money::parse_money("not money", "amount"), Err(money::AmountError::Message(m)) if m.contains("is not an amount"))
    );
}

#[test]
fn accounting_negation_overflow_is_a_contextual_amount_error() {
    let magnitude = format!("1{}", "0".repeat(1_000_000));
    for raw in [format!("{magnitude} CR"), format!("({magnitude})")] {
        assert_eq!(
            money::parse_money(&raw, "estimate").unwrap_err(),
            money::AmountError::Message(format!("estimate: {REASON}"))
        );
    }
}

#[test]
fn json_command_refuses_range_errors_with_context_and_exit_2() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_wip-tally"))
        .arg("parse-money")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            json!({"raw": TOO_LARGE, "where": "estimate"})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr)
            .unwrap()
            .replace("\r\n", "\n"),
        format!("error: estimate: {REASON}\n")
    );
}

#[test]
fn csv_range_errors_accumulate_with_row_contract_and_field() {
    let csv = format!(
        "contract_id,original_contract_sum,costs_incurred,estimated_cost_to_complete,certified_billings,outcome_reasonably_measurable\nCROFT-1,{TOO_LARGE},200,300,100,yes\nCROFT-2,1000,200,300,{TOO_LARGE},yes\n"
    );
    let expected = format!(
        "invented.csv has unreadable cells:\n  row 2 (CROFT-1): original_contract_sum: {REASON}\n  row 3 (CROFT-2): certified_billings: {REASON}"
    );
    let error = csv_io::read_contracts(
        "invented.csv",
        csv.as_bytes(),
        &csv_io::load_mapping(None).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error, csv_io::Failure::Message(expected));
}

#[test]
fn schedule_cli_refuses_out_of_range_cells_without_outputs() {
    let folder = std::env::temp_dir().join(format!("wip-range-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let source = folder.join("invented.csv");
    let output = folder.join("schedule.csv");
    std::fs::write(&source, format!("contract_id,original_contract_sum,costs_incurred,estimated_cost_to_complete,certified_billings,outcome_reasonably_measurable\nCROFT-1,{TOO_LARGE},200,300,100,yes\n")).unwrap();
    let (code, stdout, stderr) = cli::run(&[
        "schedule".into(),
        source.to_string_lossy().into(),
        "--as-at".into(),
        "2026-10-25".into(),
        "--output".into(),
        output.to_string_lossy().into(),
    ]);
    assert_eq!(code, 1, "{stderr}");
    assert!(stdout.is_empty());
    assert!(
        stderr.contains(&format!("row 2 (CROFT-1): original_contract_sum: {REASON}")),
        "{stderr}"
    );
    assert!(!output.exists());
    std::fs::remove_dir_all(folder).unwrap();
}
