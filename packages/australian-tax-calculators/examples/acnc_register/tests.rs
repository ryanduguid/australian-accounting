// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The dataset reader's tests, on fabricated files in its column layout.
use super::*;

/// A header with the kept columns, one dropped column and the subtypes.
fn header_line() -> String {
    let mut columns = vec![
        "ABN",
        "Charity_Legal_Name",
        "Charity_Website",
        "Registration_Date",
    ];
    columns.extend(SUBTYPE_COLUMNS);
    columns.join(",")
}

/// A row: ABN, name, website, date, then 14 subtype cells.
fn row(abn: &str, name: &str, registered: &str, subtypes: &[usize]) -> String {
    let cells: Vec<&str> = (1..=14)
        .map(|i| if subtypes.contains(&i) { "Y" } else { "" })
        .collect();
    format!(
        "{abn},{name},https://example.invalid,{registered},{}",
        cells.join(",")
    )
}

fn file(rows: &[String]) -> String {
    format!("\u{feff}{}\r\n{}\r\n", header_line(), rows.join("\r\n"))
}

#[test]
fn keeps_the_named_columns_sorted_and_drops_a_withheld_abn() {
    let text = file(&[
        row(
            "61000000002",
            "\"Example, Charity \"\"Two\"\"\"",
            "03/12/2012",
            &[1, 14],
        ),
        row("", "Withheld Fund", "11/05/2021", &[]),
        row("23000000004", "Example Charity One", "", &[2]),
        row("23000000005", "Example Group", "", &[]),
    ]);
    let read = rows(&text).unwrap();
    assert_eq!((read.withheld, read.invalid, read.read), (1, 1, 4));
    assert_eq!(
        read.kept,
        vec![
            (
                "23000000004".into(),
                "Example Charity One".into(),
                None,
                vec![2]
            ),
            (
                "61000000002".into(),
                "Example, Charity \"Two\"".into(),
                Some("2012-12-03".into()),
                vec![1, 14]
            ),
        ]
    );
}

#[test]
fn keeps_a_line_break_inside_quotes_and_a_leap_day() {
    let text = file(&[row(
        "23000000004",
        "\"Line one\r\nline two\"",
        "29/02/2024",
        &[],
    )]);
    let read = rows(&text).unwrap();
    assert_eq!(read.kept[0].1, "Line one\nline two");
    assert_eq!(read.kept[0].2.as_deref(), Some("2024-02-29"));
    assert_eq!(read.read, 1);
}

fn assert_stops(bad: Vec<(String, &str)>) {
    for (text, why) in bad {
        let error = rows(&text).unwrap_err();
        assert!(error.contains(why), "{why}: {error}");
    }
}

#[test]
fn stops_on_a_value_it_cannot_read() {
    assert_stops(vec![
        (
            file(&[row("23000000004", "A", "2012-12-03", &[])]),
            "DD/MM/YYYY",
        ),
        (
            file(&[
                row("23000000004", "A", "", &[]).replace(",Y", ",N"),
                row("42000000003", "B", "", &[3]).replace(",Y", ",N"),
            ]),
            "not Y or blank",
        ),
        (
            file(&[
                row("23000000004", "A", "", &[]),
                row("23000000004", "B", "", &[]),
            ]),
            "appears twice",
        ),
        (
            file(&[row("23000000004", "A", "31/02/2026", &[])]),
            "DD/MM/YYYY",
        ),
        // A row dropped for its ABN still stops the build when malformed.
        (file(&[row("", "A", "2012-12-03", &[])]), "DD/MM/YYYY"),
        (
            file(&[row("42000000004", "A", "", &[2]).replace(",Y", ",N")]),
            "not Y or blank",
        ),
    ]);
}

#[test]
fn stops_on_a_layout_it_cannot_read() {
    assert_stops(vec![
        (
            file(&[row("23000000004", "\"A", "", &[])]),
            "unclosed quote",
        ),
        (
            file(&[format!("{},extra", row("23000000004", "A", "", &[]))]),
            "fields, not",
        ),
        (file(&[]).replace("PBI", "Other"), "no PBI column"),
        (
            file(&[row("23000000004", "\"A\"tail", "", &[])]),
            "after a closing quote",
        ),
        (
            file(&[row("23000000004", "A\"B\"", "", &[])]),
            "inside an unquoted field",
        ),
    ]);
}

#[test]
fn the_header_records_the_copy_and_its_licence() {
    let read = Counts {
        kept: vec![("23000000004".into(), "A".into(), None, vec![])],
        withheld: 1,
        invalid: 1,
        read: 3,
    };
    let first = header("2026-11-22T17:39:55", &"a".repeat(64), &read);
    assert_eq!(first["snapshot_date"], "2026-11-22");
    assert_eq!(
        first["licence_url"],
        "https://creativecommons.org/licenses/by/3.0/au/"
    );
    assert_eq!(first["charities"], 1);
    assert_eq!(first["abn_invalid_dropped"], 1);
    let text = first.to_string().to_lowercase();
    assert!(!text.contains("endorse") && !text.contains("affiliat"));
}
