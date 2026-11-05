// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `usi-check`: read USIs against an operator-supplied copy of the daily USI
//! extract from Super Fund Lookup (`SflUsiExtract.txt`).
//!
//! The extract is a public, fixed-width file. Nothing here downloads, bundles
//! or republishes it, and it changes no deadline or verdict. `LISTED` means only
//! that the extract shows the USI in date with no contribution restrictions;
//! anything else is `UNVERIFIED` with a reason, never a pass. SMSFs have no USI
//! and the extract covers APRA funds, so an SMSF cannot be `LISTED`.

use std::collections::BTreeMap;

use crate::csv_io;
use crate::csvread::{ReadError, Reader};
use crate::csvwrite;
use crate::py::Date;
use crate::pytext;
use crate::{Failure, Result};

pub const LISTED: &str = "LISTED";
pub const UNVERIFIED: &str = "UNVERIFIED";

/// The longest gap, in days, between a daily extract and `--as-at` by default.
/// An operational safeguard: the source page says only that it is generated daily.
pub const DEFAULT_MAX_AGE_DAYS: i64 = 2;

/// The header's columns at their 0-based byte offsets, from the layout at
/// superfundlookup.gov.au/Tools/DownloadUsiList. A header that moved means the
/// layout moved, so every offset below is wrong: the extract is unreadable.
const HEADER: [(usize, &str); 7] = [
    (0, "ABN"),
    (12, "FundName"),
    (213, "USI"),
    (234, "ProductName"),
    (435, "ContributionRestrictions"),
    (460, "From"),
    (471, "To"),
];
const ROW_BYTES: usize = 481;
const RUN_AT: &str = "This Report Run at:";
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// One product line of the extract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Product {
    restricted: bool,
    from: i64,
    to: i64,
}

/// A readable, dated extract.
#[derive(Debug, PartialEq, Eq)]
pub struct Extract {
    pub run_on: Date,
    products: BTreeMap<String, Vec<Product>>,
}

fn text(bytes: &[u8]) -> Option<&str> {
    std::str::from_utf8(bytes).ok().map(str::trim)
}

fn date(bytes: &[u8]) -> Option<i64> {
    Date::fromisoformat(text(bytes)?).map(|day| day.ordinal())
}

/// One data row, or None when any field it needs cannot be read.
fn product(line: &[u8]) -> Option<(String, Product)> {
    let usi = text(&line[213..234])?;
    let restricted = match text(&line[435..460])? {
        "Y" => true,
        "N" => false,
        _ => return None,
    };
    let (from, to) = (date(&line[460..470])?, date(&line[471..481])?);
    (!usi.is_empty() && from <= to).then(|| {
        (
            usi.to_string(),
            Product {
                restricted,
                from,
                to,
            },
        )
    })
}

/// `This Report Run at:  Nov  5 2026  7:46AM`, the only date in the file.
fn run_date(line: &str) -> Option<Date> {
    let mut words = line.strip_prefix(RUN_AT)?.split_whitespace();
    let month = words.next()?;
    let month = MONTHS.iter().position(|m| *m == month)? as i64 + 1;
    let day = words.next()?.parse().ok()?;
    let year = words.next()?.parse().ok()?;
    Date::new(year, month, day).ok()
}

fn rows_affected(line: &str) -> Option<usize> {
    line.strip_prefix('(')?
        .strip_suffix(" rows affected)")?
        .parse()
        .ok()
}

/// Read the extract, or name why it cannot be used: `extract_unreadable` for a
/// layout, row or row count that does not hold, `extract_undated` for no run date.
pub fn parse_extract(data: &[u8]) -> std::result::Result<Extract, &'static str> {
    let unreadable = "extract_unreadable";
    let mut lines = data
        .split(|byte| *byte == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line));
    let header = lines.next().ok_or(unreadable)?;
    let rule = lines.next().ok_or(unreadable)?;
    let columns_match = header.len() == ROW_BYTES
        && HEADER.iter().all(|(at, name)| {
            header[*at..].starts_with(name.as_bytes())
                && (*at == 0 || header[at - 1] == b' ')
                && header.get(at + name.len()).is_none_or(|b| *b == b' ')
        });
    if !columns_match || rule.len() != ROW_BYTES {
        return Err(unreadable);
    }
    let mut products: BTreeMap<String, Vec<Product>> = BTreeMap::new();
    let mut count = 0;
    let mut footer = Vec::new();
    let mut past_rows = false;
    for line in lines {
        if !past_rows && line.len() == ROW_BYTES {
            let (usi, product) = product(line).ok_or(unreadable)?;
            products.entry(usi).or_default().push(product);
            count += 1;
            continue;
        }
        past_rows = true;
        let line = text(line).ok_or(unreadable)?;
        if !line.is_empty() {
            footer.push(line);
        }
    }
    // One row count, one run date and nothing after them. Anything else could
    // let a later row or value stand in for the real one, and a missing or short
    // count means the copy was cut.
    let (rows, run) = match footer[..] {
        [rows] => (rows, None),
        [rows, run] => (rows, Some(run)),
        _ => return Err(unreadable),
    };
    if rows_affected(rows) != Some(count) {
        return Err(unreadable);
    }
    let run = run.ok_or("extract_undated")?;
    if !run.starts_with(RUN_AT) {
        return Err(unreadable);
    }
    Ok(Extract {
        run_on: run_date(run).ok_or("extract_undated")?,
        products,
    })
}

