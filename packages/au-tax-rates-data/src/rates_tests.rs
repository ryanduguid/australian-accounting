//! The behaviours of `tests/test_dataset.py`, `test_numeric_validation.py`,
//! `test_pattern_reports.py` and `test_ato_tables.py` at 24fcf24, one test
//! here per Python test. Where Python monkeypatched `subprocess.run`,
//! `fetch_text`, `HTTPSConnection`, `load_records` or `fetch_all`, these call
//! the step below the patched one with the same fabricated input.
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use super::*;

// ------------------------------------------------------------------ helpers

fn s(text: &str) -> S {
    pts(text)
}

fn st(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

fn j(text: &str) -> Json {
    pyjson::loads_standard(text).expect("test JSON")
}

fn int(n: i64) -> Json {
    Json::Int(BigInt::from(n))
}

fn nl() -> &'static str {
    if cfg!(windows) { "\r\n" } else { "\n" }
}

fn ctx() -> Ctx {
    Ctx::new()
}

fn records() -> Vec<Json> {
    load_records(&ctx()).expect("the committed records load")
}

fn by_id(records: &[Json], id: &str) -> Json {
    records
        .iter()
        .find(|r| r.at("id").as_str() == Some(id))
        .cloned()
        .expect("record id")
}

/// `dict(record, key=value)`.
fn with(record: &Json, key: &str, value: Json) -> Json {
    let mut record = record.clone();
    set(&mut record, key, value);
    record
}

fn set(record: &mut Json, key: &str, value: Json) {
    let Json::Object(pairs) = record else {
        panic!("not a dict")
    };
    match pairs.iter_mut().find(|(k, _)| k.as_str() == Some(key)) {
        Some(pair) => pair.1 = value,
        None => pairs.push((PyStr::Text(key.into()), value)),
    }
}

fn without(record: &Json, key: &str) -> Json {
    let Json::Object(pairs) = record else {
        panic!("not a dict")
    };
    Json::Object(
        pairs
            .iter()
            .filter(|(k, _)| k.as_str() != Some(key))
            .cloned()
            .collect(),
    )
}

fn row_mut(scale: &mut Json, i: usize) -> &mut Json {
    let Json::Array(rows) = scale else {
        panic!("not a list")
    };
    &mut rows[i]
}

fn snapshot(record: &Json) -> S {
    let snap = snapshot_path(&ctx(), &str_points(record.at("source_url"))).expect("snapshot path");
    pts(&snap.read().expect("snapshot text"))
}

fn replace(text: &[u32], from: &str, to: &str) -> S {
    pts(&st(text).replace(from, to))
}

fn texts(problems: &[S]) -> Vec<String> {
    problems.iter().map(|p| st(p)).collect()
}

fn validate(record: &Json) -> Vec<String> {
    texts(&validate_record(record, None).expect("no exception"))
}

fn page(record: &Json, text: &str) -> Vec<String> {
    texts(&page_problems(record, &s(text)).expect("no exception"))
}

fn date(text: &str) -> Date {
    fromisoformat(&Json::str(text)).expect("ISO date")
}

/// `{r["source_url"]: snapshot(r) for r in records}`.
fn snapshot_pages(records: &[Json]) -> Vec<(Json, Page)> {
    let mut pages: Vec<(Json, Page)> = Vec::new();
    for record in records {
        let url = record.at("source_url").clone();
        let text = Ok(snapshot(record));
        match pages.iter_mut().find(|(u, _)| *u == url) {
            Some(slot) => slot.1 = text,
            None => pages.push((url, text)),
        }
    }
    pages
}

fn report(records: &[Json], pages: &[(Json, Page)], today: &str) -> (String, usize) {
    let (report, findings) = build_report(&ctx(), records, pages, today).expect("no exception");
    (st(&report), findings)
}

fn found(number: &str) -> Found {
    Found::Number(Decimal::parse(number).expect("decimal"))
}

fn same(a: &Found, b: &Found) -> bool {
    match (a, b) {
        (Found::Date(x), Found::Date(y)) => x == y,
        (Found::Number(x), Found::Number(y)) => {
            x.equals(y).unwrap_or(false) && x.to_string() == y.to_string()
        }
        _ => false,
    }
}

/// A fresh folder under the system temporary folder.
fn tmp(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, AtomicOrdering::Relaxed);
    let path = std::env::temp_dir().join(format!("rates-test-{}-{name}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("temporary folder");
    path
}

fn out() -> Out<Vec<u8>> {
    Out::new(Vec::new())
}

fn printed(out: Out<Vec<u8>>) -> String {
    String::from_utf8(out.w).expect("UTF-8 output")
}

/// `cmd_validate` with `DATA` pointed at a folder holding these files.
fn validate_folder(files: &[(&str, Vec<u8>)]) -> (i32, String) {
    let folder = tmp("data");
    for (name, content) in files {
        std::fs::write(folder.join(name), content).expect("write record");
    }
    let real = ctx();
    let ctx = Ctx {
        data: PyPath::new(&folder.to_string_lossy()),
        root: real.root,
        snapshots: real.snapshots,
    };
    let mut out = out();
    let code = cmd_validate(&ctx, &mut out).expect("no exception");
    let _ = std::fs::remove_dir_all(&folder);
    (code, printed(out))
}

fn first_aud() -> Json {
    records()
        .into_iter()
        .find(|r| r.at("unit").as_str() == Some("AUD"))
        .expect("an AUD record")
}

fn dumps(value: &Json) -> Vec<u8> {
    pyjson::dumps(value, pyjson::Layout::Canonical).into_bytes()
}

/// `json.dumps(value, indent=2, ensure_ascii=False)`.
fn dumps_indented(value: &Json, level: usize, out: &mut String) {
    let pad = |level: usize| "  ".repeat(level);
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Int(i) => out.push_str(&i.to_string()),
        Json::Float(f) => out.push_str(&py::float_repr(*f)),
        Json::Str(text) => dump_text(&text.code_points(), out),
        Json::Array(items) if items.is_empty() => out.push_str("[]"),
        Json::Array(items) => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                out.push_str(&pad(level + 1));
                dumps_indented(item, level + 1, out);
            }
            out.push('\n');
            out.push_str(&pad(level));
            out.push(']');
        }
        Json::Object(pairs) if pairs.is_empty() => out.push_str("{}"),
        Json::Object(pairs) => {
            out.push_str("{\n");
            for (i, (key, item)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                out.push_str(&pad(level + 1));
                dump_text(&key.code_points(), out);
                out.push_str(": ");
                dumps_indented(item, level + 1, out);
            }
            out.push('\n');
            out.push_str(&pad(level));
            out.push('}');
        }
    }
}

fn dump_text(points: &[u32], out: &mut String) {
    out.push('"');
    for &cp in points {
        match cp {
            0x22 => out.push_str("\\\""),
            0x5c => out.push_str("\\\\"),
            0x0a => out.push_str("\\n"),
            0x0d => out.push_str("\\r"),
            0x09 => out.push_str("\\t"),
            0x08 => out.push_str("\\b"),
            0x0c => out.push_str("\\f"),
            0..0x20 => out.push_str(&format!("\\u{cp:04x}")),
            _ => out.push(char::from_u32(cp).expect("no lone surrogate in the data")),
        }
    }
    out.push('"');
}

// ------------------------------------------------------------------ test_dataset.py

