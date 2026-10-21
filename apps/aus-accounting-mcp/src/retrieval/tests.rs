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
                assert_eq!(result.unwrap(), item["result"], "{}", item["test"]);
            }
        }
        count += 1;
    }
    assert_eq!(count, 127);
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
            assert_eq!(result.unwrap(), item["result"], "{}", item["test"]);
        }
        count += 1;
    }
    assert_eq!(count, 76);
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
            assert_eq!(result.unwrap(), item["result"], "{}", item["test"]);
        }
        count += 1;
    }
    assert_eq!(count, 38);
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
