// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::super::*;
use super::{Tree, fixture, restore, scratch};
#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::ffi::OsStringExt;

/// A row nested past any parser's depth is a broken corpus, not a crash. Python's
/// RecursionError text and serde_json's depth error differ, so only the verdict is compared.
#[test]
fn doctor_reports_a_row_nested_past_the_limit_as_broken() {
    let tree = Tree(scratch("native-mcp-nested"));
    let titles = tree.0.join("markdown/C9999A00001");
    fs::create_dir_all(&titles).unwrap();
    fs::create_dir_all(tree.0.join("rates")).unwrap();
    let row = r#"{"row_id":"C9999A00001:0001:1","text":"Synthetic levy."}"#;
    fs::write(
        titles.join("sections.jsonl"),
        format!("{row}\n{}\n", "[".repeat(1_000_000)),
    )
    .unwrap();
    fs::write(
        tree.0.join("rates/rates.jsonl"),
        r#"{"rate_id":"r1","content":"A cap of $6."}"#.to_owned() + "\n",
    )
    .unwrap();
    let context = Context {
        root: Some(tree.0.clone()),
        limits: Default::default(),
        denied: false,
        principal_acts: None,
        fault: None,
    };
    let (ok, _) = crate::doctor::judged("AUS_ACCOUNTING_CORPUS_ROOT", corpus::census(&context));
    assert!(!ok);
}

fn original_case<'a>(fixture: &'a Value, test: &str, function: &str) -> &'a Value {
    fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["test"] == test && item["function"] == function)
        .unwrap()
}

fn relation_tree(label: &str) -> Tree {
    let tree = Tree(scratch(&format!("native-mcp-relation-{label}")));
    fs::create_dir(&tree.0).unwrap();
    tree
}

#[test]
fn corpus_link_refusal_with_a_valid_readable_external_index() {
    let fixture = fixture();
    let tree = relation_tree("corpus-link");
    let valid = restore(
        &tree.0,
        original_case(
            &fixture,
            "tests/test_corpus.py::test_search_returns_a_traceable_citation",
            "search_sections",
        ),
        &fixture["blobs"],
    );
    let linked = restore(
        &tree.0,
        original_case(
            &fixture,
            "tests/test_corpus.py::test_a_linked_markdown_directory_is_refused_by_read_as_well_as_search",
            "read_section",
        ),
        &fixture["blobs"],
    );
    let target = tree.0.join("outside/C9999A00001/sections.jsonl");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(
        valid
            .root
            .as_ref()
            .unwrap()
            .join("markdown/C9999A00001/sections.jsonl"),
        &target,
    )
    .unwrap();
    let args = json!({"row_id":"C9999A00001:0003:5-15","neighbours":0,"start":0});
    assert!(corpus::evaluate(&valid, "read_tax_legislation_section", &args).is_ok());
    assert!(
        linked
            .root
            .as_ref()
            .unwrap()
            .join("markdown/C9999A00001/sections.jsonl")
            .is_file()
    );
    assert_eq!(
        corpus::evaluate(&linked, "read_tax_legislation_section", &args).unwrap_err(),
        "No title index for that row_id in the configured corpus."
    );
    assert_eq!(corpus::evaluate(&linked, "search_tax_legislation", &json!({"query":"synthetic levy rate","limit":5,"offset":0,"act":null,"in_force_only":true})).unwrap_err(), "The corpus folder must contain a markdown directory of title indexes.");
}

#[test]
fn rulings_link_refusal_with_a_valid_searchable_external_run() {
    let fixture = fixture();
    let tree = relation_tree("rulings-link");
    let valid = restore(
        &tree.0,
        original_case(
            &fixture,
            "tests/test_rulings.py::test_a_same_day_tie_goes_to_the_later_folder_name",
            "search_rulings",
        ),
        &fixture["blobs"],
    );
    let linked = restore(
        &tree.0,
        original_case(
            &fixture,
            "tests/test_rulings.py::test_a_linked_run_is_not_followed",
            "search_rulings",
        ),
        &fixture["blobs"],
    );
    let outside = tree.0.join("outside");
    copy_ruling_files(valid.root.as_ref().unwrap().join("run-b"), &outside);
    rewrite_linked_wording(&outside);
    let direct = Context {
        root: Some(outside),
        limits: Default::default(),
        denied: false,
        principal_acts: None,
        fault: None,
    };
    let args = json!({"query":"linked outside wording","limit":5,"offset":0,"family":null});
    assert!(!ruling_matches(&direct, &args).is_empty());
    assert!(ruling_matches(&linked, &args).is_empty());
}

fn copy_ruling_files(from: PathBuf, outside: &Path) {
    for name in ["manifest.json", "rulings.jsonl", "LICENCE-NOTICE.md"] {
        fs::copy(from.join(name), outside.join(name)).unwrap();
    }
}