#[test]
fn every_file_is_named_after_its_unique_id() {
    let records = records();
    let mut ids: Vec<String> = records
        .iter()
        .map(|r| pyvalue::py_str(r.at("id")))
        .collect();
    let count = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), count);
    let mut stems: Vec<String> = glob_json(&ctx().data)
        .iter()
        .map(|e| st(&e.stem()))
        .collect();
    stems.sort();
    assert_eq!(stems, ids);
}

#[test]
fn record_passes_schema_quote_and_pattern_checks() {
    for record in records() {
        let problems = validate_record(&record, Some(&snapshot(&record))).expect("no exception");
        assert_eq!(
            texts(&problems),
            Vec::<String>::new(),
            "{}",
            pyvalue::py_str(record.at("id"))
        );
    }
}

#[test]
fn pattern_reads_the_committed_value_from_the_snapshot() {
    let mut checked = 0;
    for record in records() {
        let Some(pattern) = record.get("pattern").filter(|p| p.truthy()) else {
            continue;
        };
        let text = norm(&snapshot(&record));
        let compiled = compile(pattern).expect("pattern compiles");
        let m = compiled.search(&text).expect("pattern matches");
        let (start, end) = m.span(1).expect("group 1");
        let read = parse_text(record.at("unit"), &text[start..end]).expect("value parses");
        let expected = if record.at("unit").as_str() == Some("date") {
            Found::Date(str_points(record.at("value")))
        } else {
            Found::Number(str_decimal(record.at("value")).expect("decimal value"))
        };
        let equal = match (&read, &expected) {
            (Found::Date(a), Found::Date(b)) => a == b,
            (Found::Number(a), Found::Number(b)) => a.equals(b).expect("comparable"),
            _ => false,
        };
        assert!(equal, "{}", pyvalue::py_str(record.at("id")));
        checked += 1;
    }
    assert!(checked > 0);
}

#[test]
fn parse_text_units() {
    for (unit, text, expected) in [
        ("AUD", "$32,500", found("32500")),
        ("AUD", "$7,483.05", found("7483.05")),
        ("percent", "8.77%", found("8.77")),
        ("percent", "12.00", found("12.00")),
        ("cents_per_km", "91 cents", found("91")),
        ("cents_per_km", "70c", found("70")),
        ("factor", "2.0802", found("2.0802")),
        ("factor", "2", found("2")),
        ("date", "1\u{a0}July 2026", Found::Date(s("2026-07-01"))),
    ] {
        let read = parse_text(&Json::str(unit), &s(text)).expect("parses");
        assert!(same(&read, &expected), "{unit} {text}");
    }
    // Decimal("12.00") == Decimal("12"), as the Python test compares.
    let Found::Number(twelve) = parse_text(&Json::str("percent"), &s("12.00")).unwrap() else {
        panic!()
    };
    assert!(twelve.equals(&decimal(12)).unwrap());
}

#[test]
fn parse_text_rejects_other_formats() {
    for (unit, text) in [
        ("AUD", "about $30k"),
        ("percent", "47 per cent"),
        ("date", "2026-07-01"),
        ("factor", "2.08%"),
        ("factor", ".5"),
    ] {
        let Err(e) = parse_text(&Json::str(unit), &s(text)) else {
            panic!("{unit} {text} parsed")
        };
        assert!(e.is("ValueError"), "{unit} {text}: {}", e.class);
    }
}

#[test]
fn check_value_rejects_bad_values() {
    for (unit, value) in [
        ("AUD", int(-1)),
        ("AUD", Json::str("75000")),
        ("AUD", Json::Bool(true)),
        ("percent", int(101)),
        ("date", Json::str("27 August 2025")),
    ] {
        assert!(!check_value(unit, &value).unwrap().is_empty(), "{unit}");
    }
}

#[test]
fn tax_scale_base_amounts_must_add_up() {
    let mut scale = records()
        .into_iter()
        .find(|r| r.at("unit").as_str() == Some("tax_scale"))
        .unwrap()
        .at("value")
        .clone();
    assert!(check_scale(&scale).unwrap().is_empty());
    let row = row_mut(&mut scale, 2);
    let bumped = match row.at("base_tax") {
        Json::Int(i) => Json::Int(i + 1),
        Json::Float(f) => Json::Float(f + 1.0),
        other => panic!("{other:?}"),
    };
    set(row, "base_tax", bumped);
    assert!(
        texts(&check_scale(&scale).unwrap())
            .iter()
            .any(|p| p.contains("base_tax"))
    );
}

#[test]
fn schema_rejects_missing_fields_and_foreign_sources() {
    let first = records()[0].clone();
    assert!(validate(&without(&first, "quote")).contains(&"missing or wrong type: quote".into()));
    let foreign = with(&first, "source_url", Json::str("https://example.com/rates"));
    assert!(validate(&foreign).contains(&"source_url is not a public ato.gov.au page".into()));
}

#[test]
fn income_year_must_describe_adjacent_calendar_years() {
    let first = records()[0].clone();
    for year in [
        "2026-99",
        "2026-26",
        "2026-25",
        "2026-27\n",
        "0000-01",
        "9999-00",
    ] {
        let record = with(
            &first,
            "period",
            Json::obj([("income_year", Json::str(year))]),
        );
        assert!(
            validate(&record).iter().any(|p| p.contains("income_year")),
            "{year:?}"
        );
    }
}

#[test]
fn valid_periods_keep_century_rollover_open_bounds_and_retrieval_dates() {
    let first = records()[0].clone();
    for period in [
        r#"{"income_year": "2026-27"}"#,
        r#"{"income_year": "1999-00"}"#,
        r#"{"income_year": "2099-00"}"#,
        r#"{"effective_from": "2026-07-01", "effective_to": "2027-06-30"}"#,
        r#"{"effective_from": "2024-02-29", "effective_to": "2024-02-29"}"#,
        r#"{"effective_from": "2026-07-01"}"#,
        r#"{"effective_to": "2026-06-30"}"#,
        r#"{"as_at": "2026-06-01", "effective_from": "2026-07-01"}"#,
    ] {
        assert_eq!(
            validate(&with(&first, "period", j(period))),
            Vec::<String>::new(),
            "{period}"
        );
    }
}

