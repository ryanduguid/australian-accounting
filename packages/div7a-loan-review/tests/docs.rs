//! Keep the documentation honest against the code. Prose checks run on a
//! whitespace-flattened copy, because the files are hard-wrapped; structural
//! checks (table rows, the banner) read the raw text.

use div7a_loan_review::register::{GATE_COLUMNS, MYR_COLUMNS, OPTIONAL_COLUMNS};
use div7a_loan_review::{LAW_COMPILATION, VERSION, api};
use serde_json::json;

const README: &str = include_str!("../README.md");
const DISCLAIMER: &str = include_str!("../DISCLAIMER.md");
const SECURITY: &str = include_str!("../SECURITY.md");
const CITATION: &str = include_str!("../CITATION.cff");
const RELEASE_NOTES: &str = include_str!("../RELEASE_NOTES.md");
const REVIEW: &str = include_str!("../docs/primary-source-review-2026-08-31.md");
const RATES_REVIEW: &str = include_str!("../docs/primary-source-review-2026-09-20.md");
const ATO_COMPARISON: &str = include_str!("../docs/ato-calculator-comparison-2026-10-07.md");

fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Everything above the first line that is only `---`.
fn first_screen() -> String {
    let lines: Vec<&str> = README.lines().collect();
    let end = lines.iter().position(|l| l.trim() == "---").unwrap();
    flat(&lines[..end].join("\n"))
}

fn assert_in(phrase: &str, text: &str, what: &str) {
    assert!(text.contains(phrase), "{what} lacks {phrase:?}");
}

const RATES: [(&str, &str); 8] = [
    ("2026-27", "8.77%"),
    ("2025-26", "8.37%"),
    ("2024-25", "8.77%"),
    ("2023-24", "8.27%"),
    ("2022-23", "4.77%"),
    ("2021-22", "4.52%"),
    ("2020-21", "4.52%"),
    ("2019-20", "5.37%"),
];

/// The engine's rate for a year, as a percentage to 2 places.
fn engine_percent(year: &str) -> String {
    let result = api::evaluate("benchmark-rate", &json!({"year_of_income": year})).unwrap();
    let rate = result["benchmark_rate"].as_str().unwrap();
    let (whole, frac) = rate.split_once('.').unwrap();
    let frac = format!("{frac:0<4}");
    let percent = format!("{}{}", whole.trim_start_matches('0'), &frac[..2]);
    format!("{}.{}%", percent.trim_start_matches('0'), &frac[2..4])
}

#[test]
fn the_first_screen_states_the_scope_and_its_gaps() {
    let screen = first_screen();
    for phrase in [
        "Experimental review aid. Not a Division 7A determination.",
        "operator-supplied amalgamated loan",
        "does not form amalgamated loans",
        "does not classify payments",
        "unpaid present entitlements",
        "109RB",
        "rising benchmark rate raises the minimum yearly repayment",
        "not just on new ones",
        "https://duguid.com.au/rates/div7a-benchmark-rate/",
    ] {
        assert_in(phrase, &screen, "the first screen");
    }
}

#[test]
fn the_banner_is_a_t_account_with_dr_and_cr_sides() {
    let opening = README.find("```").unwrap();
    let close = README[opening + 3..].find("```").unwrap() + opening + 3;
    let banner = &README[opening..close];
    assert!(banner.contains("DR  what it gives you"));
    assert!(banner.contains("CR  what it needs"));
    let widths: std::collections::BTreeSet<usize> = banner
        .lines()
        .filter(|l| l.starts_with('+') || l.starts_with('|'))
        .map(|l| l.chars().count())
        .collect();
    assert_eq!(widths.len(), 1, "banner rows are ragged");
}

#[test]
fn the_rate_tables_match_the_frozen_table() {
    for (year, percent) in RATES {
        assert_eq!(engine_percent(year), percent, "{year}");
        let row = format!("| {year} | {percent} |");
        assert_in(&row, README, "README");
        assert_in(&row, REVIEW, "the source review");
    }
}

#[test]
fn the_readme_explains_the_rates_and_their_review() {
    let readme = flat(README);
    for phrase in [
        "May 2025 was 8.37% and June 2025 was 8.27%",
        "8.37 is the benchmark rate",
        "FILRHLBVS",
        "F5",
        "28 August 2026",
        "ROUND_HALF_UP",
        "term of the loan",
        "C1936A00027",
        "docs/ato-calculator-comparison-2026-10-07.md",
        "docs/primary-source-review-2026-09-20.md",
        "docs/primary-source-review-2026-08-31.md",
    ] {
        assert_in(phrase, &readme, "README");
    }
    assert!(
        readme
            .to_lowercase()
            .contains("the act prescribes no rounding")
    );
    let table = api::evaluate("load-table", &json!({"path": "data/benchmark_rates.csv"})).unwrap();
    assert_eq!(table["reviewed_on"], "2026-08-28");
    assert!(LAW_COMPILATION.contains("C1936A00027"));
}

#[test]
fn every_register_column_is_documented_and_no_other() {
    let known: Vec<&str> = GATE_COLUMNS
        .iter()
        .chain(&MYR_COLUMNS)
        .chain(&OPTIONAL_COLUMNS)
        .copied()
        .collect();
    for column in &known {
        assert_in(&format!("`{column}`"), README, "README");
    }
    let start = README.find("## The register CSV").unwrap();
    let end = README.find("## Benchmark rates").unwrap();
    for line in README[start..end].lines().filter(|l| l.starts_with("| `")) {
        let name = line.split('|').nth(1).unwrap().trim().trim_matches('`');
        assert!(known.contains(&name), "documented but unread: {name}");
    }
}

