// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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