#[test]
fn effective_period_cannot_end_before_it_starts() {
    let record = with(
        &records()[0],
        "period",
        j(r#"{"effective_from": "2026-07-01", "effective_to": "2026-06-30"}"#),
    );
    assert!(validate(&record).contains(&"effective_to precedes effective_from".into()));
}

#[test]
fn invalid_period_dates_report_a_problem_without_comparing_them() {
    let first = records()[0].clone();
    for invalid in [
        Json::str("2026-02-29"),
        Json::str("2024-02-30"),
        Json::str("0000-01-01"),
        int(20260701),
        Json::Null,
    ] {
        let period = Json::obj([
            ("effective_from", invalid.clone()),
            ("effective_to", Json::str("2026-06-30")),
        ]);
        let problems = validate(&with(&first, "period", period));
        assert!(
            problems.iter().any(|p| p.starts_with("effective_from:")),
            "{invalid:?}"
        );
    }
}

#[test]
fn page_problems_report_changed_value_and_missing_quote() {
    let record = by_id(&records(), "concessional-cap-2026-27");
    let changed = replace(&snapshot(&record), "$32,500", "$35,000");
    let problems = texts(&page_problems(&record, &changed).unwrap());
    assert!(problems.contains(&"quote not found on page".into()));
    assert!(problems.contains(&"value changed: 32500 -> $35,000".into()));
}

#[test]
fn report_flags_missing_pages_and_passes_unchanged_pages() {
    let records = records();
    let mut pages = snapshot_pages(&records);
    let (text, findings) = report(&records, &pages, "2026-09-24");
    assert!(findings == 0 && text.contains("0 finding(s)"));

    let missing = pages[0].0.clone();
    pages[0].1 = Err(Exc::msg("RuntimeError", "HTTP 404"));
    let (text, findings) = report(&records, &pages, "2026-09-24");
    let affected = records
        .iter()
        .filter(|r| *r.at("source_url") == missing)
        .count();
    assert_eq!(findings, affected);
    assert!(text.contains("page missing or unreadable (HTTP 404)"));
}

#[test]
fn report_notes_only_pages_whose_body_changed() {
    // A footer edit reaches every page at once; noting all of them buried the
    // one page that gained a new year's figure.
    let records = records();
    let mut pages = snapshot_pages(&records);
    for (_, page) in &mut pages {
        let text = page.as_ref().unwrap().clone();
        *page = Ok(replace(&text, "Privacy policy", "Privacy notice"));
    }
    let (text, findings) = report(&records, &pages, "2026-09-24");
    assert!(findings == 0 && !text.contains("## Pages with changed wording"));
    // A header edit too, including the GIC page whose capture reads "MenuSearch".
    for (_, page) in &mut pages {
        let text = page.as_ref().unwrap().clone();
        let mut lines: Vec<S> = splitlines(&text).into_iter().map(<[u32]>::to_vec).collect();
        lines.insert(1, s("A new header link"));
        *page = Ok(lines.join(&[u32::from('\n')][..]));
    }
    let (text, findings) = report(&records, &pages, "2026-09-24");
    assert!(findings == 0 && !text.contains("## Pages with changed wording"));

    let url = by_id(&records, "concessional-cap-2026-27")
        .at("source_url")
        .clone();
    let slot = pages.iter_mut().find(|(u, _)| *u == url).unwrap();
    // re.sub(r"(?m)^(QC\s?\d+)$", "A new cap applies from 1 July 2027.\n\1", page)
    let text = slot.1.as_ref().unwrap().clone();
    let lines: Vec<S> = text
        .split(|&c| c == u32::from('\n'))
        .map(|line| {
            if QC_LINE.fullmatch(line).is_some() {
                p!("A new cap applies from 1 July 2027.\n", line)
            } else {
                line.to_vec()
            }
        })
        .collect();
    slot.1 = Ok(lines.join(&[u32::from('\n')][..]));
    let (text, findings) = report(&records, &pages, "2026-09-24");
    assert_eq!(findings, 0);
    assert_eq!(text.matches("\n- https://").count(), 1);
    assert!(text.contains(&format!("- {}", pyvalue::py_str(&url))));
}

#[test]
fn a_record_ending_without_a_successor_is_a_finding() {
    // After 30 June 2027 the SG record kept passing with no 2027-28 figure.
    let records = records();
    let sg = by_id(&records, "super-guarantee-rate-2026-27");
    let sg_findings = |records: &[Json], today: &str| -> Vec<String> {
        texts(&expiring(records, &date(today)).unwrap())
            .into_iter()
            .filter(|f| f.starts_with("`super-guarantee"))
            .collect()
    };
    assert!(expiring(&records, &date("2026-09-24")).unwrap().is_empty());
    assert!(sg_findings(&records, "2027-06-15").is_empty());
    assert_eq!(
        sg_findings(&records, "2027-06-17"),
        [
            "`super-guarantee-rate-2026-27`: ends 2027-06-30 and no record for this figure starts 2027-07-01"
        ]
    );
    let successor = with(
        &with(&sg, "id", Json::str("super-guarantee-rate-2027-28")),
        "period",
        j(r#"{"effective_from": "2027-07-01", "effective_to": "2028-06-30"}"#),
    );
    let mut more = records.clone();
    more.push(successor);
    assert!(sg_findings(&more, "2027-06-17").is_empty());
    // Consecutive GIC quarters succeed one another; the last one does not.
    let gic: Vec<String> = texts(&expiring(&records, &date("2026-12-20")).unwrap())
        .iter()
        .map(|f| f.split(':').next().unwrap().to_string())
        .collect();
    assert_eq!(gic, ["`gic-annual-rate-oct-dec-2026`"]);
    let (text, _) = report(&records, &snapshot_pages(&records), "2027-07-01");
    assert!(text.contains("`super-guarantee-rate-2026-27`: ends 2027-06-30"));
}

fn renderings(value: &Json) -> Vec<String> {
    texts(&scale_renderings(&json_scale_rows(value).unwrap()).unwrap())
}

#[test]
fn a_scale_must_match_the_brackets_its_quote_prints() {
    // Firecrawl once returned the 2025-26 scale for 2026-27, and every
    // arithmetic check passed it.
    let records = records();
    let current = by_id(&records, "resident-tax-rates-2026-27");
    let previous = by_id(&records, "resident-tax-rates-2025-26");
    assert!(validate(&current).is_empty() && validate(&previous).is_empty());
    assert_eq!(
        renderings(current.at("value"))[..2],
        [
            "15c for each $1 over $18,200",
            "$4,020 plus 30c for each $1 over $45,000"
        ]
    );
    let problems = validate(&with(&current, "value", previous.at("value").clone()));
    assert!(
        problems.contains(&"quote does not show the bracket '16c for each $1 over $18,200'".into())
    );
    assert_eq!(problems.len(), 4);
}

#[test]
fn a_scale_taxed_from_the_first_dollar_renders_without_an_over_amount() {
    let foreign = by_id(&records(), "foreign-resident-tax-rates-2025-26");
    assert_eq!(
        renderings(foreign.at("value")),
        [
            "30c for each $1",
            "$40,500 plus 37c for each $1 over $135,000",
            "$60,850 plus 45c for each $1 over $190,000"
        ]
    );
    let mut shifted = foreign.at("value").clone();
    set(row_mut(&mut shifted, 0), "marginal_rate", Json::Float(32.5));
    set(row_mut(&mut shifted, 1), "base_tax", int(43875));
    set(row_mut(&mut shifted, 2), "base_tax", int(64225));
    assert!(check_scale(&shifted).unwrap().is_empty());
    assert!(
        validate(&with(&foreign, "value", shifted))
            .contains(&"quote does not show the bracket '32.5c for each $1'".into())
    );
}

#[test]
fn data_files_are_stable_json() {
    for entry in glob_json(&ctx().data) {
        let text = read_text(&entry.path, &entry.display).unwrap();
        let mut canonical = String::new();
        dumps_indented(&loads(&text).unwrap(), 0, &mut canonical);
        canonical.push('\n');
        assert_eq!(text, canonical, "{}", entry.display);
    }
}

/// The browser's JSON reply, as `browser_reply` fabricated it.
fn browser_reply(code: i64, status: &str, loaded: bool, text: &str) -> String {
    if code == 2 {
        return String::new();
    }
    format!(
        r#"{{"status": {status}, "loaded": {loaded}, "url": "https://www.ato.gov.au/p", "title": "t", "text": "{text}"}}"#
    )
}

#[test]
fn fetch_text_returns_the_page_text_with_plain_spaces() {
    let text = format!("Cap $30,000 {}", "x".repeat(250));
    let read = read_reply(0, &browser_reply(0, "200", true, &text), "").unwrap();
    assert!(st(&read).starts_with("Cap $30,000 "));
    // The browser's non-breaking spaces become plain ones; line breaks and tabs stay.
    let read = read_reply(
        0,
        &browser_reply(
            0,
            "200",
            true,
            &format!("a\\u00a0b\\n\\tc{}", "x".repeat(250)),
        ),
        "",
    )
    .unwrap();
    assert!(st(&read).starts_with("a b\n\tc"));
}

#[test]
fn fetch_text_rejects_unusable_pages() {
    let x = "x".repeat(250);
    for (code, status, loaded, text, message) in [
        (1, "404", true, x.as_str(), "HTTP 404"),
        (0, "200", false, x.as_str(), "HTTP 200"),
        (0, "200", true, "Access denied", "nearly empty"),
        (2, "null", false, "", "ERR_NAME_NOT_RESOLVED"),
    ] {
        let stdout = browser_reply(code, status, loaded, text);
        let e = read_reply(code, &stdout, "ToolError: net::ERR_NAME_NOT_RESOLVED").unwrap_err();
        assert_eq!(e.class, "RuntimeError");
        assert!(st(&e.text).contains(message), "{message}: {}", st(&e.text));
    }
}

#[test]
fn fetch_all_keeps_each_failure_as_that_page_s_value() {
    let urls = [
        Json::str("https://www.ato.gov.au/ok"),
        Json::str("https://www.ato.gov.au/bad"),
    ];
    let pages = fetch_all_with(
        &urls,
        |url| {
            if pyvalue::py_str(url).ends_with("bad") {
                Err(Exc::msg(
                    "subprocess.TimeoutExpired",
                    "Command 'browser' timed out after 180 seconds",
                ))
            } else {
                Ok(s("text"))
            }
        },
        false,
    )
    .unwrap();
    assert_eq!(pages[0].1.as_ref().unwrap(), &s("text"));
    assert_eq!(
        pages[1].1.as_ref().unwrap_err().class,
        "subprocess.TimeoutExpired"
    );
    // An exception outside fetch_all's list ends the run.
    let e = fetch_all_with(&urls, |_| Err(type_error("boom")), false).unwrap_err();
    assert_eq!(e.class, "TypeError");
}

#[test]
fn fetch_text_rejects_a_success_shaped_reply_from_a_failed_browser() {
    let e = read_reply(3, &browser_reply(3, "200", true, &"x".repeat(250)), "").unwrap_err();
    assert_eq!(e.class, "RuntimeError");
    assert!(st(&e.text).contains("exited with code 3"));
}

#[test]
fn a_malformed_reply_fails_only_its_own_page() {
    let good = browser_reply(0, "200", true, &"x".repeat(250));
    let urls = [
        Json::str("https://www.ato.gov.au/bad"),
        Json::str("https://www.ato.gov.au/ok"),
    ];
    let pages = fetch_all_with(
        &urls,
        |url| {
            let bad = pyvalue::py_str(url).ends_with("bad");
            read_reply(0, if bad { "null" } else { &good }, "")
        },
        false,
    )
    .unwrap();
    assert_eq!(pages[0].1.as_ref().unwrap_err().class, "RuntimeError");
    assert_eq!(st(pages[1].1.as_ref().unwrap()), "x".repeat(250));
}

#[test]
fn the_payg_section_is_checked_not_its_contents_entry() {
    let record = by_id(&records(), "payg-tax-tables-effective");
    let heading = "Tax tables that were updated and apply from 1 July 2026";
    for edit in ["change", "remove"] {
        let text = snapshot(&record);
        let mut lines: Vec<S> = splitlines(&text).into_iter().map(<[u32]>::to_vec).collect();
        let at: Vec<usize> = (0..lines.len())
            .filter(|&i| strip(&lines[i]) == s(heading).as_slice())
            .collect();
        let [contents, section] = at[..] else {
            panic!("the heading appears twice")
        };
        lines[section] = if edit == "change" {
            s(&heading.replace("2026", "2027"))
        } else {
            S::new()
        };
        assert_eq!(strip(&lines[contents]), s(heading).as_slice());
        let changed = lines.join(&[u32::from('\n')][..]);
        assert!(
            !page_problems(&record, &changed).unwrap().is_empty(),
            "{edit}"
        );
    }
}

// ------------------------------------------------------------------ test_numeric_validation.py

#[test]
fn non_finite_values_are_refused() {
    for unit in ["AUD", "percent", "cents_per_km", "factor"] {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                !check_value(unit, &Json::Float(value)).unwrap().is_empty(),
                "{unit} {value}"
            );
        }
    }
}

