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

#[test]
fn frozen_corpus_contracts() {
    let fixture = fixture();
    let tree = Tree(std::env::temp_dir().join(format!("native-mcp-corpus-{}", std::process::id())));
    assert!(!tree.0.exists());
    fs::create_dir(&tree.0).unwrap();
    let mut count = 0;
    for item in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["module"] == "corpus" || i["module"] == "boundary")
    {
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
            let words = strings("terms")
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            assert_eq!(
                json!(corpus::tier(
                    &words,
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
        }
        count += 1;
    }
    assert_eq!(count, 130);
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
        .filter(|i| i["module"] == "rulings")
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
    assert_eq!(count, 77);
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
        .filter(|i| i["module"] == "library")
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
    assert_eq!(count, 43);
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
            corpus::tier(&terms(query), &["heading"], &["text"], &row),
            expected,
            "{query}: {row}"
        );
    }
}

#[cfg(unix)]
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
