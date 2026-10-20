//! Reviewing a loan register: one amalgamated loan per row, for one year of
//! income. A row is SKIPPED where the operator marks it out of scope or the
//! loan was made in a year that may straddle 4 December 1997 (s 109D(5)).

use std::cmp::Ordering;

use serde_json::{Value, json};

use crate::Failure;
use crate::csvread;
use crate::decimal::{Calc, Decimal};
use crate::facts::{optional_year_of_income, text_cell};
use crate::gate::{GateFacts, GateResult, complying_loan_gate};
use crate::money::{cents_str, d};
use crate::myr::{GateSummary, MyrFacts, MyrResult, minimum_yearly_repayment};
use crate::py::{self, PyPath};
use crate::rates::{BenchmarkTable, RateOverride, TableSource, dict_row, frozen_table};
use crate::raw::Raw;
use crate::years::YearOfIncome;

pub const GATE_COLUMNS: [&str; 8] = [
    "loan_id",
    "written_agreement",
    "terms_in_place_before_lodgment_day",
    "maximum_term_years",
    "secured_by_registered_mortgage_over_real_property",
    "security_coverage_at_first_made",
    "interest_rate_for_years_after_year_loan_made",
    "year_loan_made",
];

pub const MYR_COLUMNS: [&str; 3] = [
    "amalgamated_loan_unpaid_at_end_of_previous_year",
    "remaining_term_years",
    "payments_applied_during_the_year",
];

pub const OPTIONAL_COLUMNS: [&str; 3] = [
    "borrower_reference",
    "out_of_scope_reason",
    "year_of_income_being_tested",
];

/// The last year of income that can contain 4 December 1997.
pub const FIRST_REVIEWABLE_YEAR: i64 = 1998;

/// The summary counts a register review reports, in display order.
pub const SUMMARY_KEYS: [&str; 7] = [
    "COMPLYING",
    "NOT_COMPLYING",
    "MYR_MET",
    "MYR_SHORT",
    "UNKNOWN",
    "REFUSED",
    "SKIPPED",
];

#[derive(Clone, Debug)]
pub struct ReviewLine {
    pub row_number: usize,
    pub loan_id: String,
    pub borrower_reference: String,
    pub gate: Option<GateResult>,
    pub myr: Option<MyrResult>,
    pub skipped_reason: String,
}

impl ReviewLine {
    pub fn is_skipped(&self) -> bool {
        !self.skipped_reason.is_empty()
    }

    /// Nil where there is no established shortfall, so the sort is total.
    pub fn shortfall(&self) -> Decimal {
        self.myr
            .as_ref()
            .and_then(|m| m.shortfall.clone())
            .unwrap_or_else(|| d("0.00"))
    }

    pub fn has_exposure(&self) -> bool {
        self.myr.as_ref().is_some_and(|m| m.verdict == "MYR_SHORT")
    }

    fn is_undecided(&self) -> bool {
        let gate = self.gate.as_ref().is_some_and(|g| g.verdict == "UNKNOWN");
        let myr = self
            .myr
            .as_ref()
            .is_some_and(|m| m.verdict == "UNKNOWN" || m.verdict == "REFUSED");
        gate || myr
    }

    pub fn needs_attention(&self) -> bool {
        let not_complying = self
            .gate
            .as_ref()
            .is_some_and(|g| g.verdict == "NOT_COMPLYING");
        self.has_exposure() || self.is_undecided() || not_complying
    }

    fn to_json(&self) -> Calc<Value> {
        Ok(json!({
            "row_number": self.row_number,
            "loan_id": self.loan_id,
            "borrower_reference": self.borrower_reference,
            "status": if self.is_skipped() { "SKIPPED" } else { "REVIEWED" },
            "skipped_reason": (!self.skipped_reason.is_empty()).then_some(&self.skipped_reason),
            "gate": self.gate.as_ref().map(GateResult::to_json),
            "myr": match &self.myr { None => Value::Null, Some(m) => m.to_json()? },
        }))
    }
}

#[derive(Clone, Debug)]
pub struct ReviewReport {
    pub year_of_income: String,
    pub lines: Vec<ReviewLine>,
    pub summary: Vec<(&'static str, usize)>,
    pub rows_reviewed: usize,
    pub sources: Vec<TableSource>,
}

impl ReviewReport {
    pub fn total_exposure(&self) -> Calc<Decimal> {
        self.lines
            .iter()
            .try_fold(d("0.00"), |sum, line| sum.add(&line.shortfall()))
    }

    pub fn needs_attention(&self) -> bool {
        self.lines.iter().any(ReviewLine::needs_attention)
    }

