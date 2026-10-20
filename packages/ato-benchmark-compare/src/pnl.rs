//! Read a profit and loss export into account and amount rows: the neutral
//! `account,amount` CSV, or a report style export on a best effort basis,
//! where subtotal rows are marked rather than dropped.

use std::sync::LazyLock;

use crate::Failure;
use crate::csvread;
use crate::decimal::Decimal;
use crate::money::parse_amount;
use crate::py::{self, PyPath};
use crate::pyre::Regex;
use crate::pytext;
use crate::unicode::casefold;
use crate::unitables::DIGIT_NOT_DECIMAL;

pub const SECTION_INCOME: &str = "income";
pub const SECTION_COST_OF_SALES: &str = "cost_of_sales";
pub const SECTION_EXPENSE: &str = "expense";

static SECTIONS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    [
        (
            r"^(less\s+)?cost of (sales|goods sold)$",
            SECTION_COST_OF_SALES,
        ),
        (r"^(trading|operating|other)?\s*income$", SECTION_INCOME),
        (r"^revenue$", SECTION_INCOME),
        (r"^(less\s+)?(operating\s+)?expenses$", SECTION_EXPENSE),
        (r"^(less\s+)?administration expenses$", SECTION_EXPENSE),
        (r"^overheads$", SECTION_EXPENSE),
    ]
    .into_iter()
    .map(|(p, s)| (Regex::ignore_case(p), s))
    .collect()
});

static TOTALS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"^total\b",
        r"^gross (profit|loss)",
        r"^net (profit|loss|income)",
        r"^operating (profit|loss)",
        r"^(profit|loss) (before|after) (income )?tax",
        r"^earnings before",
    ]
    .into_iter()
    .map(Regex::ignore_case)
    .collect()
});

/// Headings of numeric columns that label a row rather than hold amounts.
static LABEL_HEADING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::ignore_case(
        r"^((account|invoice)\s+)?(code|codes|note|notes|ref|reference|no\.?|number|id|gl code)$|^invoices?$",
    )
});
/// Cells that stand in for an amount: dashes, n/a, nil, TBC, spreadsheet errors.
static PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::ignore_case(r"^[-–—.\s]*$|^(n/?a|nil|tbc|tba|#n/a)$|^#[a-z0-9/_]+[!?]$")
});

#[derive(Clone, Debug)]
pub struct PnlRow {
    pub account: String,
    pub amount: Decimal,
    pub line_number: usize,
    pub section: Option<String>,
    pub is_total: bool,
}

#[derive(Clone, Debug)]
pub struct PnlFile {
    pub rows: Vec<PnlRow>,
    pub layout: &'static str,
    pub amount_column: String,
    pub skipped: Vec<String>,
}

fn fail<T>(message: impl Into<String>) -> Result<T, Failure> {
    Err(Failure::Value(message.into()))
}

pub fn is_total_row(label: &str) -> bool {
    let stripped = pytext::strip(label);
    TOTALS.iter().any(|p| p.matches_start(stripped))
}

pub fn section_for(label: &str) -> Option<&'static str> {
    let stripped = pytext::strip(label).trim_end_matches(':');
    SECTIONS
        .iter()
        .find(|(p, _)| p.matches_start(stripped))
        .map(|(_, s)| *s)
}

fn strip(cell: &str) -> &str {
    pytext::strip(cell)
}

/// Decode `utf-8-sig`, reporting a bad byte's position in the file.
pub fn decode_sig(raw: &[u8], shown: &str, what: &str) -> Result<String, Failure> {
    let bom = raw.starts_with(b"\xef\xbb\xbf");
    let body = if bom { &raw[3..] } else { raw };
    py::decode_utf8(body).map_err(|e| {
        let offset = e.start + if bom { 3 } else { 0 };
        Failure::Value(format!(
            "{shown}: not valid UTF-8 text at byte {offset}. Re-save {what} as CSV UTF-8. A \
             Windows export is usually cp1252, which this tool cannot read."
        ))
    })
}

/// `list(csv.reader(io.StringIO(text, newline="")))`.
pub fn csv_rows(text: &str) -> Result<Vec<Vec<String>>, Failure> {
    let mut reader = csvread::Reader::new(text.as_bytes());
    let mut rows = Vec::new();
    loop {
        match reader.next_row() {
            Ok(Some(row)) => rows.push(row),
            Ok(None) => return Ok(rows),
            Err(csvread::ReadError::Csv(message)) => {
                return Err(Failure::Uncaught(format!("_csv.Error: {message}")));
            }
            Err(csvread::ReadError::Decode { .. }) => unreachable!("decoded text"),
        }
    }
}

