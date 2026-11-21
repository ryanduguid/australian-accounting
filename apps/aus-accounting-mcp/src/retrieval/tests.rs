// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::*;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};

struct Tree(PathBuf);
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn scratch(label: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("scratch");
    fs::create_dir_all(&parent).unwrap();
    parent.join(format!(
        "{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
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

fn plain_path(value: &str) -> &Path {
    let relative = Path::new(value);
    assert!(
        !relative.is_absolute()
            && relative
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)))
    );
    relative
}

fn clear_path(root: &Path) {
    if root.is_dir() {
        fs::remove_dir_all(root).unwrap();
    } else if root.exists() {
        fs::remove_file(root).unwrap();
    }
}

fn write_file(path: &Path, entry: &Value, blobs: &Value) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let raw = bytes(&blobs[entry["blob"].as_str().unwrap()]);
    assert_eq!(
        Sha256::digest(&raw)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        entry["blob"].as_str().unwrap()
    );
    fs::write(path, raw).unwrap();
    if let Some(time) = entry["mtime_ns"].as_u64() {
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_nanos(time))
            .unwrap();
    }
}

fn write_link(tree: &Path, path: &Path, entry: &Value) {
    // The capture records links without following their targets.
    // Recreate them only against a fabricated sibling tree.
    let outside = tree.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    if entry["directory"] == true {
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&outside, path).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, path).unwrap();
    } else {
        let target = outside.join("linked.md");
        fs::write(&target, "Synthetic untrusted reference").unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, path).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, path).unwrap();
    }
}

fn write_entries(tree: &Path, root: &Path, state: &Value, blobs: &Value) {
    fs::create_dir_all(root).unwrap();
    for entry in state["files"].as_array().unwrap() {
        let path = root.join(plain_path(entry["path"].as_str().unwrap()));
        match entry["kind"].as_str().unwrap() {
            "directory" => fs::create_dir_all(&path).unwrap(),
            "file" => write_file(&path, entry, blobs),
            "link" => write_link(tree, &path, entry),
            other => panic!("Unexpected fixture kind {other}"),
        }
    }
}

