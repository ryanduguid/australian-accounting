// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::*;
use sha2::{Digest, Sha256};

struct Tree(PathBuf);
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fixture() -> Value {
    let mut fixture: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native/retrieval-contracts.json"
    ))
    .unwrap();
    let edges: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native/retrieval-edges.json"
    ))
    .unwrap();
    fixture["cases"]
        .as_array_mut()
        .unwrap()
        .extend(edges["cases"].as_array().unwrap().iter().cloned());
    fixture["blobs"].as_object_mut().unwrap().extend(
        edges["blobs"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    fixture
}

fn bytes(blob: &Value) -> Vec<u8> {
    if let Some(text) = blob["utf8"].as_str() {
        return text.as_bytes().to_vec();
    }
    if let Some(hex) = blob["hex"].as_str() {
        return hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
            .collect();
    }
    vec![blob["repeat_byte"].as_u64().unwrap() as u8; blob["length"].as_u64().unwrap() as usize]
}

fn restore(tree: &Path, item: &Value, blobs: &Value) -> Context {
    let state = &item["root"];
    let root = state["configured"].as_bool().unwrap_or(false).then(|| {
        let relative = Path::new(state["root_id"].as_str().unwrap());
        assert!(
            !relative.is_absolute()
                && relative
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_)))
        );
        tree.join(relative)
    });
    if let Some(root) = &root {
        if root.is_dir() {
            fs::remove_dir_all(root).unwrap();
        } else if root.exists() {
            fs::remove_file(root).unwrap();
        }
        if state["exists"] == true {
            fs::create_dir_all(root.parent().unwrap()).unwrap();
            if state["kind"] == "file" {
                fs::write(root, bytes(&blobs[state["file_blob"].as_str().unwrap()])).unwrap();
            } else {
                fs::create_dir_all(root).unwrap();
                for entry in state["files"].as_array().unwrap() {
                    let relative = Path::new(entry["path"].as_str().unwrap());
                    assert!(
                        !relative.is_absolute()
                            && relative
                                .components()
                                .all(|c| matches!(c, std::path::Component::Normal(_)))
                    );
                    let path = root.join(relative);
                    match entry["kind"].as_str().unwrap() {
                        "directory" => fs::create_dir_all(&path).unwrap(),
                        "file" => {
                            fs::create_dir_all(path.parent().unwrap()).unwrap();
                            let raw = bytes(&blobs[entry["blob"].as_str().unwrap()]);
                            assert_eq!(
                                Sha256::digest(&raw)
                                    .iter()
                                    .map(|b| format!("{b:02x}"))
                                    .collect::<String>(),
                                entry["blob"].as_str().unwrap()
                            );
                            fs::write(&path, raw).unwrap();
                            if let Some(time) = entry["mtime_ns"].as_u64() {
                                fs::File::options()
                                    .write(true)
                                    .open(path)
                                    .unwrap()
                                    .set_modified(
                                        std::time::UNIX_EPOCH
                                            + std::time::Duration::from_nanos(time),
                                    )
                                    .unwrap();
                            }
                        }
                        "link" => {
                            // The capture records links without following their targets.
                            // Recreate them only against a fabricated sibling tree.
                            let outside = tree.join("outside");
                            fs::create_dir_all(&outside).unwrap();
                            fs::create_dir_all(path.parent().unwrap()).unwrap();
                            if entry["directory"] == true {
                                #[cfg(windows)]
                                std::os::windows::fs::symlink_dir(&outside, &path).unwrap();
                                #[cfg(unix)]
                                std::os::unix::fs::symlink(&outside, &path).unwrap();
                            } else {
                                let target = outside.join("linked.md");
                                fs::write(&target, "Synthetic untrusted reference").unwrap();
                                #[cfg(windows)]
                                std::os::windows::fs::symlink_file(&target, &path).unwrap();
                                #[cfg(unix)]
                                std::os::unix::fs::symlink(&target, &path).unwrap();
                            }
                        }
                        other => panic!("Unexpected fixture kind {other}"),
                    }
                }
            }
        }
    }
    Context {
        root,
        limits: item["limits"]
            .as_object()
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_u64().map(|n| (k.clone(), n as usize)))
                    .collect()
            })
            .unwrap_or_default(),
        denied: item["fault"] == "library-permission-denied",
        principal_acts: item["principal_acts"]
            .as_array()
            .map(|a| a.iter().map(|v| v.as_str().unwrap().to_owned()).collect()),
        fault: item["fault"].as_str().map(str::to_owned),
    }
}

