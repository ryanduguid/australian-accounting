//! Account buckets, the mapping file and the suggestions that seed it. A
//! suggestion is recorded as one and the report says so until a person
//! changes it.

use std::sync::LazyLock;

use sha2::{Digest, Sha256};

use crate::Failure;
use crate::atomic::atomic_write;
use crate::csvsafe::guard;
use crate::decimal::Decimal;
use crate::pnl::{PnlRow, decode_sig};
use crate::py::PyPath;
use crate::pyre::Regex;
use crate::pytext;
use crate::unicode::casefold;

pub const REVIEW: &str = "REVIEW";
pub const SOURCE_SUGGESTED: &str = "suggested";
pub const SOURCE_REVIEWED: &str = "reviewed";

/// Bucket name and what the ATO does with it, in the published order.
pub const BUCKETS: [(&str, &str); 11] = [
    (
        "turnover",
        "Sales of goods and services. The ATO's turnover label.",
    ),
    (
        "other_income",
        "Business income that is not sales, for example interest or grants.",
    ),
    (
        "cost_of_sales",
        "Cost of sales, excluding any salary and wages inside it.",
    ),
    (
        "cost_of_sales_labour",
        "Salary and wages included in cost of sales.",
    ),
    ("salary_wages", "Salary and wages outside cost of sales."),
    (
        "contractor_commission",
        "Contractor, subcontractor and commission expenses.",
    ),
    ("associated_persons", "Payments to associated persons."),
    ("rent", "Rent expenses."),
    ("motor_vehicle", "Motor vehicle expenses."),
    (
        "other_expense",
        "Every other expense, including superannuation and depreciation.",
    ),
    (
        "excluded",
        "Not part of the ATO calculation, for example income tax expense.",
    ),
];

pub const INCOME_BUCKETS: [&str; 2] = ["turnover", "other_income"];
/// The expense buckets, in sorted order.
pub const EXPENSE_BUCKETS: [&str; 8] = [
    "associated_persons",
    "contractor_commission",
    "cost_of_sales",
    "cost_of_sales_labour",
    "motor_vehicle",
    "other_expense",
    "rent",
    "salary_wages",
];

pub const FIELDNAMES: [&str; 6] = [
    "account",
    "account_key",
    "bucket",
    "source",
    "amount",
    "note",
];

pub fn is_bucket(name: &str) -> bool {
    BUCKETS.iter().any(|(b, _)| *b == name)
}

pub fn is_expense(name: &str) -> bool {
    EXPENSE_BUCKETS.contains(&name)
}

fn sorted_buckets() -> Vec<&'static str> {
    let mut names: Vec<&str> = BUCKETS.iter().map(|(b, _)| *b).collect();
    names.sort();
    names
}

static RULES: LazyLock<Vec<(Regex, &'static str, &'static str)>> = LazyLock::new(|| {
    [
        (
            r"income tax (expense|provision)",
            "excluded",
            "income tax is outside the ATO expense labels",
        ),
        (
            r"associated (person|entity)|related part(y|ies)|spouse",
            "associated_persons",
            "named as an associate, related party or spouse",
        ),
        (
            r"payroll tax",
            "other_expense",
            "payroll tax is not salary and wages",
        ),
        (
            r"(sub[- ]?contractor|contractor)",
            "contractor_commission",
            "contractor wording",
        ),
        (r"commission", "contractor_commission", "commission wording"),
        (
            r"(direct|production|factory) (labour|labor|wages)",
            "cost_of_sales_labour",
            "labour inside cost of sales",
        ),
        (
            r"superannuation|super contribution",
            "other_expense",
            "superannuation is its own ATO label",
        ),
        (
            r"(wages|salaries|salary)",
            "salary_wages",
            "salary and wages wording",
        ),
        (
            r"cost of (sales|goods)|opening stock|closing stock|purchases",
            "cost_of_sales",
            "cost of sales wording",
        ),
        (
            r"motor vehicle|vehicle running|car expense",
            "motor_vehicle",
            "motor vehicle wording",
        ),
        (
            r"rent(al)? (received|income)",
            "other_income",
            "rent received is income",
        ),
        (r"\brent\b", "rent", "rent wording"),
        (
            r"interest (income|received)|dividend|government (grant|payment)|fuel tax credit",
            "other_income",
            "not sales of goods or services",
        ),
        (
            r"(gain|profit) on (sale|disposal)",
            "other_income",
            "not sales of goods or services",
        ),
        (
            r"\bsales\b|services income|trading income|revenue|\bfees? income\b|fees charged",
            "turnover",
            "sales wording",
        ),
    ]
    .into_iter()
    .map(|(p, b, r)| (Regex::ignore_case(p), b, r))
    .collect()
});
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+"));
static ACCOUNT_KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[0-9a-f]{64}"));