/// `read(path, amount_column)`.
pub fn read(path: &PyPath, amount_column: Option<&str>) -> Result<PnlFile, Failure> {
    let shown = path.display();
    if !path.fs().is_file() {
        return fail(format!("profit and loss file not found: {shown}"));
    }
    let raw = std::fs::read(path.fs())
        .map_err(|e| Failure::Value(py::oserror_str(&e, &shown, py::OsStyle::Errno)))?;
    let text = decode_sig(&raw, &shown, "the export")?;
    if pytext::strip(&text).is_empty() {
        return fail(format!("{shown}: file is empty"));
    }
    let rows = csv_rows(&text)?;
    let header: Vec<String> = rows[0].iter().map(|c| casefold(strip(c))).collect();
    let has = |name: &str| header.iter().any(|h| h == name);
    if has("account") && has("amount") {
        if amount_column.is_some() {
            return fail(format!(
                "{shown} has an account and amount header, so --amount-column does not apply. \
                 Remove it, or drop the header row to read the file as a report style export."
            ));
        }
        let repeated: Vec<&str> = ["account", "amount", "section"]
            .into_iter()
            .filter(|name| header.iter().filter(|h| h == name).count() > 1)
            .collect();
        if !repeated.is_empty() {
            return fail(format!(
                "{shown}: more than one column is named {}, so the column to read is \
                 ambiguous. Keep one column with each of those names.",
                repeated.join(", ")
            ));
        }
        return read_neutral(&shown, &rows, &header);
    }
    read_report(&shown, &rows, amount_column)
}

fn read_neutral(shown: &str, rows: &[Vec<String>], header: &[String]) -> Result<PnlFile, Failure> {
    let index = |name: &str| header.iter().position(|h| h == name);
    let account_at = index("account").unwrap();
    let amount_at = index("amount").unwrap();
    let section_at = index("section");
    let width = header.len();
    let mut parsed = Vec::new();
    let mut skipped = Vec::new();
    for (i, row) in rows[1..].iter().enumerate() {
        let number = i + 2;
        if row.iter().all(|c| strip(c).is_empty()) {
            continue;
        }
        if row.len() > width && row[width..].iter().any(|c| !strip(c).is_empty()) {
            return fail(format!(
                "{shown} line {number}: expected {width} column(s), found {}. An unquoted \
                 comma inside an account name is the usual cause.",
                row.len()
            ));
        }
        let mut row = row.clone();
        if row.len() < width {
            row.resize(width, String::new());
        }
        let account = strip(&row[account_at]).to_string();
        if account.is_empty() {
            skipped.push(format!("line {number}: no account name"));
            continue;
        }
        let amount = parse_amount(Some(&row[amount_at]), &format!("{shown} line {number}"))
            .map_err(|e| Failure::Value(e.0))?;
        let section = match section_at {
            None => None,
            Some(at) => {
                let value = casefold(strip(&row[at]));
                if value.is_empty() {
                    None
                } else if [SECTION_INCOME, SECTION_COST_OF_SALES, SECTION_EXPENSE]
                    .contains(&value.as_str())
                {
                    Some(value)
                } else {
                    return fail(format!(
                        "{shown} line {number}: unknown section {}. Use one of \
                         {SECTION_INCOME}, {SECTION_COST_OF_SALES}, {SECTION_EXPENSE}.",
                        pytext::repr(&value)
                    ));
                }
            }
        };
        parsed.push(PnlRow {
            is_total: is_total_row(&account),
            account,
            amount,
            line_number: number,
            section,
        });
    }
    if parsed.is_empty() {
        return fail(format!("{shown}: no account rows found"));
    }
    let heading = strip(&rows[0][amount_at]);
    Ok(PnlFile {
        rows: parsed,
        layout: "neutral",
        amount_column: if heading.is_empty() {
            "amount".into()
        } else {
            heading.into()
        },
        skipped,
    })
}

fn cell(row: &[String], index: usize) -> &str {
    row.get(index).map_or("", |c| strip(c))
}

fn parses(text: &str) -> bool {
    parse_amount(Some(text), "amount").is_ok()
}

