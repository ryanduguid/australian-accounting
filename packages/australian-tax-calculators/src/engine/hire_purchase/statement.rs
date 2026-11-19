// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! A schedule or statement as a Markdown table, ready to print.

/// `rows` under `headers`, one line a row, with amounts right-aligned.
/// `right` marks the columns that hold amounts.
pub(super) fn markdown(headers: &[&str], right: &[bool], rows: &[Vec<String>]) -> String {
    let line = |cells: &[String]| format!("| {} |\n", cells.join(" | "));
    let rule: Vec<String> = right
        .iter()
        .map(|&r| if r { "---:".into() } else { "---".into() })
        .collect();
    let mut out = line(&headers.iter().map(|h| h.to_string()).collect::<Vec<_>>());
    out.push_str(&line(&rule));
    for row in rows {
        out.push_str(&line(row));
    }
    out
}
