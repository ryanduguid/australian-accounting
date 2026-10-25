//! The workbook as tools/build_workbook.py builds it before its desktop Excel
//! pass. Formula text is the Python builder's, character for character; the
//! frozen model of the Python output holds the two together.

use std::collections::BTreeMap;
use std::error::Error;

use payday_super_checker::assess::TRANSITION_END;
use payday_super_checker::csv_io::{self, Cell, FALSE_WORDS, LATEST_SANE_YEAR, TRUE_WORDS};
use payday_super_checker::deadlines::REGIME_START;
use payday_super_checker::py::Date;
use rust_xlsxwriter::{
    Color, ConditionalFormatCell, ConditionalFormatCellRule, DataValidation, DataValidationRule,
    Format, FormatAlign, Formula, ProtectionOptions, Table, TableColumn, TableStyle, Workbook,
    Worksheet,
};
use serde_json::Value;

use crate::{ARRAY_CALCS, repair_array_columns};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_AS_AT: (i64, i64, i64) = (2026, 8, 10);

const PURPLE: u32 = 0x5C2D91;
const LAVENDER: u32 = 0xF3F1F6;
const GREY: u32 = 0xE2E0DF;
const TEXT: &str = "@";
const MONEY: &str = r#"_(* #,##0.00_);_(* (#,##0.00);_(* "-"??_);_(@_)"#;
const DATE: &str = "yyyy-mm-dd";

const INPUT_COLUMNS: [&str; 11] = [
    "employee_id",
    "payment_date",
    "sg_amount",
    "remitted_date",
    "remitted_amount",
    "matched_amount",
    "fund_received_date",
    "first_contribution_to_fund",
    "out_of_cycle",
    "next_standard_payday",
    "defined_benefit",
];
const DATE_COLUMNS: [&str; 4] = [
    "payment_date",
    "remitted_date",
    "fund_received_date",
    "next_standard_payday",
];
const AMOUNT_COLUMNS: [&str; 3] = ["sg_amount", "remitted_amount", "matched_amount"];
const BOOL_COLUMNS: [&str; 3] = [
    "first_contribution_to_fund",
    "out_of_cycle",
    "defined_benefit",
];

// The register compares whole calendar days, as the checker does: B2 and B3
// stay the cells the accountant types into; D2 and D3 hold the whole-day values
// every formula reads.
const AS_AT_INPUT: &str = "Summary!$B$2";
const ASSESS_INPUT: &str = "Summary!$B$3";
const AS_AT: &str = "Summary!$D$2";
const ASSESS: &str = "Summary!$D$3";
const TRANSITION_OK: &str = "Summary!$B$4";
const REMIT_ONLY_OK: &str = "Summary!$B$5";
const COVERAGE: &str = "Summary!$B$6";
const GIC_LAST: &str = "Summary!$B$7";

const CALC_ORDER: [&str; 54] = [
    "Pay_day",
    "Remit_day",
    "Recv_day",
    "Next_day",
    "Flag_db",
    "Flag_ooc",
    "Flag_ftf",
    "Row_problem",
    "Guard",
    "Cap",
    "Due_ooc",
    "Due_ftf",
    "Own_due",
    "Own_pathway",
    "Earliest_prepay",
    "Settled",
    "Remit",
    "Receipt_credit",
    "Credit",
    "Evidence_own",
    "Confirmed_latest",
    "Possible_latest",
    "Final_due",
    "Pathway",
    "Possible_item4",
    "Past_horizon",
    "Branch",
    "Verdict",
    "Unassessable_between",
    "Exposed",
    "Stale_prepay",
    "OTRC",
    "Base_shortfall",
    "Offset_s18D",
    "Final_shortfall",
    "Lateness_basis",
    "Outstanding_to",
    "Accrual_to",
    "NEC_end",
    "Days_late",
    "Past_gic_table",
    "NEC",
    "Shortfall_r",
    "NEC_r",
    "Uplift_best",
    "Uplift_worst",
    "SGC_low",
    "SGC_high",
    "Transition_row",
    "Receipt_established",
    "Assessable",
    "Duplicate",
    "Case_variant",
    "Sample_row",
];
const DATE_CALCS: [&str; 17] = [
    "Pay_day",
    "Remit_day",
    "Recv_day",
    "Next_day",
    "Due_ooc",
    "Due_ftf",
    "Own_due",
    "Earliest_prepay",
    "Settled",
    "Remit",
    "Confirmed_latest",
    "Possible_latest",
    "Final_due",
    "Possible_item4",
    "Outstanding_to",
    "Accrual_to",
    "NEC_end",
];
const MONEY_CALCS: [&str; 14] = [
    "Cap",
    "Receipt_credit",
    "Credit",
    "OTRC",
    "Base_shortfall",
    "Offset_s18D",
    "Final_shortfall",
    "NEC",
    "Shortfall_r",
    "NEC_r",
    "Uplift_best",
    "Uplift_worst",
    "SGC_low",
    "SGC_high",
];

/// Branch codes: verdict, unassessable outer outcomes, whether the on-time
/// receipt credit applies, whether the receipt is a stale pre-payment, whether
/// exposed.
const CODES: [(&str, &str, &str, u8, u8, u8); 29] = [
    ("DB", "SKIPPED", "", 0, 0, 0),
    ("NIL", "UNKNOWN", "", 0, 0, 0),
    ("A1", "ON_TIME", "", 1, 0, 0),
    ("A2", "UNKNOWN", "", 1, 0, 0),
    ("A3", "UNKNOWN", "UNPAID or NOT_YET_DUE", 1, 0, 0),
    ("A4", "UNPAID", "", 1, 0, 1),
    ("S1", "UNKNOWN", "", 0, 1, 0),
    ("S2", "UNKNOWN", "LATE or NOT_YET_DUE", 0, 1, 0),
    ("S3", "LATE", "", 0, 1, 1),
    ("B1", "UNKNOWN", "LATE or ON_TIME", 0, 0, 0),
    ("B2", "UNKNOWN", "LATE or NOT_YET_DUE", 0, 0, 0),
    ("B3", "UNKNOWN", "LATE or UNPAID", 0, 0, 0),
    ("C1", "UNKNOWN", "LATE or ON_TIME", 0, 0, 0),
    ("C2", "UNKNOWN", "LATE or NOT_YET_DUE", 0, 0, 0),
    ("D1", "ON_TIME", "", 1, 0, 0),
    ("D2", "UNKNOWN", "UNPAID or NOT_YET_DUE", 1, 0, 0),
    ("D3", "UNPAID", "", 1, 0, 1),
    ("D4", "UNKNOWN", "", 1, 0, 0),
    ("E", "LATE", "", 0, 0, 1),
    ("R1", "UNKNOWN", "LATE or AT_RISK", 0, 0, 0),
    ("R2", "UNKNOWN", "LATE or AT_RISK", 0, 0, 0),
    ("R3", "AT_RISK", "", 0, 0, 0),
    ("R4", "LATE", "", 0, 0, 1),
    ("N1", "UNKNOWN", "UNPAID or NOT_YET_DUE", 0, 0, 0),
    ("N2", "UNPAID", "", 0, 0, 1),
    ("N3", "UNKNOWN", "", 0, 0, 0),
    // A fund-receipt date with neither amount column filled evidences timing
    // only. Where that timing could earn ON_TIME the engine leaves the row
    // UNKNOWN between the full-receipt and partial-receipt outcomes.
    ("U1", "UNKNOWN", "LATE or ON_TIME", 0, 0, 0),
    ("U2", "UNKNOWN", "UNPAID or ON_TIME", 0, 0, 0),
    ("U3", "UNKNOWN", "NOT_YET_DUE or ON_TIME", 0, 0, 0),
];

