//! The workbook before its desktop Excel pass. Formula text is that of the
//! Python builder it replaced (tools/build_workbook.py), character for
//! character; the frozen model of the Python output holds the two together.
//!
//! The workbook restates the engine's rules in ordinary worksheet formulas so
//! an accountant with desktop Excel and no Python can paste a loan register,
//! pick the year of income and read the s 109N gate and the s 109E minimum
//! yearly repayment per loan.

use std::collections::BTreeMap;
use std::error::Error;

use div7a_loan_review::facts::{FALSE, TRUE, UNKNOWN_TOKENS};
use div7a_loan_review::money::{MAX_MONEY_DECIMAL_PLACES, MAX_MONEY_MAGNITUDE};
use div7a_loan_review::py::{self, PyPath};
use div7a_loan_review::rates::load_table;
use div7a_loan_review::raw::Raw;
use div7a_loan_review::register::{FIRST_REVIEWABLE_YEAR, load_rows};
use div7a_loan_review::years::{EARLIEST, LATEST};
use rust_xlsxwriter::{
    Color, ConditionalFormatCell, ConditionalFormatCellRule, DataValidation, Format, FormatAlign,
    Formula, ProtectionOptions, Table, TableColumn, TableStyle, Workbook, Worksheet,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_YEAR: &str = "2026-27";

const PURPLE: u32 = 0x5C2D91;
const LAVENDER: u32 = 0xF3F1F6;
const GREY: u32 = 0xE2E0DF;
const TEXT: &str = "@";
const MONEY: &str = r#"_(* #,##0.00_);_(* (#,##0.00);_(* "-"??_);_(@_)"#;

const INPUT_COLUMNS: [&str; 14] = [
    "loan_id",
    "borrower_reference",
    "year_loan_made",
    "written_agreement",
    "terms_in_place_before_lodgment_day",
    "maximum_term_years",
    "secured_by_registered_mortgage_over_real_property",
    "security_coverage_at_first_made",
    "interest_rate_for_years_after_year_loan_made",
    "amalgamated_loan_unpaid_at_end_of_previous_year",
    "remaining_term_years",
    "payments_applied_during_the_year",
    "out_of_scope_reason",
    "year_of_income_being_tested",
];
const TEXT_COLUMNS: [&str; 5] = [
    "loan_id",
    "borrower_reference",
    "year_loan_made",
    "out_of_scope_reason",
    "year_of_income_being_tested",
];
const BOOL_COLUMNS: [&str; 3] = [
    "written_agreement",
    "terms_in_place_before_lodgment_day",
    "secured_by_registered_mortgage_over_real_property",
];
const NUMBER_COLUMNS: [&str; 6] = [
    "maximum_term_years",
    "security_coverage_at_first_made",
    "interest_rate_for_years_after_year_loan_made",
    "amalgamated_loan_unpaid_at_end_of_previous_year",
    "remaining_term_years",
    "payments_applied_during_the_year",
];
/// Each year column and the hidden helper column holding its normalised text.
const YEAR_HELPERS: [(&str, &str); 2] = [
    ("year_loan_made", "Year_made_text"),
    ("year_of_income_being_tested", "Year_tested_text"),
];
/// Python `str.strip()` whitespace, which the engine trims around a year
/// label. Replace it with spaces rather than deleting it: an internal tab must
/// keep a malformed label such as 2023-\t24 invalid.
const YEAR_WHITESPACE: [char; 29] = [
    '\t', '\n', '\u{b}', '\u{c}', '\r', '\u{1c}', '\u{1d}', '\u{1e}', '\u{1f}', ' ', '\u{85}',
    '\u{a0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
    '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}',
    '\u{205f}', '\u{3000}',
];
const MONEY_COLUMNS: [&str; 5] = [
    "amalgamated_loan_unpaid_at_end_of_previous_year",
    "payments_applied_during_the_year",
    "MYR_required",
    "Shortfall",
    "Exposure",
];

const YEAR: &str = "Summary!$B$2";
const YEAR_RATE: &str = "Summary!$B$3";
const YEAR_START: &str = "Summary!$B$4";

const CALC_ORDER: [&str; 22] = [
    "Status",
    "Floor_year",
    "Floor_rate",
    "Limb_a_written",
    "Limb_lodgment",
    "Limb_b_rate",
    "Limb_secured",
    "Allowed_term_raw",
    "Limb_c_term",
    "Gate_verdict",
    "Max_term_allowed",
    "Term_used",
    "MYR_raw",
    "MYR_verdict",
    "MYR_reason",
    "MYR_required",
    "Shortfall",
    "Exposure",
    "Undecided",
    "Guard",
    "Input_problem",
    "Sample_row",
];

/// Every calculated column in table order: the public ones, then the helpers.
fn all_calcs() -> Vec<&'static str> {
    CALC_ORDER
        .iter()
        .copied()
        .chain(YEAR_HELPERS.iter().map(|(_, helper)| *helper))
        .collect()
}

fn t(column: &str) -> String {
    format!("tblLoans[[#This Row],[{column}]]")
}

/// `{"a","b"}` members, sorted as Python sorts a set of strings.
fn quoted_sorted(words: &[&str]) -> String {
    let mut words = words.to_vec();
    words.sort_unstable();
    words
        .iter()
        .map(|w| format!("\"{w}\""))
        .collect::<Vec<_>>()
        .join(",")
}

fn unknown(expr: &str) -> String {
    let tokens = quoted_sorted(&UNKNOWN_TOKENS);
    format!("OR(LOWER(TRIM({expr}&\"\"))={{{tokens}}})")
}

fn tristate(column: &str) -> String {
    let yes = quoted_sorted(&TRUE);
    let no = quoted_sorted(&FALSE);
    let v = format!("LOWER(TRIM({}&\"\"))", t(column));
    format!(
        "=IF({},\"UNKNOWN\",IF(OR({v}={{{yes}}}),\"PASS\",IF(OR({v}={{{no}}}),\"FAIL\",\"UNKNOWN\")))",
        unknown(&t(column))
    )
}

fn helper(column: &str) -> &'static str {
    YEAR_HELPERS
        .iter()
        .find(|(c, _)| *c == column)
        .map(|(_, h)| *h)
        .expect("a year column")
}

/// Read the shared normalised year instead of repeating its formula.
fn year_text(column: &str) -> String {
    t(helper(column))
}

/// Match strip() for year labels and unknown tokens, preserving internal gaps.
fn normalise_year(column: &str) -> String {
    let mut text = format!("{}&\"\"", t(column));
    for character in YEAR_WHITESPACE {
        if character != ' ' {
            text = format!(
                "SUBSTITUTE({text},_xlfn.UNICHAR({}),\" \")",
                u32::from(character)
            );
        }
    }
    format!("TRIM({text})")
}

