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

fn edge_context(root: PathBuf) -> Context {
    Context {
        root: Some(root),
        limits: Default::default(),
        denied: false,
        principal_acts: None,
        fault: None,
    }
}

fn write_def_act(dir: &Path, id: &str, act: &str, text: &str) {
    fs::create_dir_all(dir.join(id)).unwrap();
    let row = json!({
        "row_id": format!("{id}:0001:2"),
        "register_id": id,
        "act": act,
        "section": "2",
        "heading": "2 Definitions",
        "text": text,
        "version_is_current": true,
    });
    fs::write(dir.join(id).join("sections.jsonl"), format!("{row}\n")).unwrap();
}

#[test]
fn define_term_nearest_terms_keep_full_partial_heads_before_limit_and_clear_on_exact() {
    let tree = relation_tree("define-nearest");
    let dir = tree.0.join("markdown");
    write_def_act(
        &dir,
        "C9999A00001",
        "Synthetic Beta Act 2099",
        "small cap means 1.\n\nsmall cap limit means 2.",
    );
    write_def_act(
        &dir,
        "C9999A00002",
        "Income Tax Assessment Act 1997",
        "small cap threshold means 3.\n\nsmall cap means 4.",
    );
    let ctx = edge_context(tree.0.clone());
    let partial = corpus::evaluate(
        &ctx,
        "define_tax_term",
        &json!({"term":"cap","act":null,"limit":1,"in_force_only":true}),
    )
    .unwrap();
    assert_eq!(partial["defined"], false);
    assert_eq!(partial["definitions"].as_array().unwrap().len(), 1);
    assert_eq!(partial["has_more"], true);
    assert!(partial["defined_note"].is_string());
    assert_eq!(
        partial["nearest_terms"],
        json!(["small cap threshold", "small cap", "small cap limit"])
    );
    let exact = corpus::evaluate(
        &ctx,
        "define_tax_term",
        &json!({"term":"small cap","act":"income tax assessment","limit":1,"in_force_only":true}),
    )
    .unwrap();
    assert_eq!(exact["defined"], true);
    assert_eq!(exact["nearest_terms"], json!([]));
    assert!(exact["defined_note"].is_null());
}

#[test]
fn cited_by_preserves_unicode_offsets_paragraph_cutoff_and_truncation() {
    let tree = relation_tree("cited-by-edges");
    let dir = tree.0.join("markdown");
    fs::create_dir_all(dir.join("C9999A00001")).unwrap();
    let target = json!({"row_id":"C9999A00001:0001:5-10","register_id":"C9999A00001",
        "act":"Synthetic Levy Act 2099","section":"5\u{2011}10","heading":"5-10 Rate",
        "text":"Rate is 7%.", "version_is_current":true});
    let long_tail = " word".repeat(160);
    let body = format!(
        "Intro \u{2011}\u{00a0}ignore.\n\nSee s\u{00a0}5\u{2011}10, s 5\u{2011}10, and sections 5-5, 5-10(2)(a) and 5-15{long_tail}.\n\nClosing."
    );
    let citing = json!({"row_id":"C9999A00001:0002:5-20","register_id":"C9999A00001",
        "act":"Synthetic Levy Act 2099","section":"5-20","heading":"5-20 Use",
        "text":body,"version_is_current":true});
    fs::write(
        dir.join("C9999A00001/sections.jsonl"),
        format!("{target}\n{citing}\n"),
    )
    .unwrap();
    let res = corpus::evaluate(
        &edge_context(tree.0.clone()),
        "find_citing_provisions",
        &json!({"row_id":"C9999A00001:0001:5-10","limit":20,"in_force_only":true}),
    )
    .unwrap();
    let entry = &res["cited_by"][0];
    assert_eq!(
        entry["citations"],
        json!(["s 5\u{2011}10", "sections 5-5, 5-10(2)(a) and 5-15"])
    );
    assert!(!entry["text"].as_str().unwrap().contains("Intro"));
    assert!(!entry["text"].as_str().unwrap().contains("Closing"));
    assert!(entry["total_chars"].as_u64().unwrap() > 600);
    assert!(
        entry["caveats"][0]
            .as_str()
            .unwrap()
            .contains("truncated at 600")
    );
}