impl Extract {
    /// Why this extract cannot speak for `as_at`, if it cannot.
    fn timing_problem(&self, as_at: Date, max_age_days: i64) -> Option<&'static str> {
        let age = as_at.ordinal() - self.run_on.ordinal();
        if age < 0 {
            Some("extract_after_as_at")
        } else if age > max_age_days {
            Some("extract_stale")
        } else {
            None
        }
    }

    /// The reason a USI is not `LISTED`, or None when it is. Rows for one USI
    /// that disagree on the day are read the cautious way.
    fn reason(&self, usi: &str, as_at: i64) -> Option<&'static str> {
        let Some(rows) = self.products.get(usi) else {
            return Some("not_listed");
        };
        let in_date: Vec<&Product> = rows
            .iter()
            .filter(|p| p.from <= as_at && as_at <= p.to)
            .collect();
        if in_date.iter().any(|p| p.restricted) {
            Some("restricted")
        } else if !in_date.is_empty() {
            None
        } else if rows.iter().any(|p| p.to < as_at) {
            Some("end_dated")
        } else {
            Some("not_yet_effective")
        }
    }
}

/// One input row's result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// The file row, counting the header as 1.
    pub row: i64,
    pub usi: String,
    pub status: &'static str,
    pub reason: &'static str,
}

/// The USI in each data row of a CSV with one `usi` column. A row of another
/// width could hide a USI, and a file with no rows checks nothing, so both refuse.
pub fn read_usis(path: &str, data: &[u8]) -> Result<Vec<(i64, String)>> {
    let body = data.strip_prefix(b"\xef\xbb\xbf").unwrap_or(data);
    let failed = |e: ReadError| match e {
        ReadError::Decode { byte, start, .. } => csv_io::not_utf8(path, byte, start),
        ReadError::Csv(message) => csv_io::csv_module_error(&message),
    };
    let mut reader = Reader::new(body);
    let Some(header) = reader.next_row().map_err(failed)? else {
        return Err(Failure::Value(format!("{path} has no header row")));
    };
    let wanted: Vec<usize> = (0..header.len())
        .filter(|i| pytext::strip(&header[*i]).eq_ignore_ascii_case("usi"))
        .collect();
    let [column] = wanted[..] else {
        return Err(Failure::Value(format!(
            "{path} needs exactly one usi column, found {}",
            wanted.len()
        )));
    };
    let mut usis = Vec::new();
    let mut row = 1;
    while let Some(cells) = reader.next_row().map_err(failed)? {
        row += 1;
        // The csv module's blank line is no row at all.
        if cells.is_empty() {
            continue;
        }
        if cells.len() != header.len() {
            return Err(Failure::Value(format!(
                "{path} row {row} has {} fields, but the header has {}",
                cells.len(),
                header.len()
            )));
        }
        usis.push((row, pytext::strip(&cells[column]).to_string()));
    }
    if usis.is_empty() {
        return Err(Failure::Value(format!("{path} has no data rows")));
    }
    Ok(usis)
}

/// Check each USI. A problem with the extract itself leaves every row
/// `UNVERIFIED` with that reason, so no row can pass on evidence that is not there.
pub fn check(
    usis: &[(i64, String)],
    extract: std::result::Result<&Extract, &'static str>,
    as_at: Date,
    max_age_days: i64,
) -> Vec<Finding> {
    let overall = match extract {
        Ok(extract) => extract.timing_problem(as_at, max_age_days),
        Err(problem) => Some(problem),
    };
    usis.iter()
        .map(|(row, usi)| {
            let reason = match (overall, extract) {
                (Some(problem), _) | (None, Err(problem)) => Some(problem),
                _ if usi.is_empty() => Some("usi_blank"),
                (None, Ok(extract)) => extract.reason(usi, as_at.ordinal()),
            };
            Finding {
                row: *row,
                usi: usi.clone(),
                status: reason.map_or(LISTED, |_| UNVERIFIED),
                reason: reason.unwrap_or(""),
            }
        })
        .collect()
}