fn rewrite_linked_wording(outside: &Path) {
    let rows = fs::read_to_string(outside.join("rulings.jsonl")).unwrap();
    assert!(rows.contains("second fetch wording"));
    fs::write(
        outside.join("rulings.jsonl"),
        rows.replace("second fetch wording", "linked outside wording"),
    )
    .unwrap();
}

fn ruling_matches(context: &Context, args: &Value) -> Vec<Value> {
    rulings::evaluate(context, "search_ato_rulings", args).unwrap()["matches"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn oversized_rulings_rows_are_refused_before_an_invalid_manifest() {
    let fixture = fixture();
    let tree = relation_tree("rulings-order");
    let mut context = restore(
        &tree.0,
        original_case(
            &fixture,
            "tests/test_rulings.py::test_an_oversized_folder_is_refused_before_anything_is_parsed",
            "search_rulings",
        ),
        &fixture["blobs"],
    );
    fs::write(
        context
            .root
            .as_ref()
            .unwrap()
            .join("run-2099-01-01/manifest.json"),
        "{not json",
    )
    .unwrap();
    let args = json!({"query":"synthetic","limit":5,"offset":0,"family":null});
    assert_eq!(
        rulings::evaluate(&context, "search_ato_rulings", &args).unwrap_err(),
        "The configured rulings runs hold more than 0 MB of rows; configure a smaller folder."
    );
    context
        .limits
        .insert("MAX_CORPUS_BYTES".into(), 320_000_000);
    let error = rulings::evaluate(&context, "search_ato_rulings", &args).unwrap_err();
    assert!(error.contains("invalid manifest.json"), "{error}");
}

#[test]
fn original_queries_are_tokenised_before_ranking() {
    for (query, row, expected) in token_cases() {
        assert_eq!(
            corpus::tier(&terms(query), &terms(query), &["heading"], &["text"], &row),
            expected,
            "{query}: {row}"
        );
    }
}

fn token_cases() -> Vec<(&'static str, serde_json::Value, usize)> {
    vec![
        (
            "small entity",
            json!({"heading":"8-5 Meaning of small entity"}),
            0,
        ),
        (
            "small entity",
            json!({"heading":"9 Entity that is small"}),
            1,
        ),
        (
            "small entity",
            json!({"heading":"4 Other","text":"a *small entity for the year"}),
            2,
        ),
        (
            "small entity",
            json!({"heading":"4 Other","text":"a small or large entity"}),
            3,
        ),
        (
            "40-230 car limit",
            json!({"heading":"40\u{2011}230 Car limit"}),
            0,
        ),
        (
            "write-off",
            json!({"heading":"1 Other","text":"the write\u{2011}off applies"}),
            2,
        ),
        (
            "car limit",
            json!({"heading":"9 Scar limitation","text":"a car limit applies"}),
            2,
        ),
        (
            "car limit",
            json!({"heading":"9 Scar limitation","text":"scar limitation"}),
            3,
        ),
    ]
}

fn dated_search(context: &Context, query: &str, on: &str) -> Result<Value, String> {
    corpus::evaluate(
        context,
        "search_tax_legislation",
        &json!({"query":query,"limit":5,"offset":0,"act":null,"in_force_only":true,"as_at":on}),
    )
}

fn dated_read(context: &Context, row_id: &str, neighbours: u64) -> Result<Value, String> {
    corpus::evaluate(
        context,
        "read_tax_legislation_section",
        &json!({"row_id":row_id,"neighbours":neighbours,"start":0,"as_at":"2098-08-01"}),
    )
}

/// Main 139d5ac (#594): a dated call classifies a title from its first row and checks
/// only the rows it returns or counts, so a disagreeing row no result uses leaves it
/// answering, and a read looks for its row before checking the rows around it.
#[test]
fn a_dated_call_checks_only_the_rows_it_uses() {
    let tree = Tree(scratch("native-mcp-agreeing"));
    assert!(!tree.0.exists());
    let index = tree.0.join("markdown/C9999A10001/sections.jsonl");
    fs::create_dir_all(index.parent().unwrap()).unwrap();
    fs::write(&index, dated_rows().join("\n") + "\n").unwrap();
    let context = Context {
        root: Some(tree.0.clone()),
        limits: Default::default(),
        denied: false,
        principal_acts: None,
        fault: None,
    };
    let search = |query: &str, on: &str| dated_search(&context, query, on);
    let read = |row_id: &str, neighbours: u64| dated_read(&context, row_id, neighbours);
    let broken = "Rows of C9999A10001 disagree about their compilation window; the title's index is broken: tell the user, and do not retry.";

    let found = search("return lodged", "2098-08-01").unwrap();
    let ids: Vec<&Value> = found["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| &hit["row_id"])
        .collect();
    assert_eq!(ids, ["C9999A10001:0002:2"]);
    assert_eq!(
        read("C9999A10001:0001:1", 0).unwrap()["section"]["as_at"]["status"],
        "captured"
    );
    assert_eq!(
        read("C9999A10001:0009:9", 0).unwrap_err(),
        "No section with that row_id in the configured corpus."
    );
    // A row a page returns, a refusal counts or a read returns beside its row is checked.
    assert_eq!(search("receipt kept", "2098-08-01").unwrap_err(), broken);
    assert_eq!(search("receipt kept", "2098-01-01").unwrap_err(), broken);
    assert_eq!(read("C9999A10001:0002:2", 1).unwrap_err(), broken);
}

// APFS refuses a file name that is not UTF-8 (EILSEQ), so macOS cannot hold the names.
#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn invalid_filename_bytes_cannot_alias_a_unicode_citation() {
    let tree = Tree(scratch("native-mcp-filename"));
    assert!(!tree.0.exists());
    fs::create_dir(&tree.0).unwrap();
    let context = Context {
        root: Some(tree.0.clone()),
        limits: Default::default(),
        denied: false,
        principal_acts: None,
        fault: None,
    };
    fs::write(tree.0.join("safe.md"), "GST safe").unwrap();
    fs::write(
        tree.0
            .join(std::ffi::OsString::from_vec(b"\xff.txt".to_vec())),
        "Unrelated file",
    )
    .unwrap();
    assert_eq!(
        library::search(&context, "GST", 5, 0).unwrap()["matches"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    fs::write(tree.0.join("\u{fffd}.md"), "GST Unicode name").unwrap();
    fs::write(
        tree.0
            .join(std::ffi::OsString::from_vec(b"\xff.md".to_vec())),
        "GST invalid byte name",
    )
    .unwrap();
    assert_eq!(
        library::search(&context, "GST", 5, 0).unwrap_err(),
        "Library paths must use Unicode names."
    );
}

/// CPython 3.15's texts for these lines (B-1 receipts, utf8_messages.py), read from one file.
#[test]
fn a_line_that_is_not_utf8_reports_pythons_decode_error() {
    let tree = Tree(scratch("native-mcp-utf8"));
    fs::create_dir(&tree.0).unwrap();
    let first = |raw: &[u8]| {
        let path = tree.0.join("lines.jsonl");
        fs::write(&path, raw).unwrap();
        Lines::open(&path)
            .unwrap()
            .find_map(Result::err)
            .unwrap()
            .to_string()
    };
    for (raw, text) in broken_utf8() {
        assert_eq!(
            first(raw),
            format!("'utf-8' codec can't decode {text}"),
            "{raw:?}"
        );
    }
}

fn dated_rows() -> Vec<String> {
    let texts = [
        "The synthetic levy applies to a dated receipt.",
        "The synthetic levy return must be lodged.",
        "A dated receipt must be kept.",
    ];
    texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            // The third row's window disagrees with the first row's.
            let compiled = if i == 2 { "2098-06-01" } else { "2098-07-01" };
            json!({"register_id":"C9999A10001","act":"Synthetic Act 2098",
                "row_id":format!("C9999A10001:{:04}:{}", i + 1, i + 1),"section":(i + 1).to_string(),
                "heading":format!("{} {}", i + 1, text.trim_end_matches('.')),"text":text,
                "compilation_date":compiled,"retrieved":"2099-01-01","version_is_current":true,
                "superseded_from":null})
            .to_string()
        })
        .collect()
}

fn broken_utf8() -> Vec<(&'static [u8], &'static str)> {
    vec![
        (
            &b"\xff\n"[..],
            "byte 0xff in position 0: invalid start byte",
        ),
        (
            b"ok\xe2\x82A\n",
            "bytes in position 2-3: invalid continuation byte",
        ),
        (
            b"\xe2A\n",
            "byte 0xe2 in position 0: invalid continuation byte",
        ),
        (
            b"\xf0\x80\n",
            "byte 0xf0 in position 0: invalid continuation byte",
        ),
        (b"\xc0\n", "byte 0xc0 in position 0: invalid start byte"),
        (
            b"\xe2\x82\n",
            "bytes in position 0-1: invalid continuation byte",
        ),
        (
            b"ab\xe2\x82",
            "bytes in position 0-1: unexpected end of data",
        ),
        (
            b"\xed\xa0\x80\n",
            "byte 0xed in position 0: invalid continuation byte",
        ),
        (
            b"a\r\nb\rc\n\xff\n",
            "byte 0xff in position 7: invalid start byte",
        ),
    ]
}