/// The case table `tests/test_citing_provisions.py` also reads: ranges, lists and single
/// sections give the same citing provisions in both servers.
#[test]
fn cited_by_follows_the_shared_range_case_table() {
    let table: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/cited-ranges.json")).unwrap();
    let (register_id, act) = (table["register_id"].as_str().unwrap(), &table["act"]);
    for case in table["cases"].as_array().unwrap() {
        let tree = relation_tree("cited-ranges");
        let dir = tree.0.join("markdown").join(register_id);
        fs::create_dir_all(&dir).unwrap();
        let mut texts: Vec<(String, Value)> = table["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|label| (label.as_str().unwrap().to_owned(), json!("Rate.")))
            .collect();
        for (label, text) in case["citing"].as_object().unwrap() {
            texts.push((label.clone(), text.clone()));
        }
        let row_id = |n: usize, label: &str| format!("{register_id}:{n:04}:{label}");
        let rows: String = texts
            .iter()
            .enumerate()
            .map(|(i, (label, text))| {
                let row = json!({"row_id": row_id(i + 1, label), "register_id": register_id,
                    "act": act, "section": label, "heading": format!("{label} Rule"),
                    "text": text, "version_is_current": true});
                format!("{row}\n")
            })
            .collect();
        fs::write(dir.join("sections.jsonl"), rows).unwrap();
        let other = table["other_register_id"].as_str().unwrap();
        let elsewhere = case.get("elsewhere").and_then(Value::as_object);
        let rows: String = elsewhere
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(i, (label, text))| {
                let row = json!({"row_id": format!("{other}:{:04}:{label}", i + 1),
                    "register_id": other, "act": table["other_act"], "section": label,
                    "heading": format!("{label} Rule"), "text": text,
                    "version_is_current": true});
                format!("{row}\n")
            })
            .collect();
        let other_dir = tree.0.join("markdown").join(other);
        fs::create_dir_all(&other_dir).unwrap();
        fs::write(other_dir.join("sections.jsonl"), rows).unwrap();
        for (cited, expected) in case["cited_by"].as_object().unwrap() {
            let n = texts.iter().position(|(label, _)| label == cited).unwrap() + 1;
            let res = corpus::evaluate(
                &edge_context(tree.0.clone()),
                "find_citing_provisions",
                &json!({"row_id": row_id(n, cited), "limit": 20, "in_force_only": true}),
            )
            .unwrap();
            let found: Vec<_> = res["cited_by"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["section"].clone())
                .collect();
            assert_eq!(json!(found), *expected, "{} {cited}", case["name"]);
        }
    }
}

#[test]
fn corpus_search_refuses_row_nested_past_the_limit() {
    let tree = relation_tree("corpus-rows-depth");
    let dir = tree.0.join("markdown/C9999A00001");
    fs::create_dir_all(&dir).unwrap();
    let deep = format!(
        r#"{}"synthetic levy"{}"#,
        "[".repeat(1001),
        "]".repeat(1001)
    );
    fs::write(dir.join("sections.jsonl"), format!("{deep}\n")).unwrap();
    let err = corpus::evaluate(
        &edge_context(tree.0.clone()),
        "search_tax_legislation",
        &json!({"query":"synthetic levy","limit":5,"offset":0,"act":null,"in_force_only":true}),
    )
    .unwrap_err();
    assert_eq!(
        err,
        "Cannot read a UTF-8 JSONL index in the configured corpus."
    );
}

#[cfg(unix)]
struct RestoreMode(PathBuf, u32);
#[cfg(unix)]
impl Drop for RestoreMode {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(self.1));
    }
}

#[cfg(unix)]
#[test]
fn corpus_provenance_ignores_unreadable_sources_json() {
    use std::os::unix::fs::PermissionsExt;
    let tree = relation_tree("corpus-sources-unreadable");
    let dir = tree.0.join("markdown");
    write_def_act(&dir, "C9999A00001", "Synthetic Act 2099", "Synthetic levy.");
    let sources = tree.0.join("sources.json");
    fs::write(&sources, r#"{"corpus":"Synthetic"}"#).unwrap();
    fs::set_permissions(&sources, fs::Permissions::from_mode(0o000)).unwrap();
    let _guard = RestoreMode(sources, 0o644);
    let res = corpus::evaluate(
        &edge_context(tree.0.clone()),
        "search_tax_legislation",
        &json!({"query":"synthetic levy","limit":5,"offset":0,"act":null,"in_force_only":true}),
    )
    .unwrap();
    assert_eq!(res["corpus"], json!({}));
}

#[cfg(unix)]
#[test]
fn library_search_skips_unreadable_subdirectory() {
    use std::os::unix::fs::PermissionsExt;
    let tree = relation_tree("library-unreadable-dir");
    fs::write(tree.0.join("ok.md"), "GST guidance").unwrap();
    let locked = tree.0.join("locked");
    fs::create_dir(&locked).unwrap();
    fs::write(locked.join("hidden.md"), "GST hidden").unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    let _guard = RestoreMode(locked, 0o755);
    let found = library::search(&edge_context(tree.0.clone()), "GST", 5, 0).unwrap();
    let matches = found["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["path"], "ok.md");
}

#[cfg(unix)]
#[test]
fn lines_iterator_surfaces_read_error_on_directory() {
    let tree = relation_tree("lines-read-dir");
    let err = Lines::open(&tree.0).unwrap().next().unwrap().unwrap_err();
    assert!(err.raw_os_error().is_some());
}

#[test]
fn rulings_census_refuses_non_object_row_after_cache_hit() {
    let fixture = fixture();
    let tree = relation_tree("rulings-census-non-object");
    let valid = restore(
        &tree.0,
        original_case(
            &fixture,
            "tests/test_rulings.py::test_a_same_day_tie_goes_to_the_later_folder_name",
            "search_rulings",
        ),
        &fixture["blobs"],
    );
    let run = tree.0.join("run-2099-01-01");
    fs::create_dir(&run).unwrap();
    copy_ruling_files(valid.root.as_ref().unwrap().join("run-b"), &run);
    let rows_path = run.join("rulings.jsonl");
    let fixed = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
    let set_time = || {
        fs::File::options()
            .write(true)
            .open(&rows_path)
            .unwrap()
            .set_modified(fixed)
            .unwrap();
    };
    set_time();
    let ctx = edge_context(run);
    assert!(rulings::census(&ctx).is_ok());
    let len = fs::metadata(&rows_path).unwrap().len() as usize;
    fs::write(&rows_path, format!("[{}]", " ".repeat(len - 2))).unwrap();
    set_time();
    let err = rulings::census(&ctx).unwrap_err();
    assert_eq!(err, "a rulings row is not a JSON object.");
}
