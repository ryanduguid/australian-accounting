// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Point-in-time legislation indexes supplied by the operator. No downloads.
mod dated;
mod define;
mod query;
mod rank;
mod read;
mod rows;
mod search;

use super::Context;
use serde_json::Value;

#[cfg(test)]
pub(in crate::retrieval) use define::definitions;
pub(in crate::retrieval) use rank::{Ranked, retain, tier};
pub(in crate::retrieval) use rows::{census, holds, joined};

pub(super) fn evaluate(context: &Context, name: &str, input: &Value) -> Result<Value, String> {
    match name {
        "search_tax_legislation" => search::search(context, input, false),
        "search_tax_rates" => search::search(context, input, true),
        "read_tax_legislation_section" => read::read(context, input),
        "define_tax_term" => define::define(context, input),
        _ => unreachable!("validated corpus tool"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::as_at;
    use serde_json::json;

    /// Python's test_a_title_changed_after_validation_cannot_return_uncaptured_text rewrites
    /// a title between classifying it and reading its rows, which a frozen replay cannot do.
    /// The guard it proves is here: a row read for a date must agree with the first row its
    /// title was classified from.
    #[test]
    fn a_row_changed_after_validation_is_refused_not_returned() {
        let on = "2098-08-01";
        let changed = json!({"register_id": "C9999A10001", "compilation_date": "2098-12-01",
            "retrieved": "2099-01-01", "version_is_current": true, "superseded_from": null,
            "text": "The synthetic levy applies to a dated receipt."});
        let mut held = changed.clone();
        held["compilation_date"] = json!("2098-07-01");
        let first = (
            json!({"register_id": "C9999A10001"}),
            as_at::standing(&held, on).unwrap(),
        );
        assert_eq!(
            super::dated::agreeing(&changed, on, &first).unwrap_err(),
            "Rows of C9999A10001 disagree about their compilation window; the title's index is broken: tell the user, and do not retry."
        );
        assert_eq!(
            super::dated::agreeing(&held, on, &first).unwrap()["status"],
            "captured"
        );
    }

    /// Python's test_a_read_failure_while_counting_refusals_stops_the_search replaces the
    /// row reader with one that fails, which a frozen replay cannot do. A title that
    /// cannot be read while its withheld rows are counted stops the search.
    #[test]
    fn a_read_failure_while_counting_refusals_stops_the_search() {
        let context = super::Context {
            root: None,
            limits: Default::default(),
            denied: false,
            principal_acts: None,
            fault: None,
        };
        let gone = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("native-mcp-unreadable-{}", std::process::id()))
            .join("sections.jsonl");
        assert!(!gone.exists());
        let query = super::query::Query::new(&["synthetic".into(), "levy".into()]);
        let first = (json!({}), json!({}));
        assert_eq!(
            super::dated::withheld_counts(
                &context,
                &gone,
                &query,
                &[],
                &|_| true,
                "2098-01-01",
                &first
            )
            .unwrap_err(),
            super::rows::READ_ERROR
        );
    }
}
