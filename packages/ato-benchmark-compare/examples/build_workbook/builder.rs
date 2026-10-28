//! The workbook before its desktop Excel pass. Formula text is that of the
//! Python builder it replaced (tools/build_workbook.py), character for
//! character; the frozen model of the Python output holds the two together.
//!
//! The workbook is a second implementation of the engine's rules written in
//! ordinary worksheet formulas, so an accountant can run the comparison with
//! desktop Excel and nothing else.

use std::collections::BTreeSet;
use std::error::Error;

use ato_benchmark_compare::VERSION;
use ato_benchmark_compare::dataset::{self, DataDir, Dataset, ratio_label};
use ato_benchmark_compare::mapping::{BUCKETS, SOURCE_SUGGESTED};
use ato_benchmark_compare::pnl::csv_rows;
use ato_benchmark_compare::py;
use ato_benchmark_compare::raw::{Raw, float_repr};
use rust_xlsxwriter::{
    Color, ConditionalFormatCell, ConditionalFormatCellRule, DataValidation, Format, FormatAlign,
    Formula, ProtectionOptions, Table, TableColumn, TableStyle, Workbook, Worksheet,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

const PURPLE: u32 = 0x5C2D91;
const LAVENDER: u32 = 0xF3F1F6;
const GREY: u32 = 0xE2E0DF;
const TEXT: &str = "@";
const MONEY: &str = r#"_(* #,##0.00_);_(* (#,##0.00);_(* "-"??_);_(@_)"#;
const PCT: &str = "0.00%";

/// cost_of_sales .. other_expense on the Calculation sheet, the rows the
/// expense sign factor applies to.
const EXPENSE_ROWS: (u32, u32) = (10, 17);

/// Ratios whose ranges tblBands carries, in the Results sheet's row order,
/// with the table columns that hold each one's minimum and maximum, and the
/// Calculation cell that holds its numerator.
const RANGE_COLUMNS: [(&str, &str, &str, &str); 5] = [
    (
        "cost_of_sales_to_turnover",
        "CostOfSalesMin",
        "CostOfSalesMax",
        "B28",
    ),
    (
        "total_expenses_to_turnover",
        "TotalExpensesMin",
        "TotalExpensesMax",
        "B27",
    ),
    ("labour_to_turnover", "LabourMin", "LabourMax", "B31"),
    ("rent_to_turnover", "RentMin", "RentMax", "B15"),
    (
        "motor_vehicle_to_turnover",
        "MotorVehicleMin",
        "MotorVehicleMax",
        "B16",
    ),
];

/// Columns A to N keep their original order; the industry-page ranges, the
/// ratios they cover and that page's text digest follow in O to V.
const BAND_COLUMNS: [&str; 22] = [
    "Year",
    "Industry",
    "KeyRatio",
    "Band",
    "Label",
    "From",
    "FromInclusive",
    "To",
    "CostOfSalesMin",
    "CostOfSalesMax",
    "TotalExpensesMin",
    "TotalExpensesMax",
    "Retrieved",
    "Sha256",
    "LabourMin",
    "LabourMax",
    "RentMin",
    "RentMax",
    "MotorVehicleMin",
    "MotorVehicleMax",
    "PageRanges",
    "PageSha256",
];

fn bucket_names() -> impl Iterator<Item = &'static str> {
    BUCKETS.iter().map(|(name, _)| *name)
}

/// Python's `re.fullmatch(r"-?\d+(\.\d+)?", text)`, `\d` being any Unicode
/// decimal digit as in a str pattern.
fn is_amount(text: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.chars().all(py::is_decimal);
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    match unsigned.split_once('.') {
        Some((whole, fraction)) => digits(whole) && digits(fraction),
        None => digits(unsigned),
    }
}

fn read_csv(name: &str) -> Result<Vec<Vec<String>>> {
    let path = format!("{ROOT}/examples/{name}");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
    csv_rows(&text).map_err(|e| format!("{name}: {e}").into())
}

/// The shipped bakery sample.
struct Bakery {
    /// The P&L lines that carry an amount.
    pnl: Vec<(String, f64)>,
    /// Account, bucket, source and note of each mapping row.
    mapping: Vec<[String; 4]>,
}

/// The account, bucket, source and note columns of the mapping header. A
/// repeated one refuses: DictReader silently took its last value.
fn mapping_columns(header: &[String]) -> Result<[usize; 4]> {
    let column = |name: &str| {
        let mut at = (0..header.len()).filter(|&i| header[i] == name);
        match (at.next(), at.next()) {
            (Some(i), None) => Ok(i),
            (None, _) => Err(format!("bakery-mapping.csv: no {name} column")),
            _ => Err(format!("bakery-mapping.csv: repeated {name} column")),
        }
    };
    Ok([
        column("account")?,
        column("bucket")?,
        column("source")?,
        column("note")?,
    ])
}

/// The bakery sample as the Python builder read it, with csv.reader and
/// csv.DictReader.
fn read_bakery() -> Result<Bakery> {
    let mut pnl = Vec::new();
    for row in read_csv("bakery-pnl.csv")? {
        if row.len() > 1 && is_amount(&row[1]) {
            // float(Decimal(text)) is the correctly rounded double of the
            // text, as Rust's parse is; a non-ASCII digit refuses instead.
            let amount: f64 = row[1]
                .parse()
                .map_err(|e| format!("bakery-pnl.csv: amount {:?}: {e}", row[1]))?;
            pnl.push((row[0].clone(), amount));
        }
    }
    let mut rows = read_csv("bakery-mapping.csv")?.into_iter();
    let header = rows.next().ok_or("bakery-mapping.csv: no header")?;
    let at = mapping_columns(&header)?;
    let mut mapping = Vec::new();
    // DictReader skips a row with no fields at all.
    for row in rows.filter(|row| !row.is_empty()) {
        if row.len() < header.len() {
            return Err(format!("bakery-mapping.csv: short row {row:?}").into());
        }
        mapping.push(at.map(|i| row[i].clone()));
    }
    Ok(Bakery { pnl, mapping })
}