#[test]
fn finite_non_negative_values_remain_valid() {
    for (unit, value) in [
        ("AUD", int(0)),
        ("AUD", Json::Float(7483.05)),
        ("percent", int(0)),
        ("percent", int(100)),
        ("cents_per_km", int(91)),
        ("factor", Json::Float(2.0802)),
        ("AUD", Json::Int(BigInt::from(10).pow(400))),
    ] {
        assert!(
            check_value(unit, &value).unwrap().is_empty(),
            "{unit} {value:?}"
        );
    }
}

fn tax_scale() -> Json {
    j(r#"[
        {"from": 0, "to": 100, "base_tax": 0, "marginal_rate": 0},
        {"from": 101, "to": 200, "base_tax": 0, "marginal_rate": 10},
        {"from": 201, "to": null, "base_tax": 10, "marginal_rate": 20}
    ]"#)
}

#[test]
fn invalid_bracket_values_return_problems() {
    let (nan, inf) = (Json::Float(f64::NAN), Json::Float(f64::INFINITY));
    for (index, field, value) in [
        (0, "from", int(-1)),
        (0, "from", int(1)),
        (0, "from", Json::Bool(true)),
        (0, "to", Json::Float(100.5)),
        (0, "to", Json::Null),
        (0, "to", Json::str("100")),
        (1, "from", Json::Null),
        (1, "from", Json::str("101")),
        (1, "from", nan.clone()),
        (1, "to", int(99)),
        (1, "to", inf.clone()),
        (1, "to", Json::Bool(false)),
        (0, "base_tax", Json::str("0")),
        (1, "base_tax", int(-1)),
        (1, "base_tax", Json::Null),
        (1, "base_tax", nan.clone()),
        (1, "base_tax", inf.clone()),
        (1, "base_tax", Json::Bool(true)),
        (1, "marginal_rate", Json::str("10")),
        (2, "marginal_rate", int(-5)),
        (2, "marginal_rate", int(101)),
        (2, "marginal_rate", Json::Null),
        (2, "marginal_rate", nan.clone()),
        (2, "marginal_rate", inf.clone()),
        (2, "marginal_rate", Json::Bool(false)),
    ] {
        let mut scale = tax_scale();
        set(row_mut(&mut scale, index), field, value.clone());
        assert!(
            !check_scale(&scale).unwrap().is_empty(),
            "{index} {field} {value:?}"
        );
    }
}