fn fail<T>(message: impl Into<String>) -> Result<T, Failure> {
    Err(Failure::Value(message.into()))
}

#[derive(Clone, Debug, PartialEq)]
pub struct MappingRow {
    pub account: String,
    pub bucket: String,
    pub source: String,
    pub note: String,
    pub amount: String,
}

#[derive(Clone, Debug)]
pub struct MappingDraft {
    pub rows: Vec<MappingRow>,
    pub duplicates: Vec<String>,
}

impl MappingDraft {
    pub fn needs_review(&self) -> usize {
        self.rows.iter().filter(|r| r.bucket == REVIEW).count()
    }
}

#[derive(Clone, Debug)]
pub struct RoutingResult {
    /// Every bucket's total, in `BUCKETS` order.
    pub totals: Vec<(String, Decimal)>,
    pub unreviewed: usize,
    pub notes: Vec<String>,
    /// Buckets at least one reviewed account reached.
    pub supplied_buckets: Vec<String>,
}

/// Propose a bucket for an account name: (bucket, reason).
pub fn suggest(account: &str, section: Option<&str>) -> (String, String) {
    for (pattern, bucket, reason) in RULES.iter() {
        if pattern.search(account) {
            if section == Some("income") && !INCOME_BUCKETS.contains(bucket) {
                return ("turnover".into(), "in the income section".into());
            }
            if matches!(section, Some("cost_of_sales" | "expense"))
                && INCOME_BUCKETS.contains(bucket)
            {
                return (
                    REVIEW.into(),
                    format!("matched {bucket} wording but sits in an expense section"),
                );
            }
            if section == Some("cost_of_sales") && *bucket == "salary_wages" {
                return (
                    "cost_of_sales_labour".into(),
                    "wages inside the cost of sales section".into(),
                );
            }
            return ((*bucket).into(), (*reason).into());
        }
    }
    match section {
        Some("income") => ("turnover".into(), "in the income section".into()),
        Some("cost_of_sales") => (
            "cost_of_sales".into(),
            "in the cost of sales section, no more specific rule matched".into(),
        ),
        Some("expense") => (
            "other_expense".into(),
            "in the expense section, no more specific rule matched".into(),
        ),
        _ => (REVIEW.into(), "no rule matched".into()),
    }
}

/// The established logical identity of an account name.
pub fn normalise_account(account: &str) -> String {
    casefold(pytext::strip(&SPACES.sub(" ", account)))
}