fn t(column: &str) -> String {
    format!("tblLines[[#This Row],[{column}]]")
}

fn excel_date(d: Date) -> String {
    format!("DATE({},{},{})", d.year, d.month, d.day)
}

/// The Excel serial number of a date (1900 date system).
fn serial(d: Date) -> f64 {
    let epoch = Date::new(1899, 12, 30).expect("valid epoch");
    (d.ordinal() - epoch.ordinal()) as f64
}

/// TAA 1953 s 8AAD divides the annual GIC rate by the days in the calendar year.
fn days_in_year(d: Date) -> Result<i64> {
    let year = i64::from(d.year);
    Ok(Date::new(year, 12, 31)?.ordinal() - Date::new(year, 1, 1)?.ordinal() + 1)
}

fn iso(text: &str) -> Result<Date> {
    Date::fromisoformat(text).ok_or_else(|| format!("not an ISO date: {text:?}").into())
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("{key}: expected text").into())
}

fn read_json(name: &str) -> Result<Value> {
    let path = format!("{ROOT}/data/{name}");
    Ok(serde_json::from_str(&std::fs::read_to_string(&path)?)?)
}

/// The sample register's rows, each as column name to raw text.
fn read_sample() -> Result<Vec<BTreeMap<String, String>>> {
    let data = std::fs::read(format!("{ROOT}/examples/sample_payrun.csv"))?;
    let mut reader = csv_io::DictReader::new(&data).map_err(|_| "sample_payrun.csv: not CSV")?;
    let mut rows = Vec::new();
    while let Some(row) = reader
        .next_row()
        .map_err(|_| "sample_payrun.csv: not CSV")?
    {
        rows.push(
            row.into_iter()
                .filter_map(|(key, cell)| match (key, cell) {
                    (Some(key), Cell::Value(value)) => Some((key, value)),
                    _ => None,
                })
                .collect(),
        );
    }
    Ok(rows)
}

/// The Holidays sheet's rows, the provisional dates it lists as not applied
/// and the date the official sources were checked through.
struct Calendar {
    holidays: Vec<(Date, String, String)>,
    provisional: Vec<(Date, String)>,
    verified_until: Date,
}

fn read_holidays() -> Result<Calendar> {
    let doc = read_json("business_days.json")?;
    let days = doc["non_business_days"]
        .as_array()
        .ok_or("non_business_days: expected a list")?;
    let mut rows = Vec::new();
    let mut provisional = Vec::new();
    for h in days {
        let day = text(h, "date")?;
        let name = text(h, "name")?.to_string();
        if h["provisional"]
            .as_bool()
            .ok_or("provisional: expected true or false")?
        {
            provisional.push((iso(day)?, name));
        } else if day >= "2026-01-01" {
            let jurisdictions = h["jurisdictions"]
                .as_array()
                .ok_or("jurisdictions: expected a list")?
                .iter()
                .map(|j| j.as_str().ok_or("jurisdictions: expected text"))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            rows.push((iso(day)?, name, jurisdictions.join(", ")));
        }
    }
    Ok(Calendar {
        holidays: rows,
        provisional,
        verified_until: iso(text(&doc, "verified_until")?)?,
    })
}

type Quarter = (Date, Date, f64, i64, String);

/// Only the ATO's published quarters. The engine refuses a day past the table
/// unless --allow-stale-gic is passed; the workbook has no opt-in, so a line
/// whose accrual runs past the table withholds the charge estimate.
fn read_gic() -> Result<(Vec<Quarter>, Date)> {
    let doc = read_json("gic_rates.json")?;
    let mut rows = Vec::new();
    for q in doc["quarters"]
        .as_array()
        .ok_or("quarters: expected a list")?
    {
        let start = iso(text(q, "from")?)?;
        let pct: f64 = text(q, "annual_pct")?.parse()?;
        rows.push((
            start,
            iso(text(q, "to")?)?,
            pct,
            days_in_year(start)?,
            text(q, "seen")?.to_string(),
        ));
    }
    let last = rows.last().ok_or("quarters: empty")?.1;
    Ok((rows, last))
}

fn bool_expr(column: &str) -> String {
    let v = format!("LOWER(TRIM({}&\"\"))", t(column));
    let words = |list: &[&str]| {
        let mut list = list.to_vec();
        list.sort_unstable();
        list.iter()
            .map(|x| format!("\"{x}\""))
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
        "IF(OR({v}={{{}}}),1,IF(OR({v}={{{}}}),0,\"\"))",
        words(&TRUE_WORDS),
        words(&FALSE_WORDS)
    )
}

fn bd(start: &str, n: u32) -> String {
    format!("_xlfn.WORKDAY.INTL({start},{n},1,tblHolidays[date])")
}

/// A register date column as a whole calendar day. A blank cell stays blank and
/// a non-numeric cell passes through, so `Row_problem`'s own ISNUMBER guards
/// still see what they are there to refuse.
fn whole_day(column: &str) -> String {
    let c = t(column);
    format!("=IF({c}=\"\",\"\",IF(ISNUMBER({c}),INT({c}),{c}))")
}