#[test]
fn valid_scale_uses_independently_calculated_base_amounts() {
    // The middle bracket covers 100 dollars at 10%, producing 10 dollars tax.
    assert!(check_scale(&tax_scale()).unwrap().is_empty());
    let one = j(r#"[{"from": 0, "to": null, "base_tax": 0, "marginal_rate": 100}]"#);
    assert!(check_scale(&one).unwrap().is_empty());
}

#[test]
fn an_unreadable_record_is_reported_and_the_rest_still_checked() {
    let later = with(&first_aud(), "value", int(-1));
    let id = pyvalue::py_str(later.at("id"));
    let deep = [vec![b'['; 100_000], vec![b']'; 100_000]].concat();
    let huge = [b"{\"value\": ".to_vec(), vec![b'9'; 5000], b"}".to_vec()].concat();
    for content in [
        b"".to_vec(),
        b"\xef\xbb\xbf".to_vec(),
        b"\xff\xfe\x00garbage".to_vec(),
        b"{not json".to_vec(),
        b"[]".to_vec(),
        b"\"x\"".to_vec(),
        b"null".to_vec(),
        deep,
        huge,
    ] {
        let (code, out) = validate_folder(&[
            (&format!("{id}.json"), dumps(&later)),
            ("aaa-broken.json", content),
        ]);
        assert_eq!(code, 1);
        assert!(out.starts_with("aaa-broken: "), "{out}");
        assert!(out.contains(&format!("\n{id}: ")), "{out}");
        assert!(out.contains("2 record(s) with problems"), "{out}");
    }
}

#[test]
fn a_source_url_that_is_not_text_is_a_problem_not_a_crash() {
    let record = first_aud();
    let id = pyvalue::py_str(record.at("id"));
    for url in [
        Json::Null,
        j("[]"),
        j("{}"),
        Json::Bool(true),
        Json::Float(1.5),
    ] {
        let record = with(&record, "source_url", url);
        let (code, out) = validate_folder(&[(&format!("{id}.json"), dumps(&record))]);
        assert_eq!(code, 1);
        assert!(out.contains("missing or wrong type: source_url"), "{out}");
    }
}

#[test]
fn validation_reports_overflowing_json_number() {
    let record = with(
        &without(&first_aud(), "pattern"),
        "value",
        Json::str("OVERFLOW"),
    );
    let id = pyvalue::py_str(record.at("id"));
    let text = String::from_utf8(dumps(&record))
        .unwrap()
        .replace("\"OVERFLOW\"", "1e999");
    let (code, out) = validate_folder(&[(&format!("{id}.json"), text.into_bytes())]);
    assert_eq!(code, 1);
    assert!(out.contains("1 record(s) with problems"), "{out}");
}

// ------------------------------------------------------------------ test_pattern_reports.py

fn fabricated(changes: &[(&str, Json)]) -> Json {
    let mut record = Json::obj([
        ("id", Json::str("fabricated-date")),
        ("unit", Json::str("date")),
        ("value", Json::str("2026-07-01")),
        ("quote", Json::str("Effective date")),
        ("pattern", Json::str("Effective date: (.*)")),
        (
            "source_url",
            Json::str("https://www.ato.gov.au/fabricated-date"),
        ),
    ]);
    for (key, value) in changes {
        set(&mut record, key, value.clone());
    }
    record
}

#[test]
fn unreadable_source_value_is_a_report_finding() {
    for text in ["31 February 2026", "1 August", "1/8/2026"] {
        let item = fabricated(&[]);
        let pages = [(
            item.at("source_url").clone(),
            Ok(s(&format!("Effective date: {text}"))),
        )];
        let (report, findings) = build_report(&ctx(), &[item], &pages, "2026-09-26").unwrap();
        let report = st(&report);
        assert_eq!(findings, 1);
        assert!(
            report.contains("value pattern could not be read"),
            "{report}"
        );
        assert!(report.contains("1 finding(s)"));
    }
}

#[test]
fn invalid_pattern_is_a_problem_instead_of_an_exception() {
    for pattern in [
        Json::str("("),
        Json::str("Effective date"),
        Json::str("(Effective) (date)"),
        int(42),
        Json::str("Effective date: (?:1 August)?(missing)?"),
    ] {
        let problems = page(
            &fabricated(&[("pattern", pattern.clone())]),
            "Effective date: 1 August 2026",
        );
        assert!(!problems.is_empty(), "{pattern:?}");
    }
}

#[test]
fn falsy_supplied_patterns_are_problems() {
    for pattern in [
        Json::str(""),
        Json::Null,
        Json::Bool(false),
        int(0),
        j("[]"),
        j("{}"),
    ] {
        let problems = page(
            &fabricated(&[("pattern", pattern.clone())]),
            "Effective date: 1 August 2026",
        );
        assert!(!problems.is_empty(), "{pattern:?}");
    }
}

#[test]
fn absent_pattern_remains_valid() {
    let item = without(&fabricated(&[]), "pattern");
    assert!(page(&item, "Effective date: 1 August 2026").is_empty());
}

#[test]
fn malformed_decimal_is_a_report_finding() {
    let item = fabricated(&[("unit", Json::str("percent")), ("value", int(12))]);
    assert!(!page(&item, "Effective date: 1..2%").is_empty());
}

#[test]
fn tax_scale_pattern_is_reported_as_unsupported() {
    let item = fabricated(&[("unit", Json::str("tax_scale")), ("value", j("[]"))]);
    assert!(!page(&item, "Effective date: 1 August 2026").is_empty());
}

#[test]
fn failed_parse_does_not_hide_later_changed_values() {
    let bad = fabricated(&[]);
    let changed = fabricated(&[
        ("id", Json::str("fabricated-changed")),
        (
            "source_url",
            Json::str("https://www.ato.gov.au/fabricated-changed"),
        ),
    ]);
    let pages = [
        (
            bad.at("source_url").clone(),
            Ok(s("Effective date: 31 February 2026")),
        ),
        (
            changed.at("source_url").clone(),
            Ok(s("Effective date: 1 August 2026")),
        ),
    ];
    let (report, findings) = build_report(&ctx(), &[bad, changed], &pages, "2026-09-26").unwrap();
    assert_eq!(findings, 2);
    assert!(st(&report).contains("value changed: 2026-07-01 -> 1 August 2026"));
}

fn check_into_folder(item: Json, page: &str) -> (i32, PathBuf) {
    let folder = tmp("reports");
    let pages = [(item.at("source_url").clone(), Ok(s(page)))];
    let mut out = out();
    let code = write_check(
        &ctx(),
        &mut out,
        &folder.to_string_lossy(),
        &[item],
        &pages,
        "2026-09-26",
    )
    .unwrap();
    (code, folder)
}

#[test]
fn check_writes_report_and_returns_failure_when_source_value_is_unreadable() {
    let (code, folder) = check_into_folder(fabricated(&[]), "Effective date: 31 February 2026");
    assert_eq!(code, 1);
    let reports: Vec<_> = std::fs::read_dir(&folder)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.starts_with("au-tax-rates-check-") && name.ends_with(".md")
        })
        .collect();
    assert_eq!(reports.len(), 1);
    let text = std::fs::read_to_string(reports[0].path()).unwrap();
    assert!(text.contains("value pattern could not be read"));
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn empty_pattern_cannot_produce_a_successful_check() {
    let (code, folder) = check_into_folder(
        fabricated(&[("pattern", Json::str(""))]),
        "Effective date: 1 August 2026",
    );
    assert_eq!(code, 1);
    let _ = std::fs::remove_dir_all(&folder);
}

// ------------------------------------------------------------------ test_ato_tables.py

