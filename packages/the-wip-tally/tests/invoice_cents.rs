use serde_json::json;
use the_wip_tally::{api, csv_io, engine, money::d};

fn contract() -> engine::ContractInput {
    let csv = b"contract_id,original_contract_sum,costs_incurred,estimated_cost_to_complete,certified_billings,outcome_reasonably_measurable\nCROFT-1,1000,200,300,100,yes\n";
    csv_io::read_contracts("invented.csv", csv, &csv_io::load_mapping(None).unwrap())
        .unwrap()
        .pop()
        .unwrap()
}

#[test]
fn invoice_revenue_and_position_use_the_same_cents() {
    for (raw, revenue, gst) in [
        ("0", "0.00", "0.00"),
        ("0.004", "0.00", "0.00"),
        ("100.00", "100.00", "10.00"),
        ("100.004", "100.00", "10.00"),
        ("100.005", "100.01", "10.00"),
        ("100.049", "100.05", "10.00"),
    ] {
        let mut input = contract();
        input.progress_method = engine::PROGRESS_RIGHT_TO_INVOICE.into();
        input.certified_billings = d(raw);
        input.prior_revenue_to_date = Some(d("90.00"));
        let position = engine::measure(&input).unwrap();
        assert_eq!(position.revenue_to_date.to_string(), revenue, "{raw}");
        assert_eq!(position.contract_asset.to_string(), "0.00", "{raw}");
        assert_eq!(position.contract_liability.to_string(), "0.00", "{raw}");
        assert_eq!(position.gst_on_certified_billings.to_string(), gst, "{raw}");
        assert_eq!(
            position.period_revenue.unwrap().to_string(),
            d(revenue).sub(&d("90.00")).unwrap().to_string(),
            "{raw}"
        );
        assert_eq!(position.contract.certified_billings.to_string(), raw);
        assert_eq!(input.certified_billings.to_string(), raw);
    }
}

#[test]
fn other_revenue_methods_keep_their_rounding() {
    for method in [engine::PROGRESS_COST_TO_COST, engine::PROGRESS_OUTPUT] {
        let mut input = contract();
        input.progress_method = method.into();
        input.output_percent = Some(d("0.4"));
        input.certified_billings = d("100.005");
        let position = engine::measure(&input).unwrap();
        assert_eq!(position.revenue_to_date.to_string(), "400.00");
        assert_eq!(position.contract_asset.to_string(), "300.00");
        assert_eq!(position.contract_liability.to_string(), "0.00");
    }
    let mut input = contract();
    input.progress_method = engine::PROGRESS_RIGHT_TO_INVOICE.into();
    input.outcome_reasonably_measurable = Some(false);
    input.recoverable_costs = Some(d("100.01"));
    input.certified_billings = d("100.005");
    let position = engine::measure(&input).unwrap();
    assert_eq!(position.revenue_to_date.to_string(), "100.01");
    assert_eq!(position.contract_asset.to_string(), "0.01");
    assert_eq!(position.contract_liability.to_string(), "0.00");
}

#[test]
fn gst_uses_supplied_billings_independently_of_progress_method() {
    for (method, measurable) in [
        (engine::PROGRESS_RIGHT_TO_INVOICE, true),
        (engine::PROGRESS_COST_TO_COST, true),
        (engine::PROGRESS_OUTPUT, true),
        (engine::PROGRESS_RIGHT_TO_INVOICE, false),
    ] {
        let mut input = contract();
        input.progress_method = method.into();
        input.outcome_reasonably_measurable = Some(measurable);
        input.output_percent = Some(d("0.4"));
        input.certified_billings = d("100.049");
        let position = engine::measure(&input).unwrap();
        assert_eq!(position.gst_on_certified_billings.to_string(), "10.00");
        if measurable && method == engine::PROGRESS_RIGHT_TO_INVOICE {
            assert_eq!(position.revenue_to_date.to_string(), "100.05");
            assert_eq!(position.contract_asset.to_string(), "0.00");
            assert_eq!(position.contract_liability.to_string(), "0.00");
        }
    }
}

#[test]
fn json_measure_returns_invoice_cents() {
    let input = json!({
        "contract_id": "CROFT-1", "line_number": 2, "customer": "", "description": "",
        "original_contract_sum": "1000", "approved_variations": "0",
        "unapproved_variations_estimate": "0", "constraint_include_ratio": "0",
        "costs_incurred": "200", "inefficiency_rework_wastage": "0",
        "uninstalled_materials": "0", "estimated_cost_to_complete": "300",
        "certified_billings": "100.005", "uncertified_claims": "0",
        "retention_withheld": "0", "retention_classification": "review",
        "committed_outstanding": "0", "outcome_reasonably_measurable": true,
        "progress_method": "right_to_invoice", "gst_rate": "0.10"
    });
    let position = api::evaluate("measure", &input).unwrap();
    assert_eq!(position["revenue_to_date"], "100.01");
    assert_eq!(position["contract_asset"], "0.00");
    assert_eq!(position["contract_liability"], "0.00");
}