#[cfg(test)]
thread_local! {
    /// A key every account gets, as the Python tests forced a digest
    /// collision by replacing `account_key`.
    static FORCED_KEY: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// A stable digest of an account's logical identity.
pub fn account_key(account: &str) -> String {
    #[cfg(test)]
    if let Some(key) = FORCED_KEY.with(|k| k.borrow().clone()) {
        return key;
    }
    let identity = normalise_account(account);
    Sha256::digest(identity.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn suggest_mapping(rows: &[PnlRow]) -> Result<MappingDraft, Failure> {
    let mut suggested = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();
    let mut duplicates = Vec::new();
    for row in rows {
        let identity = normalise_account(&row.account);
        let key = account_key(&row.account);
        if let Some((_, previous)) = seen.iter().find(|(k, _)| *k == key) {
            if *previous != identity {
                return fail(format!(
                    "account identity hash collision between {} and {}; no mapping was written",
                    pytext::repr(previous),
                    pytext::repr(&identity)
                ));
            }
            duplicates.push(row.account.clone());
            continue;
        }
        seen.push((key, identity));
        let (bucket, note) = if row.is_total {
            (
                "excluded".to_string(),
                "looks like a subtotal row, so it is left out".to_string(),
            )
        } else {
            suggest(&row.account, row.section.as_deref())
        };
        suggested.push(MappingRow {
            account: row.account.clone(),
            bucket,
            source: SOURCE_SUGGESTED.into(),
            note,
            amount: row.amount.to_sci_string(),
        });
    }
    Ok(MappingDraft {
        rows: suggested,
        duplicates,
    })
}

/// A mapping keyed by account digest, in file order.
pub type Mapping = Vec<(String, MappingRow)>;

pub fn route(rows: &[PnlRow], mapping: &Mapping, flip: bool) -> Result<RoutingResult, Failure> {
    let mut totals: Vec<(String, Decimal)> = BUCKETS
        .iter()
        .map(|(b, _)| (b.to_string(), Decimal::from_int(0)))
        .collect();
    let mut notes = Vec::new();
    let mut unreviewed = 0;
    let mut missing = Vec::new();
    let mut repeated = Vec::new();
    let mut counted: Vec<String> = Vec::new();
    let mut counted_buckets: Vec<String> = Vec::new();
    for row in rows {
        let identity = normalise_account(&row.account);
        let key = account_key(&row.account);
        let Some((_, entry)) = mapping.iter().find(|(k, _)| *k == key) else {
            missing.push(format!("line {}: {}", row.line_number, row.account));
            continue;
        };
        if normalise_account(&entry.account) != identity {
            return fail(format!(
                "line {}: account identity hash collision between {} and {}; no amount was routed",
                row.line_number,
                pytext::repr(&entry.account),
                pytext::repr(&row.account)
            ));
        }
        if counted.contains(&key) {
            repeated.push(format!("line {}: {}", row.line_number, row.account));
            continue;
        }
        counted.push(key);
        let suggested = casefold(pytext::strip(&entry.source)) == SOURCE_SUGGESTED;
        if suggested {
            unreviewed += 1;
        }
        if row.is_total && entry.bucket != "excluded" {
            notes.push(format!(
                "{} looks like a subtotal row but is mapped to {}. The mapping wins, so check it \
                 is not double counting.",
                pytext::repr(&row.account),
                entry.bucket
            ));
        }
        let amount = if flip && is_expense(&entry.bucket) {
            row.amount.neg()?
        } else {
            row.amount.clone()
        };
        let Some(slot) = totals.iter_mut().find(|(b, _)| *b == entry.bucket) else {
            return Err(Failure::Uncaught(format!(
                "KeyError: {}",
                pytext::repr(&entry.bucket)
            )));
        };
        slot.1 = slot.1.add(&amount)?;
        if !suggested && !counted_buckets.contains(&entry.bucket) {
            counted_buckets.push(entry.bucket.clone());
        }
    }
    let listing = |items: &[String]| {
        let listed = items
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ");
        let more = if items.len() <= 20 {
            String::new()
        } else {
            format!("\n  ... and {} more", items.len() - 20)
        };
        format!("{listed}{more}")
    };
    if !missing.is_empty() {
        return fail(format!(
            "these profit and loss rows have no mapping entry:\n  {}\nRerun the map command, \
             or add them to the mapping file.",
            listing(&missing)
        ));
    }
    if !repeated.is_empty() {
        return fail(format!(
            "these account names appear more than once in the export, so one mapping row \
             cannot cover them:\n  {}\nGive them distinct names in the export, or combine them \
             into one row.",
            listing(&repeated)
        ));
    }
    let mut unused: Vec<&String> = mapping
        .iter()
        .map(|(k, _)| k)
        .filter(|k| !counted.contains(k))
        .collect();
    unused.sort();
    if !unused.is_empty() {
        let names: Vec<&str> = unused
            .iter()
            .take(5)
            .map(|k| {
                mapping
                    .iter()
                    .find(|(key, _)| key == *k)
                    .map(|(_, r)| r.account.as_str())
                    .unwrap()
            })
            .collect();
        notes.push(format!(
            "{} mapping row(s) did not match any account in the export: {}",
            unused.len(),
            names.join(", ")
        ));
    }
    Ok(RoutingResult {
        totals,
        unreviewed,
        notes,
        supplied_buckets: counted_buckets,
    })
}

fn duplicate_or_collision(
    shown: &str,
    number: Option<usize>,
    account: &str,
    previous: &str,
) -> Failure {
    let location = match number {
        Some(n) => format!("{shown} line {n}"),
        None => shown.to_string(),
    };
    if normalise_account(previous) == normalise_account(account) {
        return Failure::Value(format!(
            "{location}: {} appears more than once",
            pytext::repr(account)
        ));
    }
    Failure::Value(format!(
        "{location}: account identity hash collision between {} and {}; no mapping was applied",
        pytext::repr(previous),
        pytext::repr(account)
    ))
}

fn resolve_keyed_account(
    shown: &str,
    number: usize,
    displayed: &str,
    supplied: &str,
) -> Result<String, Failure> {
    if !ACCOUNT_KEY.fullmatch(supplied) {
        return fail(format!(
            "{shown} line {number}: account_key must be exactly 64 lower-case hexadecimal \
             characters. Regenerate the mapping and reapply the reviewed bucket, source and note \
             values."
        ));
    }
    let mut candidates = Vec::new();
    if guard(Some(displayed)) == displayed {
        candidates.push(displayed.to_string());
    }
    if let Some(unguarded) = displayed.strip_prefix('\'')
        && guard(Some(unguarded)) == displayed
    {
        candidates.push(unguarded.to_string());
    }
    let matches: Vec<String> = candidates
        .into_iter()
        .filter(|c| !normalise_account(c).is_empty() && account_key(c) == supplied)
        .collect();
    match matches.len() {
        0 => fail(format!(
            "{shown} line {number}: account display and account_key do not identify the same \
             safely guarded account. Regenerate the mapping and reapply the reviewed bucket, \
             source and note values."
        )),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => fail(format!(
            "{shown} line {number}: account display and account_key are ambiguous, which may \
             indicate an account identity hash collision. Regenerate the mapping and reapply the \
             reviewed bucket, source and note values."
        )),
    }
}

fn resolve_legacy_account(shown: &str, number: usize, displayed: &str) -> Result<String, Failure> {
    let guarded_candidate = displayed
        .strip_prefix('\'')
        .is_some_and(|rest| guard(Some(rest)) == displayed);
    if guard(Some(displayed)) != displayed || guarded_candidate {
        return fail(format!(
            "{shown} line {number}: legacy account {} is formula-like or could be a \
             spreadsheet guard. Regenerate the mapping and reapply the reviewed bucket, source \
             and note values.",
            pytext::repr(displayed)
        ));
    }
    Ok(displayed.to_string())
}

/// `csv.writer` with the excel dialect: minimal quoting and CRLF rows.
pub fn csv_line(fields: &[String]) -> String {
    let quoted: Vec<String> = fields
        .iter()
        .map(|field| {
            let needs =
                field.contains([',', '"', '\r', '\n']) || (fields.len() == 1 && field.is_empty());
            if needs {
                format!("\"{}\"", field.replace('"', "\"\""))
            } else {
                field.clone()
            }
        })
        .collect();
    format!("{}\r\n", quoted.join(","))
}

pub fn write_mapping(path: &PyPath, rows: &[MappingRow]) -> Result<(), Failure> {
    let shown = path.display();
    let mut prepared = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();
    for row in rows {
        if normalise_account(&row.account).is_empty() {
            return fail(format!(
                "{shown}: mapping account cannot be empty after normalisation"
            ));
        }
        let key = account_key(&row.account);
        if let Some((_, previous)) = seen.iter().find(|(k, _)| *k == key) {
            return Err(duplicate_or_collision(&shown, None, &row.account, previous));
        }
        seen.push((key.clone(), row.account.clone()));
        prepared.push((row, key));
    }
    let mut text = csv_line(&FIELDNAMES.map(String::from));
    for (row, key) in prepared {
        text.push_str(&csv_line(&[
            guard(Some(&row.account)),
            key,
            guard(Some(&row.bucket)),
            guard(Some(&row.source)),
            guard(Some(&row.amount)),
            guard(Some(&row.note)),
        ]));
    }
    atomic_write(path, text.as_bytes())
}

/// Read a mapping file, keyed by stable account digest.
pub fn read_mapping(path: &PyPath) -> Result<Mapping, Failure> {
    let shown = path.display();
    if !path.fs().is_file() {
        return fail(format!("mapping file not found: {shown}"));
    }
    let data = std::fs::read(path.fs()).map_err(|e| {
        Failure::Value(crate::py::oserror_str(
            &e,
            &shown,
            crate::py::OsStyle::Errno,
        ))
    })?;
    let text = decode_sig(&data, &shown, "the mapping")?;
    // Rows are read as the loop reaches them, so an unreadable row later in
    // the file does not pre-empt a refusal of an earlier one.
    let mut reader = crate::csvread::Reader::new(text.as_bytes());
    let next = |reader: &mut crate::csvread::Reader| match reader.next_row() {
        Ok(row) => Ok(row),
        Err(crate::csvread::ReadError::Csv(message)) => {
            Err(Failure::Uncaught(format!("_csv.Error: {message}")))
        }
        Err(crate::csvread::ReadError::Decode { .. }) => unreachable!("decoded text"),
    };
    let Some(header) = next(&mut reader)? else {
        return fail(format!("{shown}: file is empty"));
    };
    let names: Vec<String> = header.iter().map(|n| casefold(pytext::strip(n))).collect();
    let unnamed: Vec<String> = names
        .iter()
        .enumerate()
        .filter(|(_, n)| n.is_empty())
        .map(|(i, _)| (i + 1).to_string())
        .collect();
    if !unnamed.is_empty() {
        return fail(format!(
            "{shown}: unnamed column(s) at position(s): {}",
            unnamed.join(", ")
        ));
    }
    let mut duplicates: Vec<&String> = names
        .iter()
        .filter(|n| !n.is_empty() && names.iter().filter(|m| m == n).count() > 1)
        .collect();
    duplicates.sort();
    duplicates.dedup();
    if !duplicates.is_empty() {
        return fail(format!(
            "{shown}: duplicate column name(s): {}",
            duplicates
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let mut missing: Vec<&str> = ["account", "bucket"]
        .into_iter()
        .filter(|n| !names.iter().any(|m| m == n))
        .collect();
    missing.sort();
    if !missing.is_empty() {
        let found: Vec<&str> = header
            .iter()
            .filter(|n| !n.is_empty())
            .map(|n| n.as_str())
            .collect();
        return fail(format!(
            "{shown}: missing required column(s): {}. Found: {}",
            missing.join(", "),
            found.join(", ")
        ));
    }
    let unknown: Vec<&str> = names
        .iter()
        .filter(|n| !n.is_empty() && !FIELDNAMES.contains(&n.as_str()))
        .map(|n| n.as_str())
        .collect();
    let has_source = names.iter().any(|n| n == "source");
    if !has_source && !unknown.is_empty() {
        return fail(format!(
            "{shown}: the source column is missing while the file carries extra column(s): {}. \
             Every row would take the legacy reviewed default, so add the source column \
             (reviewed or suggested for each row) or remove the extras.",
            unknown.join(", ")
        ));
    }
    let keyed = names.iter().any(|n| n == "account_key");
    let width = header.len();
    let sorted = sorted_buckets().join(", ");
    let mut rows: Mapping = Vec::new();
    let mut number = 1;
    while let Some(raw) = next(&mut reader)? {
        number += 1;
        if raw.iter().all(|c| pytext::strip(c).is_empty()) {
            continue;
        }
        if raw.len() < width {
            return fail(format!(
                "{shown} line {number}: truncated row has {} column(s); the header has {width}",
                raw.len()
            ));
        }
        if raw.len() > width && raw[width..].iter().any(|c| !c.is_empty()) {
            return fail(format!(
                "{shown} line {number}: populated extra column(s) do not match the mapping header"
            ));
        }
        let raw = &raw[..width];
        let field = |name: &str| -> Option<&str> {
            names
                .iter()
                .rposition(|n| n == name)
                .map(|at| raw[at].as_str())
        };
        let displayed = field("account").unwrap_or("");
        if normalise_account(displayed).is_empty() {
            return fail(format!(
                "{shown} line {number}: mapping account is empty after normalisation"
            ));
        }
        let (account, key) = if keyed {
            let supplied = field("account_key").unwrap_or("");
            (
                resolve_keyed_account(&shown, number, displayed, supplied)?,
                supplied.to_string(),
            )
        } else {
            let account = resolve_legacy_account(&shown, number, displayed)?;
            let key = account_key(&account);
            (account, key)
        };
        let bucket = pytext::strip(field("bucket").unwrap_or("")).to_string();
        if bucket.is_empty() {
            return fail(format!(
                "{shown} line {number}: {} has no bucket",
                pytext::repr(&account)
            ));
        }
        if bucket == REVIEW {
            return fail(format!(
                "{shown} line {number}: {} is still marked {REVIEW}. Choose one of: {sorted}",
                pytext::repr(&account)
            ));
        }
        if !is_bucket(&bucket) {
            return fail(format!(
                "{shown} line {number}: {} has unknown bucket {}. Choose one of: {sorted}",
                pytext::repr(&account),
                pytext::repr(&bucket)
            ));
        }
        let source = match field("source") {
            None => SOURCE_REVIEWED.to_string(),
            Some(raw) => pytext::strip(raw).to_string(),
        };
        let folded = casefold(&source);
        if source.is_empty() || (folded != SOURCE_REVIEWED && folded != SOURCE_SUGGESTED) {
            return fail(format!(
                "{shown} line {number}: {} has source {}. Choose one of: reviewed, suggested",
                pytext::repr(&account),
                pytext::repr(&source)
            ));
        }
        if let Some((_, previous)) = rows.iter().find(|(k, _)| *k == key) {
            return Err(duplicate_or_collision(
                &shown,
                Some(number),
                &account,
                &previous.account,
            ));
        }
        rows.push((
            key,
            MappingRow {
                account,
                bucket,
                source,
                note: pytext::strip(field("note").unwrap_or("")).to_string(),
                amount: pytext::strip(field("amount").unwrap_or("")).to_string(),
            },
        ));
    }
    if rows.is_empty() {
        return fail(format!("{shown}: no mapping rows found"));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestions_follow_the_ordered_rules() {
        let s = |a: &str, sec: Option<&str>| suggest(a, sec).0;
        assert_eq!(s("Income tax expense", None), "excluded");
        assert_eq!(
            s("Wages - directors (related party)", None),
            "associated_persons"
        );
        assert_eq!(s("Sub-contractor payments", None), "contractor_commission");
        assert_eq!(s("Office rent", None), "rent");
        assert_eq!(s("Rental income", None), "other_income");
        assert_eq!(s("Parental leave", None), REVIEW);
        assert_eq!(s("Fees income", None), "turnover");
        assert_eq!(s("Bank fees", None), REVIEW);
        assert_eq!(s("Wages", Some("cost_of_sales")), "cost_of_sales_labour");
        assert_eq!(s("Interest received", Some("expense")), REVIEW);
    }

    use crate::decimal::Decimal;
    use crate::py::PyPath;

    fn forced<T>(f: impl FnOnce() -> T) -> T {
        FORCED_KEY.with(|k| *k.borrow_mut() = Some("a".repeat(64)));
        let out = f();
        FORCED_KEY.with(|k| *k.borrow_mut() = None);
        out
    }

    fn row(account: &str, amount: &str, line: usize) -> PnlRow {
        PnlRow {
            account: account.into(),
            amount: Decimal::parse(amount).unwrap(),
            line_number: line,
            section: None,
            is_total: false,
        }
    }

    fn mapping_row(account: &str, bucket: &str) -> MappingRow {
        MappingRow {
            account: account.into(),
            bucket: bucket.into(),
            source: "reviewed".into(),
            note: String::new(),
            amount: String::new(),
        }
    }

    fn scratch(name: &str, text: &str) -> PyPath {
        let dir = std::env::temp_dir().join(format!("ato-collision-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        if !text.is_empty() {
            std::fs::write(&path, text).unwrap();
        }
        PyPath::new(&path.to_string_lossy())
    }

    fn message(result: Result<impl std::fmt::Debug, Failure>) -> String {
        result.unwrap_err().to_string()
    }

    #[test]
    fn write_rejects_a_forced_digest_collision() {
        let path = scratch("write.csv", "");
        let rows = [
            mapping_row("Sales", "turnover"),
            mapping_row("Rent", "rent"),
        ];
        assert!(forced(|| message(write_mapping(&path, &rows))).contains("collision"));
    }

    #[test]
    fn read_rejects_a_forced_digest_collision() {
        let key = "a".repeat(64);
        let text = format!(
            "account,account_key,bucket,source,amount,note
Sales,{key},turnover,reviewed,1,
Rent,{key},rent,reviewed,1,
"
        );
        let path = scratch("read.csv", &text);
        assert!(forced(|| message(read_mapping(&path))).contains("collision"));
    }

    #[test]
    fn a_key_both_spellings_of_a_guarded_account_match_is_refused_as_ambiguous() {
        let key = "a".repeat(64);
        let text = format!(
            "account,account_key,bucket,source,amount,note
'=Rent,{key},rent,reviewed,1,
"
        );
        let path = scratch("ambiguous.csv", &text);
        assert!(
            forced(|| message(read_mapping(&path)))
                .contains("line 2: account display and account_key are ambiguous")
        );
    }

    #[test]
    fn suggest_mapping_rejects_a_hash_collision_before_returning_a_draft() {
        let rows = [row("Sales", "100", 2), row("Rent", "10", 3)];
        assert_eq!(
            forced(|| message(suggest_mapping(&rows))),
            "account identity hash collision between 'sales' and 'rent'; no mapping was written"
        );
    }

    #[test]
    fn route_rejects_a_hash_collision_before_routing_amounts() {
        let mapping: Mapping = vec![("a".repeat(64), mapping_row("Sales", "turnover"))];
        assert_eq!(
            forced(|| message(route(&[row("Rent", "10", 7)], &mapping, false))),
            "line 7: account identity hash collision between 'Sales' and 'Rent'; no amount was routed"
        );
    }

    #[test]
    fn map_fails_when_distinct_accounts_share_a_forced_digest() {
        let pnl = scratch(
            "source.csv",
            "account,amount
Sales,100
Rent,10
",
        );
        let out = scratch("m.csv", "");
        let args: Vec<String> = [
            "map",
            "--profit-and-loss",
            &pnl.display(),
            "--out",
            &out.display(),
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let (code, _, err) = forced(|| crate::cli::run(&args));
        assert_eq!(code, 1);
        assert!(err.to_lowercase().contains("collision"), "{err}");
    }

    #[test]
    fn csv_lines_quote_minimally() {
        let line = csv_line(&["a,b".into(), "q\"t".into(), "plain".into(), String::new()]);
        assert_eq!(line, "\"a,b\",\"q\"\"t\",plain,\r\n");
        assert_eq!(csv_line(&[String::new()]), "\"\"\r\n");
    }
}