const TC2: [&str; 7] = [
    "DT_EFFECT",
    "DT_END",
    "TX_DESC",
    "AM_INC_MIN",
    "AM_INC_MAX",
    "AM_OFFSET",
    "PC_TAX_RATE",
];
const TC9: [&str; 6] = [
    "NM_CALCN_TYPE",
    "DT_EFFECT",
    "DT_END",
    "AM_MIN_INCOME",
    "AM_MAX_INCOME",
    "PC_TAX_RATE_A",
];
const OPEN: &str = "99999999999";
const SCALE_LABEL: &str = "Resident individual income tax rates";

fn table(columns: &[&str], rows: &[&[&str]]) -> Json {
    let columns = columns
        .iter()
        .map(|c| Json::obj([("name", Json::str(c))]))
        .collect();
    let rows = rows
        .iter()
        .map(|row| Json::Array(row.iter().map(|v| Json::str(v)).collect()))
        .collect();
    Json::obj([
        ("columns", Json::Array(columns)),
        ("rows", Json::Array(rows)),
    ])
}

fn tables(tc2: &[&[&str]], tc9: &[&[&str]]) -> Vec<(&'static str, Json)> {
    vec![
        ("TC2TAXRTE", table(&TC2, tc2)),
        ("TC9GENTAC", table(&TC9, tc9)),
    ]
}

fn ato_record(id: &str, label: &str, unit: &str, value: Json, period: &str) -> Json {
    Json::obj([
        ("id", Json::str(id)),
        ("label", Json::str(label)),
        ("unit", Json::str(unit)),
        ("value", value),
        ("period", j(period)),
    ])
}

fn scale() -> Json {
    j(
        r#"[{"from": 0, "to": 18200, "base_tax": 0, "marginal_rate": 0},
          {"from": 18201, "to": 45000, "base_tax": 0, "marginal_rate": 15},
          {"from": 45001, "to": null, "base_tax": 4020, "marginal_rate": 30}]"#,
    )
}

const SCALE_ROWS: [&[&str]; 4] = [
    &[
        "2020-07-01",
        "9999-12-31",
        "Resident Full Year",
        "-99999999999",
        "18200",
        "0",
        "0",
    ],
    &[
        "2024-07-01",
        "2026-06-30",
        "Resident Full Year",
        "18201",
        "45000",
        "0",
        "0.16",
    ],
    &[
        "2026-07-01",
        "9999-12-31",
        "Resident Full Year",
        "45001",
        OPEN,
        "4020",
        "0.3",
    ],
    &[
        "2026-07-01",
        "9999-12-31",
        "Resident Full Year",
        "18201",
        "45000",
        "0",
        "0.15",
    ],
];
const LEVY_ROWS: [&[&str]; 3] = [
    &[
        "2025-07-01",
        "9999-12-31",
        "Medicare Basic Levy",
        "35014",
        OPEN,
        "0",
        "0.02",
    ],
    &[
        "2025-07-01",
        "9999-12-31",
        "Medicare Basic Levy",
        "-99999999999",
        "28011",
        "0",
        "0",
    ],
    &[
        "2025-07-01",
        "9999-12-31",
        "Medicare Basic Levy",
        "28012",
        "35013",
        "0",
        "0.1",
    ],
];
const FBT: &[&str] = &[
    "2017-04-01",
    "9999-03-31",
    "FBT rate",
    "0",
    "0",
    "0",
    "0.47",
];

fn levy_records() -> Vec<Json> {
    vec![
        ato_record(
            "lower",
            "Medicare levy low-income lower threshold, single, not SAPTO",
            "AUD",
            int(28011),
            r#"{"income_year": "2025-26"}"#,
        ),
        ato_record(
            "upper",
            "Medicare levy low-income upper threshold, single, not SAPTO",
            "AUD",
            int(35013),
            r#"{"income_year": "2025-26"}"#,
        ),
        ato_record(
            "rate",
            "Medicare levy rate",
            "percent",
            int(2),
            r#"{"as_at": "2026-09-24"}"#,
        ),
    ]
}

/// `ato_table_report(records, tables)` as text: (lines, findings).
fn compare(records: &[Json], tables: &[(&str, Json)]) -> (Vec<String>, Vec<String>) {
    let (lines, count) = ato_table_report(records, tables).expect("no exception");
    let lines = texts(&lines);
    let findings = lines[lines.len() - 1 - count..lines.len() - 1].to_vec();
    (lines, findings)
}

#[test]
fn matching_figures_produce_no_finding() {
    let records = [
        ato_record(
            "scale",
            SCALE_LABEL,
            "tax_scale",
            scale(),
            r#"{"income_year": "2026-27"}"#,
        ),
        ato_record(
            "sg",
            "Super guarantee percentage (general)",
            "percent",
            int(12),
            r#"{"effective_from": "2026-07-01", "effective_to": "2027-06-30"}"#,
        ),
        ato_record(
            "div7a",
            "Division 7A benchmark interest rate",
            "percent",
            Json::Float(8.77),
            r#"{"income_year": "2026-27"}"#,
        ),
    ];
    let mut tc2 = SCALE_ROWS.to_vec();
    tc2.push(&[
        "2025-07-01",
        "9999-12-31",
        "Superannuation Guarantee Rate",
        "0",
        "0",
        "0",
        "0.12",
    ]);
    let tc9: [&[&str]; 2] = [
        &["DIV7A", "2026-07-01", "2027-06-30", "0", "0", "0.0877"],
        &["DIV7A", "2025-07-01", "2026-06-30", "0", "0", "0.0837"],
    ];
    let (lines, findings) = compare(&records, &tables(&tc2, &tc9));
    assert!(findings.is_empty());
    assert_eq!(
        lines[..3],
        [
            "scale: matches TC2TAXRTE",
            "sg: matches TC2TAXRTE",
            "div7a: matches TC9GENTAC"
        ]
    );
    assert!(
        lines
            .last()
            .unwrap()
            .starts_with("3 record(s) compared, 0 finding(s).")
    );
}

#[test]
fn the_medicare_figures_come_from_the_basic_levy_rows() {
    assert!(
        compare(&levy_records(), &tables(&LEVY_ROWS, &[]))
            .1
            .is_empty()
    );
}

#[test]
fn a_different_figure_is_a_finding_that_shows_both() {
    let records = [
        ato_record(
            "fbt",
            "Fringe benefits tax rate",
            "percent",
            int(48),
            r#"{"effective_from": "2026-04-01"}"#,
        ),
        ato_record(
            "scale",
            SCALE_LABEL,
            "tax_scale",
            scale(),
            r#"{"income_year": "2025-26"}"#,
        ),
    ];
    let mut tc2: Vec<&[&str]> = vec![FBT];
    tc2.extend(SCALE_ROWS);
    let (_, findings) = compare(&records, &tables(&tc2, &[]));
    assert_eq!(
        findings,
        [
            "fbt: TC2TAXRTE shows 47; the record has 48",
            "scale: TC2TAXRTE shows 16c for each $1 over $18,200; the record has \
             15c for each $1 over $18,200; $4,020 plus 30c for each $1 over $45,000"
        ]
    );
}

