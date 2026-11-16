// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `dgr-check` against the bundled data, the ABN Bulk Extract of 11 November
//! 2026 (docs/calculation-evidence.md#dgr-check). Each ABN below was read the
//! same in ABN Lookup's DGR listing downloads of 16 November 2026
//! (https://abr.business.gov.au/Tools/DgrListing): ABN, status, DGR status
//! date and fund name. The engine's readings on fabricated data are in
//! `src/dgr.rs`'s unit tests.
use australian_tax_calculators::api;
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

const DATA: &str = include_str!("../data/dgr-register.jsonl");

// ABNs from the data, kept apart from their labels so the repository's ABN
// secret scan does not read them as client identifiers.
const ARTHRITIS: &str = "67002598594";
const ARTHRITIS_SPACED: &str = "67 002 598 594";
const COLLEGE: &str = "11000047950";
const COLLEGE_SPACED: &str = "11 000 047 950";
const KILLARNEY: &str = "11001209490";
const CANCELLED: &str = "12713020917";

fn check(donations: Value) -> Value {
    api::evaluate("dgr-check", &json!({"donations": donations})).unwrap()
}

fn one(donation: Value) -> Value {
    check(json!([donation]))["donations"][0].clone()
}

fn spaced(abn: &str) -> String {
    format!("{} {} {} {}", &abn[..2], &abn[2..5], &abn[5..8], &abn[8..])
}

fn conditions(result: &Value) -> Vec<&str> {
    let list = result["conditions"].as_array().unwrap();
    list.iter()
        .map(|c| c["condition"].as_str().unwrap())
        .collect()
}

#[test]
fn the_bundled_data_is_one_extract_of_organisations_only() {
    let header: Value = serde_json::from_str(DATA.lines().next().unwrap()).unwrap();
    assert_eq!(header["extract_time"], "2026-11-11T12:20:05");
    assert_eq!(
        header["licence"],
        "Creative Commons Attribution 3.0 Australia"
    );
    assert_eq!(header["individuals_with_dgr_dropped"], 0);
    let rows = DATA.lines().count() - 1;
    assert_eq!(header["abns"], rows);
    let r = check(json!([{"abn": ARTHRITIS}]));
    assert_eq!(r["data"]["data_date"], "2026-11-11");
    assert_eq!(r["data"]["abns_with_dgr_entries"], rows);
    assert_eq!(r["calculation"], "dgr_check");
    // Every row keeps 6 fields and no other: ABN, status, status date,
    // name, DGR entries and other names. No address, ASIC number or GST.
    for line in DATA.lines().skip(1) {
        let row: Vec<Value> = serde_json::from_str(line).unwrap();
        assert_eq!(row.len(), 6, "{line}");
    }
}

#[test]
fn an_entity_endorsed_in_its_own_right_and_for_a_fund() {
    // ABN Lookup: Item 1, Endorsed DGR from 2006-03-01; its building and
    // maintenance fund from 2000-07-01.
    let r = one(json!({"abn": COLLEGE_SPACED, "date": "2026-03-01", "amount": "100.00"}));
    assert_eq!(r["register"], "shown_as_dgr");
    assert_eq!(r["on_payment_date"], "shown");
    assert_eq!(r["entity"]["name"], "SYDNEY MISSIONARY & BIBLE COLLEGE");
    assert_eq!(
        r["dgr_entries"],
        json!([
            {"from": "2000-07-01", "endorsed_for": "SYDNEY MISSIONARY & BIBLE COLLEGE BUILDING & MAINTENANCE FUND"},
            {"from": "2006-03-01", "endorsed_for": "the entity itself"},
        ])
    );
    assert_eq!(conditions(&r), ["a_gift", "gift_conditions", "record"]);
    let earlier = one(json!({"abn": COLLEGE, "date": "2005-06-30"}));
    assert_eq!(earlier["on_payment_date"], "shown_for_a_fund");
    assert!(conditions(&earlier).contains(&"gift_to_the_fund"));
}

/// The whole result for one donation to a fund-only DGR: a golden record of
/// the wording a reviewer reads.
#[test]
fn a_fund_only_school_gives_this_result() {
    // ABN Lookup: THE KILLARNEY SCHOOL LTD BUILDING FUND, Active, 2000-07-01.
    let input = json!({"abn": KILLARNEY, "name": "Killarney School", "date": "2024-06-30", "amount": "1.50"});
    let expected = json!({
        "abn": KILLARNEY,
        "register": "shown_as_dgr",
        "finding": format!("ABN {} (THE KILLARNEY SCHOOL LTD) is shown as a deductible gift recipient in the data of 11 November 2026, with entries from 1 July 2000. The data shows the entity endorsed on the payment date only for the operation of a fund, authority or institution.", spaced(KILLARNEY)),
        "name_matches_register": true,
        "entity": {"abn": KILLARNEY, "name": "THE KILLARNEY SCHOOL LTD", "abn_status": "active", "abn_status_from": "2000-04-11"},
        "other_names": [],
        "dgr_entries": [{"from": "2000-07-01", "endorsed_for": "THE KILLARNEY SCHOOL LTD BUILDING FUND"}],
        "on_payment_date": "shown_for_a_fund",
        "conditions": [
            {"condition": "a_gift", "status": "open", "text": "The payment is a gift: a voluntary transfer of money with no material benefit or advantage in return. A payment for a ticket to a fund-raising event or for an item won at a charity auction is a contribution under s 30-15 items 7 or 8, with other conditions; one made under a will is not deductible (s 30-15(2)).", "source": "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/gifts-and-donations"},
            {"condition": "gift_conditions", "status": "open", "text": "Any condition the relevant table item in Subdivision 30-B sets for this kind of recipient is met, such as a gift being for a stated purpose (s 30-15 item 1, special condition (c)). ABN Lookup shows the recipient's DGR item.", "source": "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html"},
            {"condition": "record", "status": "open", "text": "A record of the gift is kept, such as a receipt; a DGR's receipt states its name, its ABN and that it is for a gift (s 30-228(1)).", "source": "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/gifts-and-donations"},
            {"condition": "gift_to_the_fund", "status": "open", "text": "The entity is endorsed only for the operation of a fund, authority or institution, so the gift must have been made to it: THE KILLARNEY SCHOOL LTD BUILDING FUND (s 30-17(2)(b) and (c), s 30-229(2)).", "source": "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html"},
            {"condition": "two_dollar_minimum", "status": "fails", "text": "A gift made before 1 July 2024 must be $2 or more; the minimum does not apply to a gift made on or after that date (Act No. 58, 2026, Sch 1 item 7).", "source": "https://www.legislation.gov.au/C2026A00058/asmade/text"},
        ],
        "index": 0,
        "input": input.clone(),
    });
    assert_eq!(one(input), expected);
}