/// A cell of the Benchmarks table.
enum Value {
    Blank,
    Number(f64),
    Text(String),
}

fn number(text: &str) -> Result<Value> {
    Ok(Value::Number(text.parse()?))
}

fn decimal(value: &ato_benchmark_compare::decimal::Decimal) -> Result<Value> {
    number(&value.to_sci_string())
}

/// A text field of a source record, which the workbook writes as it is.
fn text_field(record: &Raw, key: &str) -> Result<String> {
    match record.get(key) {
        Some(Raw::Str(text)) => Ok(text.clone()),
        other => Err(format!("source field {key} is not text: {other:?}").into()),
    }
}

/// `str()` of a source field, as the Python builder's f-strings wrote it.
fn str_field(record: &Raw, key: &str) -> Result<String> {
    record
        .get(key)
        .map(Raw::py_str)
        .ok_or_else(|| format!("source field {key} is missing").into())
}

/// Every band as the engine loads it, industry-page ranges merged in.
struct Bands {
    /// The tblBands rows.
    rows: Vec<Vec<Value>>,
    /// Each benchmark year's dataset, in year order.
    datasets: Vec<(String, Dataset)>,
}

/// The bands and datasets of every shipped year.
///
/// Reading through the engine's loader rather than the JSON keeps the
/// workbook on the engine's own view: the same merge, the same checks, the
/// same refusals.
fn read_bands() -> Result<Bands> {
    let mut rows = Vec::new();
    let mut datasets = Vec::new();
    for year in dataset::available_years(&DataDir::Shipped) {
        let data = dataset::load(Some(&year), &DataDir::Shipped)
            .map_err(|e| format!("dataset {year}: {e}"))?;
        let retrieved = text_field(&data.source, "retrieved")?;
        let sha256 = text_field(&data.source, "sha256")?;
        for bt in &data.business_types {
            for band in &bt.bands {
                let bounds = |key: &str| -> Result<[Value; 2]> {
                    Ok(match band.ratio(key) {
                        None => [Value::Blank, Value::Blank],
                        Some(range) => [decimal(&range.minimum)?, decimal(&range.maximum)?],
                    })
                };
                let inclusive = if band.turnover_from_inclusive {
                    "Y"
                } else {
                    "N"
                };
                let mut row = vec![
                    Value::Text(year.clone()),
                    Value::Text(bt.name.clone()),
                    Value::Text(bt.key_ratio.clone()),
                    Value::Text(band.band.clone()),
                    Value::Text(band.label.clone()),
                    decimal(&band.turnover_from)?,
                    Value::Text(inclusive.into()),
                    match &band.turnover_to {
                        None => Value::Blank,
                        Some(to) => decimal(to)?,
                    },
                ];
                row.extend(bounds("cost_of_sales_to_turnover")?);
                row.extend(bounds("total_expenses_to_turnover")?);
                row.push(Value::Text(retrieved.clone()));
                row.push(Value::Text(sha256.clone()));
                row.extend(bounds("labour_to_turnover")?);
                row.extend(bounds("rent_to_turnover")?);
                row.extend(bounds("motor_vehicle_to_turnover")?);
                let page_ranges: Vec<&str> = RANGE_COLUMNS
                    .iter()
                    .map(|(key, ..)| *key)
                    .filter(|key| band.is_page_ratio(key))
                    .collect();
                row.push(Value::Text(page_ranges.join(" ")));
                row.push(match &bt.page {
                    Some(page) if !band.page_ratios.is_empty() => {
                        let page = Raw::Dict(page.clone());
                        Value::Text(text_field(&page, "text_sha256")?)
                    }
                    _ => Value::Blank,
                });
                rows.push(row);
            }
        }
        datasets.push((year, data));
    }
    Ok(Bands { rows, datasets })
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
///
/// A calculated column is what makes Excel fill the formula into rows an
/// accountant pastes below the example, instead of leaving them blank and
/// BLOCKED.
fn add_table(
    ws: &mut Worksheet,
    name: &str,
    titles: &[&str],
    last_row: u32,
    formulas: &[(&str, &str)],
    head: &Format,
) -> Result<()> {
    let columns: Vec<TableColumn> = titles
        .iter()
        .map(|title| {
            let column = TableColumn::new()
                .set_header(*title)
                .set_header_format(head);
            match formulas.iter().find(|(t, _)| t == title) {
                Some((_, f)) => column.set_formula(Formula::new(&f[1..])),
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

/// A list validation as openpyxl writes `allow_blank=False`: blanks are not
/// ignored, and neither prompt nor error alert is shown.
fn list_rule() -> DataValidation {
    DataValidation::new()
        .ignore_blank(false)
        .show_input_message(false)
        .show_error_message(false)
}

/// The account on the last row where `cond` holds, so the accountant can
/// find it.
fn offender(table: &str, cond: &str) -> String {
    format!("INDEX({table}[Account],SUMPRODUCT(MAX({cond}*(ROW({table}[Account])-1))))")
}

/// Both duplicate checks read the normalised Key columns, matching the
/// engine's trimmed, case-folded identity, so "Sales" and " Sales " count as
/// one name. Compare names literally: COUNTIF interprets wildcard and escape
/// characters.
fn duplicate_names(table: &str) -> String {
    let keys = format!("{table}[Key]");
    format!("(MMULT(--({keys}=TRANSPOSE({keys})),ROW({keys})^0)>1)*({keys}<>\"\")")
}

pub fn build() -> Result<Vec<u8>> {
    let Bakery { pnl, mapping } = read_bakery()?;
    let Bands {
        rows: bands,
        datasets,
    } = read_bands()?;
    let s = Styles::new();
    // openpyxl writes a bold-only font with no colour element: automatic.
    let bold = Format::new().set_bold().set_font_color(Color::Automatic);
    let mut wb = Workbook::new();

    // 1. Start Here
    let ws = wb.add_worksheet().set_name("Start Here")?;
    ws.write_string_with_format(
        0,
        0,
        "ATO small business benchmark comparison",
        &Format::new()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::RGB(PURPLE)),
    )?;
    let steps = [
        "1. On P&L Import, paste account names and amounts over the example rows. Account cells are Text; nothing pasted there is evaluated.",
        "2. On Mapping, give every account a bucket and set Source to reviewed. A suggested or missing mapping blocks the result.",
        "3. On Calculation, choose the benchmark year and industry. Enter W1 if the activity statement labour figure is higher than wages. Set B4 to Y if the export shows expenses as negatives.",
        "4. Read Results and Review Checks. The overall status below is BLOCKED, REVIEW or PASS; blank or an error is never a pass.",
        "5. Sitting outside a published range is not a finding. The ATO ranges are review prompts, and the judgement stays with you. Nothing leaves this workbook.",
        "6. Benchmarks and Sources & Version hold the published ranges, their provenance and the engine version this workbook was checked against.",
    ];
    let wrap_top = Format::new().set_text_wrap().set_align(FormatAlign::Top);
    for (i, step) in steps.iter().enumerate() {
        ws.write_string_with_format(2 + i as u32, 0, *step, &wrap_top)?;
    }
    width(ws, 0, 110)?;
    ws.write_string_with_format(9, 0, "Overall status", &bold)?;
    ws.write_formula_with_format(
        10,
        0,
        Formula::new("='Review Checks'!B13"),
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
            "ato-benchmark-compare workbook, engine {VERSION}. Macro-free. Not advice; see DISCLAIMER.md in the repository."
        ),
    )?;

    // 2. P&L Import: tblPnl = Account, Amount, Bucket, Guard, Sample, Key
    let ws = wb.add_worksheet().set_name("P&L Import")?;
    // A fabricated bakery line left in the P&L would count in the ratios;
    // flag the exact account-and-amount pairs the shipped sample carries.
    let sample_names = pnl
        .iter()
        .map(|(account, _)| format!("\"{}\"", account.trim_matches(py::is_space).to_lowercase()))
        .collect::<Vec<_>>()
        .join(",");
    let sample_amounts = pnl
        .iter()
        .map(|(_, amount)| float_repr(*amount))
        .collect::<Vec<_>>()
        .join(",");
    let sample_formula = format!(
        "=IF(SUMPRODUCT((LOWER(TRIM(tblPnl[[#This Row],[Account]]))={{{sample_names}}})*(tblPnl[[#This Row],[Amount]]={{{sample_amounts}}}))>0,1,0)"
    );
    // The engine compares normalised names literally, so the lookup matches
    // on an equality array rather than MATCH's text form, which reads ? and *
    // as wildcards.
    let pnl_key_formula = "=TRIM(LOWER(tblPnl[[#This Row],[Account]]))";
    let bucket_formula = "=IFERROR(INDEX(tblMapping[Bucket],MATCH(TRUE,INDEX(tblMapping[Key]=tblPnl[[#This Row],[Key]],0),0)),\"\")";
    let guard_formula = "=IF(OR(_xlfn.ISFORMULA(tblPnl[[#This Row],[Account]]),_xlfn.ISFORMULA(tblPnl[[#This Row],[Amount]]),AND(tblPnl[[#This Row],[Account]]<>\"\",NOT(ISNUMBER(tblPnl[[#This Row],[Amount]])))),1,0)";
    let pnl_calcs = [
        ("Bucket", bucket_formula),
        ("Guard", guard_formula),
        ("Sample", sample_formula.as_str()),
        ("Key", pnl_key_formula),
    ];
    let titles = ["Account", "Amount", "Bucket", "Guard", "Sample", "Key"];
    add_table(ws, "tblPnl", &titles, pnl.len() as u32, &pnl_calcs, &s.head)?;
    let input_text = s.input.clone().set_num_format(TEXT);
    for (i, (account, amount)) in pnl.iter().enumerate() {
        let r = 1 + i as u32;
        ws.write_string_with_format(r, 0, account, &input_text)?;
        ws.write_number_with_format(r, 1, *amount, &s.input.clone().set_num_format(MONEY))?;
        for (c, (_, formula)) in pnl_calcs.iter().enumerate() {
            ws.write_formula_with_format(r, 2 + c as u16, Formula::new(*formula), &s.calc)?;
        }
    }
    width(ws, 0, 34)?;
    width(ws, 1, 16)?;
    width(ws, 2, 24)?;

    // 3. Mapping: tblMapping = Account, Bucket, Source, Note, Key
    let ws = wb.add_worksheet().set_name("Mapping")?;
    let key = "=TRIM(LOWER(tblMapping[[#This Row],[Account]]))";
    let n = mapping.len() as u32 + 1;
    add_table(
        ws,
        "tblMapping",
        &["Account", "Bucket", "Source", "Note", "Key"],
        n - 1,
        &[("Key", key)],
        &s.head,
    )?;
    for (i, [account, bucket, source, note]) in mapping.iter().enumerate() {
        let r = 1 + i as u32;
        ws.write_string_with_format(r, 0, account, &input_text)?;
        ws.write_string_with_format(r, 1, bucket, &s.input)?;
        ws.write_string_with_format(r, 2, source, &s.input)?;
        ws.write_string_with_format(r, 3, note, &input_text)?;
        ws.write_formula_with_format(r, 4, Formula::new(key), &s.calc)?;
    }
    let buckets: Vec<&str> = bucket_names().collect();
    ws.add_data_validation(1, 1, n + 199, 1, &list_rule().allow_list_strings(&buckets)?)?;
    ws.add_data_validation(
        1,
        2,
        n + 199,
        2,
        &list_rule().allow_list_strings(&["suggested", "reviewed"])?,
    )?;
    for (c, chars) in [34, 24, 12, 60, 34].into_iter().enumerate() {
        width(ws, c as u16, chars)?;
    }

    // 4. Benchmarks: tblBands plus the industry list used for validation
    let ws = wb.add_worksheet().set_name("Benchmarks")?;
    add_table(
        ws,
        "tblBands",
        &BAND_COLUMNS,
        bands.len() as u32,
        &[],
        &s.head,
    )?;
    for (i, row) in bands.iter().enumerate() {
        let r = 1 + i as u32;
        for (c, value) in row.iter().enumerate() {
            let c = c as u16;
            match value {
                Value::Blank => {}
                Value::Number(n) => {
                    ws.write_number(r, c, *n)?;
                }
                Value::Text(text) => {
                    ws.write_string(r, c, text)?;
                }
            }
        }
    }
    let industries: BTreeSet<&str> = bands
        .iter()
        .map(|row| match &row[1] {
            Value::Text(name) => name.as_str(),
            _ => unreachable!("an industry name"),
        })
        .collect();
    ws.write_string_with_format(0, 23, "Industries", &s.head)?;
    for (i, name) in industries.iter().enumerate() {
        ws.write_string(1 + i as u32, 23, *name)?;
    }
    let industry_ref = format!("Benchmarks!$X$2:$X${}", industries.len() + 1);
    for (c, chars) in [(1, 48), (4, 26), (20, 40), (23, 48)] {
        width(ws, c, chars)?;
    }
    protect(ws);

    // 5. Calculation
    let ws = wb.add_worksheet().set_name("Calculation")?;
    ws.write_string_with_format(0, 0, "Inputs", &s.head)?;
    ws.write_blank(0, 1, &s.head)?;
    let years: Vec<&str> = datasets.iter().map(|(year, _)| year.as_str()).collect();
    let latest = years.last().ok_or("no benchmark year")?;
    let unlocked = s.input.clone().set_unlocked();
    let inputs = [
        ("Benchmark year", Some(*latest)),
        ("Industry", Some("Bakeries and hot bread shops")),
        ("Expenses shown as negatives? (Y/N)", Some("N")),
        ("Activity statement W1 (blank if not supplied)", None),
    ];
    for (i, (label, value)) in inputs.iter().enumerate() {
        let r = 1 + i as u32;
        ws.write_string(r, 0, *label)?;
        match value {
            Some(text) => ws.write_string_with_format(r, 1, *text, &unlocked)?,
            None => ws.write_blank(r, 1, &unlocked.clone().set_num_format(MONEY))?,
        };
    }
    ws.write_string(5, 0, "Expense sign factor")?;
    ws.write_formula(5, 1, Formula::new("=IF(B4=\"Y\",-1,1)"))?;
    ws.add_data_validation(1, 1, 1, 1, &list_rule().allow_list_strings(&years)?)?;
    ws.add_data_validation(
        2,
        1,
        2,
        1,
        &list_rule().allow_list_formula(Formula::new(format!("={industry_ref}"))),
    )?;
    ws.add_data_validation(3, 1, 3, 1, &list_rule().allow_list_strings(&["Y", "N"])?)?;

    ws.write_string_with_format(6, 0, "Bucket totals (SUMIFS over tblPnl)", &s.head)?;
    ws.write_blank(6, 1, &s.head)?;
    ws.write_string_with_format(6, 2, "What the ATO does with it", &s.head)?;
    let calc_money = s.calc.clone().set_num_format(MONEY);
    for (i, (bucket, meaning)) in BUCKETS.iter().enumerate() {
        let r = 8 + i as u32;
        let sign = if (EXPENSE_ROWS.0..=EXPENSE_ROWS.1).contains(&r) {
            "*$B$6"
        } else {
            ""
        };
        ws.write_string(r - 1, 0, *bucket)?;
        ws.write_formula_with_format(
            r - 1,
            1,
            Formula::new(format!("=SUMIFS(tblPnl[Amount],tblPnl[Bucket],A{r}){sign}")),
            &calc_money,
        )?;
        ws.write_string(r - 1, 2, *meaning)?;
    }

    // First matching band, as the engine does: the minimum matching row
    // index, with non-matching rows pushed to 1E9 so MIN ignores them. 1E9
    // then means no band.
    let band_match = "((tblBands[Year]=B2)*(tblBands[Industry]=B3)*(((B23>tblBands[From])+((tblBands[FromInclusive]=\"Y\")*(B23=tblBands[From])))>0)*(((tblBands[To]=\"\")+(B23<=tblBands[To]))>0))";
    let band_row_raw =
        format!("=SUMPRODUCT(MIN({band_match}*(ROW(tblBands[Year])-1)+(1-{band_match})*1E9))");
    let expense_rows = format!("B{}:B{}", EXPENSE_ROWS.0, EXPENSE_ROWS.1);
    let calc: [(u32, &str, String); 18] = [
        (20, "Sales of goods and services", "=B8".into()),
        (21, "Other business income", "=B9".into()),
        (22, "Total business income", "=B20+B21".into()),
        (23, "Turnover", "=IF(OR(B20<=0,B20*2<B22),B22,B20)".into()),
        (
            24,
            "Turnover basis",
            "=IF(OR(B20<=0,B20*2<B22),\"total business income\",\"sales of goods and services\")"
                .into(),
        ),
        (
            25,
            "Total expenses reported",
            format!("=SUM({expense_rows})"),
        ),
        (26, "Payments to associated persons", "=B14".into()),
        (27, "Total expenses for ratio", "=B25-B26".into()),
        (
            28,
            "Cost of sales for ratio (excludes wages)",
            "=B10".into(),
        ),
        (
            29,
            "Salary and wages label (wages + cost of sales wages + associates)",
            "=B12+B11+B26".into(),
        ),
        (
            30,
            "Labour base (W1 when it exceeds the label)",
            "=IF(AND(B5<>\"\",B5>B29),B5,B29)".into(),
        ),
        (
            31,
            "Labour (base less associates plus contractors and commission)",
            "=B30-B26+B13".into(),
        ),
        (32, "Band lookup (1E9 = no match)", band_row_raw),
        (
            33,
            "Band row in tblBands (0 = none)",
            "=IF(B32>=1E9,0,B32)".into(),
        ),
        (
            34,
            "Turnover band",
            "=IF(B33=0,\"none applies\",INDEX(tblBands[Label],B33))".into(),
        ),
        (
            35,
            "Industry row in tblBands (0 = not published for the year)",
            "=SUMPRODUCT(MAX((tblBands[Year]=B2)*(tblBands[Industry]=B3)*(ROW(tblBands[Year])-1)))"
                .into(),
        ),
        (
            36,
            "Published key ratio",
            "=IF(B35=0,\"\",INDEX(tblBands[KeyRatio],B35))".into(),
        ),
        (
            37,
            "Key ratio used",
            "=IF(AND(B36=\"cost_of_sales_to_turnover\",B28=0),\"total_expenses_to_turnover\",B36)"
                .into(),
        ),
    ];
    ws.write_string_with_format(18, 0, "Figures", &s.head)?;
    ws.write_blank(18, 1, &s.head)?;
    for (r, label, formula) in &calc {
        ws.write_string(r - 1, 0, *label)?;
        // B20 to B31 are money; the band and key ratio lookups are not.
        let format = if *r <= 31 { &calc_money } else { &s.calc };
        ws.write_formula_with_format(r - 1, 1, Formula::new(formula), format)?;
    }
    for (c, chars) in [62, 30, 70].into_iter().enumerate() {
        width(ws, c as u16, chars)?;
    }
    protect(ws);

    // 6. Results
    let ws = wb.add_worksheet().set_name("Results")?;
    header(
        ws,
        &[
            "Ratio",
            "Label",
            "Actual",
            "Benchmark min",
            "Benchmark max",
            "Status",
            "Key",
            "Range source",
        ],
        &s.head,
    )?;
    let calc_pct = s.calc.clone().set_num_format(PCT);
    for (i, (key, lo, hi, cell)) in RANGE_COLUMNS.iter().enumerate() {
        let r = 2 + i as u32;
        let row = r - 1;
        ws.write_string(row, 0, *key)?;
        ws.write_string(row, 1, ratio_label(key))?;
        ws.write_formula_with_format(
            row,
            2,
            Formula::new(format!(
                "=IFERROR(ROUND(Calculation!{cell}/Calculation!B23,4),\"\")"
            )),
            &calc_pct,
        )?;
        for (c, name) in [(3, lo), (4, hi)] {
            ws.write_formula_with_format(
                row,
                c,
                Formula::new(format!(
                    "=IF(OR(Calculation!$B$33=0,INDEX(tblBands[{name}],Calculation!$B$33)=\"\"),\"\",INDEX(tblBands[{name}],Calculation!$B$33))"
                )),
                &calc_pct,
            )?;
        }
        // The engine's benchmark_source: blank without a range, otherwise
        // the industry page when the band's PageRanges names this ratio.
        ws.write_formula_with_format(
            row,
            7,
            Formula::new(format!(
                "=IF(D{r}=\"\",\"\",IF(ISNUMBER(SEARCH(A{r},INDEX(tblBands[PageRanges],Calculation!$B$33))),\"ato_industry_page\",\"ato_dataset\"))"
            )),
            &s.calc,
        )?;
        ws.write_formula_with_format(
            row,
            5,
            Formula::new(format!(
                "=IF(C{r}=\"\",\"not calculated\",IF(Calculation!$B$33=0,\"no turnover band applies\",IF(D{r}=\"\",\"no benchmark in this dataset\",IF(AND(C{r}>=D{r},C{r}<=E{r}),\"within\",IF(C{r}<D{r},\"below\",\"above\")))))"
            )),
            &s.calc,
        )?;
        ws.write_formula_with_format(
            row,
            6,
            Formula::new(format!("=IF(A{r}=Calculation!$B$37,\"key\",\"\")")),
            &s.calc,
        )?;
    }
    ws.write_string(7, 0, "Turnover")?;
    ws.write_formula_with_format(7, 1, Formula::new("=Calculation!B23"), &calc_money)?;
    ws.write_string(8, 0, "Turnover band")?;
    ws.write_formula_with_format(8, 1, Formula::new("=Calculation!B34"), &s.calc)?;
    ws.write_string(9, 0, "Benchmark year and industry")?;
    ws.write_formula_with_format(
        9,
        1,
        Formula::new("=Calculation!B2&\" \"&Calculation!B3"),
        &s.calc,
    )?;
    for (c, chars) in [28, 34, 12, 16, 16, 28, 6, 20].into_iter().enumerate() {
        width(ws, c as u16, chars)?;
    }
    protect(ws);

    // 7. Review Checks
    let ws = wb.add_worksheet().set_name("Review Checks")?;
    header(
        ws,
        &["Check", "Result", "Count or value", "Example account"],
        &s.head,
    )?;
    let pnl_dup = duplicate_names("tblPnl");
    let map_dup = duplicate_names("tblMapping");
    let bucket_list = format!(
        "{{{}}}",
        bucket_names()
            .map(|b| format!("\"{b}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
    let ones = format!("{{{}}}", vec!["1"; BUCKETS.len()].join(";"));
    // A bucket outside the engine's list falls outside every SUMIFS row, so
    // an unrecognised value is as blocking as a blank one. Literal equality,
    // not MATCH: a pasted "*" or "?" would otherwise match the list as a
    // wildcard.
    let bad_bucket =
        format!("(tblPnl[Account]<>\"\")*(MMULT(--(tblPnl[Bucket]={bucket_list}),{ones})=0)");
    let suggested = format!("(TRIM(LOWER(tblMapping[Source]))=\"{SOURCE_SUGGESTED}\")");
    let example = |cell: &str, table: &str, cond: &str| {
        Some(format!("=IF({cell}=0,\"\",{})", offender(table, cond)))
    };
    let checks: [(&str, &str, String, Option<String>); 11] = [
        (
            "Every P&L account has a recognised bucket",
            "=IF(C2=0,\"PASS\",\"BLOCKED\")",
            format!("=SUMPRODUCT({bad_bucket})"),
            example("C2", "tblPnl", &bad_bucket),
        ),
        (
            "No mapping is still suggested",
            "=IF(C3=0,\"PASS\",\"BLOCKED\")",
            format!("=SUMPRODUCT(--{suggested})"),
            example("C3", "tblMapping", &suggested),
        ),
        (
            "No formula or non-numeric cell in the P&L",
            "=IF(C4=0,\"PASS\",\"BLOCKED\")",
            "=SUM(tblPnl[Guard])".into(),
            example("C4", "tblPnl", "(tblPnl[Guard]=1)"),
        ),
        (
            "No P&L account name appears twice",
            "=IF(C5=0,\"PASS\",\"BLOCKED\")",
            format!("=SUMPRODUCT({pnl_dup})"),
            example("C5", "tblPnl", &pnl_dup),
        ),
        (
            "No mapping account appears twice",
            "=IF(C6=0,\"PASS\",\"BLOCKED\")",
            format!("=SUMPRODUCT({map_dup})"),
            example("C6", "tblMapping", &map_dup),
        ),
        (
            "W1 is not negative",
            "=IF(AND(Calculation!B5<>\"\",Calculation!B5<0),\"BLOCKED\",\"PASS\")",
            "=Calculation!B5".into(),
            None,
        ),
        (
            "Turnover is positive",
            "=IF(Calculation!B23>0,\"PASS\",\"BLOCKED\")",
            "=Calculation!B23".into(),
            None,
        ),
        (
            "Industry is published for the benchmark year",
            "=IF(Calculation!B35=0,\"BLOCKED\",\"PASS\")",
            "=Calculation!B2&\" \"&Calculation!B3".into(),
            None,
        ),
        (
            "Turnover falls in a published band",
            "=IF(Calculation!B33=0,\"REVIEW\",\"PASS\")",
            "=Calculation!B34".into(),
            None,
        ),
        (
            "Key ratio is within its range",
            "=IF(C11=\"within\",\"PASS\",\"REVIEW\")",
            "=IFERROR(INDEX(Results!F2:F6,MATCH(Calculation!B37,Results!A2:A6,0)),\"not calculated\")"
                .into(),
            None,
        ),
        (
            "No fabricated example account from the shipped bakery sample remains in the P&L",
            "=IF(C12=0,\"PASS\",\"REVIEW\")",
            "=SUM(tblPnl[Sample])".into(),
            example("C12", "tblPnl", "(tblPnl[Sample]=1)"),
        ),
    ];
    for (i, (label, result, detail, example)) in checks.iter().enumerate() {
        let row = 1 + i as u32;
        ws.write_string(row, 0, *label)?;
        ws.write_formula_with_format(row, 1, Formula::new(*result), &s.calc)?;
        // Keep whole columns in duplicate checks and offender examples. Excel
        // otherwise inserts implicit intersection into nested array
        // expressions.
        if matches!(row + 1, 5 | 6) {
            ws.write_array_formula_with_format(row, 2, row, 2, Formula::new(detail), &s.calc)?;
        } else {
            ws.write_formula_with_format(row, 2, Formula::new(detail), &s.calc)?;
        }
        match example {
            Some(f) => {
                ws.write_array_formula_with_format(row, 3, row, 3, Formula::new(f), &s.calc)?
            }
            None => ws.write_blank(row, 3, &s.calc)?,
        };
    }
    let last = checks.len() + 1;
    ws.write_string_with_format(12, 0, "Overall status", &bold)?;
    ws.write_formula_with_format(
        12,
        1,
        Formula::new(format!(
            "=IF(COUNTIF(B2:B{last},\"BLOCKED\")>0,\"BLOCKED\",IF(COUNTIF(B2:B{last},\"REVIEW\")>0,\"REVIEW\",\"PASS\"))"
        )),
        &s.calc.clone().set_bold().set_font_color(Color::Automatic),
    )?;
    ws.write_string_with_format(
        14,
        0,
        "Notes (the engine prints these as warnings; they do not change the status)",
        &s.head,
    )?;
    let notes = [
        "=IF(Calculation!B24=\"total business income\",\"Sales of goods and services are not positive or are less than half of total business income, so total business income is used as turnover. That is the ATO rule, but check the mapping.\",\"\")",
        "=IF(AND(Calculation!B5<>\"\",Calculation!B5>Calculation!B29),\"Activity statement W1 is greater than the salary and wages label, so W1 is used in the labour ratio.\",\"\")",
        "=IF(AND(Calculation!B5=\"\",Calculation!B11+Calculation!B12>0),\"No activity statement W1 amount was supplied. The ATO uses W1 for the labour ratio when it exceeds the salary and wages figure.\",\"\")",
        "=IF(Calculation!B26=0,\"No payments to associated persons were mapped. They are deducted from total expenses, so a zero here raises the total expenses ratio.\",\"\")",
        "=IF(AND(Calculation!B28>0,Calculation!B11=0),\"No salary and wages were mapped inside cost of sales. The ATO excludes wages from the cost of sales ratio, so confirm none are sitting in those accounts.\",\"\")",
        "=IF(MIN(Calculation!B10:B17)<0,\"An expense bucket is negative. Check the sign convention of the export, or set Calculation!B4 to Y.\",\"\")",
        "=IF(Calculation!B36=\"cost_of_sales_to_turnover\",IF(Calculation!B28=0,\"Cost of sales is the ATO key range for this industry but none was mapped, so total expenses to turnover is the key range used here.\",\"The ATO says to use total expenses to turnover as the key range where cost of sales is only a small amount. Both ranges are reported and that judgement is yours.\"),\"\")",
        "=IF(COUNTIF(Results!H2:H6,\"ato_industry_page\")>0,\"Some ranges on Results come from the ATO's page for this industry, not the data.gov.au dataset (Range source column). The ATO says not every business reports these expenses and to use these ranges only as a guide where they apply.\",\"\")",
    ];
    let calc_wrap = s.calc.clone().set_text_wrap();
    for (i, formula) in notes.iter().enumerate() {
        ws.write_formula_with_format(15 + i as u32, 0, Formula::new(*formula), &calc_wrap)?;
    }
    for (c, chars) in [100, 12, 40, 34].into_iter().enumerate() {
        width(ws, c as u16, chars)?;
    }
    protect(ws);

    // 8. Sources & Version
    let ws = wb.add_worksheet().set_name("Sources & Version")?;
    header(ws, &["Item", "Value"], &s.head)?;
    let mut items: Vec<(String, String)> = vec![
        ("Workbook engine version".into(), VERSION.into()),
        (
            "Desktop Excel build used to calculate cached values".into(),
            String::new(),
        ),
    ];
    for (year, data) in &datasets {
        let src = &data.source;
        items.push((format!("{year} publisher"), text_field(src, "publisher")?));
        items.push((format!("{year} dataset"), text_field(src, "dataset")?));
        items.push((format!("{year} resource"), text_field(src, "resource_url")?));
        items.push((format!("{year} retrieved"), text_field(src, "retrieved")?));
        items.push((format!("{year} sha256"), text_field(src, "sha256")?));
        items.push((
            format!("{year} licence"),
            format!(
                "{} {}",
                str_field(src, "licence")?,
                str_field(src, "licence_url")?
            ),
        ));
    }
    for (year, data) in &datasets {
        let Some(src) = &data.page_source else {
            continue;
        };
        items.push((
            format!("{year} industry pages"),
            format!(
                "{}, {}",
                str_field(src, "publisher")?,
                str_field(src, "pages")?
            ),
        ));
        items.push((
            format!("{year} industry page index"),
            text_field(src, "index_url")?,
        ));
        items.push((
            format!("{year} industry pages retrieved"),
            text_field(src, "retrieved")?,
        ));
        items.push((
            format!("{year} industry pages licence"),
            format!(
                "{} {}",
                str_field(src, "licence")?,
                str_field(src, "licence_url")?
            ),
        ));
        items.push((
            format!("{year} industry pages caution"),
            text_field(src, "caution")?,
        ));
    }
    for (i, (key, value)) in items.iter().enumerate() {
        let r = 1 + i as u32;
        ws.write_string(r, 0, key)?;
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
    for (sheet, (r1, c, r2)) in [("Review Checks", (1, 1, 12)), ("Start Here", (10, 0, 10))] {
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
    use ato_benchmark_compare::dataset::{PAGE_RATIO_KEYS, RATIO_KEYS};
    use ato_benchmark_compare::mapping::EXPENSE_BUCKETS;

    /// Accepted, as for the payday and div7a workbooks after their Gate B
    /// desktop Excel checks:
    /// - openpyxl writes a status colour as a `solid` dxf fill with fgColor,
    ///   which Excel does not display; rust_xlsxwriter writes bgColor, the form
    ///   Excel writes and displays, so the Rust workbook shows the colours;
    /// - setting openpyxl's margins makes rust_xlsxwriter write a page setup
    ///   with portrait orientation, A4 paper and 200 dpi, where openpyxl leaves
    ///   them to the printer.
    ///
    /// And one of this workbook's own: openpyxl writes the industry list's
    /// source as given, `=Benchmarks!…`; rust_xlsxwriter writes the OOXML
    /// form without the `=`, which is what Excel saves for either.
    fn accepted(line: &str) -> String {
        line.replace("dxf{fill{solid fgColor=", "dxf{fill{(absent) bgColor=")
            .replace(
                "| page orientation=default paperSize=1 scale=100 fitToWidth=1 fitToHeight=1 horizontalDpi=600 verticalDpi=600 ",
                "| page orientation=portrait paperSize=9 scale=100 fitToWidth=1 fitToHeight=1 horizontalDpi=200 verticalDpi=200 ",
            )
            .replace("formula1=\"=Benchmarks!$X$2:", "formula1=\"Benchmarks!$X$2:")
    }

    #[test]
    fn the_workbook_matches_the_frozen_python_model() {
        // The model of tools/build_workbook.py --no-excel at 374f83d.
        let frozen: Vec<String> = include_str!("../../tests/data/workbook-model.txt")
            .lines()
            .map(accepted)
            .collect();
        let built = model(&super::build().unwrap()).unwrap();
        let found = differences(&frozen, &built);
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    /// The sign factor and the total expenses sum cover exactly the engine's
    /// expense buckets.
    #[test]
    fn the_expense_rows_are_the_engines_expense_buckets() {
        let rows: BTreeSet<&str> = bucket_names()
            .enumerate()
            .filter(|(i, _)| (EXPENSE_ROWS.0..=EXPENSE_ROWS.1).contains(&(8 + *i as u32)))
            .map(|(_, name)| name)
            .collect();
        assert_eq!(rows, EXPENSE_BUCKETS.into_iter().collect());
    }

    /// Results carries every ratio the engine compares, dataset and page.
    #[test]
    fn the_results_rows_are_the_engines_ratios() {
        let rows: BTreeSet<&str> = RANGE_COLUMNS.iter().map(|(key, ..)| *key).collect();
        let engine: BTreeSet<&str> = RATIO_KEYS.into_iter().chain(PAGE_RATIO_KEYS).collect();
        assert_eq!(rows, engine);
        for (key, ..) in RANGE_COLUMNS {
            assert_ne!(ratio_label(key), key, "{key} has a label");
        }
    }

    #[test]
    fn the_amount_filter_matches_the_python_pattern() {
        for text in ["0", "-12", "520000.00", "1.5", "\u{0661}\u{0662}"] {
            assert!(is_amount(text), "{text}");
        }
        for text in ["", "-", "1.", ".5", "+1", " 1", "1e3", "1,000", "--1"] {
            assert!(!is_amount(text), "{text}");
        }
    }

    #[test]
    fn a_repeated_or_missing_mapping_column_refuses() {
        let header = |names: &str| names.split(',').map(String::from).collect::<Vec<_>>();
        let columns = mapping_columns(&header("account,bucket,source,amount,note")).unwrap();
        assert_eq!(columns, [0, 1, 2, 4]);
        let error = mapping_columns(&header("account,bucket,source,note,bucket"));
        assert!(error.unwrap_err().to_string().contains("repeated bucket"));
        let error = mapping_columns(&header("account,bucket,note"));
        assert!(error.unwrap_err().to_string().contains("no source"));
    }

    /// The length of a formula cell's formula, from its model line
    /// `... | cell A1 formula "..."`: the model writes it `{:?}` quoted, and
    /// each escape stands for one character. None for any other line.
    fn formula_chars(line: &str) -> Option<usize> {
        let (_, cell) = line.split_once(" | cell ")?;
        let (_, rest) = cell.split_once(' ')?;
        let mut chars = rest.strip_prefix("formula \"")?.chars();
        let mut count = 0;
        loop {
            match chars.next()? {
                '"' => return Some(count),
                '\\' => {
                    let escaped = chars.next()?;
                    if escaped == 'u' {
                        while chars.next()? != '}' {}
                    }
                }
                _ => {}
            }
            count += 1;
        }
    }

    #[test]
    fn every_formula_fits_excels_formula_limit() {
        let fits = |formula: &str| {
            formula_chars(&format!("sheet S | cell A1 formula {formula:?}")).unwrap() <= 8192
        };
        assert!(fits(&"A".repeat(8192)) && !fits(&"A".repeat(8193)));
        assert!(fits(&format!("{}\"\\\u{7f}", "A".repeat(8189))));
        assert!(!fits(&format!("\"{}", "A".repeat(8192))));
        let built = model(&build().unwrap()).unwrap();
        assert_eq!(
            formula_chars("sheet S | cell A4 text \"No formula \\\"x\\\"\""),
            None
        );
        let formulas: Vec<(&String, usize)> = built
            .iter()
            .filter_map(|line| Some((line, formula_chars(line)?)))
            .collect();
        assert!(formulas.len() > 100);
        for (line, chars) in formulas {
            assert!(chars <= 8192, "{line}");
        }
    }
}