#[test]
fn a_missing_or_doubled_row_is_a_finding() {
    let records = [
        ato_record(
            "div7a",
            "Division 7A benchmark interest rate",
            "percent",
            int(9),
            r#"{"income_year": "2027-28"}"#,
        ),
        ato_record(
            "fbt",
            "Fringe benefits tax rate",
            "percent",
            int(47),
            r#"{"effective_from": "2026-04-01"}"#,
        ),
    ];
    let tc9: [&[&str]; 1] = [&["DIV7A", "2026-07-01", "2027-06-30", "0", "0", "0.0877"]];
    let (_, findings) = compare(&records, &tables(&[FBT, FBT], &tc9));
    assert_eq!(
        findings,
        [
            "div7a: cannot read TC9GENTAC {'NM_CALCN_TYPE': 'DIV7A'}: no row in force on 2027-07-01",
            "fbt: cannot read TC2TAXRTE {'TX_DESC': 'FBT rate'}: rows in force overlap from 0"
        ]
    );
}

#[test]
fn a_doubled_or_overlapping_medicare_row_is_a_finding() {
    // Doubling the highest bracket left every positional reader on a plausible
    // figure, and the comparison reported 3 matches and 0 findings.
    let overlap: &[&str] = &[
        "2025-07-01",
        "9999-12-31",
        "Medicare Basic Levy",
        "35000",
        "35013",
        "0",
        "0.1",
    ];
    for (extra, start) in [(LEVY_ROWS[0], 35014), (overlap, 35000)] {
        let mut tc2 = LEVY_ROWS.to_vec();
        tc2.push(extra);
        let (lines, findings) = compare(&levy_records(), &tables(&tc2, &[]));
        let expected: Vec<String> = ["lower", "upper", "rate"]
            .iter()
            .map(|id| format!("{id}: cannot read TC2TAXRTE {{'TX_DESC': 'Medicare Basic Levy'}}: rows in force overlap from {start}"))
            .collect();
        assert_eq!(findings, expected);
        assert!(
            lines
                .last()
                .unwrap()
                .starts_with("3 record(s) compared, 3 finding(s).")
        );
    }
}

#[test]
fn two_separate_rows_for_a_single_rate_are_a_finding() {
    let records = [ato_record(
        "fbt",
        "Fringe benefits tax rate",
        "percent",
        int(47),
        r#"{"effective_from": "2026-04-01"}"#,
    )];
    let second: &[&str] = &[
        "2017-04-01",
        "9999-03-31",
        "FBT rate",
        "1",
        "1",
        "0",
        "0.47",
    ];
    let (_, findings) = compare(&records, &tables(&[FBT, second], &[]));
    assert_eq!(
        findings,
        ["fbt: cannot read TC2TAXRTE {'TX_DESC': 'FBT rate'}: 2 rows in force, expected 1"]
    );
}

#[test]
fn records_the_tables_do_not_hold_are_listed_without_a_finding() {
    let records = [ato_record(
        "gst",
        "GST registration threshold",
        "AUD",
        int(75000),
        r#"{"as_at": "2026-09-24"}"#,
    )];
    let (lines, findings) = compare(&records, &tables(&[], &[]));
    assert!(findings.is_empty());
    assert_eq!(
        lines,
        ["0 record(s) compared, 0 finding(s). Not in the ATO tables: gst"]
    );
}

#[test]
fn every_label_the_comparison_reads_belongs_to_a_record() {
    let labels: Vec<String> = records()
        .iter()
        .map(|r| pyvalue::py_str(r.at("label")))
        .collect();
    for (label, ..) in ATO_TABLE_FIGURES {
        assert!(labels.iter().any(|l| l == label), "{label}");
    }
}

#[test]
fn the_command_reads_both_tables_and_exits_1_on_a_finding() {
    assert_eq!(ATO_HOST, "onlineservices.ato.gov.au");
    assert_eq!(crate::https::TIMEOUT, Duration::from_secs(60));
    let served = tables(
        &[&["2017-04-01", "9999-03-31", "FBT rate", "0", "0", "0", "0.5"]],
        &[],
    );
    let mut requested = Vec::new();
    let mut out = out();
    let fbt = ato_record(
        "fbt",
        "Fringe benefits tax rate",
        "percent",
        int(47),
        r#"{"effective_from": "2026-04-01"}"#,
    );
    let code = ato_tables_with(
        &mut out,
        |name, path| {
            requested.push(path.to_string());
            let table = &served.iter().find(|(n, _)| *n == name).unwrap().1;
            Ok((200, dumps(table)))
        },
        || Ok(vec![fbt]),
    )
    .unwrap();
    assert_eq!(code, 1);
    assert_eq!(
        requested,
        [
            "/cdn/static-data/codes-tables/TC2TAXRTE.json",
            "/cdn/static-data/codes-tables/TC9GENTAC.json"
        ]
    );
    assert!(printed(out).contains("fbt: TC2TAXRTE shows 50; the record has 47"));
}

#[test]
fn an_unreadable_table_ends_the_command_with_exit_1() {
    let mut out = out();
    let code = ato_tables_with(
        &mut out,
        |_, _| Err("network unreachable".into()),
        || panic!("records read"),
    )
    .unwrap();
    assert_eq!(code, 1);
    assert_eq!(
        printed(out),
        format!("cannot read the ATO tables: network unreachable{}", nl())
    );
}

#[test]
fn redirects_and_http_errors_end_the_command_without_following_them() {
    for status in [301, 302, 404, 500] {
        let mut requested = Vec::new();
        let mut out = out();
        let code = ato_tables_with(
            &mut out,
            |_, path| {
                requested.push(path.to_string());
                Ok((status, Vec::new()))
            },
            || panic!("records read"),
        )
        .unwrap();
        assert_eq!(code, 1);
        assert_eq!(
            printed(out),
            format!(
                "cannot read the ATO tables: TC2TAXRTE: HTTP {status}{}",
                nl()
            )
        );
        assert_eq!(requested, ["/cdn/static-data/codes-tables/TC2TAXRTE.json"]);
    }
}

#[test]
fn failed_requests_and_invalid_json_close_the_connection() {
    let responses: [TableResponse; 2] = [
        Err("invalid response".into()),
        Ok((200, b"invalid JSON".to_vec())),
    ];
    for response in responses {
        let mut out = out();
        let code =
            ato_tables_with(&mut out, |_, _| response.clone(), || panic!("records read")).unwrap();
        assert_eq!(code, 1);
        assert!(printed(out).starts_with("cannot read the ATO tables: "));
    }
}

// ------------------------------------------------------------------ regressions

#[test]
fn a_url_path_keeps_supplementary_characters_and_lone_surrogates_apart() {
    // CPython 3.15's urlparse(...).path for each tail.
    let path = |tail: &[u32]| {
        url_path(&[s("https://example.test/a"), tail.to_vec()].concat()).expect("a path")
    };
    let want = |tail: &[u32]| [s("/a"), tail.to_vec()].concat();
    assert_eq!(path(&[0x10_0000]), want(&[0x10_0000]));
    assert_eq!(path(&[0x10_07FF, 0xD800]), want(&[0x10_07FF, 0xD800]));
    assert_eq!(path(&[0xD800, 0x4_0000]), want(&[0xD800, 0x4_0000]));
    assert_eq!(path(&[0xD800, 0x3B, 0x78]), want(&[0xD800]));
}