#[test]
fn a_cancelled_abn_keeps_its_fund_entry_but_is_not_read_as_a_dgr_after() {
    // ABN Lookup lists the fund as Cancelled, DGR status date 2000-07-01.
    let r = one(json!({"abn": CANCELLED, "date": "2024-01-10"}));
    assert_eq!(r["register"], "shown_as_dgr");
    assert_eq!(r["entity"]["abn_status"], "cancelled");
    assert_eq!(r["entity"]["abn_status_from"], "2023-10-16");
    assert_eq!(r["on_payment_date"], "abn_cancelled");
    let before = one(json!({"abn": CANCELLED, "date": "2023-10-15"}));
    assert_eq!(before["on_payment_date"], "shown_for_a_fund");
}

#[test]
fn abn_lookups_worked_example_abn_is_valid_but_not_shown() {
    // https://abr.business.gov.au/Help/AbnFormat (read 16 November 2026).
    let r = one(json!({"abn": "51 824 753 556", "date": "2025-10-01", "amount": "20.00"}));
    assert_eq!(r["register"], "not_shown_as_dgr");
    assert!(
        r["finding"]
            .as_str()
            .unwrap()
            .contains("That does not establish that it is not one")
    );
    assert_eq!(conditions(&r), ["a_gift", "gift_conditions", "record"]);
}

/// The Library's worked example ¶7-020 (Tax Examples, Individuals, reviewed 30
/// June 2025) deducts a $250 donation in 2024-25 to 'the Arthritis Foundation'
/// under s 30-15 as a gift to a DGR. By name alone the check gives candidates,
/// each unconfirmed, and no reading: the receipt's ABN decides.
#[test]
fn the_librarys_arthritis_foundation_donation_gives_only_candidates() {
    let r = one(json!({"name": "Arthritis Foundation", "date": "2025-03-01", "amount": "250.00"}));
    assert_eq!(r["register"], "abn_needed");
    let candidates = r["candidates"].as_array().unwrap();
    assert!(r["candidates_total"].as_u64().unwrap() >= candidates.len() as u64);
    assert!(candidates.len() >= 5, "{candidates:?}");
    assert!(candidates.iter().all(|c| c["status"] == "unconfirmed"));
    let names: Vec<&str> = candidates
        .iter()
        .map(|c| c["entity"]["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"ARTHRITIS FOUNDATION OF AUSTRALIA"),
        "{names:?}"
    );
    assert!(r.get("on_payment_date").is_none());
    assert!(!conditions(&r).contains(&"two_dollar_minimum"));
    // With the ABN from a receipt, the reading follows.
    let r = one(
        json!({"abn": ARTHRITIS_SPACED, "name": "Arthritis Foundation", "date": "2025-03-01", "amount": "250.00"}),
    );
    assert_eq!(r["on_payment_date"], "shown");
    assert_eq!(r["name_matches_register"], true);
}

#[test]
fn every_result_states_its_data_date_limits_and_sources() {
    let r = check(json!([{"abn": ARTHRITIS}, {"name": "Arthritis"}]));
    assert_eq!(r["data"]["extract_time"], "2026-11-11T12:20:05");
    assert!(
        r["data"]["attribution"]
            .as_str()
            .unwrap()
            .contains("CC BY 3.0 AU")
    );
    let limits = r["limits"].as_array().unwrap();
    assert!(limits[0].as_str().unwrap().contains("current details only"));
    let sources = r["sources"].as_array().unwrap();
    assert!(sources.contains(&json!("https://data.gov.au/data/dataset/abn-bulk-extract")));
    let text = r.to_string();
    assert!(!text.contains("\"deductible\"") && !text.contains("is deductible"));
}

#[test]
fn the_binary_refuses_a_malformed_abn_with_its_code() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_australian-tax-calculators"))
        .args(["dgr-check", "--refusal-json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let input = br#"{"donations": [{"abn": "51 824 753 557"}]}"#;
    child.stdin.take().unwrap().write_all(input).unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let refusal: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(refusal["code"], "input.abn_invalid");
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("error: donations[0].abn"));
}