    pub fn to_json(&self) -> Calc<Value> {
        let mut summary = serde_json::Map::new();
        for (key, count) in &self.summary {
            summary.insert((*key).into(), json!(count));
        }
        Ok(json!({
            "year_of_income": self.year_of_income,
            "rows_reviewed": self.rows_reviewed,
            "summary": summary,
            "experimental_total_exposure": cents_str(&self.total_exposure()?)?,
            "manifest": {"rate_table_uris": self.sources.iter().map(TableSource::to_json).collect::<Vec<_>>()},
            "lines": self.lines.iter().map(ReviewLine::to_json).collect::<Calc<Vec<_>>>()?,
            "note": "Counts are per question, not per row: a reviewed row answers both the \
                     s 109N terms question and the s 109E minimum yearly repayment question, \
                     so the counts can sum to more than rows_reviewed.",
        }))
    }
}

/// Refuse a register that does not carry the columns a command needs.
pub fn require_columns(fieldnames: &[String], needed: &[&str], where_: &str) -> Result<(), String> {
    let missing: Vec<&str> = needed
        .iter()
        .copied()
        .filter(|name| !fieldnames.iter().any(|f| f == name))
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{where_} is missing required column(s): {}. The register columns are documented in \
         the README.",
        missing.join(", ")
    ))
}

/// `load_rows(path, needed)`: the register as DictReader rows.
pub fn load_rows(path: &PyPath, needed: &[&str]) -> Result<Vec<Raw>, Failure> {
    let shown = path.display();
    let bytes = std::fs::read(path.fs()).map_err(|e| {
        Failure::Value(format!(
            "cannot read the loan register at {shown}: {}",
            py::oserror_str(&e, &shown, py::OsStyle::Errno)
        ))
    })?;
    let read_error = |e: csvread::ReadError| match e {
        csvread::ReadError::Decode { byte, .. } => Failure::Value(format!(
            "{shown} is not valid UTF-8 text (invalid byte {byte:#04x}). Re-save it as \
             'CSV UTF-8 (Comma delimited)' and run again."
        )),
        csvread::ReadError::Csv(message) => Failure::Value(format!(
            "cannot parse the loan register at {shown}: {message}"
        )),
    };
    let mut reader = csvread::Reader::new(&bytes);
    let header = reader.next_row().map_err(read_error)?.unwrap_or_default();
    require_columns(&header, needed, &shown).map_err(Failure::Value)?;
    let mut rows = Vec::new();
    while let Some(row) = reader.next_row().map_err(read_error)? {
        if row.is_empty() {
            continue;
        }
        rows.push(dict_row(&header, &row));
    }
    if rows.is_empty() {
        return Err(Failure::Value(format!(
            "{shown} carries a header but no loan rows"
        )));
    }
    Ok(rows)
}

fn skip_reason(row: &Raw, where_: &str) -> Result<String, String> {
    let marked = text_cell(row.get("out_of_scope_reason"));
    if !marked.is_empty() {
        return Ok(format!("Operator marked out of scope: {marked}"));
    }
    let made = optional_year_of_income(
        row.get("year_loan_made"),
        &format!("{where_} year_loan_made"),
    )?;
    if let Some(made) = made
        && made.start_year < FIRST_REVIEWABLE_YEAR
    {
        return Ok(format!(
            "Loan made in {}. Division 7A reaches a loan made before 4 December 1997 only \
             where its terms were varied on or after that day (s 109D(5)), and a year label \
             cannot place the loan either side of that date. Out of scope for v1.",
            made.label()
        ));
    }
    Ok(String::new())
}

/// How a review is run: the gate alone, the repayment alone, or both.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mode {
    pub gate_only: bool,
    pub myr_only: bool,
}