#[test]
fn the_browser_timeout_covers_a_pipe_a_descendant_holds_open() {
    // The command exits at once, leaving a 6-second descendant on its pipes.
    let command: Vec<Json> = if cfg!(windows) {
        ["cmd", "/c", "start", "/b", "ping", "-n", "7", "127.0.0.1"]
            .map(Json::str)
            .to_vec()
    } else {
        ["sh", "-c", "sleep 6 & exit 0"].map(Json::str).to_vec()
    };
    let started = Instant::now();
    let error = run_with_limit(&command, Duration::from_secs(1)).expect_err("a timeout");
    assert_eq!(error.class, "subprocess.TimeoutExpired");
    assert!(st(&error.text).ends_with("' timed out after 1 seconds"));
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn a_stdout_that_cannot_be_written_ends_with_status_120() {
    struct Full;
    impl Write for Full {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(28))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut out = Out::new(Full);
    out.print_str("one")
        .expect("print keeps going, as a buffered print does");
    out.print_str("two")
        .expect("print keeps going, as a buffered print does");
    assert_eq!(out.finish(), Some(120));
    assert_eq!(Out::new(Vec::new()).finish(), None);
}

#[test]
fn the_browser_is_given_the_url_where_test_dataset_py_looked_for_it() {
    // Python's fake subprocess.run picked its reply by args[3].
    let url = Json::str("https://www.ato.gov.au/rates");
    if std::env::var_os("RATES_BROWSER").is_none() {
        assert_eq!(browser_args(&browser_command(), &url)[3], url);
    }
}

#[test]
fn check_out_names_the_report_folder() {
    let args = ["check", "--out", "reports"].map(String::from);
    let Ok(namespace) = build_parser(&ctx()).parse(&args) else {
        panic!("check --out parses");
    };
    assert!(matches!(namespace.get("command"), Some(Value::Str(s)) if s == "check"));
    assert!(matches!(namespace.get("out"), Some(Value::Str(s)) if s == "reports"));
}

// --------------------------------------------------------- the command line

/// The package with its data folder holding just these records.
fn package(records: &[Json]) -> Ctx {
    let folder = tmp("data");
    for (i, record) in records.iter().enumerate() {
        std::fs::write(folder.join(format!("record-{i}.json")), dumps(record))
            .expect("write record");
    }
    let real = ctx();
    Ctx {
        data: PyPath::new(&folder.to_string_lossy()),
        root: real.root,
        snapshots: real.snapshots,
    }
}

/// `main(args)` with the browser and the ATO's server replaced: the exit
/// status and stdout.
fn run_with(
    ctx: &Ctx,
    args: &[&str],
    fetch: &mut dyn FnMut(&Json) -> R<S>,
    get: &mut dyn FnMut(&str, &str) -> TableResponse,
) -> (i32, String) {
    let argv: Vec<OsString> = args.iter().map(OsString::from).collect();
    let mut stdout = Vec::new();
    let world = World {
        fetch,
        pause: false,
        get,
    };
    let code = run(ctx, &argv, world, &mut stdout);
    (code, String::from_utf8(stdout).expect("UTF-8 output"))
}

#[test]
fn main_check_fetches_each_source_and_writes_its_report() {
    let item = fabricated(&[]);
    let url = item.at("source_url").clone();
    let ctx = package(&[item]);
    let folder = tmp("reports");
    let mut fetched = Vec::new();
    let (code, printed) = run_with(
        &ctx,
        &["check", "--out", &folder.to_string_lossy()],
        &mut |u| {
            fetched.push(u.clone());
            Ok(s("Effective date: 31 February 2026"))
        },
        &mut |_, _| panic!("a table read"),
    );
    assert_eq!(code, 1);
    assert_eq!(fetched, [url]);
    assert!(printed.starts_with("1 finding(s); report at "), "{printed}");
    assert_eq!(
        std::fs::read_dir(&folder)
            .expect("the report folder")
            .count(),
        1
    );
}

#[test]
fn main_ato_tables_reads_both_tables_and_keeps_escaped_pairs_apart_from_raw_surrogates() {
    // Rates review 5, finding 1: the raw U+D800 in "note" once took U+40000
    // as its stand-in, so the escaped U+40000 in DT_EFFECT came back as
    // U+D800 and the row counted as in force. CPython 3.15 at 24fcf24 prints
    // these lines and exits 1.
    let tc2 = b"{\"note\":\"\xed\xa0\x80\",\"columns\":[{\"name\":\"DT_EFFECT\"},{\"name\":\"DT_END\"},\
{\"name\":\"TX_DESC\"},{\"name\":\"AM_INC_MIN\"},{\"name\":\"AM_INC_MAX\"},{\"name\":\"AM_OFFSET\"},\
{\"name\":\"PC_TAX_RATE\"}],\"rows\":[[\"\\ud8c0\\udc00\",\"\\ud900\\udc00\",\"FBT rate\",\"0\",\"0\",\"0\",\"0.47\"]]}";
    let tc9 = dumps(&table(&TC9, &[]));
    let fbt = ato_record(
        "fbt",
        "Fringe benefits tax rate",
        "percent",
        int(47),
        "{\"income_year\": \"\u{FF12}\u{FF10}\u{FF12}\u{FF16}-\u{FF12}\u{FF17}\"}",
    );
    let mut requested = Vec::new();
    let (code, printed) = run_with(
        &package(&[fbt]),
        &["ato-tables"],
        &mut |_| panic!("a page fetched"),
        &mut |name, _| {
            requested.push(name.to_string());
            Ok((
                200,
                if name == "TC2TAXRTE" {
                    tc2.to_vec()
                } else {
                    tc9.clone()
                },
            ))
        },
    );
    assert_eq!(code, 1);
    assert_eq!(requested, ["TC2TAXRTE", "TC9GENTAC"]);
    let want = "fbt: cannot read TC2TAXRTE {'TX_DESC': 'FBT rate'}: no row in force on \u{FF12}\u{FF10}\u{FF12}\u{FF16}-07-01\n\
1 record(s) compared, 1 finding(s). Not in the ATO tables: none\n";
    assert_eq!(printed, want.replace('\n', nl()));
}

#[test]
fn main_help_is_written_to_its_stdout() {
    let (code, printed) = run_with(
        &ctx(),
        &["--help"],
        &mut |_| panic!("a page fetched"),
        &mut |_, _| panic!("a table read"),
    );
    assert_eq!(code, 0);
    assert!(printed.starts_with("usage: rates"), "{printed}");
}

#[test]
fn an_invalid_browser_reply_fails_only_its_own_page_through_fetch_text() {
    // Python's fake subprocess.run answered by URL; here a script does.
    let good = browser_reply(0, "200", true, &"x".repeat(250));
    let folder = tmp("browser");
    let browser: Vec<String> = if cfg!(windows) {
        let script = folder.join("browser.cmd");
        let body = format!(
            "@echo off\r\nif \"%2\"==\"https://www.ato.gov.au/bad\" (echo null) else (echo {good})\r\n"
        );
        std::fs::write(&script, body).expect("write the browser");
        vec![script.to_string_lossy().into_owned()]
    } else {
        let script = folder.join("browser.sh");
        let body = format!("case \"$2\" in *bad) echo null ;; *) echo '{good}' ;; esac\n");
        std::fs::write(&script, body).expect("write the browser");
        vec!["sh".into(), script.to_string_lossy().into_owned()]
    };
    let urls = [
        Json::str("https://www.ato.gov.au/bad"),
        Json::str("https://www.ato.gov.au/ok"),
    ];
    let pages = fetch_all_with(&urls, |url| fetch_text_with(&browser, url), false).unwrap();
    let error = pages[0].1.as_ref().unwrap_err();
    assert_eq!(error.class, "RuntimeError");
    assert_eq!(st(&error.text), "the browser's reply is not a page");
    assert_eq!(st(pages[1].1.as_ref().unwrap()), "x".repeat(250));
}