/// Original tests a frozen replay cannot reproduce. The first 2 change the corpus or its
/// reader in the middle of the call; corpus::tests proves each guard. The third measures
/// search relevance with corpus.py's partial-match floor and stop words replaced, a
/// configuration the server never runs.
const UNREPLAYABLE: [&str; 3] = [
    "tests/test_as_at.py::test_a_title_changed_after_validation_cannot_return_uncaptured_text",
    "tests/test_as_at.py::test_a_read_failure_while_counting_refusals_stops_the_search",
    "tests/test_search_relevance.py::test_the_one_word_floor_fails_the_gate_as_recorded",
];

#[test]
fn frozen_corpus_contracts() {
    let fixture = fixture();
    let tree = Tree(std::env::temp_dir().join(format!("native-mcp-corpus-{}", std::process::id())));
    assert!(!tree.0.exists());
    fs::create_dir(&tree.0).unwrap();
    let mut count = 0;
    for item in fixture["cases"].as_array().unwrap().iter().filter(|i| {
        (i["module"] == "corpus" || i["module"] == "boundary") && i["function"] != "census"
    }) {
        if UNREPLAYABLE.contains(&item["test"].as_str().unwrap()) {
            continue;
        }
        let args = &item["arguments"];
        let function = item["function"].as_str().unwrap();
        if item["module"] == "boundary" {
            let response = crate::handler::AccountingServer.call(function, args);
            if item.get("error").is_some() {
                assert_eq!(
                    response["content"][0]["text"], item["error"]["message"],
                    "{}",
                    item["test"]
                );
            } else {
                assert_eq!(response, item["result"], "{}", item["test"]);
            }
        } else if function == "_tier" {
            let strings = |field: &str| {
                args[field]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>()
            };
            let owned = |field: &str| -> Vec<String> {
                match args[field].as_array() {
                    Some(_) => strings(field).into_iter().map(str::to_owned).collect(),
                    None => Vec::new(),
                }
            };
            let words = owned("terms");
            let phrase = owned("phrase_terms");
            assert_eq!(
                json!(corpus::tier(
                    &words,
                    if phrase.is_empty() { &words } else { &phrase },
                    &strings("heading"),
                    &strings("body"),
                    &args["row"]
                )),
                item["result"],
                "{}",
                item["test"]
            );
        } else if function == "_definitions" {
            assert_eq!(
                json!(corpus::definitions(args["text"].as_str().unwrap())),
                item["result"],
                "{}",
                item["test"]
            );
        } else {
            let context = restore(&tree.0, item, &fixture["blobs"]);
            let name = match function {
                "search_sections" => "search_tax_legislation",
                "read_section" => "read_tax_legislation_section",
                "define_term" => "define_tax_term",
                "search_rates" => "search_tax_rates",
                other => panic!("Unknown corpus function {other}"),
            };
            let result = corpus::evaluate(&context, name, args);
            if item.get("error").is_some() {
                assert_eq!(
                    result.expect_err(item["test"].as_str().unwrap()),
                    item["error"]["message"],
                    "{}",
                    item["test"]
                );
            } else {
                let result = result.unwrap();
                crate::contracts::validate_output(name, &result).unwrap();
                assert_eq!(result, item["result"], "{}", item["test"]);
            }
        }
        count += 1;
    }
    assert_eq!(count, 659);
}