#[test]
fn the_readme_lists_each_refusal_and_avoids_the_wrong_names() {
    let readme = flat(README);
    for phrase in [
        "s 109C payments",
        "s 109F forgiven debts",
        "109XA",
        "PCG 2017/13",
        "TD 2022/11",
        "s 109RB",
        "4 December 1997",
        "Public companies",
        "s 109Y",
        "s 109R",
        "s 109D(6)",
        "does not write \"the ATO will assess",
    ] {
        assert_in(phrase, &readme, "README");
    }
    for banned in [
        "div 7a solver",
        "division 7a solver",
        "compliance engine",
        "deemed dividend calculator",
    ] {
        assert!(!readme.to_lowercase().contains(banned), "{banned}");
    }
    assert_in(
        "same engine runs through `aus-accounting-mcp`",
        &readme,
        "README",
    );
}

#[test]
fn the_bendel_note_separates_the_holding_the_ato_view_and_what_stays_undecided() {
    let readme = flat(README);
    let start = readme.find("## Unpaid present entitlements after").unwrap();
    let end = readme.find("## The shortfall is not the dividend").unwrap();
    let note = &readme[start..end];
    for phrase in [
        "[2026] HCA 18",
        "10 June 2026",
        "dismissed 5 to 2",
        "read at source on 31 October 2026",
        "adds no calculation",
        "subject to formal revision",
        "'when a private company does nothing' ([72])",
        "Subdivision EA",
        "26 June 2026",
        "is the Commissioner's view, not the Court's holding",
        "TD 2022/11",
        "whether an unpaid present entitlement, or any dealing with one, is a loan under s 109D(3)",
        "whether Subdivision EA, including s 109XA, or s 100A applies",
    ] {
        assert_in(phrase, note, "the Bendel note");
    }
    assert_in("[2026] HCA 18", &flat(DISCLAIMER), "DISCLAIMER");
    assert_in(
        "See [*Bendel*](#unpaid-present-entitlements-after-bendel)",
        README,
        "the refusal table",
    );
}

#[test]
fn the_house_files_cover_the_required_ground() {
    let disclaimer = flat(DISCLAIMER);
    for phrase in [
        "does not provide professional advice",
        "not a Division 7A determination and not an ATO assessment",
        "Chartered Accountants Australia and New Zealand",
        "experimental review aid",
        "Do not publish private tax records",
        "using it creates no professional relationship",
        "not an endorsement of this software by CA ANZ",
        "Outputs can be wrong, incomplete, stale, or unsuitable",
    ] {
        assert_in(phrase, &disclaimer, "DISCLAIMER");
    }
    let readme = flat(README);
    for phrase in [
        "Provisional member of Chartered Accountants Australia and New Zealand",
        "not an endorsement of this software by CA ANZ",
        "own time, on his own equipment",
        "keep the copyright notices in [NOTICE](NOTICE)",
    ] {
        assert_in(phrase, &readme, "README");
    }
    let security = flat(SECURITY);
    assert_in("No network", &security, "SECURITY");
    assert_in("never as instructions", &security, "SECURITY");
}

#[test]
fn the_release_metadata_names_this_version() {
    assert_in(&format!("version: {VERSION}"), CITATION, "CITATION.cff");
    assert_in(
        "https://github.com/ryanduguid/australian-accounting",
        CITATION,
        "CITATION.cff",
    );
    assert!(RELEASE_NOTES.starts_with(&format!("# v{VERSION}\n")));
    // The README cites the last published release, the Python 0.3.2.
    assert_in("div7a-loan-review/v0.3.2", README, "README");
}

#[test]
fn the_ato_calculator_example_is_this_engines_arithmetic() {
    assert_in("12,800", &flat(ATO_COMPARISON), "the ATO comparison");
    for (balance, amount) in [("80000", "15,772.78"), ("100000", "19,715.97")] {
        let figure = api::evaluate(
            "minimum-yearly-repayment-amount",
            &json!({"principal": balance, "rate": "0.0877", "remaining_term": "7"}),
        )
        .unwrap();
        let figure = figure.as_str().unwrap();
        let (whole, frac) = figure.split_once('.').unwrap();
        let grouped: String = whole
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                let comma = i > 0 && (whole.len() - i) % 3 == 0;
                comma.then_some(',').into_iter().chain(std::iter::once(c))
            })
            .collect();
        assert_eq!(format!("{grouped}.{frac}"), amount);
        assert_in(&format!("${amount}"), &flat(README), "README");
    }
}

#[test]
fn the_source_reviews_record_what_was_read() {
    assert!(RATES_REVIEW.contains("F5") && RATES_REVIEW.contains("FILRHLBVS"));
    assert!(RATES_REVIEW.contains("changes no rate"));
    let review = flat(REVIEW);
    for phrase in [
        "C1936A00027",
        "1 July 2026",
        "volume 2",
        "109D",
        "109E",
        "109N",
        "109P",
        "109R",
        "109ZD",
        "image.017.png",
        "bitmap",
        "Reviewed position",
        "Confirmed by Ryan Duguid on 31 August 2026",
        "ROUND_HALF_UP",
        "prescribes none",
        "2026-27",
        "shelf life",
        "does not establish",
        "109XA",
        "PCG 2017/13",
        "109RB",
        "public companies",
        "AustLII",
        "respected rather than circumvented",
        "not an endorsement of this software or this review by CA ANZ",
    ] {
        assert_in(phrase, &review, "the source review");
    }
}