/// Review a loan register for one year of income. A `Failure::Value` from a
/// row is the FactError the file-level review wraps with the path.
pub fn review_register(
    rows: &[Raw],
    year: YearOfIncome,
    mode: Mode,
    gate_benchmark_year: Option<YearOfIncome>,
    table: Option<&BenchmarkTable>,
    over: Option<&RateOverride>,
) -> Result<ReviewReport, Failure> {
    if mode.gate_only && mode.myr_only {
        return Err(Failure::Value(
            "gate_only and myr_only cannot both be True".into(),
        ));
    }
    // The table is read at most once, on the first reviewed row.
    let mut merged: Option<BenchmarkTable> = None;
    let mut lines = Vec::new();
    let mut summary: Vec<(&'static str, usize)> = SUMMARY_KEYS.iter().map(|k| (*k, 0)).collect();
    let bump = |summary: &mut Vec<(&'static str, usize)>, key: &str| {
        if let Some(slot) = summary.iter_mut().find(|(k, _)| *k == key) {
            slot.1 += 1;
        }
    };
    let mut reviewed = 0;
    for (index, row) in rows.iter().enumerate() {
        let n = index + 1;
        let where_ = format!("row {n}");
        let loan_id = match text_cell(row.get("loan_id")) {
            id if id.is_empty() => format!("(row {n})"),
            id => id,
        };
        let reference = text_cell(row.get("borrower_reference"));
        let skip = skip_reason(row, &where_).map_err(Failure::Value)?;
        if !skip.is_empty() {
            bump(&mut summary, "SKIPPED");
            lines.push(ReviewLine {
                row_number: n,
                loan_id,
                borrower_reference: reference,
                gate: None,
                myr: None,
                skipped_reason: skip,
            });
            continue;
        }
        reviewed += 1;
        let table = merged.get_or_insert_with(|| {
            let base = table.cloned().unwrap_or_else(frozen_table);
            match over {
                Some(over) => base.with_override(over),
                None => base,
            }
        });
        let mut gate_facts = GateFacts::from_mapping(row, &where_).map_err(Failure::Value)?;
        if gate_benchmark_year.is_some() {
            gate_facts.year_of_income_being_tested = gate_benchmark_year;
        }
        let gate = complying_loan_gate(&gate_facts, Some(table), None)?;
        let myr = if mode.gate_only {
            None
        } else {
            let facts = MyrFacts::from_mapping(row, year, Some(GateSummary::from(&gate)), &where_)
                .map_err(Failure::Value)?;
            Some(minimum_yearly_repayment(&facts, Some(table), None)?)
        };
        if !mode.myr_only {
            match gate.verdict {
                "COMPLYING" => bump(&mut summary, "COMPLYING"),
                "NOT_COMPLYING" => bump(&mut summary, "NOT_COMPLYING"),
                _ => {}
            }
        }
        let mut undecided = gate.verdict == "UNKNOWN" && !mode.myr_only;
        if let Some(myr) = &myr {
            match myr.verdict {
                "MYR_MET" => bump(&mut summary, "MYR_MET"),
                "MYR_SHORT" => bump(&mut summary, "MYR_SHORT"),
                "REFUSED" => bump(&mut summary, "REFUSED"),
                _ => undecided = true,
            }
        }
        if undecided {
            bump(&mut summary, "UNKNOWN");
        }
        lines.push(ReviewLine {
            row_number: n,
            loan_id,
            borrower_reference: reference,
            gate: (!mode.myr_only).then_some(gate),
            myr,
            skipped_reason: String::new(),
        });
    }
    // Exposure first, largest shortfall first; then work; then clean rows;
    // then skipped rows. Ties keep the register's order.
    let band = |line: &ReviewLine| {
        if line.is_skipped() {
            3
        } else if line.has_exposure() {
            0
        } else if line.needs_attention() {
            1
        } else {
            2
        }
    };
    let mut failure = None;
    lines.sort_by(|a, b| {
        band(a).cmp(&band(b)).then_with(|| {
            match b.shortfall().compare(&a.shortfall()) {
                Ok(order) => order,
                Err(signal) => {
                    failure = Some(signal);
                    Ordering::Equal
                }
            }
            .then(a.row_number.cmp(&b.row_number))
        })
    });
    if let Some(signal) = failure {
        return Err(Failure::Signal(signal));
    }
    Ok(ReviewReport {
        year_of_income: year.label(),
        lines,
        summary,
        rows_reviewed: reviewed,
        // A caller's table names its sources even when every row was skipped.
        sources: match (merged, table) {
            (Some(merged), _) => merged.sources,
            (None, Some(table)) => table.sources.clone(),
            (None, None) => Vec::new(),
        },
    })
}

/// `review_register_file`: read the register and review it; a row's fact
/// error is reported with the path as given.
pub fn review_register_file(
    input: &str,
    year: YearOfIncome,
    mode: Mode,
    gate_benchmark_year: Option<YearOfIncome>,
    table: Option<&BenchmarkTable>,
    over: Option<&RateOverride>,
) -> Result<ReviewReport, Failure> {
    if mode.gate_only && mode.myr_only {
        return Err(Failure::Value(
            "gate_only and myr_only cannot both be True".into(),
        ));
    }
    let mut needed: Vec<&str> = GATE_COLUMNS.to_vec();
    if !mode.gate_only {
        needed.extend(MYR_COLUMNS);
    }
    let rows = load_rows(&PyPath::new(input), &needed)?;
    review_register(&rows, year, mode, gate_benchmark_year, table, over).map_err(|e| match e {
        Failure::Value(message) => Failure::Value(format!("{input}: {message}")),
        other => other,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_review_mode_is_checked_before_any_row_or_table() {
        let both = Mode {
            gate_only: true,
            myr_only: true,
        };
        let year = YearOfIncome::new(2026).unwrap();
        let refused = review_register(&[Raw::None], year, both, None, None, None).unwrap_err();
        assert_eq!(
            refused,
            Failure::Value("gate_only and myr_only cannot both be True".into())
        );
        let file = review_register_file("missing.csv", year, both, None, None, None).unwrap_err();
        assert_eq!(file, refused);
    }
}