/// True where the engine's parse_year would refuse the label.
///
/// The label is trimmed, because parse_year strips before matching and
/// ` 2023-24 ` is a year to it. Every one of the 6 positions has to be a
/// digit, because VALUE reads `20e2` as 2000 and accepted a label the engine's
/// 4-digit pattern refuses. And the start year has to fall inside the engine's
/// own 1900 to 2100 typo guard, so `1800-01` is refused here too instead of
/// being silently skipped as historical.
fn year_bad(column: &str) -> String {
    let y = year_text(column);
    let digits = [1, 2, 3, 4, 6, 7]
        .iter()
        .map(|k| format!("ISNUMBER(VALUE(MID({y},{k},1)))"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "AND(NOT({}),NOT(IFERROR(AND(LEN({y})=7,MID({y},5,1)=\"-\",{digits},VALUE(RIGHT({y},2))=MOD(VALUE(LEFT({y},4))+1,100),VALUE(LEFT({y},4))>={EARLIEST},VALUE(LEFT({y},4))<={LATEST}),FALSE)))",
        unknown(&y)
    )
}

fn bool_bad(column: &str) -> String {
    let v = format!("LOWER(TRIM({}&\"\"))", t(column));
    let words: Vec<&str> = TRUE.iter().chain(&FALSE).copied().collect();
    format!(
        "AND(NOT({}),NOT(OR({v}={{{}}})))",
        unknown(&t(column)),
        quoted_sorted(&words)
    )
}

/// Text where a number belongs, or a negative, which the engine refuses for
/// every numeric column (parse_money, parse_rate, parse_ratio); a rate above 1
/// is refused too.
///
/// parse_money adds 2 limits: an amount cannot carry more than 2 decimal
/// places, and cannot exceed MAX_MONEY_MAGNITUDE. They apply to the 2 money
/// columns only, because parse_rate and parse_ratio do not impose them.
fn number_bad(column: &str) -> String {
    let x = t(column);
    let extra = if column == "interest_rate_for_years_after_year_loan_made" {
        format!(",{x}>1")
    } else if MONEY_COLUMNS.contains(&column) {
        // ROUND(x,2)<>x, exact: parse_money refuses any value with a third
        // decimal place, so a tolerance would let 100.00000001 through here
        // and be refused by the engine. A typed 1234.56 and ROUND(1234.56,2)
        // are the same double in Excel, so a clean cent figure is not refused.
        // N() around every arithmetic use of the cell: AND and OR evaluate
        // all of their arguments, so ROUND on a text cell such as `unknown`
        // returns #VALUE! and the error propagates out of the guard that was
        // meant to catch it. N() reads text as 0 and leaves a real number
        // untouched.
        let n = format!("N({x})");
        format!(",ROUND({n},{MAX_MONEY_DECIMAL_PLACES})<>{n},{n}>{MAX_MONEY_MAGNITUDE}")
    } else {
        String::new()
    };
    format!(
        "AND(NOT({}),OR(NOT(ISNUMBER({x})),AND(ISNUMBER({x}),OR({x}<0{extra}))))",
        unknown(&x)
    )
}

/// Every calculated column's formula, with a leading `=`. Sample_row still
/// carries its `{SAMPLE_IDS}` and `{SAMPLE_YEARS}` placeholders.
fn formulas() -> BTreeMap<&'static str, String> {
    let (a, lo, b, c) = (
        "Limb_a_written",
        "Limb_lodgment",
        "Limb_b_rate",
        "Limb_c_term",
    );
    let principal = t("amalgamated_loan_unpaid_at_end_of_previous_year");
    let payments = t("payments_applied_during_the_year");
    let term = t("maximum_term_years");
    let rate = t("interest_rate_for_years_after_year_loan_made");
    let coverage = t("security_coverage_at_first_made");
    let made = year_text("year_loan_made");
    let tested = year_text("year_of_income_being_tested");
    let year_start = format!("VALUE(LEFT({made},4))");
    let limbs = [t(a), t(lo), t(b), t(c)];
    let fails = limbs
        .iter()
        .map(|x| format!("{x}=\"FAIL\""))
        .collect::<Vec<_>>()
        .join(",");
    let unknowns = limbs
        .iter()
        .map(|x| format!("{x}=\"UNKNOWN\""))
        .collect::<Vec<_>>()
        .join(",");
    let v = t("MYR_verdict");
    let no_floor_year = unknown(&t("Floor_year"));
    let status = t("Status");
    let gate = t("Gate_verdict");
    let floor_year = t("Floor_year");
    let floor_rate = t("Floor_rate");
    let secured = t("Limb_secured");
    let allowed = t("Allowed_term_raw");
    let term_used = t("Term_used");
    let remaining = t("remaining_term_years");
    let myr_raw = t("MYR_raw");
    let out_of_scope = t("out_of_scope_reason");
    let unknown_made = unknown(&made);

    let mut calc = BTreeMap::new();
    // register._skip_reason skips ANY nonblank out_of_scope_reason before it
    // parses a single loan fact. Passing the cell through the unknown-token
    // helper instead meant a reason of "unknown" or "n/a" left the row in the
    // review, and the workbook reported a shortfall on a loan the engine
    // skipped with exit 0.
    calc.insert(
        "Status",
        format!(
            "=IF(TRIM({out_of_scope}&\"\")<>\"\",\"SKIPPED\",IF(IFERROR({year_start}<{FIRST_REVIEWABLE_YEAR},FALSE),\"SKIPPED\",\"REVIEWED\"))"
        ),
    );
    calc.insert(
        "Floor_year",
        format!("=IF({},{made},{tested})", unknown(&tested)),
    );
    calc.insert(
        "Floor_rate",
        format!(
            "=IFERROR(INDEX(tblRates[rate],MATCH({floor_year},tblRates[year_of_income],0)),\"\")"
        ),
    );
    calc.insert(a, tristate("written_agreement"));
    calc.insert(lo, tristate("terms_in_place_before_lodgment_day"));
    calc.insert(
        b,
        format!(
            "=IF(NOT(ISNUMBER({rate})),\"UNKNOWN\",IF({floor_rate}=\"\",\"UNKNOWN\",IF({rate}>={floor_rate},\"PASS\",\"FAIL\")))"
        ),
    );
    calc.insert(
        "Allowed_term_raw",
        format!(
            "=IF({secured}=\"FAIL\",7,IF({secured}=\"PASS\",IF(ISNUMBER({coverage}),IF({coverage}>=1.1,25,7),\"\"),\"\"))"
        ),
    );
    calc.insert(
        "Limb_secured",
        tristate("secured_by_registered_mortgage_over_real_property"),
    );
    calc.insert(
        c,
        format!(
            "=IF(NOT(ISNUMBER({term})),\"UNKNOWN\",IF({allowed}=\"\",IF({term}<=7,\"PASS\",\"UNKNOWN\"),IF({term}<={allowed},\"PASS\",\"FAIL\")))"
        ),
    );
    calc.insert(
        "Gate_verdict",
        format!(
            "=IF({status}=\"SKIPPED\",\"\",IF({no_floor_year},\"UNKNOWN\",IF(OR({fails}),\"NOT_COMPLYING\",IF(OR({unknowns}),\"UNKNOWN\",\"COMPLYING\"))))"
        ),
    );
    calc.insert(
        "Max_term_allowed",
        format!(
            "=IF(OR({status}=\"SKIPPED\",{no_floor_year}),\"\",IF({allowed}<>\"\",{allowed},IF(AND(ISNUMBER({term}),{term}<=7),7,\"\")))"
        ),
    );
    calc.insert(
        "Term_used",
        format!(
            "=IF({status}=\"SKIPPED\",\"\",IF(ISNUMBER({remaining}),ROUNDUP({remaining},0),\"\"))"
        ),
    );
    calc.insert(
        "MYR_raw",
        format!(
            "=IF(AND(ISNUMBER({principal}),ISNUMBER({YEAR_RATE}),{YEAR_RATE}>0,ISNUMBER({term_used}),{term_used}>0),ROUND({principal}*{YEAR_RATE}/(1-(1/(1+{YEAR_RATE}))^{term_used}),2),\"\")"
        ),
    );
    calc.insert(
        "MYR_verdict",
        format!(
            "=IF({status}=\"SKIPPED\",\"\",IF(OR({gate}<>\"COMPLYING\",AND(NOT({unknown_made}),{floor_year}<>{made}),{made}={YEAR},IFERROR({year_start}>{YEAR_START},FALSE),AND(ISNUMBER({term_used}),{term_used}<=0),AND(ISNUMBER({YEAR_RATE}),{YEAR_RATE}<=0)),\"REFUSED\",IF(OR({unknown_made},NOT(ISNUMBER({YEAR_RATE})),NOT(ISNUMBER({principal})),NOT(ISNUMBER({payments})),NOT(ISNUMBER({term_used}))),\"UNKNOWN\",IF({myr_raw}-ROUND({payments},2)<=0,\"MYR_MET\",\"MYR_SHORT\"))))"
        ),
    );
    calc.insert(
        "MYR_reason",
        format!(
            "=IF({v}=\"REFUSED\",IF({gate}<>\"COMPLYING\",\"s 109N gate is \"&{gate}&\", so s 109E produces no repayment figure\",IF({floor_year}<>{made},\"gate benchmark year differs from year_loan_made\",IF({made}={YEAR},\"year of income is the year the loan was made (s 109E(1)(a), s 109P)\",IF(IFERROR({year_start}>{YEAR_START},FALSE),\"loan made after the year of income\",IF(AND(ISNUMBER({term_used}),{term_used}<=0),\"nil remaining term under s 109E(6)\",\"nil benchmark rate\"))))),IF({v}=\"UNKNOWN\",IF({unknown_made},\"year_loan_made not established\",IF(NOT(ISNUMBER({YEAR_RATE})),\"no reviewed benchmark rate for the year of income\",IF(NOT(ISNUMBER({principal})),\"unpaid balance at end of previous year not established\",IF(NOT(ISNUMBER({payments})),\"payments applied not established\",\"remaining term not established\")))),\"\"))"
        ),
    );
    calc.insert(
        "MYR_required",
        format!("=IF(OR({v}=\"MYR_MET\",{v}=\"MYR_SHORT\"),{myr_raw},\"\")"),
    );
    calc.insert(
        "Shortfall",
        format!(
            "=IF(OR({v}=\"MYR_MET\",{v}=\"MYR_SHORT\"),MAX(0,ROUND({myr_raw}-ROUND({payments},2),2)),\"\")"
        ),
    );
    calc.insert(
        "Exposure",
        format!("=IF({v}=\"MYR_SHORT\",{},\"\")", t("Shortfall")),
    );
    calc.insert(
        "Undecided",
        format!("=IF(AND({status}=\"REVIEWED\",OR({gate}=\"UNKNOWN\",{v}=\"UNKNOWN\")),1,0)"),
    );
    // One ISFORMULA per cell: a multi-cell ISFORMULA inside SUMPRODUCT
    // implicitly intersects in a plain formula and a pasted formula slipped
    // through unflagged.
    calc.insert(
        "Guard",
        format!(
            "=IF(OR({}),1,0)",
            INPUT_COLUMNS
                .iter()
                .map(|c| format!("_xlfn.ISFORMULA({})", t(c)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    );
    // What the engine reads, and when. register._skip_reason returns on a
    // nonblank out_of_scope_reason before it parses anything, so such a row
    // carries no input problem at all. Otherwise it parses year_loan_made to
    // decide whether the loan is pre-1998, so that one label has to be
    // readable even on a row the workbook then skips: `1800-01` is exit 1 in
    // the CLI, not a silent historical skip. Every remaining fact is parsed
    // after the skip, so it is validated only on a reviewed row: blocking the
    // workbook over deliberately unused values refused a file the CLI reads
    // with exit 0. The formula guard stays on every row.
    let bools = BOOL_COLUMNS
        .iter()
        .map(|c| bool_bad(c))
        .collect::<Vec<_>>()
        .join(",");
    let numbers = NUMBER_COLUMNS
        .iter()
        .map(|c| number_bad(c))
        .collect::<Vec<_>>()
        .join(",");
    calc.insert(
        "Input_problem",
        format!(
            "=IF(TRIM({out_of_scope}&\"\")<>\"\",\"\",TRIM(IF(OR({},AND({status}=\"REVIEWED\",{})),\"year label \",\"\")&IF({status}<>\"REVIEWED\",\"\",IF(OR({bools}),\"true/false/unknown \",\"\")&IF(OR({numbers}),\"number \",\"\"))))",
            year_bad("year_loan_made"),
            year_bad("year_of_income_being_tested")
        ),
    );
    // A fabricated loan left in the register would count in the summary; flag
    // the loan_id and year pairs the shipped sample carries (filled in by
    // build()).
    calc.insert(
        "Sample_row",
        format!(
            "=IF(SUMPRODUCT(({}&\"\"={{SAMPLE_IDS}})*({made}&\"\"={{SAMPLE_YEARS}}))>0,1,0)",
            t("loan_id")
        ),
    );
    for (column, helper) in YEAR_HELPERS {
        calc.insert(helper, format!("={}", normalise_year(column)));
    }
    calc
}

/// The sample register's rows as DictReader reads them.
fn read_sample() -> Result<Vec<Raw>> {
    let path = PyPath::new(&format!("{ROOT}/examples/sample_loans_mixed.csv"));
    load_rows(&path, &[]).map_err(|e| format!("sample_loans_mixed.csv: {e}").into())
}

/// A register cell's raw text; a missing cell reads as blank.
fn raw_text(row: &Raw, column: &str) -> String {
    match row.get(column) {
        None | Some(Raw::None) => String::new(),
        Some(raw) => raw.py_str(),
    }
}

/// A CSV token as the workbook stores it.
enum Value {
    Blank,
    Number(f64),
    Text(String),
}

/// Numbers as numbers, everything else text. The number test is Python's
/// `float()` over the shipped sample: surrounding whitespace is ignored.
fn cell_value(column: &str, raw: &str) -> Value {
    if raw.is_empty() {
        return Value::Blank;
    }
    if TEXT_COLUMNS.contains(&column) || BOOL_COLUMNS.contains(&column) {
        return Value::Text(raw.to_string());
    }
    match raw.trim_matches(py::is_space).parse::<f64>() {
        Ok(number) => Value::Number(number),
        Err(_) => Value::Text(raw.to_string()),
    }
}

/// Cell formats, as the Python builder's style dictionaries compose them.
struct Styles {
    head: Format,
    input: Format,
    calc: Format,
}

impl Styles {
    fn new() -> Styles {
        Styles {
            head: Format::new()
                .set_bold()
                .set_font_color(Color::RGB(0xFFFFFF))
                .set_background_color(Color::RGB(PURPLE)),
            input: Format::new()
                .set_font_color(Color::RGB(0x1F4E9E))
                .set_background_color(Color::RGB(GREY)),
            calc: Format::new().set_background_color(Color::RGB(LAVENDER)),
        }
    }
}

fn header(ws: &mut Worksheet, titles: &[&str], head: &Format) -> Result<()> {
    for (c, title) in titles.iter().enumerate() {
        ws.write_string_with_format(0, c as u16, *title, head)?;
    }
    Ok(())
}

/// A light-striped table with no autofilter over `titles`, whose header cells
/// keep the head style. `formulas` gives calculated columns with their `=`.
fn add_table(
    ws: &mut Worksheet,
    name: &str,
    titles: &[&str],
    last_row: u32,
    formulas: &BTreeMap<&str, String>,
    head: &Format,
) -> Result<()> {
    let columns: Vec<TableColumn> = titles
        .iter()
        .map(|title| {
            let column = TableColumn::new()
                .set_header(*title)
                .set_header_format(head);
            match formulas.get(title) {
                Some(f) => column.set_formula(Formula::new(&f[1..])),
                None => column,
            }
        })
        .collect();
    let table = Table::new()
        .set_name(name)
        .set_columns(&columns)
        .set_autofilter(false)
        .set_style(TableStyle::Light1);
    ws.add_table(0, 0, last_row, titles.len() as u16 - 1, &table)?;
    header(ws, titles, head)
}

/// Excel character widths, stored exactly as openpyxl stores them: 7 pixels
/// per character at the default font.
fn width(ws: &mut Worksheet, col: u16, chars: u32) -> Result<()> {
    ws.set_column_width_pixels(col, chars * 7)?;
    Ok(())
}

/// Sheet protection as openpyxl writes `protection.sheet = True`: objects and
/// scenarios stay editable.
fn protect(ws: &mut Worksheet) {
    ws.protect_with_options(&ProtectionOptions {
        edit_objects: true,
        edit_scenarios: true,
        ..ProtectionOptions::default()
    });
}

fn position(titles: &[&str], title: &str) -> u16 {
    titles.iter().position(|x| *x == title).expect("a title") as u16
}

pub fn build() -> Result<Vec<u8>> {
    let rows = read_sample()?;
    let rates = load_table(&PyPath::new(&format!("{ROOT}/data/benchmark_rates.csv")))
        .map_err(|e| format!("benchmark_rates.csv: {e}"))?;
    let mut calc = formulas();
    let column = |column: &str| -> Vec<String> {
        rows.iter()
            .map(|r| format!("\"{}\"", raw_text(r, column)))
            .collect()
    };
    let sample_ids = column("loan_id").join(",");
    let sample_years = column("year_loan_made").join(",");
    let sample = calc.get_mut("Sample_row").expect("Sample_row");
    *sample = sample
        .replace("{SAMPLE_IDS}", &format!("{{{sample_ids}}}"))
        .replace("{SAMPLE_YEARS}", &format!("{{{sample_years}}}"));
    let calcs = all_calcs();
    assert!(calc.keys().copied().eq({
        let mut order = calcs.clone();
        order.sort_unstable();
        order
    }));
    let s = Styles::new();
    let mut wb = Workbook::new();

    // 1. Start Here
    let ws = wb.add_worksheet().set_name("Start Here")?;
    ws.write_string_with_format(
        0,
        0,
        "Division 7A loan review: s 109N terms and s 109E minimum yearly repayment",
        &Format::new()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::RGB(PURPLE)),
    )?;
    let steps = [
        "1. On Register, paste one amalgamated loan per row over the example rows, using the same columns as the CSV the command line tool reads. Booleans are true, false or unknown; a blank is unknown. Rates are fractions (8.77 per cent is 0.0877). Years are written 2026-27.",
        "2. On Summary, choose the year of income to review. The s 109N gate reads each loan's benchmark from the year the loan was made; the repayment uses the chosen year's rate.",
        "3. Read the calculated columns on Register: each s 109N(1) limb, the gate verdict, the s 109E(6) repayment, the shortfall and the experimental exposure. REFUSED and UNKNOWN are answers, never a number; the reason column says why.",
        "4. Review Checks: BLOCKED means the register cannot be read (a bad year label, a value that is not true, false or unknown, text where a number belongs, a formula pasted into an input, or no reviewed rate for the year). REVIEW means a loan is not on s 109N terms, a repayment is short, a row is undecided, or a repayment figure is refused (read the reason; refusal alone is not a breach). PASS means nothing exposed, nothing undecided.",
        "5. A rising benchmark rate raises the minimum yearly repayment on existing complying loans, not just on new ones: s 109E(6) uses the current year's rate.",
        "6. Experimental review aid. Not a Division 7A determination. The shortfall is not the dividend (s 109Y, s 109Q and s 109RB are not modelled) and s 109R is not applied.",
    ];
    let wrap_top = Format::new().set_text_wrap().set_align(FormatAlign::Top);
    for (i, step) in steps.iter().enumerate() {
        ws.write_string_with_format(2 + i as u32, 0, *step, &wrap_top)?;
    }
    width(ws, 0, 110)?;
    // openpyxl writes a bold-only font with no colour element: automatic.
    let bold = Format::new().set_bold().set_font_color(Color::Automatic);
    ws.write_string_with_format(9, 0, "Overall status", &bold)?;
    ws.write_formula_with_format(
        10,
        0,
        Formula::new("='Review Checks'!B10"),
        &s.calc
            .clone()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::Automatic),
    )?;
    ws.write_string(
        12,
        0,
        format!(
            "div7a-loan-review workbook, engine {VERSION}. Macro-free. Not advice; see DISCLAIMER.md in the repository."
        ),
    )?;

    // 2. Register: inputs then calculated columns in one table
    let ws = wb.add_worksheet().set_name("Register")?;
    let titles: Vec<&str> = INPUT_COLUMNS.iter().copied().chain(calcs).collect();
    let last_row = rows.len() as u32;
    add_table(ws, "tblLoans", &titles, last_row, &calc, &s.head)?;
    for (r, row) in rows.iter().enumerate() {
        let r = r as u32 + 1;
        for (c, column) in INPUT_COLUMNS.iter().enumerate() {
            let c = c as u16;
            let mut format = s.input.clone();
            if TEXT_COLUMNS.contains(column) {
                format = format.set_num_format(TEXT);
            } else if MONEY_COLUMNS.contains(column) {
                format = format.set_num_format(MONEY);
            }
            match cell_value(column, &raw_text(row, column)) {
                Value::Blank => ws.write_blank(r, c, &format)?,
                Value::Number(n) => ws.write_number_with_format(r, c, n, &format)?,
                Value::Text(text) => ws.write_string_with_format(r, c, text, &format)?,
            };
        }
        for (c, column) in titles.iter().enumerate().skip(INPUT_COLUMNS.len()) {
            let format = if MONEY_COLUMNS.contains(column) {
                s.calc.clone().set_num_format(MONEY)
            } else {
                s.calc.clone()
            };
            ws.write_formula_with_format(r, c as u16, Formula::new(&calc[column]), &format)?;
        }
    }
    for column in BOOL_COLUMNS {
        let c = position(&INPUT_COLUMNS, column);
        let rule = DataValidation::new()
            .allow_list_strings(&["true", "false", "unknown"])?
            .show_input_message(false)
            .show_error_message(false);
        ws.add_data_validation(1, c, last_row + 500, c, &rule)?;
    }
    for (c, column) in titles.iter().enumerate() {
        let c = c as u16;
        width(
            ws,
            c,
            if MONEY_COLUMNS.contains(column) {
                14
            } else {
                12
            },
        )?;
        if YEAR_HELPERS.iter().any(|(_, h)| h == column) {
            ws.set_column_hidden(c)?;
        }
    }
    width(ws, 0, 10)?;
    width(ws, position(&titles, "MYR_reason"), 40)?;
    width(ws, position(&titles, "out_of_scope_reason"), 24)?;
    ws.set_freeze_panes(1, 1)?;

    // 3. Rates: editable, so a reviewed override is a row the accountant adds
    // and cites
    let ws = wb.add_worksheet().set_name("Rates")?;
    let rate_cols = [
        "year_of_income",
        "rate",
        "rba_table",
        "rba_series",
        "rba_month",
        "source",
        "seen",
    ];
    add_table(
        ws,
        "tblRates",
        &rate_cols,
        rates.entries.len() as u32,
        &BTreeMap::new(),
        &s.head,
    )?;
    let input_text = s.input.clone().set_num_format(TEXT);
    for (r, entry) in rates.entries.values().enumerate() {
        let r = r as u32 + 1;
        ws.write_string_with_format(r, 0, entry.year.label(), &input_text)?;
        let rate: f64 = entry.rate.to_sci_string().parse()?;
        ws.write_number_with_format(r, 1, rate, &s.input)?;
        for (c, text) in [
            &entry.rba_table,
            &entry.rba_series,
            &entry.rba_month,
            &entry.source,
            &entry.seen,
        ]
        .into_iter()
        .enumerate()
        {
            ws.write_string_with_format(r, 2 + c as u16, text, &s.input)?;
        }
    }
    width(ws, 5, 90)?;
    ws.write_string(
        rates.entries.len() as u32 + 3,
        0,
        "To review a year outside this table, add a row with the May RBA F5 FILRHLBVS figure and cite where you read it. A blank source is not a review.",
    )?;

    // 4. Summary
    let ws = wb.add_worksheet().set_name("Summary")?;
    ws.write_string_with_format(0, 0, "Review", &s.head)?;
    ws.write_blank(0, 1, &s.head)?;
    ws.write_string(1, 0, "Year of income under review")?;
    ws.write_string_with_format(
        1,
        1,
        DEFAULT_YEAR,
        &s.input.clone().set_num_format(TEXT).set_unlocked(),
    )?;
    // The Rates sheet tells the reviewer to add a year outside the frozen
    // table, so the selector has to grow with it. A fixed A2:A<n> range froze
    // it at build time and the added year could never be chosen. Excel refuses
    // a structured reference directly in a validation formula, so it goes
    // through a defined name, which is the one form that both resolves to the
    // table column and expands with it.
    let rule = DataValidation::new()
        .allow_list_formula(Formula::new("RateYears"))
        .ignore_blank(false)
        .show_input_message(false)
        .show_error_message(false);
    ws.add_data_validation(1, 1, 1, 1, &rule)?;
    ws.write_string(2, 0, "Benchmark interest rate for that year (s 109N(2))")?;
    ws.write_formula_with_format(
        2,
        1,
        Formula::new("=IFERROR(INDEX(tblRates[rate],MATCH(B2,tblRates[year_of_income],0)),\"\")"),
        &s.calc.clone().set_num_format("0.00%"),
    )?;
    ws.write_string(3, 0, "Year start")?;
    ws.write_formula_with_format(
        3,
        1,
        Formula::new("=IFERROR(VALUE(LEFT(B2,4)),\"\")"),
        &s.calc,
    )?;
    ws.write_string_with_format(
        5,
        0,
        "Summary (counts are per question, not per row)",
        &s.head,
    )?;
    ws.write_blank(5, 1, &s.head)?;
    let summary = [
        ("rows_reviewed", "=COUNTIF(tblLoans[Status],\"REVIEWED\")"),
        (
            "COMPLYING",
            "=COUNTIF(tblLoans[Gate_verdict],\"COMPLYING\")",
        ),
        (
            "NOT_COMPLYING",
            "=COUNTIF(tblLoans[Gate_verdict],\"NOT_COMPLYING\")",
        ),
        ("MYR_MET", "=COUNTIF(tblLoans[MYR_verdict],\"MYR_MET\")"),
        ("MYR_SHORT", "=COUNTIF(tblLoans[MYR_verdict],\"MYR_SHORT\")"),
        ("UNKNOWN", "=SUM(tblLoans[Undecided])"),
        ("REFUSED", "=COUNTIF(tblLoans[MYR_verdict],\"REFUSED\")"),
        ("SKIPPED", "=COUNTIF(tblLoans[Status],\"SKIPPED\")"),
        ("experimental_total_exposure", "=SUM(tblLoans[Exposure])"),
    ];
    let calc_money = s.calc.clone().set_num_format(MONEY);
    for (i, (label, formula)) in summary.iter().enumerate() {
        let r = 6 + i as u32;
        ws.write_string(r, 0, *label)?;
        // B15, the exposure total, is money.
        let format = if r == 14 { &calc_money } else { &s.calc };
        ws.write_formula_with_format(r, 1, Formula::new(*formula), format)?;
    }
    ws.write_string(16, 0, "Overall status")?;
    let calc_bold = s.calc.clone().set_bold().set_font_color(Color::Automatic);
    ws.write_formula_with_format(16, 1, Formula::new("='Review Checks'!B10"), &calc_bold)?;
    width(ws, 0, 52)?;
    width(ws, 1, 18)?;
    protect(ws);
    wb.define_name("RateYears", "=tblRates[year_of_income]")?;

    // 5. Review Checks
    let ws = wb.add_worksheet().set_name("Review Checks")?;
    header(
        ws,
        &["Check", "Result", "Count or value", "Example loan"],
        &s.head,
    )?;
    let offender = |cond: &str| {
        format!("INDEX(tblLoans[loan_id],SUMPRODUCT(MAX({cond}*(ROW(tblLoans[loan_id])-1))))")
    };
    let example = |cell: &str, cond: &str| Some(format!("=IF({cell}=0,\"\",{})", offender(cond)));
    let checks: [(&str, &str, &str, Option<String>); 8] = [
        (
            "Every register value can be read",
            "=IF(C2=0,\"PASS\",\"BLOCKED\")",
            "=SUMPRODUCT(--(tblLoans[Input_problem]<>\"\"))",
            example("C2", "(tblLoans[Input_problem]<>\"\")"),
        ),
        (
            "No formula in an input cell",
            "=IF(C3=0,\"PASS\",\"BLOCKED\")",
            "=SUM(tblLoans[Guard])",
            example("C3", "(tblLoans[Guard]=1)"),
        ),
        (
            "A reviewed benchmark exists, rate years are unique and rates are from zero to one",
            "=IFERROR(IF(AND(ISNUMBER(Summary!B3),COUNT(tblRates[rate])=ROWS(tblRates[rate]),SUMPRODUCT(--(COUNTIF(tblRates[year_of_income],tblRates[year_of_income])<>1))=0,MIN(tblRates[rate])>=0,MAX(tblRates[rate])<=1),\"PASS\",\"BLOCKED\"),\"BLOCKED\")",
            "=Summary!B2",
            None,
        ),
        (
            "Every reviewed loan is on s 109N terms",
            "=IF(C5=0,\"PASS\",\"REVIEW\")",
            "=COUNTIF(tblLoans[Gate_verdict],\"NOT_COMPLYING\")",
            example("C5", "(tblLoans[Gate_verdict]=\"NOT_COMPLYING\")"),
        ),
        (
            "No minimum yearly repayment is short",
            "=IF(C6=0,\"PASS\",\"REVIEW\")",
            "=COUNTIF(tblLoans[MYR_verdict],\"MYR_SHORT\")",
            example("C6", "(tblLoans[MYR_verdict]=\"MYR_SHORT\")"),
        ),
        (
            "No row is undecided (UNKNOWN)",
            "=IF(C7=0,\"PASS\",\"REVIEW\")",
            "=SUM(tblLoans[Undecided])",
            example("C7", "(tblLoans[Undecided]=1)"),
        ),
        (
            "Rows refused a repayment figure (read the reason; not a breach by itself)",
            "=IF(C8=0,\"PASS\",\"REVIEW\")",
            "=COUNTIF(tblLoans[MYR_verdict],\"REFUSED\")",
            example("C8", "(tblLoans[MYR_verdict]=\"REFUSED\")"),
        ),
        (
            "No fabricated example loan from the shipped sample remains in the register",
            "=IF(C9=0,\"PASS\",\"REVIEW\")",
            "=SUM(tblLoans[Sample_row])",
            example("C9", "(tblLoans[Sample_row]=1)"),
        ),
    ];
    for (i, (label, result, detail, example)) in checks.iter().enumerate() {
        let r = 1 + i as u32;
        ws.write_string(r, 0, *label)?;
        ws.write_formula_with_format(r, 1, Formula::new(*result), &s.calc)?;
        ws.write_formula_with_format(r, 2, Formula::new(*detail), &s.calc)?;
        match example {
            Some(f) => ws.write_array_formula_with_format(r, 3, r, 3, Formula::new(f), &s.calc)?,
            None => ws.write_blank(r, 3, &s.calc)?,
        };
    }
    let last = checks.len() + 1;
    ws.write_string_with_format(9, 0, "Overall status", &bold)?;
    ws.write_formula_with_format(
        9,
        1,
        Formula::new(format!(
            "=IF(COUNTIF(B2:B{last},\"BLOCKED\")>0,\"BLOCKED\",IF(COUNTIF(B2:B{last},\"REVIEW\")>0,\"REVIEW\",\"PASS\"))"
        )),
        &calc_bold,
    )?;
    ws.write_string_with_format(
        11,
        0,
        "Caveats the engine attaches to every result",
        &s.head,
    )?;
    let caveats = [
        "Interest-floor interpretation unresolved: ATO guidance requires the benchmark interest rate for each year of the loan. This workbook compares one selected year's benchmark and defaults to the year the loan was made. COMPLYING does not establish that the agreement meets the annual interest requirement. Review the agreement and relevant years before relying on this result. See docs/primary-source-review-2026-08-31.md. The lodgment day (s 109D(6)) is asserted, not computed. Refinancing reductions to the maximum term (s 109N(3A) to (3D)) and the other Subdivision D exclusions are not modelled.",
        "s 109R: the workbook does not decide whether the payments applied are genuine repayments. The figure used is the operator's assertion.",
        "Any shortfall shown is an experimental review aid, not an ATO assessment and not a s 109E(1) determination. s 109E(2) makes the dividend the shortfall subject to s 109Y, and s 109E(1)(d) removes it where s 109Q applies. Neither is modelled, nor is s 109RB.",
        "The s 109E(6) amount is rounded to cents half up. The Act prescribes no rounding; this is the engine's documented choice. A fractional remaining term is rounded up to a whole number under the closing words of s 109E(6).",
        "A loan year before 1998-99 is skipped: Division 7A reaches a loan made before 4 December 1997 only where its terms were varied on or after that day (s 109D(5)).",
    ];
    let wrap = Format::new().set_text_wrap();
    for (i, caveat) in caveats.iter().enumerate() {
        ws.write_string_with_format(12 + i as u32, 0, *caveat, &wrap)?;
    }
    ws.set_row_height(12, 135)?;
    for (c, chars) in [(0, 100), (1, 12), (2, 16), (3, 16)] {
        width(ws, c, chars)?;
    }
    protect(ws);

    // 6. Sources & Version
    let ws = wb.add_worksheet().set_name("Sources & Version")?;
    header(ws, &["Item", "Value"], &s.head)?;
    let items = [
        ("Workbook engine version", VERSION.to_string()),
        (
            "Desktop Excel build used to calculate cached values",
            String::new(),
        ),
        (
            "Law",
            "Income Tax Assessment Act 1936 (Cth) Part III Division 7A, C1936A00027, compilation in force 1 July 2026: s 109D, 109E, 109N, 109P, 109R".into(),
        ),
        (
            "Law source",
            "https://www.legislation.gov.au/C1936A00027/2026-07-01/text".into(),
        ),
        (
            "Benchmark rates",
            "RBA statistical table F5 Indicator Lending Rates, series FILRHLBVS, May figure for each year of income (s 109N(2))".into(),
        ),
        ("Rates reviewed until", rates.reviewed_until.label()),
        ("Rates reviewed on", rates.reviewed_on.clone()),
        (
            "Rates explainer",
            "https://duguid.com.au/rates/div7a-benchmark-rate/".into(),
        ),
        (
            "Disclaimer",
            "Experimental review aid. Not a Division 7A determination, not an ATO assessment, and not tax, legal or financial advice.".into(),
        ),
    ];
    for (i, (key, value)) in items.iter().enumerate() {
        let r = 1 + i as u32;
        ws.write_string(r, 0, *key)?;
        if !value.is_empty() {
            ws.write_string(r, 1, value)?;
        }
    }
    width(ws, 0, 52)?;
    width(ws, 1, 110)?;

    // Status colours, added last as the Python builder adds them.
    let fills = [
        ("BLOCKED", 0xF8D7DA),
        ("REVIEW", 0xFFF3CD),
        ("PASS", 0xD4EDDA),
    ];
    for (sheet, (r1, c, r2)) in [
        ("Review Checks", (1, 1, 9)),
        ("Start Here", (10, 0, 10)),
        ("Summary", (16, 1, 16)),
    ] {
        let ws = wb.worksheet_from_name(sheet)?;
        for (word, colour) in fills {
            let rule = ConditionalFormatCell::new()
                .set_rule(ConditionalFormatCellRule::EqualTo(format!("\"{word}\"")))
                .set_format(Format::new().set_background_color(Color::RGB(colour)));
            ws.add_conditional_format(r1, c, r2, c, &rule)?;
        }
    }

    // openpyxl's print margins, which Excel keeps on save; rust_xlsxwriter
    // otherwise writes Excel's narrower defaults. Setting them writes a page
    // setup, whose paper would then default to Letter, so name A4 (paper 9),
    // what the Python workbook prints on with an Australian printer default.
    for ws in wb.worksheets_mut() {
        ws.set_margins(0.75, 0.75, 1.0, 1.0, 0.5, 0.5);
        ws.set_paper_size(9);
    }

    Ok(wb.save_to_buffer()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{differences, model};

    /// Accepted, as for the payday workbook after its Gate B desktop Excel
    /// check:
    /// - openpyxl writes a status colour as a `solid` dxf fill with fgColor,
    ///   which Excel does not display; rust_xlsxwriter writes bgColor, the form
    ///   Excel writes and displays, so the Rust workbook shows the colours;
    /// - setting openpyxl's margins makes rust_xlsxwriter write a page setup
    ///   with portrait orientation, A4 paper and 200 dpi, where openpyxl leaves
    ///   them to the printer.
    fn accepted(line: &str) -> String {
        line.replace("dxf{fill{solid fgColor=", "dxf{fill{(absent) bgColor=")
            .replace(
                "| page orientation=default paperSize=1 scale=100 fitToWidth=1 fitToHeight=1 horizontalDpi=600 verticalDpi=600 ",
                "| page orientation=portrait paperSize=9 scale=100 fitToWidth=1 fitToHeight=1 horizontalDpi=200 verticalDpi=200 ",
            )
    }

    #[test]
    fn the_workbook_matches_the_frozen_python_model() {
        // The model of tools/build_workbook.py --no-excel at 175e64c.
        let frozen: Vec<String> = include_str!("../../tests/data/workbook-model.txt")
            .lines()
            .map(accepted)
            .collect();
        let built = model(&super::build().unwrap()).unwrap();
        let found = differences(&frozen, &built);
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    const SKIP_GATE: &str = "tblLoans[[#This Row],[Status]]<>\"REVIEWED\",\"\"";

    /// register._skip_reason skips a nonblank reason; "unknown" is a reason,
    /// not a blank.
    #[test]
    fn any_nonblank_exclusion_reason_skips_the_row() {
        let status = &formulas()["Status"];
        assert!(status.contains("TRIM(tblLoans[[#This Row],[out_of_scope_reason]]&\"\")<>\"\""));
        // The unknown-token helper is not what decides scope any more.
        assert!(!unknown(&t("year_loan_made")).contains("out_of_scope_reason"));
        assert!(!status.contains(&unknown(&t("out_of_scope_reason"))));
    }

    /// The engine skips before parsing, so unused facts on a skipped row block
    /// nothing.
    #[test]
    fn only_reviewed_rows_are_validated() {
        let calc = formulas();
        let problem = &calc["Input_problem"];
        assert!(
            problem.starts_with(
                "=IF(TRIM(tblLoans[[#This Row],[out_of_scope_reason]]&\"\")<>\"\",\"\","
            )
        );
        // The bool and number tests are the ones the engine reaches only after
        // the skip.
        for guard in [
            bool_bad("written_agreement"),
            number_bad("payments_applied_during_the_year"),
        ] {
            let head = problem.find(&guard).expect("guard");
            assert!(problem[..head].contains(SKIP_GATE));
        }
        // The formula guard stays on every row: a pasted formula is a tamper
        // signal.
        assert!(!calc["Guard"].contains("Status"));
    }

    /// register._skip_reason parses year_loan_made to decide whether the loan
    /// is pre-1998. An out-of-bounds label such as `1800-01` is exit 1 in the
    /// CLI, so the workbook cannot suppress it as a silent historical skip.
    /// Only a row the operator has marked out of scope escapes the label test,
    /// because the engine returns on that reason before it parses anything.
    #[test]
    fn the_loan_year_label_is_read_before_the_historical_skip() {
        let problem = &formulas()["Input_problem"];
        let made = year_bad("year_loan_made");
        let tested = year_bad("year_of_income_being_tested");
        let at = problem.find(&made).expect("year_loan_made guard");
        assert!(!problem[..at].contains(SKIP_GATE));
        // The control: the other year label is parsed after the skip, so it is
        // gated.
        assert!(problem.contains(&format!(
            "AND(tblLoans[[#This Row],[Status]]=\"REVIEWED\",{tested})"
        )));
    }

    #[test]
    fn the_year_guard_matches_the_engine_grammar_and_bounds() {
        let guard = year_bad("year_loan_made");
        assert!(guard.contains(&year_text("year_loan_made")));
        assert!(guard.contains(&format!(">={EARLIEST}")) && guard.contains(&format!("<={LATEST}")));
        // Every one of the 6 digit positions is tested, so VALUE cannot read
        // 20e2 as 2000.
        for position in [1, 2, 3, 4, 6, 7] {
            assert!(guard.contains(&format!(",{position},1)))")), "{position}");
        }
    }

    #[test]
    fn the_money_guard_carries_the_engine_limits() {
        for column in [
            "amalgamated_loan_unpaid_at_end_of_previous_year",
            "payments_applied_during_the_year",
        ] {
            let guard = number_bad(column);
            let cell = t(column);
            assert!(guard.contains(&format!(">{MAX_MONEY_MAGNITUDE}")));
            assert!(guard.contains(&format!(
                "ROUND(N({cell}),{MAX_MONEY_DECIMAL_PLACES})<>N({cell})"
            )));
            assert_eq!(MAX_MONEY_DECIMAL_PLACES, 2);
        }
        // parse_rate and parse_ratio impose neither limit, so neither does the
        // workbook.
        for column in [
            "interest_rate_for_years_after_year_loan_made",
            "security_coverage_at_first_made",
        ] {
            let guard = number_bad(column);
            assert!(!guard.contains(MAX_MONEY_MAGNITUDE));
            assert!(!guard.contains("ROUND("));
        }
    }

    /// AND and OR evaluate every argument, so the arithmetic must not error.
    /// ROUND on a cell holding `unknown` returns #VALUE!, and the error
    /// propagates out through the ISNUMBER guard that was there to catch it:
    /// the shipped sample's L-104, whose payments column is `unknown`, turned
    /// Input_problem and the whole 'Every register value can be read' check
    /// into #VALUE!.
    #[test]
    fn the_money_guard_survives_a_text_cell() {
        for column in [
            "amalgamated_loan_unpaid_at_end_of_previous_year",
            "payments_applied_during_the_year",
        ] {
            let guard = number_bad(column);
            let cell = t(column);
            assert!(!guard.contains(&format!("ROUND({cell},")));
            assert!(guard.contains(&format!("ROUND(N({cell}),2)<>N({cell})")));
            // The control: the raw cell is still what ISNUMBER and the sign
            // test read.
            assert!(guard.contains(&format!("ISNUMBER({cell})")));
            assert!(guard.contains(&format!("{cell}<0")));
        }
    }

    #[test]
    fn scope_and_lookup_read_the_trimmed_year() {
        let calc = formulas();
        for name in ["Status", "Floor_year"] {
            assert!(!calc[name].contains("LEFT(tblLoans[[#This Row],[year_loan_made]]"));
        }
        assert!(calc["Floor_year"].contains(&year_text("year_loan_made")));
        // The repayment verdict compares the same trimmed label. Comparing the
        // raw cell with the trimmed Floor_year made ` 2023-24 ` REFUSED for a
        // benchmark-year mismatch against itself, where the engine trims and
        // returns MYR_MET, exit 0.
        for name in ["MYR_verdict", "MYR_reason"] {
            assert!(calc[name].contains(&year_text("year_loan_made")));
            assert!(!calc[name].contains(&t("year_loan_made")));
        }
    }

    /// The engine half (every such character trims around a year and an
    /// unknown token, and none inside one) is the years.rs unit tests.
    #[test]
    fn year_whitespace_matches_the_engine_without_erasing_internal_gaps() {
        let whitespace: Vec<char> = (0..=0x10FFFF)
            .filter_map(char::from_u32)
            .filter(|c| py::is_space(*c))
            .collect();
        assert_eq!(YEAR_WHITESPACE.to_vec(), whitespace);
        for (column, _) in YEAR_HELPERS {
            let formula = normalise_year(column);
            assert!(!formula.contains("CLEAN("));
            for character in whitespace.iter().filter(|c| **c != ' ') {
                assert!(
                    formula.contains(&format!("UNICHAR({}),\" \")", u32::from(*character))),
                    "{character:?}"
                );
            }
        }
        for character in ['\u{200b}', '\u{feff}'] {
            assert!(!whitespace.contains(&character));
        }
    }

    #[test]
    fn every_semantic_year_read_uses_the_shared_helper() {
        for (name, formula) in formulas() {
            for (column, helper) in YEAR_HELPERS {
                if name == "Guard" || name == helper {
                    continue;
                }
                assert!(!formula.contains(&t(column)), "{name} reads {column}");
            }
        }
    }

    #[test]
    fn missing_floor_year_precedes_gate_failures_and_maximum_term() {
        let calc = formulas();
        let no_year = unknown(&t("Floor_year"));
        let gate = &calc["Gate_verdict"];
        assert!(gate.contains(&format!("IF({no_year},\"UNKNOWN\",")));
        assert!(gate.find(&no_year) < gate.find(&t("Limb_a_written")));
        let maximum = &calc["Max_term_allowed"];
        let at = maximum.find(&no_year).expect("floor year test");
        assert!(at < maximum.find(&t("Allowed_term_raw")).expect("allowed term"));
        // An unreviewed rate is different: the maximum term can still be
        // established.
        assert!(!maximum.contains(&t("Floor_rate")));
    }

    /// Existing native probes and consumers use these column positions. The
    /// helpers come last, hidden, and are calculated columns on every row.
    #[test]
    fn year_helpers_extend_the_table_without_moving_public_columns() {
        let titles: Vec<&str> = INPUT_COLUMNS.iter().copied().chain(all_calcs()).collect();
        assert_eq!(INPUT_COLUMNS.len(), 14);
        assert_eq!(position(&titles, "Gate_verdict"), 23);
        assert_eq!(position(&titles, "Max_term_allowed"), 24);
        assert_eq!(position(&titles, "Input_problem"), 34);
        assert_eq!(
            &titles[titles.len() - 2..],
            ["Year_made_text", "Year_tested_text"]
        );
        let built = model(&build().unwrap()).unwrap();
        let rows = read_sample().unwrap().len() as u32;
        for (_, helper) in YEAR_HELPERS {
            let c = u32::from(position(&titles, helper)) + 1;
            let letters = if c <= 26 {
                char::from(b'A' + c as u8 - 1).to_string()
            } else {
                format!("A{}", char::from(b'A' + (c - 27) as u8))
            };
            assert!(
                built.iter().any(|l| l
                    .starts_with(&format!("sheet Register | col {letters} width="))
                    && l.contains(" hidden=1 ")),
                "{helper} hidden"
            );
            assert!(
                built.iter().any(|l| l.starts_with(&format!(
                    "table tblLoans | column {c} \"{helper}\" formula="
                ))),
                "{helper} calculated"
            );
            for r in 2..=rows + 1 {
                let cell = format!("sheet Register | cell {letters}{r} formula ");
                assert!(built.iter().any(|l| l.starts_with(&cell)), "{cell}");
            }
        }
    }

    #[test]
    fn changed_formulas_fit_excels_formula_limit() {
        for (name, formula) in formulas() {
            assert!(formula.chars().count() <= 8192, "{name}");
        }
    }
}