/// The calculated register columns, each a formula with its leading `=`.
fn formulas() -> BTreeMap<&'static str, String> {
    let (pay, sg) = (t("Pay_day"), t("sg_amount"));
    let (rem, rem_amt, matched) = (t("Remit_day"), t("remitted_amount"), t("matched_amount"));
    let (rec, nxt) = (t("Recv_day"), t("Next_day"));
    let (db, ooc, ftf) = (t("Flag_db"), t("Flag_ooc"), t("Flag_ftf"));
    let (due, settled, remit) = (t("Final_due"), t("Settled"), t("Remit"));
    let poss = t("Possible_item4");
    let horizon = format!("({due}>{COVERAGE})");
    let unc = format!("({poss}<>\"\")");
    let later_gate = format!("OR({horizon},AND({unc},{AS_AT}<={poss}))");
    // Both sides in cents; a receipt date evidences an amount only where a row
    // states one.
    let evidenced = format!("OR(ISNUMBER({matched}),ISNUMBER({rem_amt}))");
    let covers = format!("AND({}>=ROUND({sg},2),{evidenced})", t("Receipt_credit"));
    // Mirrors _receipt_could_be_timely: a pre-payment is judged only by the
    // 12-month window; every other receipt is timely on or before the proved
    // deadline or where the real deadline is unsettled.
    let timely_possible = format!(
        "IF({settled}<{pay},{settled}>={},OR({settled}<={due},{horizon},AND({unc},{settled}<={poss})))",
        t("Earliest_prepay")
    );
    let nec_from = format!("({due}+1)");
    let nec_end = t("NEC_end");
    // Elementwise min and max via ABS: MIN and MAX aggregate a whole column
    // inside SUMPRODUCT.
    let seg_end = format!("(({nec_end}+tblGic[to]-ABS({nec_end}-tblGic[to]))/2)");
    let seg_start = format!("(({nec_from}+tblGic[from]+ABS({nec_from}-tblGic[from]))/2)");
    let seg_days = format!("({seg_end}-{seg_start}+1)");
    let segments = format!("(({seg_days}>0)*{seg_days})");
    let v = t("Verdict");
    let exposed = format!("({}=1)", t("Exposed"));
    let join = |f: &dyn Fn(&str) -> String, columns: &[&str]| {
        columns.iter().map(|c| f(c)).collect::<Vec<_>>().join(",")
    };
    let date_bad = join(
        &|c| format!("AND({0}<>\"\",NOT(ISNUMBER({0})))", t(c)),
        &DATE_COLUMNS,
    );
    let amount_bad = join(
        &|c| format!("AND({0}<>\"\",OR(NOT(ISNUMBER({0})),{0}<0))", t(c)),
        &AMOUNT_COLUMNS,
    );
    // The checker refuses a placeholder such as 9999-12-31 and a nonzero amount
    // that rounds to nothing.
    let far = format!("DATE({LATEST_SANE_YEAR},12,31)");
    let date_far = join(
        &|c| format!("AND(ISNUMBER({0}),{0}>{far})", t(c)),
        &DATE_COLUMNS,
    );
    let sub_cent = join(
        &|c| format!("AND(ISNUMBER({0}),{0}<>0,ROUND({0},2)=0)", t(c)),
        &AMOUNT_COLUMNS,
    );
    let bool_bad = join(
        &|s| format!("{}=\"\"", t(&format!("Flag_{s}"))),
        &["ftf", "ooc", "db"],
    );
    let (own_due, cap, prepay) = (t("Own_due"), t("Cap"), t("Earliest_prepay"));
    let (confirmed, possible) = (t("Confirmed_latest"), t("Possible_latest"));
    let (branch, stale, credit) = (t("Branch"), t("Stale_prepay"), t("Receipt_credit"));
    let (base, final_short, past_gic) = (
        t("Base_shortfall"),
        t("Final_shortfall"),
        t("Past_gic_table"),
    );
    let emp = t("employee_id");
    let code = |column: &str| format!("INDEX(tblCodes[{column}],MATCH({branch},tblCodes[code],0))");
    let row_problem = [
        format!("=TRIM(IF(TRIM({emp}&\"\")=\"\",\"employee_id \",\"\")"),
        format!("&IF(OR({date_bad}),\"date \",\"\")"),
        format!("&IF(OR({date_far}),\"date-not-real \",\"\")"),
        format!("&IF(NOT(ISNUMBER({pay})),\"payment_date \",\"\")"),
        format!(
            "&IF(AND(ISNUMBER({pay}),{pay}<{}),\"pre-1-Jul-2026 \",\"\")",
            excel_date(REGIME_START)
        ),
        format!("&IF(OR({amount_bad}),\"amount \",\"\")"),
        format!("&IF(OR({sub_cent}),\"amount-under-half-a-cent \",\"\")"),
        format!("&IF(NOT(ISNUMBER({sg})),\"sg_amount \",\"\")"),
        format!("&IF(OR({bool_bad}),\"yes/no \",\"\")"),
        format!("&IF(AND(ISNUMBER({rem_amt}),ISNUMBER({sg}),ROUND({rem_amt},2)>ROUND({sg},2)),\"remitted_amount>sg \",\"\")"),
        format!("&IF(AND(ISNUMBER({rem_amt}),NOT(ISNUMBER({rem}))),\"remitted_amount-needs-date \",\"\")"),
        format!("&IF(AND(ISNUMBER({matched}),ISNUMBER({sg}),ROUND({matched},2)>ROUND({sg},2)),\"matched_amount>sg \",\"\")"),
        format!("&IF(AND(ISNUMBER({matched}),ISNUMBER({rem_amt}),ROUND({rem_amt},2)>ROUND({matched},2)),\"remitted_amount>matched \",\"\")"),
        format!("&IF(AND(ISNUMBER({matched}),ISNUMBER({sg}),ROUND({matched},2)<ROUND({sg},2),ISNUMBER({rem}),NOT(ISNUMBER({rem_amt}))),\"partial-match-needs-remitted_amount \",\"\")"),
        format!("&IF(AND(ISNUMBER({rec}),ISNUMBER({rem}),{rec}<{rem}),\"receipt-before-remittance \",\"\")"),
        format!("&IF(AND({ooc}=1,NOT(ISNUMBER({nxt}))),\"out_of_cycle-needs-next_standard_payday \",\"\")"),
        format!("&IF(AND({ooc}=1,ISNUMBER({nxt}),ISNUMBER({pay}),{nxt}<={pay}),\"next_standard_payday-not-after-payday \",\"\"))"),
    ]
    .concat();
    let guard = INPUT_COLUMNS
        .iter()
        .map(|c| format!("_xlfn.ISFORMULA({})", t(c)))
        .collect::<Vec<_>>()
        .join(",");
    let branch_formula = [
        format!("=IF({db}=1,\"DB\",IF({due}=\"\",\"\",IF(ROUND({sg},2)<=0,\"NIL\","),
        format!("IF({settled}<>\"\","),
        // Received: a receipt with no stated amount that could be timely is
        // left between the full and partial outcomes (U codes).
        format!("IF(AND(NOT({evidenced}),{timely_possible}),"),
        format!("IF({settled}>{due},\"U1\",IF({due}<{AS_AT},\"U2\",\"U3\")),"),
        format!("IF({settled}<{pay},"),
        format!("IF({settled}>={prepay},"),
        format!("IF({covers},\"A1\",IF({due}>={AS_AT},\"A2\",IF({later_gate},\"A3\",\"A4\"))),"),
        format!("IF({due}>={AS_AT},\"S1\",IF({later_gate},\"S2\",\"S3\"))),"),
        format!("IF(AND({unc},{due}<{settled},{settled}<={poss}),"),
        format!("IF({covers},\"B1\",IF({AS_AT}<={poss},\"B2\",\"B3\")),"),
        format!("IF(AND({horizon},{settled}>{due}),IF({covers},\"C1\",\"C2\"),"),
        format!("IF({settled}<={due},IF({covers},\"D1\",IF({later_gate},\"D2\",IF({due}<{AS_AT},\"D3\",\"D4\"))),\"E\"))))),"),
        // Without receipt.
        format!("IF(AND({remit}<>\"\",{sg}-{}<=0),", t("Credit")),
        format!("IF(AND({unc},{due}<{remit},{remit}<={poss}),\"R1\","),
        format!("IF(AND({horizon},{remit}>{due}),\"R2\",IF({remit}<={due},\"R3\",\"R4\"))),"),
        format!("IF({due}<{AS_AT},IF({later_gate},\"N1\",\"N2\"),\"N3\"))))))"),
    ]
    .concat();
    let receipt_applies = format!("{settled}<>\"\",{stale}=0");
    let latest = |evidence: &str| {
        format!(
            "=IF(OR({db}=1,{own_due}=0),\"\",SUMPRODUCT(MAX(EXACT(tblLines[employee_id],{emp})*(tblLines[payment_date]<{pay})*(tblLines[Evidence_own]{evidence})*tblLines[Own_due])))"
        )
    };
    let withheld = |then: String| format!("=IF({exposed},IF({past_gic}=1,\"\",{then}),\"\")");
    BTreeMap::from([
        ("Pay_day", whole_day("payment_date")),
        ("Remit_day", whole_day("remitted_date")),
        ("Recv_day", whole_day("fund_received_date")),
        ("Next_day", whole_day("next_standard_payday")),
        ("Flag_db", format!("={}", bool_expr("defined_benefit"))),
        ("Flag_ooc", format!("={}", bool_expr("out_of_cycle"))),
        (
            "Flag_ftf",
            format!("={}", bool_expr("first_contribution_to_fund")),
        ),
        ("Row_problem", row_problem),
        ("Guard", format!("=IF(OR({guard}),1,0)")),
        (
            "Cap",
            format!("=IF(ISNUMBER({matched}),{matched},IF(ISNUMBER({rem_amt}),{rem_amt},{sg}))"),
        ),
        (
            "Due_ooc",
            format!("=IF(AND({ooc}=1,ISNUMBER({nxt})),{},\"\")", bd(&nxt, 7)),
        ),
        ("Due_ftf", format!("=IF({ftf}=1,{},\"\")", bd(&pay, 20))),
        // 0, not "", where there is no deadline: the item 4 lookups multiply this
        // column inside SUMPRODUCT.
        (
            "Own_due",
            format!(
                "=IF(OR({db}=1,NOT(ISNUMBER({pay}))),0,IF(OR({ooc}=1,{ftf}=1),MAX({},{}),{}))",
                t("Due_ooc"),
                t("Due_ftf"),
                bd(&pay, 7)
            ),
        ),
        (
            "Own_pathway",
            format!(
                "=IF({db}=1,\"SKIP_DB\",IF({ooc}=1,IF(AND({ftf}=1,{}>{}),\"EXTENDED_20BD\",\"OUT_OF_CYCLE\"),IF({ftf}=1,\"EXTENDED_20BD\",\"USUAL_7BD\")))",
                t("Due_ftf"),
                t("Due_ooc")
            ),
        ),
        (
            "Earliest_prepay",
            format!("=IF(ISNUMBER({pay}),_xlfn.EDATE({pay}-1,-12)+1,\"\")"),
        ),
        (
            "Settled",
            format!("=IF(AND(ISNUMBER({rec}),{rec}<={AS_AT}),{rec},\"\")"),
        ),
        (
            "Remit",
            format!("=IF(AND(ISNUMBER({rem}),{rem}<={AS_AT}),{rem},\"\")"),
        ),
        (
            "Receipt_credit",
            format!("=IF({settled}<>\"\",MIN(ROUND({cap},2),ROUND({sg},2)),0)"),
        ),
        (
            "Credit",
            format!(
                "=IF(ISNUMBER({rem_amt}),IF({remit}<>\"\",MIN(ROUND({rem_amt},2),ROUND({sg},2)),0),IF({remit}<>\"\",ROUND({sg},2),0))"
            ),
        ),
        // Evidence against the row's own deadline.
        (
            "Evidence_own",
            format!(
                "=IF(OR({own_due}=0,ROUND({sg},2)<=0,{db}=1,ROUND({cap},2)<=0),\"impossible\",IF(ISNUMBER({rec}),IF({rec}<{prepay},\"impossible\",IF({rec}<={AS_AT},IF(OR({rec}<{pay},{rec}<={own_due}),\"confirmed\",IF({rec}>{own_due},\"impossible\",\"possible\")),IF({rec}<={own_due},\"possible\",\"impossible\"))),IF(AND(ISNUMBER({rem}),{rem}>{own_due}),\"impossible\",\"possible\")))"
            ),
        ),
        // EXACT keeps employee ids case-sensitive, as the engine groups them.
        ("Confirmed_latest", latest("=\"confirmed\"")),
        ("Possible_latest", latest("<>\"impossible\"")),
        (
            "Final_due",
            format!(
                "=IF({own_due}=0,\"\",IF(AND(ISNUMBER({confirmed}),{confirmed}>{own_due}),{confirmed},{own_due}))"
            ),
        ),
        (
            "Pathway",
            format!(
                "=IF({own_due}=0,{0},IF(AND(ISNUMBER({confirmed}),{confirmed}>{own_due}),\"ITEM4_ALIGNED\",{0}))",
                t("Own_pathway")
            ),
        ),
        (
            "Case_variant",
            format!(
                "=IF(COUNTIF(tblLines[employee_id],{emp})>SUMPRODUCT(--EXACT(tblLines[employee_id],{emp})),1,0)"
            ),
        ),
        (
            "Sample_row",
            format!(
                "=IF(AND(ISNUMBER(MATCH({emp},{{SAMPLE_IDS}},0)),ISNUMBER(MATCH({pay},{{SAMPLE_DATES}},0))),1,0)"
            ),
        ),
        (
            "Possible_item4",
            format!(
                "=IF({due}=\"\",\"\",IF(AND(ISNUMBER({possible}),{possible}>{due}),{possible},\"\"))"
            ),
        ),
        (
            "Past_horizon",
            format!("=IF({due}=\"\",\"\",IF({horizon},1,0))"),
        ),
        ("Branch", branch_formula),
        (
            "Verdict",
            format!("=IF({branch}=\"\",\"\",{})", code("verdict")),
        ),
        // T() turns the 0 that INDEX returns for an empty Codes cell back into "".
        (
            "Unassessable_between",
            format!("=IF({branch}=\"\",\"\",T({}))", code("unassessable")),
        ),
        (
            "Exposed",
            format!("=IF({branch}=\"\",0,{})", code("exposed")),
        ),
        (
            "Stale_prepay",
            format!("=IF({branch}=\"\",0,{})", code("stale")),
        ),
        (
            "OTRC",
            format!("=IF({branch}=\"\",0,{}*{credit})", code("otrc")),
        ),
        (
            "Base_shortfall",
            format!("=IF({exposed},MAX(ROUND({sg},2)-{},0),\"\")", t("OTRC")),
        ),
        (
            "Offset_s18D",
            format!(
                "=IF({exposed},IF(AND({receipt_applies},{settled}>{due},{evidenced},OR({ASSESS}=\"\",{settled}<{ASSESS})),MIN({credit},{base}),0),\"\")"
            ),
        ),
        (
            "Final_shortfall",
            format!("=IF({exposed},MAX({base}-{},0),\"\")", t("Offset_s18D")),
        ),
        (
            "Lateness_basis",
            format!(
                "=IF({exposed},IF(AND({receipt_applies},{covers}),\"fund receipt\",IF(AND({receipt_applies},NOT({evidenced})),\"as-at date (fund receipt amount not evidenced)\",IF(AND({receipt_applies}),\"as-at date (shortfall remains after part receipt)\",IF(AND(OR({settled}<>\"\",{remit}<>\"\"),{stale}=0),\"as-at date (no fund receipt recorded)\",\"as-at date (nothing applied to this payday)\")))),\"\")"
            ),
        ),
        (
            "Outstanding_to",
            format!("=IF({exposed},IF(AND({receipt_applies},{covers}),{settled},{AS_AT}),\"\")"),
        ),
        // No fixed horizon: the accrual ends on the assessment date or the day
        // the money stops being outstanding, exactly as the checker does.
        (
            "NEC_end",
            format!(
                "=IF({exposed},IF({ASSESS}=\"\",{0},MIN({ASSESS}-1,{0})),\"\")",
                t("Accrual_to")
            ),
        ),
        (
            "Accrual_to",
            format!(
                "=IF({exposed},IF(AND({receipt_applies},{covers},{final_short}=0),{settled},{AS_AT}),\"\")"
            ),
        ),
        (
            "Days_late",
            format!(
                "=IF({exposed},IF({}=1,\"\",MAX({}-{due},0)),\"\")",
                t("Past_horizon"),
                t("Outstanding_to")
            ),
        ),
        // A day past the last published GIC quarter has no rate; the workbook
        // withholds the notional earnings and both exposure totals.
        (
            "Past_gic_table",
            format!("=IF({exposed},IF({nec_end}>{GIC_LAST},1,0),\"\")"),
        ),
        (
            "NEC",
            withheld(format!(
                "IF({nec_end}>{due},{base}*(EXP(SUMPRODUCT({segments}*LN(1+tblGic[annual_pct]/100/tblGic[divisor])))-1),0)"
            )),
        ),
        (
            "Shortfall_r",
            format!("=IF({exposed},ROUND({final_short},2),\"\")"),
        ),
        ("NEC_r", withheld(format!("ROUND({},2)", t("NEC")))),
        ("Uplift_best", withheld("0".into())),
        (
            "Uplift_worst",
            withheld(format!("ROUND(0.6*({final_short}+{}),2)", t("NEC"))),
        ),
        (
            "SGC_low",
            withheld(format!(
                "{}+{}+{}",
                t("Shortfall_r"),
                t("NEC_r"),
                t("Uplift_best")
            )),
        ),
        (
            "SGC_high",
            withheld(format!(
                "{}+{}+{}",
                t("Shortfall_r"),
                t("NEC_r"),
                t("Uplift_worst")
            )),
        ),
        (
            "Transition_row",
            format!(
                "=IF(AND({db}<>1,ISNUMBER({sg}),ROUND({sg},2)>0,ROUND({cap},2)>0,OR(ISNUMBER({rec}),ISNUMBER({rem})),IF(ISNUMBER({rec}),{rec},{rem})<={}),1,0)",
                excel_date(TRANSITION_END)
            ),
        ),
        ("Receipt_established", format!("=IF({settled}<>\"\",1,0)")),
        (
            "Assessable",
            format!("=IF(AND({v}<>\"\",{v}<>\"SKIPPED\",ISNUMBER({sg}),ROUND({sg},2)>0),1,0)"),
        ),
        (
            "Duplicate",
            format!(
                "=IF(COUNTIFS(tblLines[employee_id],{emp},tblLines[payment_date],{pay},tblLines[sg_amount],{sg},tblLines[remitted_date],{rem}&\"\",tblLines[fund_received_date],{rec}&\"\")>1,1,0)"
            ),
        ),
    ])
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
/// keep the head style. `formulas` gives calculated columns without their `=`.
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