fn restore(tree: &Path, item: &Value, blobs: &Value) -> Context {
    let state = &item["root"];
    let root = state["configured"]
        .as_bool()
        .unwrap_or(false)
        .then(|| tree.join(plain_path(state["root_id"].as_str().unwrap())));
    if let Some(root) = &root {
        clear_path(root);
        if state["exists"] == true {
            fs::create_dir_all(root.parent().unwrap()).unwrap();
            if state["kind"] == "file" {
                fs::write(root, bytes(&blobs[state["file_blob"].as_str().unwrap()])).unwrap();
            } else {
                write_entries(tree, root, state, blobs);
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

fn replay_boundary(item: &Value, function: &str, args: &Value) {
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
}

fn field_strings<'a>(args: &'a Value, field: &str) -> Vec<&'a str> {
    args[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect()
}

fn owned_field(args: &Value, field: &str) -> Vec<String> {
    match args[field].as_array() {
        Some(_) => field_strings(args, field)
            .into_iter()
            .map(str::to_owned)
            .collect(),
        None => Vec::new(),
    }
}

fn replay_tier(item: &Value, args: &Value) {
    let words = owned_field(args, "terms");
    let phrase = owned_field(args, "phrase_terms");
    assert_eq!(
        json!(corpus::tier(
            &words,
            if phrase.is_empty() { &words } else { &phrase },
            &field_strings(args, "heading"),
            &field_strings(args, "body"),
            &args["row"]
        )),
        item["result"],
        "{}",
        item["test"]
    );
}

// Replay the setup wording that replaced the original Python build-only guidance.
fn current_corpus_setup(expected: &str) -> String {
    let old = concat!(
        "`python -m fadden quick --out <empty folder>` in PaulGoodman builds 9 ",
        "principal Acts on this machine and prints the setting. "
    );
    let current = concat!(
        "PaulGoodman's published snapshot of 9 principal Acts needs no build: download ",
        "https://github.com/ryanduguid/PaulGoodman/releases/tag/corpus-2026-11-18, unpack it ",
        "and set the variable to its dist folder as an absolute path. Otherwise ",
        "`python -m fadden quick --out <empty folder>` in PaulGoodman builds the same Acts on ",
        "this machine and prints the setting. "
    );
    expected.replacen(old.trim_end(), current.trim_end(), 1)
}

fn replay_corpus_call(tree: &Path, item: &Value, blobs: &Value, function: &str, args: &Value) {
    let context = restore(tree, item, blobs);
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
            current_corpus_setup(item["error"]["message"].as_str().unwrap()),
            "{}",
            item["test"]
        );
    } else {
        let result = result.unwrap();
        crate::contracts::validate_output(name, &result).unwrap();
        assert_eq!(result, item["result"], "{}", item["test"]);
    }
}

fn replay_corpus(tree: &Path, item: &Value, blobs: &Value) {
    let args = &item["arguments"];
    let function = item["function"].as_str().unwrap();
    if item["module"] == "boundary" {
        replay_boundary(item, function, args);
    } else if function == "_tier" {
        replay_tier(item, args);
    } else if function == "_definitions" {
        assert_eq!(
            json!(corpus::definitions(args["text"].as_str().unwrap())),
            item["result"],
            "{}",
            item["test"]
        );
    } else {
        replay_corpus_call(tree, item, blobs, function, args);
    }
}

#[test]
fn frozen_corpus_contracts() {
    let fixture = fixture();
    let tree = Tree(scratch("native-mcp-corpus"));
    assert!(!tree.0.exists());
    fs::create_dir(&tree.0).unwrap();
    let mut count = 0;
    for item in fixture["cases"].as_array().unwrap().iter().filter(|i| {
        (i["module"] == "corpus" || i["module"] == "boundary") && i["function"] != "census"
    }) {
        if UNREPLAYABLE.contains(&item["test"].as_str().unwrap()) {
            continue;
        }
        replay_corpus(&tree.0, item, &fixture["blobs"]);
        count += 1;
    }
    assert_eq!(count, 659);
}

#[test]
fn frozen_rulings_contracts() {
    let fixture = fixture();
    let tree = Tree(scratch("native-mcp-rulings"));
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

fn library_call(context: &Context, item: &Value) -> Result<Value, String> {
    let args = &item["arguments"];
    match item["function"].as_str().unwrap() {
        "read_reference" => library::read(
            context,
            args["path"].as_str().unwrap(),
            position(&args["start_line"]),
            position(&args["line_count"]),
        ),
        "search_references" => library::search(
            context,
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
    }
}

fn replay_library(tree: &Path, item: &Value, blobs: &Value) {
    let context = restore(tree, item, blobs);
    let result = library_call(&context, item);
    if item.get("error").is_some() {
        assert_eq!(
            result.unwrap_err(),
            item["error"]["message"],
            "{}",
            item["test"]
        );
        return;
    }
    let result = result.unwrap();
    let name = match item["function"].as_str().unwrap() {
        "read_reference" => "read_accounting_library",
        "search_references" => "search_accounting_library",
        other => panic!("Unexpected successful library helper {other}"),
    };
    crate::contracts::validate_output(name, &result).unwrap();
    assert_eq!(result, item["result"], "{}", item["test"]);
}

#[test]
fn frozen_library_contracts() {
    let fixture = fixture();
    let tree = Tree(scratch("native-mcp-library"));
    assert!(!tree.0.exists());
    fs::create_dir(&tree.0).unwrap();
    let mut count = 0;
    for item in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["module"] == "library" && i["function"] != "census")
    {
        replay_library(&tree.0, item, &fixture["blobs"]);
        count += 1;
    }
    assert_eq!(count, 55);
}

/// The Python doctor's census of each folder its tests build, and the line it printed.
#[test]
fn frozen_census_contracts() {
    let fixture = fixture();
    let tree = Tree(scratch("native-mcp-census"));
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
                &current_corpus_setup(error["message"].as_str().unwrap()),
                "{}",
                item["test"]
            ),
            None => assert_eq!(found.as_ref().unwrap(), &item["result"], "{}", item["test"]),
        }
        let (ok, line) = crate::doctor::judged(variable, found);
        assert_eq!(ok, item["check"][0].as_bool().unwrap(), "{}", item["test"]);
        assert_eq!(
            line,
            current_corpus_setup(item["check"][1].as_str().unwrap()),
            "{}",
            item["test"]
        );
        count += 1;
    }
    assert_eq!(count, 28);
}

mod edges;
