// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A caller's benchmark rate table read from a file: blank lines, the source
//! it names and a file that is not UTF-8. All rates are fabricated.

use div7a_loan_review::Failure;
use div7a_loan_review::py::PyPath;
use div7a_loan_review::rates::{self, TableSource};

/// A fresh table file holding `data`, named for this process and test.
fn table_file(name: &str, data: &[u8]) -> PyPath {
    let dir = std::env::temp_dir().join(format!("div7a-table-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scratch directory");
    let path = dir.join("rates.csv");
    std::fs::write(&path, data).expect("scratch file");
    PyPath::new(&path.to_string_lossy())
}

const HEAD: &str = "# reviewed_until: 2026-27\n# reviewed_on: 2026-07-01\n\
year_of_income,rate,rba_table,rba_series,rba_month,source,seen,\
primary_url,retrieved_on,snapshot_sha256\n";

/// One rate row for `year` citing the snapshot `url`, `on` and `sha`.
fn row(year: &str, url: &str, on: &str, sha: &str) -> String {
    format!("{year},0.05,F5,S,2025-05,RBA,2025-06-01,{url},{on},{sha}\n")
}

fn source(name: &str, rows: &str) -> TableSource {
    let table =
        rates::load_table(&table_file(name, format!("{HEAD}{rows}").as_bytes())).expect("a table");
    table.sources.into_iter().next().expect("one source")
}

fn cited(source: &TableSource) -> (&str, &str, &str) {
    (
        source.primary_url.as_str(),
        source.retrieved_on.as_str(),
        source.snapshot_sha256.as_str(),
    )
}

const URL: &str = "https://example.test/f5";

#[test]
fn rows_citing_one_snapshot_name_it_as_the_tables_source() {
    // The blank line between the rows is skipped, as csv.reader's empty row is.
    let rows = format!(
        "{}\n{}",
        row("2025-26", URL, "2026-07-01", "aa"),
        row("2026-27", URL, "2026-07-01", "aa")
    );
    assert_eq!(cited(&source("one", &rows)), (URL, "2026-07-01", "aa"));
}

#[test]
fn rows_citing_different_snapshots_leave_the_source_unnamed() {
    let rows = format!(
        "{}{}",
        row("2025-26", URL, "2026-07-01", "aa"),
        row("2026-27", URL, "2026-07-02", "bb")
    );
    assert_eq!(cited(&source("mixed", &rows)), ("", "", ""));
}

#[test]
fn a_table_that_is_not_utf8_raises_python_s_decode_error() {
    let mut data = HEAD.as_bytes().to_vec();
    data.extend(b"2025-26,0.05,\xff\n");
    let at = data.len() - 2;
    assert_eq!(
        rates::load_table(&table_file("bad-byte", &data)).unwrap_err(),
        Failure::Uncaught(format!(
            "UnicodeDecodeError: 'utf-8' codec can't decode byte 0xff in position {at}: \
             invalid start byte"
        ))
    );
}