/// The workbook, with the array flag repaired on the calculated columns that
/// need it.
pub fn build() -> Result<Vec<u8>> {
    let rows = read_sample()?;
    let Calendar {
        holidays,
        provisional,
        verified_until,
    } = read_holidays()?;
    let (gic, gic_last) = read_gic()?;
    let mut calc = formulas();
    let sample_ids = rows
        .iter()
        .map(|r| format!("\"{}\"", r.get("employee_id").map_or("", String::as_str)))
        .collect::<Vec<_>>()
        .join(",");
    let sample_dates = rows
        .iter()
        .map(|r| Ok(serial(iso(r.get("payment_date").map_or("", String::as_str))?).to_string()))
        .collect::<Result<Vec<_>>>()?
        .join(",");
    let sample = calc.get_mut("Sample_row").expect("Sample_row");
    *sample = sample
        .replace("{SAMPLE_IDS}", &format!("{{{sample_ids}}}"))
        .replace("{SAMPLE_DATES}", &format!("{{{sample_dates}}}"));
    assert!(calc.keys().copied().eq({
        let mut order = CALC_ORDER.to_vec();
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
        "Payday Super review: s 18C deadlines and the SG charge estimate",
        &Format::new()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::RGB(PURPLE)),
    )?;
    let steps = [
        "1. On Register, paste one contribution line per row over the example rows, in the eleven canonical columns the command line tool reads and the importer writes. Dates are real Excel dates; amounts are dollars; yes/no columns take yes, no or blank (no).",
        "2. On Summary, set the as-at date (notional earnings on unpaid lines run to it), an assessment date if the ATO has assessed, and the two confirmations the command line tool asks for: LCR 2026/1 transition allocation and remittance-only review.",
        "3. Read the calculated columns on Register: pathway, final deadline, verdict (ON_TIME, AT_RISK, LATE, UNPAID, UNKNOWN, SKIPPED), the conservative outer outcomes where the facts cannot decide, days late, final shortfall after s 18D, notional earnings, the uplift range and the experimental SG charge estimate range.",
        "4. Review Checks: BLOCKED means the register cannot be read or a required confirmation is missing. REVIEW means a line is exposed, a line is undecided, or the file cannot produce ON_TIME and remittance-only review is not confirmed. PASS mirrors the command line tool's exit code 0.",
        "5. The statutory test is receipt by the fund. A remittance date alone gives AT_RISK, never ON_TIME. Fill fund_received_date from the clearing house or fund before treating a verdict as final.",
        "6. Experimental review aid. Not a compliance determination. The ATO makes the assessment; choice loading, the maximum contributions base and post-assessment charges are not estimated.",
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
        Formula::new("='Review Checks'!B16"),
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
            "payday-super-checker workbook, engine {VERSION}. Macro-free. Not advice; see DISCLAIMER.md in the repository."
        ),
    )?;

    // 2. Register
    let ws = wb.add_worksheet().set_name("Register")?;
    let titles: Vec<&str> = INPUT_COLUMNS.iter().chain(&CALC_ORDER).copied().collect();
    let last_row = rows.len() as u32;
    add_table(ws, "tblLines", &titles, last_row, &calc, &s.head)?;
    for (r, row) in rows.iter().enumerate() {
        let r = r as u32 + 1;
        for (c, column) in INPUT_COLUMNS.iter().enumerate() {
            let c = c as u16;
            let raw = row.get(*column).map_or("", String::as_str);
            if DATE_COLUMNS.contains(column) {
                let format = s.input.clone().set_num_format(DATE);
                match raw {
                    "" => ws.write_blank(r, c, &format)?,
                    _ => ws.write_number_with_format(r, c, serial(iso(raw)?), &format)?,
                };
            } else if AMOUNT_COLUMNS.contains(column) {
                let format = s.input.clone().set_num_format(MONEY);
                match raw {
                    "" => ws.write_blank(r, c, &format)?,
                    _ => ws.write_number_with_format(r, c, raw.parse::<f64>()?, &format)?,
                };
            } else {
                let format = s.input.clone().set_num_format(TEXT);
                match raw {
                    "" => ws.write_blank(r, c, &format)?,
                    _ => ws.write_string_with_format(r, c, raw, &format)?,
                };
            }
        }
        for (c, column) in CALC_ORDER.iter().enumerate() {
            let c = (INPUT_COLUMNS.len() + c) as u16;
            let mut format = s.calc.clone();
            if DATE_CALCS.contains(column) {
                format = format.set_num_format(DATE);
            } else if MONEY_CALCS.contains(column) {
                format = format.set_num_format(MONEY);
            }
            // Array formulas: a same-table column inside EXACT would otherwise
            // be rewritten to a this-row reference on load.
            let formula = Formula::new(&calc[column]);
            if ARRAY_CALCS.contains(column) {
                ws.write_array_formula_with_format(r, c, r, c, formula, &format)?;
            } else {
                ws.write_formula_with_format(r, c, formula, &format)?;
            }
        }
    }
    for column in BOOL_COLUMNS {
        let c = INPUT_COLUMNS
            .iter()
            .position(|x| *x == column)
            .expect("input") as u16;
        let rule = DataValidation::new()
            .allow_list_strings(&["yes", "no"])?
            .show_input_message(false)
            .show_error_message(false);
        ws.add_data_validation(1, c, last_row + 1000, c, &rule)?;
    }
    for c in 0..titles.len() {
        width(ws, c as u16, 13)?;
    }
    for (title, chars) in [
        ("Row_problem", 30),
        ("Lateness_basis", 34),
        ("Unassessable_between", 22),
    ] {
        let c = titles.iter().position(|x| *x == title).expect("title") as u16;
        width(ws, c, chars)?;
    }
    ws.set_freeze_panes(1, 1)?;

    // 3. Codes (verdict lookup)
    let ws = wb.add_worksheet().set_name("Codes")?;
    let titles = [
        "code",
        "verdict",
        "unassessable",
        "otrc",
        "stale",
        "exposed",
    ];
    add_table(
        ws,
        "tblCodes",
        &titles,
        CODES.len() as u32,
        &BTreeMap::new(),
        &s.head,
    )?;
    for (r, (code, verdict, between, otrc, stale, exposed)) in CODES.iter().enumerate() {
        let r = r as u32 + 1;
        ws.write_string(r, 0, *code)?;
        ws.write_string(r, 1, *verdict)?;
        if !between.is_empty() {
            ws.write_string(r, 2, *between)?;
        }
        for (c, value) in [(3, otrc), (4, stale), (5, exposed)] {
            ws.write_number(r, c, f64::from(*value))?;
        }
    }
    width(ws, 2, 24)?;
    protect(ws);

    // 4. Holidays
    let ws = wb.add_worksheet().set_name("Holidays")?;
    let titles = ["date", "name", "jurisdictions"];
    add_table(
        ws,
        "tblHolidays",
        &titles,
        holidays.len() as u32,
        &BTreeMap::new(),
        &s.head,
    )?;
    let input_date = s.input.clone().set_num_format(DATE);
    for (r, (day, name, jurisdictions)) in holidays.iter().enumerate() {
        let r = r as u32 + 1;
        ws.write_number_with_format(r, 0, serial(*day), &input_date)?;
        ws.write_string_with_format(r, 1, name, &s.input)?;
        ws.write_string_with_format(r, 2, jurisdictions, &s.input)?;
    }
    width(ws, 1, 50)?;
    width(ws, 2, 40)?;
    let note_row = holidays.len() as u32 + 3;
    ws.write_string(
        note_row,
        0,
        format!(
            "Whole-of-jurisdiction public holidays under SGAA s 6(1); weekends are computed. Official sources checked through {}. To review a later deadline, add the official dates here and move the coverage date on Summary.",
            verified_until.isoformat()
        ),
    )?;
    ws.write_string(
        note_row + 1,
        0,
        format!(
            "Provisional dates not applied (business days until confirmed): {}",
            provisional
                .iter()
                .map(|(d, n)| format!("{} {n}", d.isoformat()))
                .collect::<Vec<_>>()
                .join("; ")
        ),
    )?;

    // 5. GIC
    let ws = wb.add_worksheet().set_name("GIC")?;
    let titles = ["from", "to", "annual_pct", "divisor", "basis", "seen"];
    add_table(
        ws,
        "tblGic",
        &titles,
        gic.len() as u32,
        &BTreeMap::new(),
        &s.head,
    )?;
    for (r, (start, end, pct, divisor, seen)) in gic.iter().enumerate() {
        let r = r as u32 + 1;
        ws.write_number_with_format(r, 0, serial(*start), &input_date)?;
        ws.write_number_with_format(r, 1, serial(*end), &input_date)?;
        ws.write_number_with_format(r, 2, *pct, &s.input)?;
        ws.write_number_with_format(r, 3, *divisor as f64, &s.input)?;
        ws.write_string_with_format(r, 4, "known", &s.input)?;
        ws.write_string_with_format(r, 5, seen, &s.input)?;
    }
    for c in 0..3 {
        width(ws, c, 14)?;
    }
    ws.write_string(
        gic.len() as u32 + 3,
        0,
        "General interest charge, TAA 1953 s 8AAD: annual rate divided by the days in the calendar year. Only the ATO's published quarters are listed. A line whose accrual runs past the last row keeps its verdict, days late and shortfall and shows no notional earnings or charge estimate, as the checker does without --allow-stale-gic; the workbook has no opt-in. Add each new quarter from the ATO GIC rates page.",
    )?;
    width(ws, 5, 14)?;

    // 6. Summary
    let ws = wb.add_worksheet().set_name("Summary")?;
    ws.write_string_with_format(0, 0, "Inputs", &s.head)?;
    ws.write_blank(0, 1, &s.head)?;
    let (y, m, d) = DEFAULT_AS_AT;
    // An input is a date (blank when None) or a Y/N word.
    enum Entry {
        Day(Option<Date>),
        Word(&'static str),
    }
    let inputs = [
        (
            1,
            "As-at date (notional earnings on unpaid lines run to it)",
            Entry::Day(Some(Date::new(y, m, d)?)),
        ),
        (
            2,
            "ATO assessment date (blank if none has issued)",
            Entry::Day(None),
        ),
        (
            3,
            "LCR 2026/1 transition allocation reconciled and confirmed (Y/N)",
            Entry::Word("N"),
        ),
        (4, "Remittance-only review accepted (Y/N)", Entry::Word("N")),
        (
            5,
            "Holiday coverage verified until",
            Entry::Day(Some(verified_until)),
        ),
    ];
    for (r, label, entry) in inputs {
        ws.write_string(r, 0, label)?;
        let format = s.input.clone().set_unlocked();
        match entry {
            Entry::Day(Some(day)) => {
                ws.write_number_with_format(r, 1, serial(day), &format.set_num_format(DATE))?
            }
            Entry::Day(None) => ws.write_blank(r, 1, &format.set_num_format(DATE))?,
            Entry::Word(word) => ws.write_string_with_format(r, 1, word, &format)?,
        };
    }
    ws.write_string(6, 0, "GIC table last known quarter ends")?;
    let calc_date = s.calc.clone().set_num_format(DATE);
    ws.write_formula_with_format(
        6,
        1,
        Formula::new("=_xlfn.MAXIFS(tblGic[to],tblGic[basis],\"known\")"),
        &calc_date,
    )?;
    for r in [3, 4] {
        let rule = DataValidation::new()
            .allow_list_strings(&["Y", "N"])?
            .ignore_blank(false)
            .show_input_message(false)
            .show_error_message(false);
        ws.add_data_validation(r, 1, r, 1, &rule)?;
    }
    for (r, label, input) in [
        (
            1,
            "As-at date used by the register (whole day)",
            AS_AT_INPUT,
        ),
        (
            2,
            "Assessment date used by the register (whole day)",
            ASSESS_INPUT,
        ),
    ] {
        ws.write_string(r, 2, label)?;
        ws.write_formula_with_format(
            r,
            3,
            Formula::new(format!(
                "=IF({input}=\"\",\"\",IF(ISNUMBER({input}),INT({input}),{input}))"
            )),
            &calc_date,
        )?;
    }
    for (r, blank_ok) in [(1, false), (2, true), (5, false)] {
        let rule = DataValidation::new()
            .allow_date_formula(DataValidationRule::GreaterThan(Formula::new("1")))
            .ignore_blank(blank_ok)
            .show_input_message(false)
            .show_error_message(false);
        ws.add_data_validation(r, 1, r, 1, &rule)?;
    }
    ws.write_string_with_format(8, 0, "Summary", &s.head)?;
    ws.write_blank(8, 1, &s.head)?;
    let not_assessed = |column: &str| {
        format!(
            "=IF(SUM(tblLines[Past_gic_table])>0,\"Not assessed\",ROUND(SUM(tblLines[{column}]),2))"
        )
    };
    let summary = [
        ("ON_TIME", "=COUNTIF(tblLines[Verdict],\"ON_TIME\")".to_string()),
        ("AT_RISK", "=COUNTIF(tblLines[Verdict],\"AT_RISK\")".into()),
        ("LATE", "=COUNTIF(tblLines[Verdict],\"LATE\")".into()),
        ("UNPAID", "=COUNTIF(tblLines[Verdict],\"UNPAID\")".into()),
        ("UNKNOWN", "=COUNTIF(tblLines[Verdict],\"UNKNOWN\")".into()),
        ("SKIPPED", "=COUNTIF(tblLines[Verdict],\"SKIPPED\")".into()),
        ("Lines with exposure", "=SUM(tblLines[Exposed])".into()),
        (
            "Total final shortfall",
            "=ROUND(SUM(tblLines[Shortfall_r]),2)".into(),
        ),
        ("Total notional earnings", not_assessed("NEC_r")),
        ("Experimental SG charge estimate, low", not_assessed("SGC_low")),
        ("Experimental SG charge estimate, high", not_assessed("SGC_high")),
        (
            "Attention-driving UNKNOWN lines",
            "=SUMPRODUCT(--(tblLines[Unassessable_between]<>\"\"))".into(),
        ),
        (
            "Remittance-only file (no assessable line has a fund receipt)",
            "=IF(AND(SUM(tblLines[Assessable])>0,SUMPRODUCT(tblLines[Assessable]*tblLines[Receipt_established])=0),\"YES\",\"NO\")".into(),
        ),
    ];
    let calc_money = s.calc.clone().set_num_format(MONEY);
    for (i, (label, formula)) in summary.iter().enumerate() {
        let r = 9 + i as u32;
        ws.write_string(r, 0, *label)?;
        let money = ["shortfall", "earnings", "estimate"]
            .iter()
            .any(|w| label.contains(w));
        let format = if money { &calc_money } else { &s.calc };
        ws.write_formula_with_format(r, 1, Formula::new(formula), format)?;
    }
    ws.write_string(23, 0, "Overall status")?;
    let calc_bold = s.calc.clone().set_bold().set_font_color(Color::Automatic);
    ws.write_formula_with_format(23, 1, Formula::new("='Review Checks'!B16"), &calc_bold)?;
    width(ws, 0, 64)?;
    width(ws, 1, 18)?;
    protect(ws);

    // 7. Review Checks
    let ws = wb.add_worksheet().set_name("Review Checks")?;
    header(
        ws,
        &[
            "Check",
            "Result",
            "Count or value",
            "Example row (employee_id)",
        ],
        &s.head,
    )?;
    let offender = |cond: &str| {
        format!(
            "INDEX(tblLines[employee_id],SUMPRODUCT(MAX({cond}*(ROW(tblLines[employee_id])-1))))"
        )
    };
    let far = format!("DATE({LATEST_SANE_YEAR},12,31)");
    let example = |cell: &str, cond: &str| Some(format!("=IF({cell}=0,\"\",{})", offender(cond)));
    let checks: [(&str, String, String, Option<String>); 13] = [
        (
            "Every register value can be read",
            "=IF(C2=0,\"PASS\",\"BLOCKED\")".into(),
            "=SUMPRODUCT(--(tblLines[Row_problem]<>\"\"))".into(),
            example("C2", "(tblLines[Row_problem]<>\"\")"),
        ),
        (
            "No formula in an input cell",
            "=IF(C3=0,\"PASS\",\"BLOCKED\")".into(),
            "=SUM(tblLines[Guard])".into(),
            example("C3", "(tblLines[Guard]=1)"),
        ),
        (
            "Contributions dated on or before 28 July 2026 are reconciled under LCR 2026/1",
            format!("=IF(OR(C4=0,{TRANSITION_OK}=\"Y\"),\"PASS\",\"BLOCKED\")"),
            "=SUM(tblLines[Transition_row])".into(),
            example("C4", "(tblLines[Transition_row]=1)"),
        ),
        (
            "No line is LATE or UNPAID",
            "=IF(C5=0,\"PASS\",\"REVIEW\")".into(),
            "=SUM(tblLines[Exposed])".into(),
            example("C5", "(tblLines[Exposed]=1)"),
        ),
        (
            "No line is left undecided by the deadline facts",
            "=IF(C6=0,\"PASS\",\"REVIEW\")".into(),
            "=SUMPRODUCT(--(tblLines[Unassessable_between]<>\"\"))".into(),
            example("C6", "(tblLines[Unassessable_between]<>\"\")"),
        ),
        (
            "The file can produce ON_TIME, or remittance-only review is accepted",
            format!("=IF(OR(Summary!B22=\"NO\",{REMIT_ONLY_OK}=\"Y\"),\"PASS\",\"REVIEW\")"),
            "=Summary!B22".into(),
            None,
        ),
        (
            "Deadlines inside the holiday coverage (figures past it are a maximum)",
            "=IF(C8=0,\"PASS\",\"NOTE\")".into(),
            "=SUM(tblLines[Past_horizon])".into(),
            example("C8", "(tblLines[Past_horizon]=1)"),
        ),
        (
            "Accruals past the GIC table (charge estimate withheld until the rate is added)",
            "=IF(C9=0,\"PASS\",\"NOTE\")".into(),
            "=SUM(tblLines[Past_gic_table])".into(),
            example("C9", "(tblLines[Past_gic_table]=1)"),
        ),
        (
            "No identical lines (a doubled export counts a payday twice)",
            "=IF(C10=0,\"PASS\",\"NOTE\")".into(),
            "=SUM(tblLines[Duplicate])".into(),
            example("C10", "(tblLines[Duplicate]=1)"),
        ),
        (
            "Lines assessed at a nil SG amount (nothing to assess)",
            "=IF(C11=0,\"PASS\",\"NOTE\")".into(),
            "=COUNTIF(tblLines[Branch],\"NIL\")".into(),
            example("C11", "(tblLines[Branch]=\"NIL\")"),
        ),
        (
            "As-at, assessment and coverage dates on Summary are real dates",
            format!(
                "=IF(AND(ISNUMBER({AS_AT}),{AS_AT}<={far},OR({ASSESS}=\"\",AND(ISNUMBER({ASSESS}),{ASSESS}<={far})),AND(ISNUMBER({COVERAGE}),{COVERAGE}<={far})),\"PASS\",\"BLOCKED\")"
            ),
            format!("={AS_AT}"),
            None,
        ),
        (
            "No fabricated example line from the shipped sample remains in the register",
            "=IF(C13=0,\"PASS\",\"REVIEW\")".into(),
            "=SUM(tblLines[Sample_row])".into(),
            example("C13", "(tblLines[Sample_row]=1)"),
        ),
        (
            "No employee ids differ only by capitalisation (treated as different people, not aligned)",
            "=IF(C14=0,\"PASS\",\"NOTE\")".into(),
            "=SUM(tblLines[Case_variant])".into(),
            example("C14", "(tblLines[Case_variant]=1)"),
        ),
    ];
    for (i, (label, result, detail, example)) in checks.iter().enumerate() {
        let r = 1 + i as u32;
        ws.write_string(r, 0, *label)?;
        ws.write_formula_with_format(r, 1, Formula::new(result), &s.calc)?;
        ws.write_formula_with_format(r, 2, Formula::new(detail), &s.calc)?;
        match example {
            Some(f) => ws.write_array_formula_with_format(r, 3, r, 3, Formula::new(f), &s.calc)?,
            None => ws.write_blank(r, 3, &s.calc)?,
        };
    }
    let last = checks.len() + 1;
    ws.write_string_with_format(15, 0, "Overall status", &bold)?;
    ws.write_formula_with_format(
        15,
        1,
        Formula::new(format!(
            "=IF(COUNTIF(B2:B{last},\"BLOCKED\")>0,\"BLOCKED\",IF(COUNTIF(B2:B{last},\"REVIEW\")>0,\"REVIEW\",\"PASS\"))"
        )),
        &calc_bold,
    )?;
    ws.write_string_with_format(
        17,
        0,
        "Assumptions the engine prints with every run",
        &s.head,
    )?;
    let caveats = [
        "The statutory test is receipt by the fund with enough information to allocate it (SGAA s 18C(1)(c)). A remittance date is operational evidence only; clearing-house transit is the employer's risk.",
        "Item 4 (s 18C(2)) aligns a later payday only to an earlier eligible contribution the register evidences as received on time; a positive amount or a remittance alone gives a possible upper bound and an attention-driving UNKNOWN where it would change the verdict.",
        "Notional earnings compound daily at the general interest charge on the base shortfall from the day after the deadline while the final shortfall remains greater than nil (s 19A, LCR 2026/3). A part receipt does not slow the accrual. Past the last published GIC quarter the notional earnings and the charge estimate are withheld, as the command line withholds them without --allow-stale-gic.",
        "The uplift range runs from 0 per cent (clean history and a voluntary disclosure within 30 days) to 60 per cent (prior history, no disclosure). The ATO, not this workbook, decides which reductions apply.",
        "Each exposure component is rounded to cents half up and the totals are built from the rounded parts. TAA 1953 s 16B applies a five-cent down-round only to the Commissioner's final assessed charge, which is not reproduced.",
        "Not estimated: choice loading, the maximum contributions base, the late payment penalty, interest after assessment, exceptional-circumstances determinations, fund deed and award obligations, and paydays before 1 July 2026.",
    ];
    let wrap = Format::new().set_text_wrap();
    for (i, caveat) in caveats.iter().enumerate() {
        ws.write_string_with_format(18 + i as u32, 0, *caveat, &wrap)?;
    }
    for (c, chars) in [(0, 100), (1, 12), (2, 16), (3, 24)] {
        width(ws, c, chars)?;
    }
    protect(ws);

    // 8. Sources & Version
    let ws = wb.add_worksheet().set_name("Sources & Version")?;
    header(ws, &["Item", "Value"], &s.head)?;
    let items = [
        ("Workbook engine version", VERSION.to_string()),
        ("Desktop Excel build used to calculate cached values", String::new()),
        (
            "Law",
            "Superannuation Guarantee (Administration) Act 1992 as amended by the Treasury Laws Amendment (Payday Superannuation) Act 2025 (No. 57 of 2025) and the Superannuation Guarantee Charge Amendment Act 2025 (No. 58 of 2025); Payday Superannuation Regulations 2026 (F2026L00133); F2026L00784; LCR 2026/1, 2026/2 and 2026/3".into(),
        ),
        (
            "Primary-source review",
            "docs/primary-source-review-2026-08-15.md in the repository".into(),
        ),
        (
            "Holiday calendar",
            format!(
                "Official whole-of-jurisdiction sources checked through {}",
                verified_until.isoformat()
            ),
        ),
        (
            "GIC rates",
            format!(
                "ATO general interest charge rates, known through {}; an accrual past that day withholds the charge estimate",
                gic_last.isoformat()
            ),
        ),
        (
            "No-install explainer",
            "https://duguid.com.au/tools/payday-super/".into(),
        ),
        (
            "Disclaimer",
            "Experimental review aid. Not a compliance determination, not an ATO assessment, and not tax, legal or financial advice.".into(),
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
        ("Review Checks", (1, 1, 15)),
        ("Start Here", (10, 0, 10)),
        ("Summary", (23, 1, 23)),
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

    let xlsx = wb.save_to_buffer()?;
    let columns: Vec<(&str, &str)> = ARRAY_CALCS.iter().map(|c| (*c, &calc[c][1..])).collect();
    Ok(repair_array_columns(
        &xlsx, "Register", "tblLines", &columns,
    )?)
}

#[cfg(test)]
mod tests {
    use crate::model::{differences, model};

    /// Accepted after the Gate B desktop Excel check:
    /// - openpyxl writes a status colour as a `solid` dxf fill with fgColor,
    ///   which Excel does not display; rust_xlsxwriter writes bgColor, the form
    ///   Excel writes and displays, so the Rust workbook shows the colours;
    /// - setting openpyxl's margins makes rust_xlsxwriter write a page setup
    ///   with portrait orientation, A4 paper and 200 dpi, where openpyxl leaves
    ///   them to the printer; Excel reports the same orientation, margins and
    ///   print quality for both, and A4 for both with an Australian printer.
    fn accepted(line: &str) -> String {
        line.replace("dxf{fill{solid fgColor=", "dxf{fill{(absent) bgColor=")
            .replace(
                "| page orientation=default paperSize=1 scale=100 fitToWidth=1 fitToHeight=1 horizontalDpi=600 verticalDpi=600 ",
                "| page orientation=portrait paperSize=9 scale=100 fitToWidth=1 fitToHeight=1 horizontalDpi=200 verticalDpi=200 ",
            )
    }

    #[test]
    fn the_workbook_matches_the_frozen_python_model() {
        // The model of tools/build_workbook.py --no-excel at 4987fd6.
        let frozen: Vec<String> = include_str!("../../tests/data/workbook-model.txt")
            .lines()
            .map(accepted)
            .collect();
        let built = model(&super::build().unwrap()).unwrap();
        let found = differences(&frozen, &built);
        assert!(found.is_empty(), "{}", found.join("\n"));
    }
}