#[test]
fn frozen_rulings_contracts() {
    let fixture = fixture();
    let tree =
        Tree(std::env::temp_dir().join(format!("native-mcp-rulings-{}", std::process::id())));
    assert!(!tree.0.exists());
    fs::create_dir(&tree.0).unwrap();
    let mut count = 0;
    for item in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["module"] == "rulings" && i["function"] != "census")
    {
        let context = restore(&tree.0, item, &fixture["blobs"]);
        let name = match item["function"].as_str().unwrap() {
            "search_rulings" => "search_ato_rulings",
            "read_ruling" => "read_ato_ruling",
            other => panic!("Unknown rulings function {other}"),
        };
        let result = rulings::evaluate(&context, name, &item["arguments"]);
        if item.get("error").is_some() {
            assert_eq!(
                result.unwrap_err(),
                item["error"]["message"],
                "{}",
                item["test"]
            );
        } else {
            let result = result.unwrap();
            crate::contracts::validate_output(name, &result).unwrap();
            assert_eq!(result, item["result"], "{}", item["test"]);
        }
        count += 1;
    }
    assert_eq!(count, 90);
}

#[test]
fn frozen_library_contracts() {
    let fixture = fixture();
    let tree =
        Tree(std::env::temp_dir().join(format!("native-mcp-library-{}", std::process::id())));
    assert!(!tree.0.exists());
    fs::create_dir(&tree.0).unwrap();
    let mut count = 0;
    for item in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["module"] == "library" && i["function"] != "census")
    {
        let context = restore(&tree.0, item, &fixture["blobs"]);
        let args = &item["arguments"];
        let result = match item["function"].as_str().unwrap() {
            "read_reference" => library::read(
                &context,
                args["path"].as_str().unwrap(),
                position(&args["start_line"]),
                position(&args["line_count"]),
            ),
            "search_references" => library::search(
                &context,
                args["query"].as_str().unwrap(),
                args["limit"].as_u64().unwrap() as usize,
                args["offset"].as_u64().unwrap() as usize,
            ),
            "_excerpt" => {
                if args["start"].as_i64().unwrap() < 1 {
                    library::excerpt("fixture.md", &[], "digest", 0, 1)
                } else {
                    panic!("Uncaptured positive direct excerpt");
                }
            }
            other => panic!("Unknown function {other}"),
        };
        if item.get("error").is_some() {
            assert_eq!(
                result.unwrap_err(),
                item["error"]["message"],
                "{}",
                item["test"]
            );
        } else {
            let result = result.unwrap();
            let name = match item["function"].as_str().unwrap() {
                "read_reference" => "read_accounting_library",
                "search_references" => "search_accounting_library",
                other => panic!("Unexpected successful library helper {other}"),
            };
            crate::contracts::validate_output(name, &result).unwrap();
            assert_eq!(result, item["result"], "{}", item["test"]);
        }
        count += 1;
    }
    assert_eq!(count, 55);
}

/// The Python doctor's census of each folder its tests build, and the line it printed.
#[test]
fn frozen_census_contracts() {
    let fixture = fixture();
    let tree = Tree(std::env::temp_dir().join(format!("native-mcp-census-{}", std::process::id())));
    assert!(!tree.0.exists());
    fs::create_dir(&tree.0).unwrap();
    let mut count = 0;
    for item in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["function"] == "census")
    {
        let context = restore(&tree.0, item, &fixture["blobs"]);
        let (variable, found) = match item["module"].as_str().unwrap() {
            "library" => ("AUS_ACCOUNTING_LIBRARY_ROOT", library::census(&context)),
            "corpus" => ("AUS_ACCOUNTING_CORPUS_ROOT", corpus::census(&context)),
            "rulings" => ("AUS_ACCOUNTING_RULINGS_ROOT", rulings::census(&context)),
            other => panic!("Unknown census module {other}"),
        };
        match item.get("error") {
            Some(error) => assert_eq!(
                found.as_ref().unwrap_err(),
                &error["message"],
                "{}",
                item["test"]
            ),
            None => assert_eq!(found.as_ref().unwrap(), &item["result"], "{}", item["test"]),
        }
        let (ok, line) = crate::doctor::judged(variable, found);
        assert_eq!(json!([ok, line]), item["check"], "{}", item["test"]);
        count += 1;
    }
    assert_eq!(count, 28);
}