/// The report CSV. Cells keep the spreadsheet formula guard of every other report.
pub fn report(findings: &[Finding], extract_run_on: Option<Date>) -> String {
    let mut out = String::new();
    let header = ["row", "usi", "status", "reason", "extract_run_on"];
    csvwrite::row(&header.map(str::to_string), &mut out);
    let run_on = extract_run_on.map_or(String::new(), |day| day.isoformat());
    for finding in findings {
        csvwrite::row(
            &[
                finding.row.to_string(),
                csv_io::csv_safe(&finding.usi),
                finding.status.to_string(),
                finding.reason.to_string(),
                run_on.clone(),
            ],
            &mut out,
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(usi: &str, restricted: &str, from: &str, to: &str) -> String {
        let fund = "Fabricated Super Fund";
        let product = "Fabricated Product";
        format!(
            "{:<11} {fund:<200} {usi:<20} {product:<200} {restricted:<24} {from:<10} {to:<10}",
            "00000000000"
        )
    }

    fn header() -> String {
        format!(
            "{:<11} {:<200} {:<20} {:<200} {:<24} {:<10} {:<10}\r\n{} {} {} {} {} {} {}\r\n",
            "ABN",
            "FundName",
            "USI",
            "ProductName",
            "ContributionRestrictions",
            "From",
            "To",
            "-".repeat(11),
            "-".repeat(200),
            "-".repeat(20),
            "-".repeat(200),
            "-".repeat(24),
            "-".repeat(10),
            "-".repeat(10),
        )
    }

    fn extract(rows: &[String], tail: &str) -> Vec<u8> {
        let mut out = header();
        for row in rows {
            out.push_str(row);
            out.push_str("\r\n");
        }
        out.push_str(tail);
        out.into_bytes()
    }

    const DATED: &str =
        "\r\n(4 rows affected)\r\nThis Report Run at:               Nov  5 2026  7:46AM\r\n \r\n";

    fn sample() -> Vec<u8> {
        extract(
            &[
                line("FAB0001", "N", "2013-07-01", "9999-12-31"),
                line("FAB0002", "Y", "2013-07-01", "9999-12-31"),
                line("FAB0003", "N", "2013-07-01", "2026-06-30"),
                line("FAB0004", "N", "2027-07-01", "9999-12-31"),
            ],
            DATED,
        )
    }

    fn day(text: &str) -> Date {
        Date::fromisoformat(text).unwrap()
    }

    fn status(data: &[u8], usi: &str, as_at: &str) -> (&'static str, &'static str) {
        let parsed = parse_extract(data);
        let finding = check(
            &[(2, usi.to_string())],
            parsed.as_ref().map_err(|e| *e),
            day(as_at),
            DEFAULT_MAX_AGE_DAYS,
        );
        (finding[0].status, finding[0].reason)
    }

    #[test]
    fn every_row_outcome_has_its_own_reason() {
        let data = sample();
        let cases = [
            ("FAB0001", (LISTED, "")),
            ("FAB0002", (UNVERIFIED, "restricted")),
            ("FAB0003", (UNVERIFIED, "end_dated")),
            ("FAB0004", (UNVERIFIED, "not_yet_effective")),
            ("FAB0005", (UNVERIFIED, "not_listed")),
            ("", (UNVERIFIED, "usi_blank")),
        ];
        for (usi, expected) in cases {
            assert_eq!(status(&data, usi, "2026-11-05"), expected, "{usi}");
        }
    }

    #[test]
    fn the_run_date_comes_from_the_footer() {
        assert_eq!(parse_extract(&sample()).unwrap().run_on, day("2026-11-05"));
    }

    #[test]
    fn an_extract_older_than_the_limit_is_stale_at_the_day_after_it() {
        let data = sample();
        assert_eq!(status(&data, "FAB0001", "2026-11-07"), (LISTED, ""));
        assert_eq!(
            status(&data, "FAB0001", "2026-11-08"),
            (UNVERIFIED, "extract_stale")
        );
    }

    #[test]
    fn an_extract_from_after_the_as_at_date_cannot_speak_for_it() {
        assert_eq!(
            status(&sample(), "FAB0001", "2026-11-04"),
            (UNVERIFIED, "extract_after_as_at")
        );
    }

    #[test]
    fn dates_bound_a_listing_inclusively() {
        let data = extract(
            &[line("FAB0001", "N", "2026-11-05", "2026-11-05")],
            &DATED.replace('4', "1"),
        );
        assert_eq!(status(&data, "FAB0001", "2026-11-05"), (LISTED, ""));
        assert_eq!(
            status(&data, "FAB0001", "2026-11-06"),
            (UNVERIFIED, "end_dated")
        );
    }

    #[test]
    fn rows_that_disagree_on_the_day_read_as_restricted() {
        let data = extract(
            &[
                line("FAB0001", "N", "2013-07-01", "9999-12-31"),
                line("FAB0001", "Y", "2026-11-06", "9999-12-31"),
            ],
            &DATED.replace('4', "2"),
        );
        assert_eq!(
            status(&data, "FAB0001", "2026-11-07"),
            (UNVERIFIED, "restricted")
        );
        assert_eq!(status(&data, "FAB0001", "2026-11-05"), (LISTED, ""));
    }

    #[test]
    fn an_end_dated_usi_with_a_later_row_stays_end_dated_in_the_gap() {
        let data = extract(
            &[
                line("FAB0001", "N", "2013-07-01", "2026-06-30"),
                line("FAB0001", "N", "2027-07-01", "9999-12-31"),
            ],
            &DATED.replace('4', "2"),
        );
        assert_eq!(
            status(&data, "FAB0001", "2026-11-05"),
            (UNVERIFIED, "end_dated")
        );
    }

    #[test]
    fn an_extract_with_no_run_date_is_undated() {
        let data = extract(
            &[line("FAB0001", "N", "2013-07-01", "9999-12-31")],
            "\r\n(1 rows affected)\r\n",
        );
        assert_eq!(parse_extract(&data), Err("extract_undated"));
        let bad_date = extract(
            &[line("FAB0001", "N", "2013-07-01", "9999-12-31")],
            "\r\n(1 rows affected)\r\nThis Report Run at: Smarch 5 2026\r\n",
        );
        assert_eq!(parse_extract(&bad_date), Err("extract_undated"));
    }

    #[test]
    fn a_damaged_extract_is_unreadable() {
        let good = line("FAB0001", "N", "2013-07-01", "9999-12-31");
        let tail = DATED.replace('4', "1");
        let moved = String::from_utf8(sample())
            .unwrap()
            .replacen("USI", " USI", 1);
        let cases: Vec<Vec<u8>> = vec![
            Vec::new(),
            b"not an extract\n".to_vec(),
            moved.into_bytes(),
            // A row count that disagrees: the copy was cut.
            extract(std::slice::from_ref(&good), DATED),
            extract(std::slice::from_ref(&good), ""),
            extract(&[line("FAB0001", "?", "2013-07-01", "9999-12-31")], &tail),
            extract(&[line("FAB0001", "N", "01/07/2013", "9999-12-31")], &tail),
            extract(&[line("FAB0001", "N", "2026-07-01", "2013-07-01")], &tail),
            extract(&[line("", "N", "2013-07-01", "9999-12-31")], &tail),
            extract(&[good[..200].to_string()], &tail),
        ];
        for (i, data) in cases.iter().enumerate() {
            assert_eq!(parse_extract(data), Err("extract_unreadable"), "case {i}");
        }
    }

    #[test]
    fn an_unusable_extract_leaves_every_row_unverified() {
        let usis = [(2, "FAB0001".to_string()), (3, String::new())];
        for problem in ["extract_unreadable", "extract_undated"] {
            let findings = check(&usis, Err(problem), day("2026-11-05"), 2);
            assert!(findings.iter().all(|f| f.status == UNVERIFIED));
            assert!(findings.iter().all(|f| f.reason == problem));
        }
    }

    #[test]
    fn input_rows_keep_their_file_row_and_a_usi_column_is_required() {
        let usis = read_usis("in.csv", b"\xef\xbb\xbfemployee, USI \n1,FAB0001\n\n2,\n").unwrap();
        assert_eq!(usis, [(2, "FAB0001".to_string()), (4, String::new())]);
        for data in [&b"employee\n1\n"[..], b"usi,USI\n1,2\n", b""] {
            assert!(read_usis("in.csv", data).is_err());
        }
    }

    #[test]
    fn the_report_guards_a_formula_in_a_usi() {
        let finding = Finding {
            row: 2,
            usi: "=1+1".to_string(),
            status: UNVERIFIED,
            reason: "not_listed",
        };
        let text = report(&[finding], Some(day("2026-11-05")));
        assert!(
            text.contains("2,'=1+1,UNVERIFIED,not_listed,2026-11-05"),
            "{text}"
        );
    }
}