/// A column's heading: its cell on the row labelled "Account", else the text
/// above its first amount, or None when an amount comes first.
fn column_heading(rows: &[Vec<String>], index: usize) -> Option<String> {
    let first = &rows[..rows.len().min(20)];
    for row in first {
        if !row.is_empty() && casefold(strip(&row[0])) == "account" {
            let heading = cell(row, index);
            return (!heading.is_empty()).then(|| heading.to_string());
        }
    }
    for row in first {
        let c = cell(row, index);
        if c.is_empty() {
            continue;
        }
        return (!parses(c)).then(|| c.to_string());
    }
    None
}

fn only_placeholders(rows: &[Vec<String>], index: usize, heading: Option<&str>) -> bool {
    let mut cells: Vec<&str> = rows
        .iter()
        .map(|row| cell(row, index))
        .filter(|c| !c.is_empty())
        .collect();
    if let Some(heading) = heading
        && let Some(at) = cells.iter().position(|c| *c == heading)
    {
        cells.remove(at);
    }
    if heading.is_none() && cells.is_empty() {
        return false;
    }
    cells.iter().all(|c| PLACEHOLDER.matches_start(c))
}

/// `str.isdigit()` and `int()` for an `--amount-column` value.
fn column_number(text: &str) -> Option<Result<u128, Failure>> {
    let cps: Vec<u32> = pytext::code_points(text).collect();
    let digit = |cp: &u32| {
        pytext::decimal_value(*cp).is_some()
            || DIGIT_NOT_DECIMAL
                .iter()
                .any(|&(low, high)| (low..=high).contains(cp))
    };
    if cps.is_empty() || !cps.iter().all(digit) {
        return None;
    }
    if cps.iter().any(|cp| pytext::decimal_value(*cp).is_none()) {
        return Some(Err(Failure::Uncaught(format!(
            "ValueError: invalid literal for int() with base 10: {}",
            pytext::repr(text)
        ))));
    }
    // int() refuses a string of more than 4,300 digits, leading zeros included.
    if cps.len() > 4300 {
        return Some(Err(Failure::Uncaught(format!(
            "ValueError: Exceeds the limit (4300 digits) for integer string conversion: \
             value has {} digits; use sys.set_int_max_str_digits() to increase the limit",
            cps.len()
        ))));
    }
    let digits: String = cps
        .iter()
        .map(|cp| char::from_digit(pytext::decimal_value(*cp).unwrap(), 10).unwrap())
        .collect();
    let digits = digits.trim_start_matches('0');
    Some(Ok(if digits.len() > 30 {
        u128::MAX
    } else {
        digits.parse().unwrap_or(0)
    }))
}

fn amount_column_index(
    rows: &[Vec<String>],
    amount_column: Option<&str>,
) -> Result<(usize, String), Failure> {
    let widest = rows.iter().map(Vec::len).max().unwrap_or(0);
    if widest < 2 {
        return fail("report style export needs at least two columns");
    }
    if let Some(wanted_raw) = amount_column {
        if let Some(number) = column_number(wanted_raw) {
            let index = number?;
            if index < 1 || index >= widest as u128 {
                return fail(format!(
                    "--amount-column {wanted_raw} is out of range: the file has {} value column(s)",
                    widest - 1
                ));
            }
            return Ok((index as usize, format!("column {index}")));
        }
        let wanted = casefold(strip(wanted_raw));
        for row in &rows[..rows.len().min(20)] {
            for (index, c) in row.iter().enumerate() {
                if index > 0 && casefold(strip(c)) == wanted {
                    return Ok((index, strip(c).to_string()));
                }
            }
        }
        return fail(format!(
            "--amount-column {} does not match any column heading",
            pytext::repr(wanted_raw)
        ));
    }
    let mut labels = Vec::new();
    for index in 1..widest {
        if let Some(heading) = column_heading(rows, index)
            && LABEL_HEADING.matches_start(&heading)
        {
            labels.push(index);
        }
    }
    let mut counts = vec![0usize; widest];
    for row in rows {
        for index in 1..row.len().min(widest) {
            let c = strip(&row[index]);
            if !c.is_empty() && parses(c) {
                counts[index] += 1;
            }
        }
    }
    let mut best = 0;
    for index in 1..widest {
        if labels.contains(&index) {
            continue;
        }
        if best == 0 || counts[index] > counts[best] {
            best = index;
        }
    }
    if counts[best] == 0 {
        return fail("no column in this file parses as amounts");
    }
    let name = |index: usize| match column_heading(rows, index) {
        Some(heading) => format!("column {index} ({heading})"),
        None => format!("column {index}"),
    };
    for (index, &count) in counts.iter().enumerate().take(best).skip(1) {
        if labels.contains(&index) {
            continue;
        }
        let heading = column_heading(rows, index);
        if count > 0 {
            return fail(format!(
                "amounts are in {} and {}, and the second is fuller, so the period to compare \
                 is ambiguous. Name it with --amount-column, for example --amount-column {index}.",
                name(index),
                name(best)
            ));
        }
        if only_placeholders(rows, index, heading.as_deref()) {
            return fail(format!(
                "{} holds no readable amount and {} does, so the period to compare is \
                 ambiguous. Re-export with the current period filled in, or read {} \
                 deliberately with --amount-column {best}.",
                name(index),
                name(best),
                name(best)
            ));
        }
    }
    Ok((best, name(best)))
}

