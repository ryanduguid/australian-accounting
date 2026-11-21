// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The extract reader's tests, on fabricated files.
use super::*;

/// One file of fabricated records in the extract's layout.
fn file(sequence: u32, count: u32, records: &str) -> String {
    format!(
        "<?xml version=\"1.0\"?><Transfer error=\"none\"><TransferInfo><FileSequenceNumber>{sequence}</FileSequenceNumber><RecordCount>{count}</RecordCount><ExtractTime>2026-11-11T12:20:05</ExtractTime></TransferInfo>\n{records}</Transfer>"
    )
}

fn record(abn: &str, status: &str, body: &str) -> String {
    format!(
        "<ABR recordLastUpdatedDate=\"20200519\" replaced=\"N\"><ABN status=\"{status}\" ABNStatusFromDate=\"20150729\">{abn}</ABN>{body}</ABR>\n"
    )
}

const NAME: &str = "<MainEntity><NonIndividualName type=\"MN\"><NonIndividualNameText>EXAMPLE &amp; CO FOUNDATION</NonIndividualNameText></NonIndividualName><BusinessAddress><AddressDetails><State>NSW</State><Postcode>2000</Postcode></AddressDetails></BusinessAddress></MainEntity>";

fn run(input: &str) -> Result<Totals> {
    let mut xml = Xml {
        bytes: BufReader::new(input.as_bytes()).bytes(),
        back: None,
    };
    let mut totals = Totals::default();
    read(&mut xml, &mut totals).map(|()| totals)
}

#[test]
fn an_unnamed_fund_is_kept_as_a_fund_and_individuals_are_dropped() {
    let records = [
        record(
            "23000000004",
            "ACT",
            &format!(
                "{NAME}<GST status=\"NON\" GSTStatusFromDate=\"19000101\" /><DGR status=\"ACT\" DGRStatusFromDate=\"20160223\" /><DGR DGRStatusFromDate=\"20150729\"><NonIndividualName type=\"DGR\"><NonIndividualNameText></NonIndividualNameText></NonIndividualName></DGR><DGR DGRStatusFromDate=\"20000701\"><NonIndividualName type=\"DGR\"><NonIndividualNameText>EXAMPLE BUILDING FUND</NonIndividualNameText></NonIndividualName></DGR><OtherEntity><NonIndividualName type=\"TRD\"><NonIndividualNameText>Example &amp; Co Foundation</NonIndividualNameText></NonIndividualName></OtherEntity><OtherEntity><NonIndividualName type=\"BN\"><NonIndividualNameText>EXAMPLE SHOP</NonIndividualNameText></NonIndividualName></OtherEntity>"
            ),
        ),
        record(
            "42000000003",
            "ACT",
            "<LegalEntity><IndividualName type=\"LGL\"><GivenName>JANE</GivenName><FamilyName>CITIZEN</FamilyName></IndividualName></LegalEntity><DGR status=\"ACT\" DGRStatusFromDate=\"20100101\" />",
        ),
        record("61000000002", "CAN", NAME),
    ]
    .concat();
    let totals = run(&file(1, 3, &records)).unwrap();
    assert_eq!(totals.records, 3);
    assert_eq!(totals.individuals_with_dgr, 1);
    assert_eq!(totals.files.get(&1), Some(&3));
    let lines: Vec<String> = totals.lines.values().map(Value::to_string).collect();
    assert_eq!(
        lines,
        [
            r#"["23000000004","ACT","2015-07-29","EXAMPLE & CO FOUNDATION",[["2000-07-01","EXAMPLE BUILDING FUND"],["2015-07-29",""],["2016-02-23",null]],["EXAMPLE SHOP"]]"#
        ]
    );
    assert!(!lines[0].contains("JANE") && !lines[0].contains("NSW"));
}

#[test]
fn a_malformed_record_stops_the_build() {
    let good = good_record();
    stops_on(&[
        (file(1, 2, &good), "declares 2 records but holds 1"),
        (
            file(1, 1, &good.replace("</ABN>", "</ABR>")),
            "closes another element",
        ),
        (
            file(1, 1, &good.replace("&amp;", "&nbsp;")),
            "an unknown entity &nbsp;",
        ),
        (
            file(
                1,
                1,
                &good.replace("status=\"ACT\" DGR", "status=\"REV\" DGR"),
            ),
            "DGR status \"REV\"",
        ),
        (
            file(1, 1, &good.replace("20160223", "2016-02-23")),
            "is not yyyymmdd",
        ),
        (
            file(1, 1, &good.replace("23000000004", "2300000000")),
            "an ABN of",
        ),
        (
            file(
                1,
                1,
                &good.replace("status=\"ACT\" ABN", "status=\"XYZ\" ABN"),
            ),
            "status \"XYZ\"",
        ),
        (
            file(1, 1, &record("23000000004", "ACT", DGR)),
            "no entity name",
        ),
    ]);
}

#[test]
fn malformed_xml_or_files_stop_the_build() {
    let good = good_record();
    stops_on(&[
        (file(1, 1, &format!("<!-- note -->{good}")), "a comment"),
        (file(1, 1, &good)[..200].to_string(), "ends inside"),
        (
            [
                file(1, 1, &good),
                file(1, 1, &good.replace("23000000004", "42000000003")),
            ]
            .concat(),
            "file 1 given twice",
        ),
        (
            [file(1, 1, &good), file(2, 1, &good)].concat(),
            "appears twice",
        ),
        (
            [
                file(1, 1, &good),
                file(2, 0, "").replace("12:20:05", "12:20:06"),
            ]
            .concat(),
            "files from 2 extracts",
        ),
    ]);
}

const DGR: &str = "<DGR status=\"ACT\" DGRStatusFromDate=\"20160223\" />";

/// A record the build keeps, for the cases to break.
fn good_record() -> String {
    record("23000000004", "ACT", &format!("{NAME}{DGR}"))
}

/// Each input stops the build with an error holding its text.
fn stops_on(cases: &[(String, &str)]) {
    for (input, error) in cases {
        let got = run(input).err().unwrap_or_default();
        assert!(got.contains(error), "{error}: got {got:?}");
    }
}