/// A row nested past any parser's depth is a broken corpus, not a crash. Python's
/// RecursionError text and serde_json's depth error differ, so only the verdict is compared.
#[test]
fn doctor_reports_a_row_nested_past_the_limit_as_broken() {
    let tree = Tree(std::env::temp_dir().join(format!("native-mcp-nested-{}", std::process::id())));
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
    let tree = Tree(std::env::temp_dir().join(format!(
        "native-mcp-relation-{label}-{}",
        std::process::id()
    )));
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
    for name in ["manifest.json", "rulings.jsonl", "LICENCE-NOTICE.md"] {
        fs::copy(
            valid.root.as_ref().unwrap().join("run-b").join(name),
            outside.join(name),
        )
        .unwrap();
    }
    let rows = fs::read_to_string(outside.join("rulings.jsonl")).unwrap();
    assert!(rows.contains("second fetch wording"));
    fs::write(
        outside.join("rulings.jsonl"),
        rows.replace("second fetch wording", "linked outside wording"),
    )
    .unwrap();
    let direct = Context {
        root: Some(outside),
        limits: Default::default(),
        denied: false,
        principal_acts: None,
        fault: None,
    };
    let args = json!({"query":"linked outside wording","limit":5,"offset":0,"family":null});
    assert!(
        !rulings::evaluate(&direct, "search_ato_rulings", &args).unwrap()["matches"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        rulings::evaluate(&linked, "search_ato_rulings", &args).unwrap()["matches"]
            .as_array()
            .unwrap()
            .is_empty()
    );
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
    for (query, row, expected) in [
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
    ] {
        assert_eq!(
            corpus::tier(&terms(query), &terms(query), &["heading"], &["text"], &row),
            expected,
            "{query}: {row}"
        );
    }
}

/// Main 139d5ac (#594): a dated call classifies a title from its first row and checks
/// only the rows it returns or counts, so a disagreeing row no result uses leaves it
/// answering, and a read looks for its row before checking the rows around it.
#[test]
fn a_dated_call_checks_only_the_rows_it_uses() {
    let tree =
        Tree(std::env::temp_dir().join(format!("native-mcp-agreeing-{}", std::process::id())));
    assert!(!tree.0.exists());
    let index = tree.0.join("markdown/C9999A10001/sections.jsonl");
    fs::create_dir_all(index.parent().unwrap()).unwrap();
    let texts = [
        "The synthetic levy applies to a dated receipt.",
        "The synthetic levy return must be lodged.",
        "A dated receipt must be kept.",
    ];
    let rows: Vec<String> = texts
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
        .collect();
    fs::write(&index, rows.join("\n") + "\n").unwrap();
    let context = Context {
        root: Some(tree.0.clone()),
        limits: Default::default(),
        denied: false,
        principal_acts: None,
        fault: None,
    };
    let search = |query: &str, on: &str| {
        corpus::evaluate(
            &context,
            "search_tax_legislation",
            &json!({"query":query,"limit":5,"offset":0,"act":null,"in_force_only":true,"as_at":on}),
        )
    };
    let read = |row_id: &str, neighbours: u64| {
        corpus::evaluate(
            &context,
            "read_tax_legislation_section",
            &json!({"row_id":row_id,"neighbours":neighbours,"start":0,"as_at":"2098-08-01"}),
        )
    };
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
    use std::os::unix::ffi::OsStringExt;
    let tree =
        Tree(std::env::temp_dir().join(format!("native-mcp-filename-{}", std::process::id())));
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
    let tree = Tree(std::env::temp_dir().join(format!("native-mcp-utf8-{}", std::process::id())));
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
    for (raw, text) in [
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
    ] {
        assert_eq!(
            first(raw),
            format!("'utf-8' codec can't decode {text}"),
            "{raw:?}"
        );
    }
}