/// Whether a section heading's amount is the total of the rows under it:
/// Some(true), Some(false), or None when a detail cell cannot be read.
fn heading_carries_section_total(
    later: &[Vec<String>],
    index: usize,
    amount: &Decimal,
) -> Option<bool> {
    let mut running = Decimal::from_int(0);
    let mut seen_detail = false;
    for row in later {
        if row.iter().all(|c| strip(c).is_empty()) {
            continue;
        }
        let label = cell(row, 0);
        let c = cell(row, index);
        if !label.is_empty() && (section_for(label).is_some() || is_total_row(label)) {
            break;
        }
        if label.is_empty() || c.is_empty() {
            continue;
        }
        match parse_amount(Some(c), "amount") {
            Ok(value) => running = running.add(&value).expect("finite"),
            Err(_) => return None,
        }
        seen_detail = true;
    }
    Some(seen_detail && running.equals(amount).expect("finite"))
}

fn read_report(
    shown: &str,
    rows: &[Vec<String>],
    amount_column: Option<&str>,
) -> Result<PnlFile, Failure> {
    let (index, column_name) = amount_column_index(rows, amount_column)?;
    let mut parsed = Vec::new();
    let mut skipped = Vec::new();
    let mut section: Option<String> = None;
    for (i, row) in rows.iter().enumerate() {
        let number = i + 1;
        if row.iter().all(|c| strip(c).is_empty()) {
            continue;
        }
        let label = cell(row, 0);
        let c = cell(row, index);
        let heading = if label.is_empty() {
            None
        } else {
            section_for(label)
        };
        let mut row_section = section.clone();
        if let Some(heading) = heading {
            if c.is_empty() {
                section = Some(heading.into());
                continue;
            }
            let Ok(amount) = parse_amount(Some(c), &format!("{shown} line {number}")) else {
                skipped.push(format!(
                    "line {number}: {} has no readable amount in {column_name}",
                    pytext::repr(label)
                ));
                section = Some(heading.into());
                continue;
            };
            if heading_carries_section_total(&rows[number..], index, &amount) != Some(false) {
                section = Some(heading.into());
                parsed.push(PnlRow {
                    account: label.into(),
                    amount,
                    line_number: number,
                    section: section.clone(),
                    is_total: true,
                });
                continue;
            }
            row_section = Some(heading.into());
        }
        if !label.is_empty() && c.is_empty() {
            skipped.push(format!(
                "line {number}: {} has no amount in {column_name}",
                pytext::repr(label)
            ));
            continue;
        }
        if label.is_empty() {
            skipped.push(format!("line {number}: amount with no account name"));
            continue;
        }
        let Ok(amount) = parse_amount(Some(c), &format!("{shown} line {number}")) else {
            skipped.push(format!(
                "line {number}: {} has no readable amount in {column_name}",
                pytext::repr(label)
            ));
            continue;
        };
        parsed.push(PnlRow {
            account: label.into(),
            amount,
            line_number: number,
            section: row_section,
            is_total: is_total_row(label),
        });
    }
    if parsed.is_empty() {
        return fail(format!(
            "{shown}: no account rows found. If this is a two column file, give it an \
             'account,amount' header row."
        ));
    }
    Ok(PnlFile {
        rows: parsed,
        layout: "report",
        amount_column: column_name,
        skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_headless_placeholder_column_counts_as_placeholders() {
        // A column with no heading in its first 20 rows still counts as placeholders after them.
        let mut rows: Vec<Vec<String>> = vec![vec!["Sales".into(), "".into(), "850000".into()]; 20];
        rows.push(vec!["Rent".into(), "-".into(), "60000".into()]);
        assert!(only_placeholders(&rows, 1, None));
    }
}
