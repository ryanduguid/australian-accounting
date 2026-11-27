// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Snapshot names, record stems, the record listing and `write_text`'s
//! failures: each answer is what CPython 3.15.0 printed for the same input
//! (`tests/data/cov_a8_rates.py`).

use super::*;

/// A new, empty folder under the crate's `target/` for one test.
fn folder(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("cov-a8-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test folder");
    path
}

/// `urlparse(url).path` drops `;params` only after the last slash, and only
/// for a scheme that takes them.
#[test]
fn snapshot_names_drop_params_as_urlparse_does() {
    let ctx = Ctx::new();
    for (url, name) in [
        ("http:a;b", "a.txt"),
        ("https://h/x;p/y;q", "y.txt"),
        ("https://h/x;p/", "x;p.txt"),
    ] {
        let snap = snapshot_path(&ctx, &pts(url)).expect("snapshot path");
        assert_eq!(snap.name, pts(name), "{url}");
    }
}

#[test]
fn record_stems_follow_pathlib() {
    for (name, stem) in [
        ("r0.json", "r0"),
        (".json", ".json"),
        ("a.b.json", "a.b"),
        ("..json", "..json"),
    ] {
        let entry = Entry {
            name: pts(name),
            path: PathBuf::from(name),
            display: name.to_string(),
        };
        assert_eq!(entry.stem(), pts(stem), "{name}");
    }
}

/// `sorted(DATA.glob("*.json"))` skips other names and keeps hidden ones.
#[test]
fn the_record_listing_keeps_only_json_names_in_order() {
    let dir = folder("glob");
    for name in ["b.json", "a.json", "notes.txt", ".h.json", "x.json.bak"] {
        std::fs::write(dir.join(name), "{}").unwrap();
    }
    let entries = glob_json(&PyPath::new(&dir.to_string_lossy())).expect("listing");
    let names: Vec<S> = entries.into_iter().map(|e| e.name).collect();
    assert_eq!(names, [pts(".h.json"), pts("a.json"), pts("b.json")]);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn write_text_fails_as_path_write_text_does() {
    let dir = folder("write");
    let missing = dir.join("no-folder").join("x.txt");
    let display = pts("T/no-folder/x.txt");
    let e = write_text(Some(&missing), &display, &pts("x")).unwrap_err();
    assert_eq!(
        (e.class, pyvalue::points_lossy(&e.text)),
        (
            "FileNotFoundError",
            "[Errno 2] No such file or directory: 'T/no-folder/x.txt'".to_string()
        )
    );
    // A name a POSIX path cannot hold has no file to write.
    let lone = p!("/r/a", vec![0xD800], ".txt");
    let e = write_text(None, &lone, &pts("x")).unwrap_err();
    assert_eq!(
        (e.class, pyvalue::points_lossy(&e.text)),
        (
            "UnicodeEncodeError",
            "'utf-8' codec can't encode character '\\ud800' in position 4: surrogates not allowed"
                .to_string()
        )
    );
    std::fs::remove_dir_all(&dir).unwrap();
}
