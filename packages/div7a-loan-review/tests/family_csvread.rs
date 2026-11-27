// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The shared `csvread.rs` reader over a list of lines, which only this crate's
//! copy has (`Reader::from_lines`, behind the benchmark rate table).
use div7a_loan_review::py::PyPath;
use div7a_loan_review::rates;

fn load(dir: &str, text: &str) -> rates::BenchmarkTable {
    let dir = std::env::temp_dir().join(format!("family-csvread-{}-{dir}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("benchmark_rates.csv");
    std::fs::write(&path, text).unwrap();
    let table = rates::load_table(&PyPath::new(&path.to_string_lossy()));
    std::fs::remove_dir_all(&dir).unwrap();
    table.expect("the table loads")
}

/// csv.reader(["a", "", "b"]) gives [["a"], [], ["b"]], and DictReader skips the
/// empty row, so a blank line in the table changes nothing.
#[test]
fn a_blank_line_in_the_rate_table_is_skipped() {
    let shipped = include_str!("../data/benchmark_rates.csv").replace("\r\n", "\n");
    let header = shipped
        .lines()
        .position(|line| !line.starts_with('#'))
        .expect("a header row");
    let mut lines: Vec<&str> = shipped.lines().collect();
    lines.insert(header + 2, "");
    let blank = lines.join("\n");
    assert_ne!(blank, shipped);
    let want = load("plain", &shipped);
    let got = load("blank", &blank);
    assert_eq!(format!("{:?}", got.entries), format!("{:?}", want.entries));
    assert!(!want.entries.is_empty());
}
